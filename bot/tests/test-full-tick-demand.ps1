# Transform the built jar with Fabric Loader, without initializing Minecraft.
param([switch]$UseRigMods)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\..\..\scripts\common.ps1"
$testBuild = Join-Path $Root 'bot\build\full-tick-demand-test'
$null = New-Item -ItemType Directory -Force $testBuild
$mod = Join-Path $Root 'bot\build\pathtracing.jar'
if (-not (Test-Path -LiteralPath $mod)) { throw 'Build the mod first.' }
$classpath = (Get-GameClasspath) -join ';'
& "$Jdk\javac.exe" --release 25 -proc:none -cp $classpath -d $testBuild "$PSScriptRoot\FullTickDemandTest.java"
if ($LASTEXITCODE) { throw 'Full-demand test compilation failed.' }
$dependencies = if ($UseRigMods) {
  Get-ChildItem -LiteralPath (Join-Path $Rig 'clients\bot\mods') -Filter '*.jar' |
    Where-Object Name -ne 'pathtracing.jar' | ForEach-Object FullName
} else { (Get-ChildItem $PackMods -Filter 'fabric-api-*.jar' | Select-Object -First 1).FullName }
$mods = (@($mod) + @($dependencies)) -join ';'
Push-Location $testBuild
try {
  foreach ($case in @(
    @{ Role = 'bot'; Full = 'false'; Follow = 'true'; Expected = 'false' },
    @{ Role = 'bot'; Full = 'true'; Follow = 'true'; Expected = 'true' },
    @{ Role = 'player'; Full = 'true'; Follow = 'true'; Expected = 'false' },
    @{ Role = 'bot'; Full = 'true'; Follow = 'false'; Expected = 'false' }
  )) {
    $jvm = @("-Dfabric.addMods=$mods", '-Dmixin.debug.export=true', '-Dmixin.debug.export.decompile=false',
      '-Dmixin.debug.export.filter=net.minecraft.client.Minecraft', "-Dpathtracing.role=$($case.Role)",
      "-Dpathtracing.fullTickDemand=$($case.Full)", "-Dpathtracing.followTickRate=$($case.Follow)")
    & "$Jdk\java.exe" @jvm -cp "$testBuild;$mod;$classpath" pathtracing.FullTickDemandTest $testBuild $case.Expected
    if ($LASTEXITCODE) { throw 'Fabric-transformed clock test failed.' }
  }
} finally { Pop-Location }
