# Headless track checks. Does not connect to the rig or start Minecraft.
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\..\..\scripts\common.ps1"
$testBuild = Join-Path $Root 'bot\build\track-tests'
New-Item -ItemType Directory -Force $testBuild | Out-Null
$classpath = (Get-GameClasspath) -join ';'
$sourceRoot = Join-Path $Root 'bot\src\pathtracing'
$beforeCommit = '396a955a5b554d120c257eaa1f7729a476dfc5b6'
$beforeSource = (& git -C $Root show "${beforeCommit}:bot/src/pathtracing/TrackMode.java") -join "`n"
if ($LASTEXITCODE) { throw 'Cannot read the frozen before implementation.' }
$beforeFile = Join-Path $testBuild 'BeforeTrackMode.java'
[IO.File]::WriteAllText($beforeFile, ($beforeSource -replace '\bTrackMode\b', 'BeforeTrackMode'))
& "$Jdk\javac.exe" --release 25 -proc:none -cp $classpath -d $testBuild `
  "$sourceRoot\NameSuggestions.java" "$sourceRoot\RaceCommands.java" "$sourceRoot\MapRegistry.java" "$sourceRoot\TrackHumanRace.java" "$sourceRoot\TrackDetector.java" "$sourceRoot\TrackUsage.java" "$sourceRoot\TrackStore.java" `
  "$sourceRoot\TrackRcon.java" "$sourceRoot\TrackRecording.java" "$sourceRoot\TrackInventory.java" `
  "$sourceRoot\TrackInventoryCodec.java" "$sourceRoot\TrackMode.java" "$PSScriptRoot\TrackModeTests.java" "$PSScriptRoot\TrackRaceTests.java" `
  "$PSScriptRoot\TrackTests.java" "$PSScriptRoot\TrackMixinAudit.java" $beforeFile
if ($LASTEXITCODE) { throw 'Track test compilation failed' }
$testRig = Join-Path $testBuild ([Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Force $testRig | Out-Null
& "$Jdk\java.exe" "-Dpathtracing.beforeCommit=$beforeCommit" "-Dpathtracing.uxEvidence=$Root\jobs\track-ux\arming.json" -cp "$testBuild;$classpath" pathtracing.TrackTests $testRig `
  "$Root\bot\build\pathtracing.jar" "$Libraries\com\mojang\minecraft\$McVersion\minecraft-$McVersion-client.jar" | Tee-Object -FilePath (Join-Path $testRig 'stdout.txt')
$testExit = $LASTEXITCODE
@{command='bot/tests/test-tracks.ps1'; baselineCommit=$beforeCommit; testRig=$testRig; exitCode=$testExit;
  result=$(if ($testExit) {'FAIL'} else {'PASS'});
  modSha256=(Get-FileHash -LiteralPath "$Root\bot\build\pathtracing.jar").Hash;
  gameSha256=(Get-FileHash -LiteralPath "$Libraries\com\mojang\minecraft\$McVersion\minecraft-$McVersion-client.jar").Hash;
  worstMargins='Exact assertions; no tolerances'; refusals='Intentional slot/query/permission/refill failures and untimed departures';
  replay='Run bot/tests/test-tracks.ps1; clock phases and transport delays are in arming.json'} |
  ConvertTo-Json | Set-Content -LiteralPath "$Root\jobs\track-ux\track-checks.json" -Encoding ASCII
if ($testExit) { throw 'Track tests failed' }
foreach ($script in @('scripts\launch-client.ps1', 'bot\build.ps1', 'bot\tests\test-tracks.ps1')) {
  $tokens = $null
  $parseErrors = $null
  $scriptPath = Join-Path $Root $script
  [void][Management.Automation.Language.Parser]::ParseFile($scriptPath, [ref]$tokens, [ref]$parseErrors)
  if ($parseErrors.Count) { throw "PowerShell parse errors in $script" }
  if (@([IO.File]::ReadAllBytes($scriptPath) | Where-Object { $_ -gt 127 }).Count) { throw "Non-ASCII script: $script" }
}
Write-Host 'PowerShell 5.1 parsing and ASCII checks passed.'
