param([Parameter(Mandatory)][string]$Output)
$ErrorActionPreference = 'Stop'
$env:PATHTRACING_DEPS = 'D:/path-tracing-deps/data'
$env:CARGO_TARGET_DIR = 'D:/path-tracing-target/raw-fluid-export'
. "$PSScriptRoot/../../scripts/common.ps1"
$folder = New-Item -ItemType Directory -Path $Output
$fixtureBot = 'D:/path-tracing-wt/fixtures/bot'
$cp = (@(Get-GameClasspath) + @((Get-ChildItem "$fixtureBot/lib/*.jar").FullName) + @("$fixtureBot/build/classes")) -join ';'
& "$Jdk/javac.exe" --release 25 -proc:none -encoding UTF-8 -cp $cp -d $folder.FullName "$PSScriptRoot/../../bot/src/pathtracing/WorldExporter.java" "$PSScriptRoot/RawExporterCheck.java"
if ($LASTEXITCODE) { throw 'exporter compile failed' }
& "$Jdk/java.exe" -cp ($folder.FullName + ';' + $cp) pathtracing.RawExporterCheck $folder.FullName
if ($LASTEXITCODE) { throw 'exporter oracle failed' }
$env:PATHTRACING_RAW_ROWS = $folder.FullName
& cargo test --offline --manifest-path "$PSScriptRoot/../../engine/Cargo.toml" --test replay raw_exporter_oracle -- --ignored --exact
if ($LASTEXITCODE) { throw 'raw oracle replay failed' }
& cargo test --offline --manifest-path "$PSScriptRoot/../../engine/Cargo.toml"
if ($LASTEXITCODE) { throw 'law suite failed' }
