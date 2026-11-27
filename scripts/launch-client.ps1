# Starts the selected Minecraft + Fabric client in offline mode on the local rig.
#   .\launch-client.ps1 -Role player
#   .\launch-client.ps1 -Role bot
param(
  [Parameter(Mandatory)][ValidateSet('player','bot')][string]$Role,
  [switch]$NoJoin,
  # The clock gate launches a second bot process with this property disabled.
  [ValidateSet('true','false')][string]$FollowTickRate = 'true',
  [ValidateSet('true','false')][string]$FullTickDemand = 'false',
  [ValidateSet('true','false')][string]$ClockAudit = 'false',
  [string]$McVersion = $env:PATHTRACING_MC_VERSION
)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\common.ps1" -McVersion $McVersion

$name    = if ($Role -eq 'bot') { $BotName } else { $PlayerName }
$gameDir = Join-Path $Rig "clients\$Role"
if (-not (Test-Path "$gameDir\mods")) { throw "Client '$Role' is not set up yet: run scripts\setup.ps1 first." }

$versionMetadata = if (Test-Path -LiteralPath $MojangMetadata) { $MojangMetadata } else { "$Meta\net.minecraft\$McVersion.json" }
$mcMeta = Get-Content -LiteralPath $versionMetadata -Raw | ConvertFrom-Json
$classpath = Get-GameClasspath

$jvm = @(
  '--enable-native-access=ALL-UNNAMED',
  $(if ($Role -eq 'bot') { '-Xmx3G' } else { '-Xmx6G' }), '-Xms1G',
  "-Dpathtracing.role=$Role", "-Dpathtracing.bot=$BotName", "-Dpathtracing.owner=$PlayerName",
  "-Dpathtracing.rig=$Rig"
)
if ($Role -eq 'bot') {
  $jvm += "-Dpathtracing.followTickRate=$FollowTickRate"
  if ($FullTickDemand -eq 'true') { $jvm += '-Dpathtracing.fullTickDemand=true' }
  if ($ClockAudit -eq 'true') {
    $jvm += @('-Dpathtracing.clockAudit=true', "-XX:FlightRecorderOptions=repository=$gameDir")
  }
}
$jvm += @(
  '-cp', ($classpath -join ';'),
  'net.fabricmc.loader.impl.launch.knot.KnotClient'
)
$game = @(
  '--username', $name, '--uuid', (Get-OfflineUuid $name), '--accessToken', '0', '--userType', 'legacy',
  '--version', "fabric-$McVersion", '--versionType', 'release',
  '--gameDir', $gameDir, '--assetsDir', $Assets, '--assetIndex', $mcMeta.assetIndex.id
)
if (-not $NoJoin) { $game += @('--quickPlayMultiplayer', "${ServerHost}:$ServerPort") }

# Argument file: the classpath is far beyond the Windows command-line limit.
$argFile = Join-Path $gameDir 'launch.args'
($jvm + $game | ForEach-Object { '"' + ($_ -replace '\\', '/') + '"' }) -join "`n" | Set-Content $argFile -Encoding ascii
Start-Process -FilePath "$Jdk\javaw.exe" -ArgumentList "@`"$argFile`"" -WorkingDirectory $gameDir -PassThru
