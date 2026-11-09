# Shared paths and helpers. Everything reuses the files your Immersive Minecraft launcher already
# downloaded (Java 25, Minecraft 26.2, Fabric Loader 0.19.5, assets); nothing in that launcher is changed.

$Root        = Split-Path -Parent $PSScriptRoot
# The rig (server\ and clients\, not in git). PATHTRACING_ROOT lets a builder's worktree use the one rig
# while building its own code from $Root.
$Rig         = if ($env:PATHTRACING_ROOT) { $env:PATHTRACING_ROOT } else { $Root }
# PATHTRACING_DEPS points sandboxed GPT jobs at a staged copy of what builds need (scripts\stage-deps.ps1).
$PrismData   = if ($env:PATHTRACING_DEPS) { $env:PATHTRACING_DEPS } else { Join-Path $env:LOCALAPPDATA 'ImmersiveMinecraft\data' }
$Jdk         = Join-Path $PrismData 'java\java-runtime-epsilon\bin'
$Libraries   = Join-Path $PrismData 'libraries'
$Assets      = Join-Path $PrismData 'assets'
$Meta        = Join-Path $PrismData 'meta'
$PackMods    = Join-Path $PrismData 'instances\immersive-minecraft\.minecraft\mods'
$PackConfig  = Join-Path $PrismData 'instances\immersive-minecraft\.minecraft\config'
$PackOptions = Join-Path $PrismData 'instances\immersive-minecraft\.minecraft\options.txt'

$McVersion     = '26.2'
$LoaderVersion = '0.19.5'

$ServerHost  = '127.0.0.1'
$ServerPort  = if ($env:PATHTRACING_SERVER_PORT) { [int]$env:PATHTRACING_SERVER_PORT } else { 25565 }
if ($ServerPort -lt 1 -or $ServerPort -gt 65525) { throw 'PATHTRACING_SERVER_PORT must leave room for RCON at port + 10.' }
$RconPort    = $ServerPort + 10
$PlayerName  = 'Brain_Forge'
$BotName     = 'PathBot'

# Performance mods copied from the Immersive pack (exact jars you already run). Both clients get these;
# none of them changes game logic.
$ClientPerfMods = @(
  'fabric-api-*', 'sodium-fabric-*', 'sodium-extra-fabric-*', 'reeses-sodium-options-*',
  'ferritecore-*', 'ImmediatelyFast-*', 'entityculling-*',
  'moreculling-*', 'cloth-config-*', 'BadOptimizations-*', 'krypton-*', 'ScalableLux-*',
  'fast-ip-ping-*', 'rsls-*', 'modmenu-*'
)
# Your client only. Dynamic FPS throttles unfocused windows, which is exactly what the bot window always is.
# Lithium and C2ME rewrite game logic (Lithium patches collisions), so the bot's client and the server,
# which are the judge of the law, run Mojang's code without them (docs/engine.md).
$PlayerOnlyMods = @('dynamic-fps-*', 'lithium-fabric-*', 'c2me-fabric-*')
$ServerPerfMods = @('ferritecore-*', 'krypton-*', 'ScalableLux-*')
$PerfConfigs = @(
  'sodium-options.json', 'sodium-extra-options.json', 'lithium.properties', 'c2me.toml',
  'immediatelyfast.json', 'entityculling.json', 'moreculling.toml', 'badoptimizations.txt',
  'rsls.properties', 'dynamic_fps.json'
)

# Same UUID the vanilla server assigns an offline-mode player: UUID.nameUUIDFromBytes("OfflinePlayer:" + name).
function Get-OfflineUuid([string]$Name) {
  $md5 = [Security.Cryptography.MD5]::Create().ComputeHash([Text.Encoding]::UTF8.GetBytes("OfflinePlayer:$Name"))
  $md5[6] = ($md5[6] -band 0x0f) -bor 0x30
  $md5[8] = ($md5[8] -band 0x3f) -bor 0x80
  $hex = -join ($md5 | ForEach-Object { $_.ToString('x2') })
  '{0}-{1}-{2}-{3}-{4}' -f $hex.Substring(0,8), $hex.Substring(8,4), $hex.Substring(12,4), $hex.Substring(16,4), $hex.Substring(20,12)
}

function Copy-Mods([string[]]$Patterns, [string]$Destination) {
  New-Item -ItemType Directory -Force $Destination | Out-Null
  foreach ($p in $Patterns) {
    $jar = Get-ChildItem $PackMods -Filter "$p.jar" | Select-Object -First 1
    if (-not $jar) { throw "Mod '$p' not found in $PackMods" }
    Copy-Item $jar.FullName $Destination -Force
  }
}

function Test-Rules($lib) {
  if (-not $lib.rules) { return $true }
  $allowed = $false
  foreach ($r in $lib.rules) {
    if (-not $r.os -or $r.os.name -eq 'windows') { $allowed = $r.action -eq 'allow' }
  }
  $allowed
}

# The launcher stores every library by its Maven coordinate (natives included, e.g. lwjgl-natives-windows).
function Get-LibraryPath($lib) {
  $g, $a, $v, $classifier = $lib.name.Split(':')
  $file = if ($classifier) { "$a-$v-$classifier.jar" } else { "$a-$v.jar" }
  Join-Path $Libraries ("{0}\{1}\{2}\{3}" -f $g.Replace('.', '\'), $a, $v, $file)
}

# Fabric Loader + Minecraft 26.2 + LWJGL, resolved from the launcher's metadata, game jar last.
function Get-GameClasspath {
  $classpath = [Collections.Generic.List[string]]::new()
  $mcMeta = Get-Content "$Meta\net.minecraft\$McVersion.json" -Raw | ConvertFrom-Json
  $metas = @(
    (Get-Content "$Meta\net.fabricmc.fabric-loader\$LoaderVersion.json" -Raw | ConvertFrom-Json),
    (Get-Content "$Meta\net.fabricmc.intermediary\$McVersion.json" -Raw | ConvertFrom-Json),
    (Get-Content "$Meta\org.lwjgl3\$($mcMeta.requires[0].suggests).json" -Raw | ConvertFrom-Json),
    $mcMeta
  )
  foreach ($m in $metas) {
    foreach ($lib in $m.libraries) {
      if (-not (Test-Rules $lib) -or $lib.name -match 'natives-windows-x86') { continue }
      $path = Get-LibraryPath $lib
      if (-not (Test-Path -LiteralPath $path)) { Write-Warning "missing library $($lib.name)"; continue }
      if (-not $classpath.Contains($path)) { $classpath.Add($path) }
    }
  }
  $classpath.Add((Join-Path $Libraries "com\mojang\minecraft\$McVersion\minecraft-$McVersion-client.jar"))
  $classpath.ToArray()
}

# One job drives the rig at a time (gates, recordings). The lock names its holder; a stale lock is
# reclaimed only when its process is gone.
function Enter-Rig([string]$Holder) {
  $lock = Join-Path $Rig 'rig.lock'
  if (Test-Path $lock) {
    $owner = Get-Content $lock -Raw | ConvertFrom-Json
    if (Get-Process -Id $owner.pid -ErrorAction SilentlyContinue) { throw "Rig is held by $($owner.holder) (pid $($owner.pid))" }
  }
  @{ holder = $Holder; pid = $PID; since = (Get-Date).ToString('s') } | ConvertTo-Json | Set-Content $lock -Encoding ASCII
}
function Exit-Rig { Remove-Item (Join-Path $Rig 'rig.lock') -ErrorAction SilentlyContinue }

function Test-Port([string]$HostName, [int]$Port) {
  $c = New-Object Net.Sockets.TcpClient
  try { $c.ConnectAsync($HostName, $Port).Wait(300) -and $c.Connected } catch { $false } finally { $c.Dispose() }
}

# The Z.ai key for GLM, stored by scripts\set-glm-key.ps1 (DPAPI, this user only). Returns $null when none is stored.
# Callers put it only into a child process's environment, and scrub it from anything they publish.
function Get-GlmKey {
  $file = Join-Path $env:LOCALAPPDATA 'path-tracing\zai-key.xml'
  if (-not (Test-Path $file)) { return $null }
  (Import-Clixml $file).GetNetworkCredential().Password
}
