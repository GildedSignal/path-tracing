param([string]$Output = 'D:\path-tracing-target\server-mod-build', [string]$McVersion = $env:PATHTRACING_MC_VERSION)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\..\scripts\common.ps1" -McVersion $McVersion

$clientRoot = Join-Path $PSScriptRoot 'client'
$Output = [IO.Path]::GetFullPath($Output)
$classes = Join-Path $Output 'client-classes'
$null = Get-VerifiedClientJar
if (Test-Path -LiteralPath $classes) { Remove-Item -LiteralPath $classes -Recurse -Force }
$null = New-Item -ItemType Directory -Force $classes

$deps = @(Get-GameClasspath) + @(Get-FabricModules "$Output/lib/$McVersion")
$sources = @(
  Get-ChildItem "$clientRoot\src" -Recurse -Filter '*.java' |
    ForEach-Object { $_.FullName }
)

$argsFile = Join-Path $Output 'javac-client.args'
(@('--release', '25', '-proc:none', '-encoding', 'UTF-8', '-Xlint:-options', '-cp', ($deps -join ';'), '-d', $classes) + $sources |
  ForEach-Object { '"' + ($_ -replace '\\', '/') + '"' }) -join "`n" | Set-Content $argsFile -Encoding ASCII

& "$Jdk\javac.exe" "@$argsFile"
if ($LASTEXITCODE) { throw 'client-mod compilation failed' }

Write-FabricMetadata "$clientRoot\fabric.mod.json" $classes
Copy-Item "$clientRoot\pathtracing-swarm-client.mixins.json" $classes
$jar = Join-Path $Output 'pathtracing-swarm-client.jar'
Remove-Item $jar -Force -ErrorAction SilentlyContinue
& "$Jdk\jar.exe" --create --file $jar -C $classes .
if ($LASTEXITCODE) { throw 'client-mod jar packaging failed' }
Write-Output $jar
