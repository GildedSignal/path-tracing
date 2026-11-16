[CmdletBinding()]
param(
  [Parameter(Mandatory)][string]$Source,
  [Parameter(Mandatory)][ValidatePattern('\A[a-z0-9_-]+\z', Options=0)][string]$Name,
  [string]$TestRig,
  [string]$TestRoot,
  [string]$ServerJar
)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\common.ps1"
if (-not $ServerJar) { $ServerJar = $VanillaServerJar }
if ($Name -ceq 'overworld') { throw 'The name overworld is reserved.' }
$sourcePath = (Resolve-Path -LiteralPath $Source).Path
$test = !!$TestRig
if ($test) {
  if (-not $TestRoot) { $TestRoot = Join-Path $Root 'bot\build\map-tests' }
  $allowedTestRoot = [IO.Path]::GetFullPath($TestRoot).TrimEnd('\','/') + [IO.Path]::DirectorySeparatorChar
  $Rig = [IO.Path]::GetFullPath($TestRig)
  if (-not $Rig.StartsWith($allowedTestRoot, [StringComparison]::OrdinalIgnoreCase)) { throw "TestRig must be inside TestRoot: $allowedTestRoot" }
  Write-Host "Would stop the server; skipped for TestRig $Rig. Importing only into this temporary rig."
} else {
  if (Test-Port $ServerHost $ServerPort) {
    if (-not (Test-Port $ServerHost $RconPort)) { throw 'Server is running but RCON is unavailable; cannot stop safely.' }
    $null = & "$PSScriptRoot\rcon.ps1" 'save-all flush' 'stop'
    $deadline = (Get-Date).AddSeconds(60)
    while ((Test-Port $ServerHost $ServerPort) -and (Get-Date) -lt $deadline) { Start-Sleep 1 }
    if (Test-Port $ServerHost $ServerPort) { throw 'Server still running after 60 seconds.' }
  }
}
$world = Join-Path $Rig 'server\world'
$target = Join-Path $world "dimensions\pathtracing\$Name"
$pack = Join-Path $world 'datapacks\pathtracing-maps'
$dimension = Join-Path $pack "data\pathtracing\dimension\$Name.json"
$registry = Join-Path $Rig 'maps.json'
$registryLock = [IO.File]::Open("$registry.lock", [IO.FileMode]::OpenOrCreate, [IO.FileAccess]::ReadWrite, [IO.FileShare]::None)
try {
$maps = [Collections.Generic.List[object]]::new()
if (Test-Path -LiteralPath $registry) {
  foreach ($map in (Get-Content -LiteralPath $registry -Raw | ConvertFrom-Json)) { $maps.Add($map) }
}
if (($maps | Where-Object { $_.name -ceq $Name }) -or (Test-Path -LiteralPath $target) -or (Test-Path -LiteralPath $dimension)) { throw "Map already exists: $Name" }
Add-Type -AssemblyName System.IO.Compression.FileSystem
$zip = [IO.Compression.ZipFile]::OpenRead($ServerJar)
try {
  $entry = $zip.GetEntry('version.json')
  if (-not $entry) { throw 'Server jar has no version.json.' }
  $reader = [IO.StreamReader]::new($entry.Open())
  try { $version = $reader.ReadToEnd() | ConvertFrom-Json } finally { $reader.Dispose() }
} finally { $zip.Dispose() }
if ($version.id -ne '26.2') { throw 'Server jar is not 26.2.' }
$temp = Join-Path ([IO.Path]::GetTempPath()) ('pathtracing-map-' + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $temp | Out-Null
try {
  if ((Get-Item -LiteralPath $sourcePath) -is [IO.FileInfo]) {
    if ([IO.Path]::GetExtension($sourcePath) -ine '.zip') { throw 'Source file must be a zip.' }
    [IO.Compression.ZipFile]::ExtractToDirectory($sourcePath, $temp)
    $levels = @(Get-ChildItem -LiteralPath $temp -Filter level.dat -Recurse -File)
    if ($levels.Count -ne 1) { throw 'Zip must contain exactly one level.dat.' }
    $sourceWorld = $levels[0].DirectoryName
  } else { $sourceWorld = $sourcePath }
  $level = Join-Path $sourceWorld 'level.dat'
  $sourceDimension = Join-Path $sourceWorld 'dimensions\minecraft\overworld'
  if (-not (Test-Path -LiteralPath (Join-Path $sourceDimension 'region') -PathType Container)) { $sourceDimension = $sourceWorld }
  if (-not (Test-Path -LiteralPath (Join-Path $sourceDimension 'region') -PathType Container)) { throw 'Source has no overworld region folder.' }
  $classpath = @((Join-Path $Root 'bot\build\pathtracing.jar')) + @(Get-GameClasspath)
  if (-not (Test-Path -LiteralPath $classpath[0])) { throw 'Build the bot first with bot\build.ps1.' }
  $spawnText = & "$Jdk\java.exe" -cp ($classpath -join ';') pathtracing.MapSpawn $level
  if ($LASTEXITCODE -ne 0 -or $spawnText -notmatch '^(-?\d+) (-?\d+) (-?\d+)$') { throw 'Could not read source spawn.' }
  $spawn = @([int]$Matches[1], [int]$Matches[2], [int]$Matches[3])
  $staged = Join-Path $temp 'dimension'
  New-Item -ItemType Directory -Path $staged | Out-Null
  foreach ($folder in @('region','entities','poi')) {
    $from = Join-Path $sourceDimension $folder
    if (Test-Path -LiteralPath $from -PathType Container) { Copy-Item -LiteralPath $from -Destination (Join-Path $staged $folder) -Recurse }
  }
  New-Item -ItemType Directory -Force (Split-Path $target), (Split-Path $dimension) | Out-Null
  Move-Item -LiteralPath $staged -Destination $target
  $json = @{ type='minecraft:overworld'; generator=@{type='minecraft:flat'; settings=@{biome='minecraft:the_void'; layers=@(); structure_overrides=@()}} } | ConvertTo-Json -Depth 10
  $dimensionFile = [IO.File]::Open($dimension, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::None)
  try {
    $bytes = [Text.Encoding]::UTF8.GetBytes($json)
    $dimensionFile.Write($bytes, 0, $bytes.Length)
    $dimensionFile.Flush($true)
  } finally { $dimensionFile.Dispose() }
  $meta = @{pack=@{description='Path tracing maps'; min_format=@($version.pack_version.data_major,$version.pack_version.data_minor); max_format=@($version.pack_version.data_major,$version.pack_version.data_minor)}} | ConvertTo-Json -Depth 5
  [IO.File]::WriteAllText((Join-Path $pack 'pack.mcmeta'), $meta, [Text.UTF8Encoding]::new($false))
  $maps.Add([pscustomobject]@{name=$Name; spawn=$spawn; source=$sourcePath; imported=[DateTime]::UtcNow.ToString('o')})
  $nextRegistry = Join-Path $Rig ('maps.json.' + [Guid]::NewGuid().ToString('N') + '.tmp')
  try {
    [IO.File]::WriteAllText($nextRegistry, (ConvertTo-Json -InputObject ($maps.ToArray()) -Depth 5), [Text.UTF8Encoding]::new($false))
    if (Test-Path -LiteralPath $registry) { [IO.File]::Replace($nextRegistry, $registry, [NullString]::Value) }
    else { [IO.File]::Move($nextRegistry, $registry) }
  } finally { if (Test-Path -LiteralPath $nextRegistry) { Remove-Item -LiteralPath $nextRegistry } }
  Write-Host "Imported $Name. Start the game with Play."
} finally {
  $tempRoot = [IO.Path]::GetFullPath([IO.Path]::GetTempPath())
  $resolvedTemp = [IO.Path]::GetFullPath($temp)
  if (-not $resolvedTemp.StartsWith($tempRoot, [StringComparison]::OrdinalIgnoreCase) -or (Split-Path $resolvedTemp -Leaf) -notmatch '^pathtracing-map-[a-f0-9]{32}$') { throw 'Unsafe temporary cleanup path.' }
  Remove-Item -LiteralPath $resolvedTemp -Recurse -Force
}
} finally { $registryLock.Dispose() }
