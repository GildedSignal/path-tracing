# One-time (re-runnable) setup: local superflat server + player and bot clients with the pack's
# performance mods, your options/keybinds, and the bot mod.
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\common.ps1"

# --- Server ---------------------------------------------------------------------------------------
$server = Join-Path $Root 'server'
New-Item -ItemType Directory -Force $server | Out-Null
$launcherJar = Join-Path $server 'fabric-server-launch.jar'
if (-not (Test-Path $launcherJar)) {
  $installers = Invoke-RestMethod 'https://meta.fabricmc.net/v2/versions/installer'
  $installer = ($installers | Where-Object stable | Select-Object -First 1).version
  Write-Host "Downloading Fabric server launcher ($McVersion, loader $LoaderVersion, installer $installer)"
  Invoke-WebRequest "https://meta.fabricmc.net/v2/versions/loader/$McVersion/$LoaderVersion/$installer/server/jar" -OutFile $launcherJar
}
Remove-Item "$server\mods\*.jar" -ErrorAction SilentlyContinue
Copy-Mods $ServerPerfMods "$server\mods"

if (-not (Test-Path "$server\server.properties")) {
@"
motd=Path tracing lab
server-ip=$ServerHost
server-port=$ServerPort
online-mode=false
enforce-secure-profile=false
level-name=world
level-type=minecraft\:flat
generate-structures=false
gamemode=creative
difficulty=peaceful
spawn-protection=0
allow-flight=true
max-players=4
view-distance=12
simulation-distance=10
pause-when-empty-seconds=-1
"@ | Set-Content "$server\server.properties" -Encoding ascii
}
# RCON on localhost lets scripts\rcon.ps1 (and you) drive the server from scripts: test courses, teleports...
$props = Get-Content "$server\server.properties"
if (-not ($props -match '^enable-rcon=true')) {
  $password = -join ((48..57) + (97..122) | Get-Random -Count 24 | ForEach-Object { [char]$_ })
  $props = $props | Where-Object { $_ -notmatch '^(enable-rcon|rcon\.port|rcon\.password|broadcast-rcon-to-ops)=' }
  $props + @('enable-rcon=true', "rcon.port=$RconPort", "rcon.password=$password", 'broadcast-rcon-to-ops=false') |
    Set-Content "$server\server.properties" -Encoding ascii
}
$ops = @($PlayerName, $BotName) | ForEach-Object {
  [ordered]@{ uuid = (Get-OfflineUuid $_); name = $_; level = 4; bypassesPlayerLimit = $false }
}
ConvertTo-Json @($ops) | Set-Content "$server\ops.json" -Encoding ascii

# Permanent noon, clear sky, no mobs. Run once, headless; the commands must wait for the world to load,
# because the console binds each command to a level at the moment it reads the line.
if ((Test-Path "$server\eula.txt") -and -not (Test-Path "$server\world-initialized")) {
  Write-Host 'Initialising the world (noon, clear weather, no mob spawning)'
  $psi = [Diagnostics.ProcessStartInfo]::new("$Jdk\java.exe", '-Xmx2G -jar fabric-server-launch.jar nogui')
  $psi.WorkingDirectory = $server; $psi.UseShellExecute = $false; $psi.CreateNoWindow = $true
  $psi.RedirectStandardInput = $true; $psi.RedirectStandardOutput = $true
  $proc = [Diagnostics.Process]::Start($psi)
  while (($line = $proc.StandardOutput.ReadLine()) -ne $null -and $line -notmatch 'Done \(') { }
  $proc.StandardInput.WriteLine('')  # absorbs the byte-order mark .NET writes first
  foreach ($c in 'time set noon', 'weather clear', 'gamerule advance_time false',
                 'gamerule advance_weather false', 'gamerule spawn_mobs false', 'stop') {
    $proc.StandardInput.WriteLine($c)
  }
  $proc.StandardOutput.ReadToEnd() -split "`r?`n" | Select-String 'Gamerule|Set the|time marker|Incorrect' | ForEach-Object { "  $_" }
  $proc.WaitForExit()
  New-Item "$server\world-initialized" -ItemType File | Out-Null
} elseif (-not (Test-Path "$server\eula.txt")) {
  Write-Warning "Accept Mojang's EULA (https://aka.ms/MinecraftEULA) by writing eula=true to $server\eula.txt, then rerun setup."
}

# --- Clients --------------------------------------------------------------------------------------
foreach ($role in 'player', 'bot') {
  $dir = Join-Path $Root "clients\$role"
  New-Item -ItemType Directory -Force "$dir\config" | Out-Null
  Remove-Item "$dir\mods\*.jar" -ErrorAction SilentlyContinue
  Copy-Mods $ClientPerfMods "$dir\mods"
  if ($role -eq 'player') { Copy-Mods $PlayerOnlyMods "$dir\mods" }
  foreach ($c in $PerfConfigs) {
    if (Test-Path "$PackConfig\$c") { Copy-Item "$PackConfig\$c" "$dir\config\$c" -Force }
  }

  # Your options and keybinds; the bot gets a lighter, always-windowed variant for the second monitor.
  $set = @{
    tutorialStep = 'none'; skipMultiplayerWarning = 'true'; onboardAccessibility = 'false'
    resourcePacks = '["vanilla"]'; incompatibleResourcePacks = '[]'
  }
  if ($role -eq 'bot') {
    $set += @{
      fullscreen = 'false'; renderDistance = '10'; simulationDistance = '8'; maxFps = '60'
      pauseOnLostFocus = 'false'; inactivityFpsLimit = '"minimized"'; soundCategory_master = '0.0'
    }
  }
  [string[]]$options = Get-Content $PackOptions -Encoding UTF8 | ForEach-Object {
    $k = $_.Split(':')[0]
    if ($set.ContainsKey($k)) { "${k}:$($set[$k])" } else { $_ }
  }
  # Keep your own later tweaks in the player client: only seed options.txt the first time.
  if ($role -eq 'bot' -or -not (Test-Path "$dir\options.txt")) {
    [IO.File]::WriteAllLines("$dir\options.txt", $options, [Text.UTF8Encoding]::new($false))
  }
}

# --- Bot mod --------------------------------------------------------------------------------------
& "$Root\bot\build.ps1"
foreach ($role in 'player', 'bot') { Copy-Item "$Root\bot\build\pathtracing.jar" "$Root\clients\$role\mods\" -Force }

Write-Host "Setup done. Start everything with: .\play.ps1"
