# One cheap Antigravity run with a given permission set, to learn what lets run_command work (jobs/agy-commands).
#   .\scripts\probe-agy-commands.ps1 -Name A-all-forms -Forms all
#   .\scripts\probe-agy-commands.ps1 -Name B-drive-only -Forms drive
# The run gets a trivial task: run three commands in a scratch Cargo project and write their outputs.
param(
  [Parameter(Mandatory)][string]$Name,
  [ValidateSet('all','drive','nodriveless')][string]$Forms = 'drive',
  [string]$Model = 'gemini-4-hydrogen-low',
  [string[]]$Commands = @('command(cargo)','command(git)'),
  [string]$Task,
  [string]$Format = 'json',
  [switch]$Sandbox,
  [switch]$SettingsInWork,
  [switch]$NoJobDir,
  [switch]$WorkWriteOnly,
  [string[]]$Extra = @()
)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\common.ps1"
$utf8 = [Text.UTF8Encoding]::new($false)
$work = 'D:\path-tracing-wt\_probe-cmd'
$dir = Join-Path $Root "jobs\_probe-cmd\$Name"; New-Item -ItemType Directory -Force $dir | Out-Null
$report = Join-Path $dir 'report.md'
function Forms([string]$p) {
  $slash = ($p -replace '\\','/').TrimEnd('/') + '/'
  $lower = $slash.Substring(0,1).ToLower() + $slash.Substring(1)
  $all = switch ($Forms) {
    'all'         { @($slash, $lower, ($slash -replace '^[A-Za-z]:',''), "/$slash", "file:///$slash", "file:///$lower", "file://$slash") }
    'nodriveless' { @($slash, $lower, "file:///$slash", "file:///$lower", "file://$slash") }
    'drive'       { @($slash, $lower) }
  }
  $all + @($all | ForEach-Object { $_ -replace ' ','%20' }) | Select-Object -Unique
}
$allow = @()
foreach ($f in (Forms $Root)) { $allow += "read_file($f)" }
foreach ($f in (Forms $work)) { if (-not $WorkWriteOnly) { $allow += "read_file($f)" }; $allow += "write_file($f)" }
foreach ($f in (Forms $dir))  { $allow += "read_file($f)"; $allow += "write_file($f)" }
$allow += $Commands; $allow += $Extra
$settings = Join-Path $(if ($SettingsInWork) { $work } else { $dir }) 'settings.json'
[IO.File]::WriteAllText($settings, (@{ permissions = @{ allow = $allow } } | ConvertTo-Json -Depth 5), $utf8)
if (-not $Task) {
  $Task = "This is a short test of your tools. Your working directory is $work, a tiny Cargo project.`n" +
          "1. Use your command tool to run ``cargo --version``.`n2. Run ``git --version``.`n3. Run ``cargo run --quiet`` in $work.`n" +
          "4. Read the file $work\src\main.rs.`n" +
          "Write the exact output (or the exact error) of each step into $report, then reply with the same text. Do nothing else."
}
$prompt = Join-Path $dir 'prompt.txt'; [IO.File]::WriteAllText($prompt, $Task, $utf8)
$agy = Join-Path $env:LOCALAPPDATA 'agy\bin\agy.exe'
$a = @('--output-format',$Format,'--model',$Model,'--print-timeout','10m','--add-dir',$dir,'--add-dir',$work,
       '--settings',$settings,'--log-file',(Join-Path $dir 'agy.log'),'--mode','accept-edits')
if ($Sandbox) { $a += '--sandbox' }
if ($NoJobDir) { $i = [array]::IndexOf($a, $dir); $a = @($a[0..($i-2)]) + @($a[($i+1)..($a.Count-1)]) }
$quoted = $a | ForEach-Object { if ($_ -match '\s') { '"' + $_ + '"' } else { $_ } }
$sw = [Diagnostics.Stopwatch]::StartNew()
$p = Start-Process $agy -ArgumentList $quoted -WorkingDirectory $work -NoNewWindow -PassThru -Wait `
  -RedirectStandardInput $prompt -RedirectStandardOutput (Join-Path $dir 'result.json') -RedirectStandardError (Join-Path $dir 'stderr.txt')
"$Name ($Forms): exit $($p.ExitCode) after $([int]$sw.Elapsed.TotalSeconds)s"
$j = Get-Content (Join-Path $dir 'result.json') -Raw -ErrorAction SilentlyContinue | ConvertFrom-Json -ErrorAction SilentlyContinue
if ($j) { "response:"; [string]$j.response; "denied: $(@($j.denied_actions | ForEach-Object { $_.action }) -join ' | ')"; "usage: in $($j.usage.input_tokens) out $($j.usage.output_tokens)" }
"stderr tail:"; Get-Content (Join-Path $dir 'stderr.txt') -Tail 6 -ErrorAction SilentlyContinue
