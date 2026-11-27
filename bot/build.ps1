# Compiles against the selected game and Fabric jars without remapping.
param([string]$McVersion = $env:PATHTRACING_MC_VERSION, [string]$Output = "$PSScriptRoot/build")
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\..\scripts\common.ps1" -McVersion $McVersion

$build = [IO.Path]::GetFullPath($Output)
$classes = Join-Path $build 'classes'
if (Test-Path -LiteralPath $classes) { Remove-Item -LiteralPath $classes -Recurse -Force }
$null = New-Item -ItemType Directory -Force $classes

$null = Get-VerifiedClientJar
$classpath = @(Get-GameClasspath) + @(Get-FabricModules "$build/lib/$McVersion")
$sources = (Get-ChildItem "$PSScriptRoot/src", "$PSScriptRoot/src-version/$McVersion" -Recurse -Filter *.java).FullName
$argFile = Join-Path $build 'javac.args'
(@('--release', '25', '-proc:none', '-encoding', 'UTF-8', '-Xlint:-options', '-cp', ($classpath -join ';'), '-d', "$build\classes") + $sources |
  ForEach-Object { '"' + ($_ -replace '\\', '/') + '"' }) -join "`n" | Set-Content $argFile -Encoding ascii
& "$Jdk\javac.exe" "@$argFile"
if ($LASTEXITCODE) { throw 'javac failed' }

Write-FabricMetadata "$PSScriptRoot\fabric.mod.json" $classes
Copy-Item "$PSScriptRoot\pathtracing.mixins.json" "$build\classes\"
& "$Jdk\jar.exe" --create --file "$build\pathtracing.jar" -C "$build\classes" .
if ($LASTEXITCODE) { throw 'jar failed' }
Write-Host "Built $build\pathtracing.jar"
