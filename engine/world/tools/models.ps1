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
    # One state per family the 2.1b rule admitted, for the bit-exact shape test.
    $states = @('minecraft:oak_fence[east=true,north=false,south=false,waterlogged=false,west=true]',
        'minecraft:cobblestone_wall[east=low,north=none,south=tall,up=true,waterlogged=false,west=none]',
        'minecraft:oak_fence_gate[facing=north,in_wall=false,open=true,powered=false]',
        'minecraft:oak_fence_gate[facing=east,in_wall=false,open=false,powered=false]',
        'minecraft:oak_door[facing=east,half=lower,hinge=left,open=false,powered=false]',
        'minecraft:oak_door[facing=north,half=upper,hinge=right,open=true,powered=false]',
        'minecraft:white_carpet', 'minecraft:mud', 'minecraft:stone_pressure_plate[powered=true]',
        'minecraft:tripwire[attached=true,disarmed=false,east=true,north=false,powered=true,south=false,west=true]')
    & "$java/java.exe" -cp $classpath ModelTable --shapes "$PSScriptRoot/../tests/fixtures/shapes.txt" @states
    if ($LASTEXITCODE -ne 0) { throw 'ModelTable shapes failed' }
} finally { Pop-Location }
Get-FileHash $jar -Algorithm SHA256

