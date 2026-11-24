# Replays the importer gate in an isolated rig; no game, server, client or RCON.
param([string]$ServerJar, [string]$Destination, [string]$Planner, [string]$TestRoot)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\..\..\scripts\common.ps1"
if (-not $TestRoot) {
  $TestRoot = if ($Destination) { Split-Path ([IO.Path]::GetFullPath($Destination)) } else { Join-Path $Root 'bot\build\map-tests' }
}
$allowedTestRoot = [IO.Path]::GetFullPath($TestRoot).TrimEnd('\','/') + [IO.Path]::DirectorySeparatorChar
if (-not $Destination) { $Destination = Join-Path $TestRoot ('maps-' + [Guid]::NewGuid().ToString('N')) }
$Destination = [IO.Path]::GetFullPath($Destination)
if (-not $Destination.StartsWith($allowedTestRoot, [StringComparison]::OrdinalIgnoreCase)) { throw "Gate destination must be inside TestRoot: $allowedTestRoot" }
if (Test-Path -LiteralPath $Destination) { throw 'Gate destination already exists; use a fresh folder.' }
$testRig = Join-Path $Destination 'rig'
$world = Join-Path $testRig 'server\world'
$source = Join-Path $Destination 'source'
$fixture = Join-Path $Root 'engine\world\tests\fixtures\world'
New-Item -ItemType Directory -Force "$source\region", "$world\data\minecraft" | Out-Null
$classpath = @((Join-Path $Root 'bot\build\pathtracing.jar')) + @(Get-GameClasspath)
& "$Jdk\javac.exe" --release 25 -proc:none -cp ($classpath -join ';') -d $Destination "$PSScriptRoot\MapTests.java"
if ($LASTEXITCODE) { throw 'Map test compilation failed.' }
$testClasspath = @($Destination) + $classpath -join ';'
function Assert-Registry([int]$Count) {
  & "$Jdk\java.exe" -cp $testClasspath pathtracing.MapTests --registry "$testRig\maps.json" $Count
  if ($LASTEXITCODE) { throw "Client map registry read-back failed after $Count imports." }
}
[IO.File]::WriteAllText("$testRig\maps.json", '[]')
Assert-Registry 0
Copy-Item -LiteralPath "$fixture\level.dat" -Destination "$source\level.dat"
Copy-Item -LiteralPath "$fixture\dimensions\minecraft\overworld\region\r.0.0.mca" -Destination "$source\region\r.0.0.mca"
Copy-Item -LiteralPath "$fixture\level.dat" -Destination "$world\level.dat"
foreach ($file in @('world_gen_settings.dat','game_rules.dat')) { Copy-Item -LiteralPath "$fixture\data\minecraft\$file" -Destination "$world\data\minecraft\$file" }
$importer = Join-Path $Root 'scripts\import-map.ps1'
$importOptions = @{TestRig=$testRig; TestRoot=$Destination}
if ($ServerJar) { $importOptions.ServerJar = $ServerJar }
& $importer -Source $source -Name hard_track_2 @importOptions
Assert-Registry 1
if (-not (Test-Path -LiteralPath "$world\dimensions\pathtracing\hard_track_2\region\r.0.0.mca")) { throw 'Imported region is missing.' }
$before = (Get-FileHash -LiteralPath "$testRig\maps.json").Hash
$refused = $false
try { & $importer -Source $source -Name hard_track_2 @importOptions } catch { $refused = $_.Exception.Message.Contains('already exists') }
if (-not $refused -or (Get-FileHash -LiteralPath "$testRig\maps.json").Hash -ne $before) { throw 'Duplicate import did not preserve the registry.' }
foreach ($name in @('A','a b','a/b')) {
  $refused = $false
  try { & $importer -Source $source -Name $name @importOptions } catch { $refused = $true }
  if (-not $refused) { throw "Invalid map name accepted: $name" }
}
# A wrapped zip with the 26.2 directory layout exercises the other source form.
$newSource = Join-Path $Destination 'zip-source\wrapped-world'
New-Item -ItemType Directory -Force "$newSource\dimensions\minecraft\overworld" | Out-Null
Copy-Item -LiteralPath "$source\level.dat" -Destination "$newSource\level.dat"
& "$Jdk\java.exe" -cp $testClasspath pathtracing.MapTests --set-spawn "$newSource\level.dat" 12 80 -7
if ($LASTEXITCODE) { throw 'Could not prepare the second synthetic spawn.' }
Copy-Item -LiteralPath "$source\region" -Destination "$newSource\dimensions\minecraft\overworld\region" -Recurse
Add-Type -AssemblyName System.IO.Compression.FileSystem
$archive = Join-Path $Destination 'map.zip'
[IO.Compression.ZipFile]::CreateFromDirectory((Split-Path $newSource), $archive)
& $importer -Source $archive -Name a-b @importOptions
Assert-Registry 2
$thirdSource = Join-Path $Destination 'third-source'
Copy-Item -LiteralPath $source -Destination $thirdSource -Recurse
& "$Jdk\java.exe" -cp $testClasspath pathtracing.MapTests --set-spawn "$thirdSource\level.dat" -8 72 9
if ($LASTEXITCODE) { throw 'Could not prepare the third synthetic spawn.' }
& $importer -Source $thirdSource -Name third_map @importOptions
Assert-Registry 3
$entries = [Collections.Generic.List[object]]::new()
foreach ($map in (Get-Content -LiteralPath "$testRig\maps.json" -Raw | ConvertFrom-Json)) { $entries.Add($map) }
if ($entries.Count -ne 3) { throw 'Registry must contain three flat map entries.' }
$names = @('hard_track_2','a-b','third_map')
$spawns = @('0,-60,0','12,80,-7','-8,72,9')
for ($i = 0; $i -lt $names.Count; $i++) {
  if ($entries[$i].name -cne $names[$i] -or ($entries[$i].spawn -join ',') -cne $spawns[$i]) { throw "Incorrect map name or spawn at entry $i." }
}
& "$Jdk\java.exe" -cp $testClasspath pathtracing.MapTests $testRig
if ($LASTEXITCODE) { throw 'Map tests failed.' }
if ($Planner) {
  # This need not find a sentence: it must reach solve after loading the imported dimension.
  $reply = & $Planner hard_track_2 --rig $testRig --world $world --start settled --mode solve --out "$Destination\plan.jsonl"
  $plannerExit = $LASTEXITCODE
  if ($plannerExit -ne 0 -and $plannerExit -ne 3) { throw "Unexpected planner exit: $plannerExit" }
  if (($reply -join "`n") -match 'track does not load|track dimension:|planner or export failed|unknown track') { throw "Planner failed before search: $reply" }
  $reply | Set-Content "$Destination\planner.txt" -Encoding ascii
}
# The audit must refuse an open port before it reads regions or writes output.
$listener = [Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback, 0)
$previousPort = $env:PATHTRACING_SERVER_PORT
try {
  $listener.Start()
  $probePort = $listener.LocalEndpoint.Port
  if ($probePort -gt 65525) { throw 'Ephemeral probe port leaves no RCON range.' }
  $env:PATHTRACING_SERVER_PORT = [string]$probePort
  $refused = $false
  try { & "$Root\scripts\audit-map-spawns.ps1" -Output "$Destination\refused.json" -Record "$Destination\refused-record.json" }
  catch { $refused = $_.Exception.Message.Contains('Refusing spawn audit: server port') }
  if (-not $refused -or (Test-Path -LiteralPath "$Destination\refused.json")) { throw 'Spawn audit did not refuse the open port before writing.' }
} finally { $listener.Stop(); $env:PATHTRACING_SERVER_PORT = $previousPort }
@{command='bot/tests/test-maps.ps1'; destination=$Destination; serverJar=$(if ($ServerJar) {$ServerJar} else {$VanillaServerJar}); fixture=$fixture; planner=$Planner; result='PASS'; registryCounts=@(0,1,2,3); names=$names; spawns=$spawns; refusals=@('duplicate','A','a b','a/b','minecraft:the_nether','non-dimension datapack resource','PowerShell array wrapper')} | ConvertTo-Json -Depth 4 | Set-Content "$Destination\record.json" -Encoding ascii
Write-Output "Map gate record: $Destination\record.json"
Copy-Item -LiteralPath "$Destination\record.json" -Destination "$Root\jobs\track-ux\map-checks.json"
