param(
    [string]$Dependencies = $env:PATHTRACING_DEPS,
    [string]$Build = 'D:/path-tracing-target/world-models',
    [string]$McVersion = $env:PATHTRACING_MC_VERSION,
    [string]$Output,
    [string]$Shapes
)
$ErrorActionPreference = 'Stop'
if (-not $Dependencies) { $Dependencies = 'D:/path-tracing-deps/data' }
. "$PSScriptRoot/../../../scripts/common.ps1" -McVersion $McVersion -Dependencies $Dependencies
$jar = Get-VerifiedClientJar
$Build = Join-Path ([IO.Path]::GetFullPath($Build)) $McVersion
if (-not $Output) {
    $Output = Join-Path $PSScriptRoot '../data'
    if ([int](Get-Content -LiteralPath (Join-Path $Output 'version.txt')) -ne $McSpec.dataVersion) {
        $Output = Join-Path $Output $McVersion
    }
}
$Output = [IO.Path]::GetFullPath($Output)
if (-not $Shapes) { $Shapes = Join-Path $Build 'shapes.txt' }
$Shapes = [IO.Path]::GetFullPath($Shapes)
$null = New-Item -ItemType Directory -Force $Build, $Output
$libs = @(Get-ServerBundleClasspath "$Build/vanilla" | Where-Object { $_ -like '*/libraries/*' -or $_ -like '*\libraries\*' })
$classpath = (@($Build,$jar) + $libs) -join ';'
& "$Jdk/javac.exe" -cp $classpath -d $Build "$PSScriptRoot/ModelTable.java" "$PSScriptRoot/UseEffectsTable.java" "$PSScriptRoot/$McVersion/JarRules.java"
if ($LASTEXITCODE -ne 0) { throw 'ModelTable compilation failed' }
# Minecraft bootstrap may create logs; keep these in the build directory.
Push-Location $Build
try {
    & "$Jdk/java.exe" -cp $classpath ModelTable $Output
    if ($LASTEXITCODE -ne 0) { throw 'ModelTable generation failed' }
    if ([int](Get-Content -LiteralPath (Join-Path $Output 'version.txt')) -ne $McSpec.dataVersion) { throw "ModelTable generated the wrong DataVersion for $McVersion" }
    # Every block's states in registry order: a cell's StateId is the jar's own state id.
    & "$Jdk/java.exe" -cp $classpath ModelTable --states "$Output/states.tsv"
    if ($LASTEXITCODE -ne 0) { throw 'ModelTable states failed' }
    & "$Jdk/java.exe" -cp $classpath UseEffectsTable "$Output/use-effects.tsv"
    if ($LASTEXITCODE -ne 0) { throw 'UseEffectsTable generation failed' }
    # One state per family the 2.1b rule admitted, for the bit-exact shape test.
    $states = @('minecraft:oak_fence[east=true,north=false,south=false,waterlogged=false,west=true]',
        'minecraft:cobblestone_wall[east=low,north=none,south=tall,up=true,waterlogged=false,west=none]',
        'minecraft:oak_fence_gate[facing=north,in_wall=false,open=true,powered=false]',
        'minecraft:oak_fence_gate[facing=east,in_wall=false,open=false,powered=false]',
        'minecraft:oak_door[facing=east,half=lower,hinge=left,open=false,powered=false]',
        'minecraft:oak_door[facing=north,half=upper,hinge=right,open=true,powered=false]',
        'minecraft:white_carpet', 'minecraft:mud', 'minecraft:stone_pressure_plate[powered=true]',
        'minecraft:tripwire[attached=true,disarmed=false,east=true,north=false,powered=true,south=false,west=true]')
    & "$Jdk/java.exe" -cp $classpath ModelTable --shapes $Shapes @states
    if ($LASTEXITCODE -ne 0) { throw 'ModelTable shapes failed' }
} finally { Pop-Location }
Get-FileHash $jar -Algorithm SHA256

