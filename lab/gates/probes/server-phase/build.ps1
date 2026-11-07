param(
    [Parameter(Mandatory)][string]$Server,
    [Parameter(Mandatory)][string]$Output,
    [string]$ClientLib
)
$ErrorActionPreference = 'Stop'
$tree = (Resolve-Path -LiteralPath "$PSScriptRoot/../../../..").Path
. "$tree/scripts/common.ps1"
if (-not $ClientLib) { $ClientLib = "$tree/bot/lib" }
$null = New-Item -ItemType Directory -Path $Output -Force
$classes = Join-Path $Output 'classes'
$null = New-Item -ItemType Directory -Path $classes
$dependencies = @(
    "$Server/versions/$McVersion/server-$McVersion.jar"
    "$Libraries/net/fabricmc/fabric-loader/$LoaderVersion/fabric-loader-$LoaderVersion.jar"
    (Get-ChildItem "$Libraries/net/fabricmc/sponge-mixin" -Recurse -Filter '*.jar').FullName
    (Get-ChildItem "$Server/libraries" -Recurse -Filter '*.jar').FullName
    (Get-ChildItem -LiteralPath $ClientLib -Filter '*.jar').FullName
)
$sources = (Get-ChildItem -LiteralPath $PSScriptRoot -Filter '*.java').FullName
& "$Jdk/javac.exe" --release 25 -proc:none -cp ($dependencies -join ';') -d $classes $sources
if ($LASTEXITCODE) { throw 'Probe compilation failed. Set up the reference rig and build the bot first.' }
Copy-Item -LiteralPath "$PSScriptRoot/fabric.mod.json", "$PSScriptRoot/phase.mixins.json" -Destination $classes
$probe = Join-Path $Output 'phase-probe.jar'
& "$Jdk/jar.exe" --create --file $probe -C $classes .
if ($LASTEXITCODE) { throw 'Cannot package the passive probe.' }
$probe
