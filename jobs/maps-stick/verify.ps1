# Offline gates for Julien racing downloaded maps and the bot racing the same tracks.
param([string]$Destination)
$ErrorActionPreference = 'Stop'
$codeRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..\..'))
if (-not $Destination) { $Destination = 'D:\path-tracing-target\maps-stick\gate-' + [Guid]::NewGuid().ToString('N') }
$Destination = [IO.Path]::GetFullPath($Destination)
if (-not $Destination.StartsWith('D:\path-tracing-target\maps-stick\', [StringComparison]::OrdinalIgnoreCase)) { throw 'Gate records must stay within the maps-stick target.' }
if (Test-Path -LiteralPath $Destination) { throw 'Use a fresh gate record folder.' }
New-Item -ItemType Directory -Path $Destination | Out-Null
$oldTarget = $env:CARGO_TARGET_DIR
$env:CARGO_TARGET_DIR = 'D:\path-tracing-target\maps-stick'
$record = [ordered]@{ consumer='Human and bot races on downloaded maps'; root=$codeRoot; cargoTarget=$env:CARGO_TARGET_DIR; started=[DateTime]::UtcNow.ToString('o'); gates=@(); result='RUNNING' }
Start-Transcript -Path (Join-Path $Destination 'output.txt') | Out-Null
try {
  # build.ps1 deletes only this workspace's generated build folder.
  $buildPath = [IO.Path]::GetFullPath((Join-Path $codeRoot 'bot\build'))
  if (-not $buildPath.StartsWith($codeRoot + '\', [StringComparison]::OrdinalIgnoreCase)) { throw 'Build folder escaped the workspace.' }
  & "$codeRoot\bot\build.ps1"
  $record.gates += @{command='bot/build.ps1'; result='PASS'}
  Push-Location "$codeRoot\engine"
  try {
    & cargo test --offline --workspace
    if ($LASTEXITCODE) { throw 'Rust workspace gate failed.' }
  } finally { Pop-Location }
  $record.gates += @{command='cargo test --offline --workspace'; result='PASS'}
  & "$codeRoot\bot\tests\test-tracks.ps1"
  $record.gates += @{command='bot/tests/test-tracks.ps1'; result='PASS'; namesAccepted=@('hard_track_2','a-b'); namesRefused=@('A','a b','a/b')}
  & "$codeRoot\bot\tests\test-maps.ps1" -Destination (Join-Path $Destination 'import') -Planner "$env:CARGO_TARGET_DIR\debug\plan-track.exe"
  $record.gates += @{command='bot/tests/test-maps.ps1'; result='PASS'; record='import/record.json'}
  foreach ($script in @('scripts\import-map.ps1','scripts\track-world-view.ps1','scripts\play-track.ps1','bot\tests\test-maps.ps1','jobs\maps-stick\verify.ps1')) {
    $tokens = $null; $errors = $null
    $file = Join-Path $codeRoot $script
    [void][Management.Automation.Language.Parser]::ParseFile($file, [ref]$tokens, [ref]$errors)
    if ($errors.Count) { throw "PowerShell parse errors: $script" }
    if (@([IO.File]::ReadAllBytes($file) | Where-Object { $_ -gt 127 }).Count) { throw "Non-ASCII script: $script" }
  }
  Push-Location $codeRoot
  try {
    & git check-ignore --no-index maps.json maps.json.lock server/world/level.dat server/world/dimensions/pathtracing/example/region/r.0.0.mca
    if ($LASTEXITCODE) { throw 'Generated map data is not ignored.' }
    & git diff --check
    if ($LASTEXITCODE) { throw 'Whitespace check failed.' }
  } finally { Pop-Location }
  $record.gates += @{command='PowerShell parsing, ASCII, git ignore and diff checks'; result='PASS'}
  $record.result = 'PASS'
} catch {
  $record.result = 'FAIL'
  $record.failure = $_.Exception.Message
  throw
} finally {
  $record.finished = [DateTime]::UtcNow.ToString('o')
  $record | ConvertTo-Json -Depth 7 | Set-Content (Join-Path $Destination 'record.json') -Encoding ascii
  $env:CARGO_TARGET_DIR = $oldTarget
  Stop-Transcript | Out-Null
}
Write-Output "Gate record: $Destination\record.json"
