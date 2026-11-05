# Headless track checks. Does not connect to the rig or start Minecraft.
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\..\..\scripts\common.ps1"
$testBuild = Join-Path $Root 'bot\build\track-tests'
New-Item -ItemType Directory -Force $testBuild | Out-Null
$classpath = (Get-GameClasspath) -join ';'
$sourceRoot = Join-Path $Root 'bot\src\pathtracing'
& "$Jdk\javac.exe" --release 25 -proc:none -cp $classpath -d $testBuild `
  "$sourceRoot\TrackDetector.java" "$sourceRoot\TrackUsage.java" "$sourceRoot\TrackStore.java" `
  "$sourceRoot\TrackRcon.java" "$sourceRoot\TrackRecording.java" "$PSScriptRoot\TrackTests.java" "$PSScriptRoot\TrackMixinAudit.java"
if ($LASTEXITCODE) { throw 'Track test compilation failed' }
$testRig = Join-Path $testBuild ([Guid]::NewGuid().ToString('N'))
& "$Jdk\java.exe" -cp "$testBuild;$classpath" pathtracing.TrackTests $testRig `
  "$Root\bot\build\pathtracing.jar" "$Libraries\com\mojang\minecraft\$McVersion\minecraft-$McVersion-client.jar"
if ($LASTEXITCODE) { throw 'Track tests failed' }
foreach ($script in @('scripts\launch-client.ps1', 'bot\build.ps1', 'bot\tests\test-tracks.ps1')) {
  $tokens = $null
  $parseErrors = $null
  $scriptPath = Join-Path $Root $script
  [void][Management.Automation.Language.Parser]::ParseFile($scriptPath, [ref]$tokens, [ref]$parseErrors)
  if ($parseErrors.Count) { throw "PowerShell parse errors in $script" }
  if (@([IO.File]::ReadAllBytes($scriptPath) | Where-Object { $_ -gt 127 }).Count) { throw "Non-ASCII script: $script" }
}
Write-Host 'PowerShell 5.1 parsing and ASCII checks passed.'
