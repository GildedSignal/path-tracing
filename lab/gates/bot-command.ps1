param(
  [Parameter(Mandatory = $true)][ValidateNotNullOrEmpty()][string]$Text,
  [string]$WaitFor,
  [ValidateRange(1, 2147483647)][int]$TimeoutSeconds = 30,
  [string]$FailOn = 'already exists|Usage|Aborted|Refusing|Recorder/dumper:|Replay is running\.|I can''t see you\.|No path to |Stuck at |Recording finalization failed'
)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\..\..\scripts\common.ps1"

$reader = $null
try {
  if ($PSBoundParameters.ContainsKey('WaitFor')) {
    $pattern = [regex]::new($WaitFor)
    $failurePattern = [regex]::new($FailOn)
    $log = Join-Path $Rig 'clients\bot\logs\latest.log'
    $stream = [IO.File]::Open($log, [IO.FileMode]::Open, [IO.FileAccess]::Read,
      [IO.FileShare]::ReadWrite -bor [IO.FileShare]::Delete)
    $null = $stream.Seek(0, [IO.SeekOrigin]::End)
    $reader = [IO.StreamReader]::new($stream, [Text.Encoding]::UTF8)
  }
  $clock = [Diagnostics.Stopwatch]::StartNew()
  $null = & "$PSScriptRoot\..\..\scripts\rcon.ps1" "msg PathBot $Text"
  if ($null -eq $reader) { exit 0 }
  $pending = ''
  while ($clock.Elapsed.TotalSeconds -lt $TimeoutSeconds) {
    $pending += $reader.ReadToEnd()
    while (($newline = $pending.IndexOf("`n")) -ge 0) {
      $line = $pending.Substring(0, $newline).TrimEnd([char]13)
      $pending = $pending.Substring($newline + 1)
      $prefix = $line.IndexOf('[pathtracing] ', [StringComparison]::Ordinal)
      if ($prefix -ge 0 -and $failurePattern.IsMatch($line)) {
        Write-Output $line
        exit 2
      }
      if ($prefix -ge 0 -and $pattern.IsMatch($line)) {
        Write-Output $line
        exit 0
      }
    }
    Start-Sleep -Milliseconds 50
  }
  throw "Timed out waiting for a [pathtracing] reply matching '$WaitFor'."
} catch {
  Write-Output $_.Exception.Message
  exit 1
} finally {
  if ($null -ne $reader) { $reader.Dispose() }
}
