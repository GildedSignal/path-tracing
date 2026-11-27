# Shared paths and helpers for the staged jars or the launcher's existing files.
param([string]$McVersion = $env:PATHTRACING_MC_VERSION, [string]$Dependencies = $env:PATHTRACING_DEPS)

$Root        = Split-Path -Parent $PSScriptRoot
# PATHTRACING_DEPS points sandboxed GPT jobs at a staged copy of what builds need (scripts\stage-deps.ps1).
$PrismData   = if ($Dependencies) { $Dependencies } else { Join-Path $env:LOCALAPPDATA 'ImmersiveMinecraft\data' }
$Jdk         = Join-Path $PrismData 'java\java-runtime-epsilon\bin'
$Libraries   = Join-Path $PrismData 'libraries'
$Assets      = Join-Path $PrismData 'assets'
$Meta        = Join-Path $PrismData 'meta'
$PackMods    = Join-Path $PrismData 'instances\immersive-minecraft\.minecraft\mods'
$PackConfig  = Join-Path $PrismData 'instances\immersive-minecraft\.minecraft\config'
$PackOptions = Join-Path $PrismData 'instances\immersive-minecraft\.minecraft\options.txt'

if (-not $McVersion) { $McVersion = '26.2' }
$versions = Get-Content (Join-Path $Root 'scripts/versions.json') -Raw | ConvertFrom-Json
$McSpec = $versions.$McVersion
if (-not $McSpec) { throw "Unsupported Minecraft version '$McVersion'." }
# The rig (server\, clients\, lab\tracks\, maps.json and the native DLL under engine\target\release\;
# not in git): one per version, named in versions.json (none means this checkout). PATHTRACING_ROOT
# overrides it, so a builder's worktree uses a rig while building its own code from $Root.
$Rig         = if ($env:PATHTRACING_ROOT) { $env:PATHTRACING_ROOT } elseif ($McSpec.rig) { $McSpec.rig } else { $Root }
$LoaderVersion = $McSpec.fabricLoader
$StagedDeps = if ($Dependencies) { Split-Path $PrismData } else { 'D:\path-tracing-deps' }
$McStage = Join-Path $StagedDeps "versions/$McVersion"
$MojangMetadata = Join-Path $McStage "$McVersion.json"
$ClientJar = Join-Path $Libraries "com/mojang/minecraft/$McVersion/$($McSpec.clientJar)"
if (-not (Test-Path -LiteralPath $ClientJar)) { $ClientJar = Join-Path $StagedDeps $McSpec.clientJar }
$VanillaServerJar = Join-Path $Rig "server/.fabric/server/$McVersion-server.jar"
if (-not (Test-Path -LiteralPath $VanillaServerJar)) { $VanillaServerJar = Join-Path $StagedDeps $McSpec.serverJar }

function Get-VerifiedClientJar {
  if ((Get-FileHash -LiteralPath $ClientJar -Algorithm SHA256).Hash -ne $McSpec.clientSha256) {
    throw "Unexpected SHA-256 for Minecraft $McVersion at $ClientJar"
  }
  $ClientJar
}

function Get-ServerBundleClasspath([string]$Output) {
  $bundle = Join-Path $StagedDeps $McSpec.serverJar
  if (-not (Test-Path -LiteralPath $bundle)) { $bundle = $VanillaServerJar }
  Add-Type -AssemblyName System.IO.Compression.FileSystem
  $zip = [IO.Compression.ZipFile]::OpenRead($bundle)
  try {
    foreach ($entry in $zip.Entries) {
      if ($entry.FullName -notmatch '^META-INF/(libraries/.+\.jar|versions/[^/]+/[^/]+\.jar)$') { continue }
      $path = Join-Path $Output $entry.FullName.Substring('META-INF/'.Length)
      $null = New-Item -ItemType Directory -Force (Split-Path $path)
      [IO.Compression.ZipFileExtensions]::ExtractToFile($entry, $path, $true)
      $path
    }
  } finally { $zip.Dispose() }
}

function Get-FabricModules([string]$Output) {
  $api = Join-Path $McStage $McSpec.fabricApiJar
  if (-not (Test-Path -LiteralPath $api)) { $api = Join-Path $PackMods $McSpec.fabricApiJar }
  Add-Type -AssemblyName System.IO.Compression.FileSystem
  $null = New-Item -ItemType Directory -Force $Output
  $zip = [IO.Compression.ZipFile]::OpenRead($api)
  try {
    foreach ($entry in $zip.Entries | Where-Object FullName -like 'META-INF/jars/*.jar') {
      $path = Join-Path $Output $entry.Name
      [IO.Compression.ZipFileExtensions]::ExtractToFile($entry, $path, $true)
      $path
    }
  } finally { $zip.Dispose() }
}

function Get-FabricLoaderClasspath {
  $loader = Get-Content (Join-Path $Meta "net.fabricmc.fabric-loader/$LoaderVersion.json") -Raw | ConvertFrom-Json
  foreach ($lib in $loader.libraries) {
    $path = Get-LibraryPath $lib
    if (-not (Test-Path -LiteralPath $path)) { throw "Fabric Loader dependency not found at $path" }
    $path
  }
}

function Write-FabricMetadata([string]$Source, [string]$Classes) {
  $metadata = Get-Content -LiteralPath $Source -Raw | ConvertFrom-Json
  $metadata.depends.minecraft = $McVersion
  $metadata.depends.fabricloader = ">=$LoaderVersion"
  [IO.File]::WriteAllText((Join-Path $Classes 'fabric.mod.json'), ($metadata | ConvertTo-Json -Depth 10))
}

$ServerHost  = '127.0.0.1'
$ServerPort  = if ($env:PATHTRACING_SERVER_PORT) { [int]$env:PATHTRACING_SERVER_PORT } elseif ($McSpec.port) { [int]$McSpec.port } else { 25565 }
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
# which are the judge of the law, run Mojang's code without them (map/detail/engine.md).
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
  # The pack's jars are built for the version the pack runs (versions.json `packMods`). Any other
  # version gets the staged Fabric API alone: performance mods change no game logic, so none is needed.
  if ($McSpec.packMods -eq $false) {
    if ($Patterns -contains 'fabric-api-*') { Copy-Item (Join-Path $McStage $McSpec.fabricApiJar) $Destination -Force }
    return
  }
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

# Fabric Loader and Minecraft, resolved from Mojang or the launcher's metadata.
function Get-GameClasspath {
  if (Test-Path -LiteralPath $MojangMetadata) {
    Get-FabricLoaderClasspath
    $official = Get-Content -LiteralPath $MojangMetadata -Raw | ConvertFrom-Json
    foreach ($lib in $official.libraries) {
      if (-not (Test-Rules $lib) -or !$lib.downloads.artifact -or $lib.name -match 'natives-windows-(x86|arm64)') { continue }
      $path = Join-Path $Libraries $lib.downloads.artifact.path
      if (-not (Test-Path -LiteralPath $path)) { $path = Join-Path "$McStage/libraries" $lib.downloads.artifact.path }
      if (Test-Path -LiteralPath $path) { $path } else { Write-Warning "missing library $($lib.name)" }
    }
    $ClientJar
    return
  }
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
  $classpath.Add($ClientJar)
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

# A provider's API key (zai for GLM, mistral for Mistral), stored by scripts\set-key.ps1 (DPAPI, this user only).
# Returns $null when none is stored. Callers put it only into a child process's environment, and scrub it from
# anything they publish.
function Get-Key([ValidateSet('zai','mistral')][string]$Provider = 'zai') {
  $file = Join-Path $env:LOCALAPPDATA "path-tracing\$Provider-key.xml"
  if (-not (Test-Path $file)) { return $null }
  (Import-Clixml $file).GetNetworkCredential().Password
}
