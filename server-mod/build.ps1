param([string]$Output = 'D:\path-tracing-target\server-mod-build', [string]$McVersion = $env:PATHTRACING_MC_VERSION)
$ErrorActionPreference = 'Stop'
$dependencies = if ($env:PATHTRACING_DEPS) { $env:PATHTRACING_DEPS } else { 'D:\path-tracing-deps\data' }
. "$PSScriptRoot\..\scripts\common.ps1" -McVersion $McVersion -Dependencies $dependencies

$Output = [IO.Path]::GetFullPath($Output)
$classes = Join-Path $Output 'classes'
if (Test-Path -LiteralPath $classes) { Remove-Item -LiteralPath $classes -Recurse -Force }
$null = New-Item -ItemType Directory -Force $classes
$deps = @(Get-ServerBundleClasspath "$Output/vanilla/$McVersion") + @(Get-FabricLoaderClasspath) +
    @(Get-FabricModules "$Output/lib/$McVersion")

$sources = @(
  Get-ChildItem "$PSScriptRoot/src", "$PSScriptRoot/src-version/$McVersion" -Recurse -Filter '*.java' |
    ForEach-Object { $_.FullName }
)

$argsFile = Join-Path $Output 'javac.args'
(@('--release', '25', '-proc:none', '-encoding', 'UTF-8', '-Xlint:-options', '-cp', ($deps -join ';'), '-d', $classes) + $sources |
  ForEach-Object { '"' + ($_ -replace '\\', '/') + '"' }) -join "`n" | Set-Content $argsFile -Encoding ASCII

& "$Jdk\javac.exe" "@$argsFile"
if ($LASTEXITCODE) { throw 'server-mod compilation failed' }

Write-FabricMetadata "$PSScriptRoot\fabric.mod.json" $classes
Copy-Item "$PSScriptRoot\pathtracing-server.mixins.json" $classes
$jar = Join-Path $Output 'pathtracing-server.jar'
& "$Jdk\jar.exe" --create --file $jar -C $classes .
if ($LASTEXITCODE) { throw 'server-mod packaging failed' }
Write-Output $jar
