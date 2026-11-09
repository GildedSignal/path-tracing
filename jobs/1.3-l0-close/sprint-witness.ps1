param([string]$Output = 'D:/path-tracing-target/l0-close-sprint-witness.txt')
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path "$PSScriptRoot/../..").Path
$deps = 'D:/path-tracing-deps/data'
$jar = "$deps/libraries/com/mojang/minecraft/26.2/minecraft-26.2-client.jar"
$trace = 'D:/path-tracing-target/sprint-diagnostic-source-20261107T231317/lab/gates/results/20261107T2213532561408Z-fixtures/bot-latest.log'
$lines = @(Get-Content -LiteralPath $trace)
$start = @(0..($lines.Count-1) | Where-Object { $lines[$_] -match 'recorder-start t=204 ' })
if ($start.Count -ne 1) { throw 'Expected one tick-204 diagnostic boundary.' }
$witness = $lines[($start[0]-2)..($start[0]+2)]
if (($witness -join "`n") -notmatch 'entity-data-packet-return playerTick=2583 sprint=false modifier=true speedBits=3e051eb9' -or
    ($witness -join "`n") -notmatch 'travel playerTick=2584 sprint=false modifier=true speedBits=3e051eb9') {
  throw 'The recorded independent sprint-state witness changed.'
}
$report = @('26.2 sprint-state witness', (Get-FileHash -LiteralPath $jar).Hash, $witness)
foreach ($spec in @(
    @('net.minecraft.client.multiplayer.ClientPacketListener','public void handleSetEntityData',39),
    @('net.minecraft.world.entity.Entity','public boolean isSprinting',6),
    @('net.minecraft.world.entity.LivingEntity','public void setSprinting',30))) {
  $code = @(& "$deps/java/java-runtime-epsilon/bin/javap.exe" -cp $jar -c -p $spec[0])
  if ($LASTEXITCODE) { throw "javap failed: $($spec[0])" }
  $report += @($code | Select-String -SimpleMatch $spec[1] -Context 0,([int]$spec[2]) | ForEach-Object { $_.ToString() })
}
$report | Set-Content -LiteralPath $Output -Encoding UTF8
$report
