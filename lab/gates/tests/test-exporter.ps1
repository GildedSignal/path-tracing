# Run the actual export classification against the staged jar without starting the game.
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\..\..\..\scripts\common.ps1"
. "$PSScriptRoot\..\courses.ps1"
$scratch = Join-Path $PSScriptRoot ('..\results\exporter-' + [Guid]::NewGuid().ToString('N'))
$null = New-Item -ItemType Directory -Force $scratch
$ids = @(Get-FixtureCourses | ForEach-Object { $_.Commands } | ForEach-Object {
  $parts = $_.Split(' ')
  if ($parts[0] -eq 'fill') { $parts[7].Split('[')[0] } else { $parts[4].Split('[')[0] }
} | Sort-Object -Unique)
$blocks = Join-Path $scratch 'blocks.txt'
$rows = Join-Path $scratch 'hook-rows.jsonl'
Set-Content -LiteralPath $blocks -Value $ids -Encoding ASCII
$classpath = (@(Get-GameClasspath) + @((Get-ChildItem "$Root\bot\lib\*.jar").FullName) + @("$Root\bot\build\classes")) -join ';'
& "$Jdk\javac.exe" -proc:none -cp $classpath -d $scratch "$PSScriptRoot\ExporterCheck.java"
if ($LASTEXITCODE) { throw 'Exporter check did not compile; build the bot first.' }
Push-Location $scratch
try {
  & "$Jdk\java.exe" -cp ($classpath + ';' + $scratch) pathtracing.ExporterCheck $blocks $rows
  if ($LASTEXITCODE) { throw 'Offline exporter check failed.' }
} finally { Pop-Location }

$previousRows = $env:PATHTRACING_HOOK_ROWS
Push-Location "$Root\engine"
try {
  $env:PATHTRACING_HOOK_ROWS = $rows
  & cargo test --offline --test replay -- registry_export_contacts --ignored --exact
  if ($LASTEXITCODE) { throw 'Offline exporter replay check failed.' }
} finally {
  $env:PATHTRACING_HOOK_ROWS = $previousRows
  Pop-Location
}
