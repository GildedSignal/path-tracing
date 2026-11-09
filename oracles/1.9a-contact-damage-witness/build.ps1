param(
    [Parameter(Mandatory)][string]$Server,
    [Parameter(Mandatory)][string]$Output,
    [switch]$CheckHooks
)
$ErrorActionPreference = 'Stop'
$tree = (Resolve-Path -LiteralPath "$PSScriptRoot/../..").Path
. "$tree/scripts/common.ps1"
$Server = (Resolve-Path -LiteralPath $Server).Path
if (Test-Path -LiteralPath $Output) { throw 'Use a fresh build output directory.' }
$classes = (New-Item -ItemType Directory -Path (Join-Path $Output 'classes') -Force).FullName
$loader = Get-Content -LiteralPath "$Meta/net.fabricmc.fabric-loader/$LoaderVersion.json" -Raw | ConvertFrom-Json
$serverLibraries = Get-ChildItem "$Server/libraries" -Recurse -Filter '*.jar' | Where-Object {
    $_.FullName -notmatch '[\\/](net[\\/]fabricmc|org[\\/]ow2[\\/]asm)[\\/]'
}
$dependencies = @(
    "$Server/versions/$McVersion/server-$McVersion.jar"
    ($loader.libraries | ForEach-Object { Get-LibraryPath $_ })
    $serverLibraries.FullName
)
$sources = (Get-ChildItem -LiteralPath $PSScriptRoot -Filter '*.java').FullName
& "$Jdk/javac.exe" --release 25 -proc:none -cp ($dependencies -join ';') -d $classes $sources
if ($LASTEXITCODE) { throw 'Contact witness compilation failed.' }
Copy-Item -LiteralPath "$PSScriptRoot/fabric.mod.json", "$PSScriptRoot/contact.mixins.json" -Destination $classes
$jar = Join-Path (Split-Path -Parent $classes) 'contact-witness.jar'
& "$Jdk/jar.exe" --create --file $jar -C $classes .
if ($LASTEXITCODE) { throw 'Contact witness packaging failed.' }
if ($CheckHooks) {
    $test = Join-Path $PSScriptRoot 'test/TransformCheck.java'
    Push-Location (Split-Path -Parent $classes)
    try {
        & "$Jdk/java.exe" "-Dfabric.addMods=$jar" -cp ($dependencies -join ';') $test
        if ($LASTEXITCODE) { throw 'Offline Fabric hook check failed.' }
    } finally { Pop-Location }
}
$jar
