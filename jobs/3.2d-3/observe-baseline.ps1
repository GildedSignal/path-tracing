# K-only runner. A builds/parses this file but NEVER invokes its Live branch.
param(
 [Parameter(Mandatory)][string]$OriginalScript,
 [Parameter(Mandatory)][string]$Track,
 [Parameter(Mandatory)][string]$Capture,
 [switch]$Live
)
$ErrorActionPreference = 'Stop'
if (!$Live) { throw 'ObserverOnly requires K live authorization and the frozen original binary/inputs; A is offline.' }
Add-Type -TypeDefinition @'
using System.Runtime.InteropServices;
public static class BaselineQpc {
 [DllImport("kernel32.dll")] static extern bool QueryPerformanceCounter(out long n);
 [DllImport("kernel32.dll")] static extern bool QueryPerformanceFrequency(out long n);
 public static long Now() { long n; if(!QueryPerformanceCounter(out n)) throw new System.Exception("QPC failed"); return n; }
 public static long Frequency() { long n; if(!QueryPerformanceFrequency(out n)) throw new System.Exception("QPF failed"); return n; }
}
'@
$script:hopRows = [Collections.Generic.List[string]]::new()
function Write-BaselineHop([string]$Kind) {
    $sample = [BaselineQpc]::Now()
    $script:hopRows.Add("$Kind`t0`t0`t$sample`t-1`t[]")
}
$text = Get-Content -LiteralPath $OriginalScript
$frozen = Get-Content -LiteralPath 'D:/path-tracing-evidence/3.2d-3-A-baseline/scripts_play-track.ps1'
if (($text -join "`n") -cne ($frozen -join "`n")) { throw 'Baseline script changed from its frozen original' }
$sites = @(
 @{pattern='$planner = & $exe';kind='legacy-planner-start'},
 @{pattern='$clock.Stop()';kind='legacy-planner-end'},
 @{pattern='$null = & "$PSScriptRoot\rcon.ps1"';kind='legacy-rcon-send'},
 @{pattern='Start-Sleep -Milliseconds 50';kind='legacy-log-poll'}
)
$breakpoints = @()
try {
    foreach ($site in $sites) {
        $found = $false
        for ($i=0;$i -lt $text.Count;$i++) { if ($text[$i].Contains($site.pattern)) {
            $found = $true
            $kind = $site.kind
            $action = [scriptblock]::Create("Write-BaselineHop '$kind'")
            $breakpoints += Set-PSBreakpoint -Script $OriginalScript -Line ($i+1) -Action $action
        } }
        if (!$found) { throw "Frozen baseline is missing capture site $($site.kind)" }
    }
    $issued = [BaselineQpc]::Now()
    $frequency = [BaselineQpc]::Frequency()
    $script:hopRows.Add("issuance`t0`t$issued`t$issued`t-1`t[$frequency]")
    & $OriginalScript -Track $Track
    if ($LASTEXITCODE) { throw 'Original baseline returned failure' }
} finally {
    $breakpoints | Remove-PSBreakpoint
    New-Item -ItemType Directory -Force $Capture | Out-Null
    [IO.File]::WriteAllLines((Join-Path $Capture "hops-issuer-$PID.tsv"),$script:hopRows,[Text.UTF8Encoding]::new($false))
}
