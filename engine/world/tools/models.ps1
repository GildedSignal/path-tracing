param([string]$Dependencies = 'D:/path-tracing-deps/data', [string]$Build = 'D:/path-tracing-target/world-models')
$ErrorActionPreference = 'Stop'
$jar = Join-Path $Dependencies 'libraries/com/mojang/minecraft/26.2/minecraft-26.2-client.jar'
if ((Get-FileHash $jar -Algorithm SHA256).Hash -ne '40896EE9F1E2BEC3C934DAAC7E93D41E9E3D9C2F8AE0CA366D52FFBFD1AFA290') { throw 'Expected the staged vanilla 26.2 client jar' }
$java = Join-Path $Dependencies 'java/java-runtime-epsilon/bin'
$libs = Get-ChildItem (Join-Path $Dependencies 'libraries') -Recurse -File -Filter '*.jar' | Where-Object { $_.FullName -notmatch 'minecraft-26.2-client.jar' }
New-Item -ItemType Directory -Force $Build | Out-Null
$classpath = (@($Build,$jar) + $libs.FullName) -join ';'
& "$java/javac.exe" -cp $classpath -d $Build "$PSScriptRoot/ModelTable.java"
if ($LASTEXITCODE -ne 0) { throw 'ModelTable compilation failed' }
# Minecraft bootstrap may create logs; keep these in the build directory.
Push-Location $Build
try {
    & "$java/java.exe" -cp $classpath ModelTable "$PSScriptRoot/../data"
    if ($LASTEXITCODE -ne 0) { throw 'ModelTable generation failed' }
} finally { Pop-Location }
Get-FileHash $jar -Algorithm SHA256

