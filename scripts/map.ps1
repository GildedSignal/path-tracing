# Serves one map as the server's whole world, so its datapacks, scoreboards, functions, gamerules
# and spawn are the map maker's own. The map's DataVersion picks the rig (versions.json): the
# oldest version that can open it.
#   .\map.ps1 spiral -Source 'D:\path-tracing-maps\hielke\Parkour Spiral.zip'   # import once, then serve
#   .\map.ps1 spiral                                                             # switch to it
#   .\map.ps1 world -McVersion 26.2                                              # back to your world
# The rig's server restarts, and its bot client with it; join again from your client.
param(
  [Parameter(Mandatory)][ValidatePattern('\A[a-z0-9_-]+\z', Options=0)][string]$Name,
  [string]$Source,
  [string]$McVersion
)
$ErrorActionPreference = 'Stop'
$versions = Get-Content (Join-Path $PSScriptRoot 'versions.json') -Raw | ConvertFrom-Json
$rows = $versions.PSObject.Properties | Sort-Object { $_.Value.dataVersion }

function Get-DataVersion([string]$LevelDat) {
  # level.dat is gzip NBT; the first int tag named DataVersion is the world's (Data.DataVersion).
  $in = [IO.Compression.GZipStream]::new([IO.File]::OpenRead($LevelDat), [IO.Compression.CompressionMode]::Decompress)
  $mem = [IO.MemoryStream]::new(); $in.CopyTo($mem); $in.Dispose(); $b = $mem.ToArray()
  $key = [Text.Encoding]::ASCII.GetBytes('DataVersion')
  for ($i = 3; $i -lt $b.Length - 15; $i++) {
    if ($b[$i - 3] -ne 3 -or $b[$i - 2] -ne 0 -or $b[$i - 1] -ne 11) { continue }
    $match = $true; for ($k = 0; $k -lt 11; $k++) { if ($b[$i + $k] -ne $key[$k]) { $match = $false; break } }
    if ($match) { $j = $i + 11; return ([int]$b[$j] -shl 24) -bor ([int]$b[$j + 1] -shl 16) -bor ([int]$b[$j + 2] -shl 8) -bor [int]$b[$j + 3] }
  }
  throw "No DataVersion in $LevelDat"
}

# The import: the map's world folder, unchanged, under a temporary name until it is in place.
$staged = $null
if ($Source) {
  $tmp = Join-Path ([IO.Path]::GetTempPath()) "map-$Name-$PID"
  if (Test-Path -LiteralPath $Source -PathType Container) { Copy-Item -Recurse -LiteralPath $Source $tmp }
  else { Add-Type -AssemblyName System.IO.Compression.FileSystem; [IO.Compression.ZipFile]::ExtractToDirectory((Resolve-Path -LiteralPath $Source).Path, $tmp) }
  $level = Get-ChildItem -LiteralPath $tmp -Recurse -Filter level.dat | Sort-Object { $_.FullName.Length } | Select-Object -First 1
  if (-not $level) { Remove-Item -Recurse -Force $tmp; throw "No level.dat in $Source" }
  $staged = $level.Directory.FullName
  $dataVersion = Get-DataVersion $level.FullName
  if (-not $McVersion) {
    $fit = $rows | Where-Object { $_.Value.dataVersion -ge $dataVersion } | Select-Object -First 1
    if (-not $fit) { throw "The map is DataVersion $dataVersion, newer than every version in versions.json." }
    $McVersion = $fit.Name
  }
}
if (-not $McVersion) {
  # Switching: the rig that already holds this map.
  foreach ($r in $rows) {
    $rig = if ($r.Value.rig) { $r.Value.rig } else { Split-Path -Parent $PSScriptRoot }
    if ($Name -ne 'world' -and (Test-Path (Join-Path $rig "server\maps\$Name\level.dat"))) { $McVersion = $r.Name; break }
  }
  if (-not $McVersion) { throw "No rig holds map '$Name'; import it with -Source, or give -McVersion." }
}
$env:PATHTRACING_MC_VERSION = $McVersion
. "$PSScriptRoot\common.ps1" -McVersion $McVersion
$server = Join-Path $Rig 'server'
$levelName = if ($Name -eq 'world') { 'world' } else { "maps/$Name" }
$world = Join-Path $server $levelName
if ($staged) {
  if (Test-Path -LiteralPath $world) { Remove-Item -Recurse -Force $tmp; throw "Map '$Name' already exists at $world." }
  if ($dataVersion -gt $McSpec.dataVersion) { Remove-Item -Recurse -Force $tmp; throw "The map is DataVersion $dataVersion; Minecraft $McVersion is $($McSpec.dataVersion)." }
  New-Item -ItemType Directory -Force (Split-Path -Parent $world) | Out-Null
  Move-Item -LiteralPath $staged $world
  Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
  Remove-Item -LiteralPath (Join-Path $world 'session.lock') -ErrorAction SilentlyContinue
  Write-Host "Imported $Name (DataVersion $dataVersion) into the $McVersion rig."
}
if (-not (Test-Path (Join-Path $world 'level.dat'))) { throw "No world at $world." }

# Stop this rig's server and bot client, point the server at the map, start both again.
if (Test-Port $ServerHost $ServerPort) {
  $null = & "$PSScriptRoot\rcon.ps1" 'save-all flush' 'stop'
  $deadline = (Get-Date).AddSeconds(60)
  while ((Test-Port $ServerHost $ServerPort) -and (Get-Date) -lt $deadline) { Start-Sleep -Milliseconds 500 }
  if (Test-Port $ServerHost $ServerPort) { throw 'The server is still running after 60 seconds.' }
}
$botArgs = (Join-Path $Rig 'clients\bot\launch.args').Replace('/', '\')
$bots = @(Get-CimInstance Win32_Process -Filter "Name='javaw.exe'" | Where-Object { $_.CommandLine -and $_.CommandLine.Contains($botArgs) })
$bots | ForEach-Object { Stop-Process -Id $_.ProcessId -Force }
$props = Join-Path $server 'server.properties'
(Get-Content $props | ForEach-Object { if ($_ -match '^level-name=') { "level-name=$levelName" } else { $_ } }) | Set-Content $props -Encoding ascii
Start-Process -FilePath "$Jdk\java.exe" -ArgumentList '-Xms2G', '-Xmx4G', '-jar', 'fabric-server-launch.jar', 'nogui' -WorkingDirectory $server -WindowStyle Minimized | Out-Null
$deadline = (Get-Date).AddSeconds(180)
while (-not (Test-Port $ServerHost $ServerPort) -and (Get-Date) -lt $deadline) { Start-Sleep -Milliseconds 500 }
if (-not (Test-Port $ServerHost $ServerPort)) { throw "The server did not open port $ServerPort; see $server\logs\latest.log." }
if ($bots) { $null = & "$PSScriptRoot\launch-client.ps1" -Role bot -McVersion $McVersion }
Write-Host "Serving $Name on Minecraft $McVersion at ${ServerHost}:$ServerPort. Its tracks: $(if ($Name -eq 'world') { 'lab\tracks' } else { "lab\maps\$Name" })."
