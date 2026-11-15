param([string]$Evidence = 'D:\path-tracing-evidence\3.2d-3-A')
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot/../../scripts/common.ps1"
$env:CARGO_TARGET_DIR = 'D:\path-tracing-target\r0-channel'
Push-Location "$Root/engine"
try { & cargo build --offline --release -p pathtracing_native; if ($LASTEXITCODE) { throw 'Native build failed' } } finally { Pop-Location }
$env:PATHTRACING_NATIVE = Join-Path $env:CARGO_TARGET_DIR 'release/pathtracing_native.dll'
$testBuild = Join-Path $env:CARGO_TARGET_DIR 'java-native-tests'
New-Item -ItemType Directory -Force $testBuild,$Evidence | Out-Null
$cp = (Get-GameClasspath) -join ';'
& "$Jdk/javac.exe" --release 25 -proc:none -encoding UTF-8 -cp $cp -d $testBuild `
 "$Root/bot/src/pathtracing/NativeChannel.java" "$Root/bot/src/pathtracing/StateBits.java" `
 "$PSScriptRoot/NativeClockTests.java" "$PSScriptRoot/NativeChannelTests.java"
if ($LASTEXITCODE) { throw 'Native Java tests did not compile' }
& "$Jdk/java.exe" --enable-native-access=ALL-UNNAMED -cp "$testBuild;$cp" pathtracing.NativeChannelTests "$PSScriptRoot/fixtures" "$Evidence/recorded-state.hex"
if ($LASTEXITCODE) { throw 'Native Java tests failed' }
