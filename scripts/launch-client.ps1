# Starts one Minecraft 26.2 + Fabric client in offline mode, joined straight to the local server.
#   .\launch-client.ps1 -Role player
#   .\launch-client.ps1 -Role bot
param(
  [Parameter(Mandatory)][ValidateSet('player','bot')][string]$Role,
  [switch]$NoJoin
)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\common.ps1"

$name    = if ($Role -eq 'bot') { $BotName } else { $PlayerName }
$gameDir = Join-Path $Rig "clients\$Role"
if (-not (Test-Path "$gameDir\mods")) { throw "Client '$Role' is not set up yet: run scripts\setup.ps1 first." }

$mcMeta = Get-Content "$Meta\net.minecraft\$McVersion.json" -Raw | ConvertFrom-Json
$classpath = Get-GameClasspath

$jvm = @(
  $(if ($Role -eq 'bot') { '-Xmx3G' } else { '-Xmx6G' }), '-Xms1G',
  "-Dpathtracing.role=$Role", "-Dpathtracing.bot=$BotName", "-Dpathtracing.owner=$PlayerName",
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
