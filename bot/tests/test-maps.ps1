# Replays the importer gate in an isolated rig; no game, server, client or RCON.
param([string]$ServerJar, [string]$Destination, [string]$Planner)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\..\..\scripts\common.ps1"
if (-not $ServerJar) { $ServerJar = Join-Path $Rig 'server\.fabric\server\26.2-server.jar' }
if (-not $Destination) { $Destination = 'D:\path-tracing-target\maps-stick\maps-' + [Guid]::NewGuid().ToString('N') }
$Destination = [IO.Path]::GetFullPath($Destination)
if (-not $Destination.StartsWith('D:\path-tracing-target\maps-stick\', [StringComparison]::OrdinalIgnoreCase)) { throw 'Gate destination must be inside the maps-stick target folder.' }
if (Test-Path -LiteralPath $Destination) { throw 'Gate destination already exists; use a fresh folder.' }
$testRig = Join-Path $Destination 'rig'
$world = Join-Path $testRig 'server\world'
$source = Join-Path $Destination 'source'
$fixture = Join-Path $Root 'engine\world\tests\fixtures\world'
New-Item -ItemType Directory -Force "$source\region", "$world\data\minecraft" | Out-Null
Copy-Item -LiteralPath "$fixture\level.dat" -Destination "$source\level.dat"
Copy-Item -LiteralPath "$fixture\dimensions\minecraft\overworld\region\r.0.0.mca" -Destination "$source\region\r.0.0.mca"
Copy-Item -LiteralPath "$fixture\level.dat" -Destination "$world\level.dat"
foreach ($file in @('world_gen_settings.dat','game_rules.dat')) { Copy-Item -LiteralPath "$fixture\data\minecraft\$file" -Destination "$world\data\minecraft\$file" }
$importer = Join-Path $Root 'scripts\import-map.ps1'
& $importer -Source $source -Name hard_track_2 -TestRig $testRig -ServerJar $ServerJar
if (-not (Test-Path -LiteralPath "$world\dimensions\pathtracing\hard_track_2\region\r.0.0.mca")) { throw 'Imported region is missing.' }
$before = (Get-FileHash -LiteralPath "$testRig\maps.json").Hash
$refused = $false
try { & $importer -Source $source -Name hard_track_2 -TestRig $testRig -ServerJar $ServerJar } catch { $refused = $_.Exception.Message.Contains('already exists') }
if (-not $refused -or (Get-FileHash -LiteralPath "$testRig\maps.json").Hash -ne $before) { throw 'Duplicate import did not preserve the registry.' }
foreach ($name in @('A','a b','a/b')) {
  $refused = $false
  try { & $importer -Source $source -Name $name -TestRig $testRig -ServerJar $ServerJar } catch { $refused = $true }
  if (-not $refused) { throw "Invalid map name accepted: $name" }
}
# A wrapped zip with the 26.2 directory layout exercises the other source form.
$newSource = Join-Path $Destination 'zip-source\wrapped-world'
New-Item -ItemType Directory -Force "$newSource\dimensions\minecraft\overworld" | Out-Null
Copy-Item -LiteralPath "$source\level.dat" -Destination "$newSource\level.dat"
Copy-Item -LiteralPath "$source\region" -Destination "$newSource\dimensions\minecraft\overworld\region" -Recurse
Add-Type -AssemblyName System.IO.Compression.FileSystem
$archive = Join-Path $Destination 'map.zip'
[IO.Compression.ZipFile]::CreateFromDirectory((Split-Path $newSource), $archive)
& $importer -Source $archive -Name a-b -TestRig $testRig -ServerJar $ServerJar
$classpath = @((Join-Path $Root 'bot\build\pathtracing.jar')) + @(Get-GameClasspath)
& "$Jdk\javac.exe" --release 25 -proc:none -cp ($classpath -join ';') -d $Destination "$PSScriptRoot\MapTests.java"
if ($LASTEXITCODE) { throw 'Map test compilation failed.' }
& "$Jdk\java.exe" -cp (@($Destination) + $classpath -join ';') pathtracing.MapTests $testRig
if ($LASTEXITCODE) { throw 'Map tests failed.' }
if ($Planner) {
  # This need not find a sentence: it must reach search after loading the imported dimension.
  $reply = & $Planner hard_track_2 --rig $testRig --world $world --start settled --mode 3 --out "$Destination\plan.jsonl" --cap 0.01
  $plannerExit = $LASTEXITCODE
  if ($plannerExit -ne 0 -and $plannerExit -ne 3) { throw "Unexpected planner exit: $plannerExit" }
  if (($reply -join "`n") -match 'track does not load|track dimension:|planner or export failed|unknown track') { throw "Planner failed before search: $reply" }
  $reply | Set-Content "$Destination\planner.txt" -Encoding ascii
}
@{command='bot/tests/test-maps.ps1'; destination=$Destination; serverJar=$ServerJar; fixture=$fixture; planner=$Planner; result='PASS'; refusals=@('duplicate','A','a b','a/b','minecraft:the_nether','non-dimension datapack resource')} | ConvertTo-Json -Depth 4 | Set-Content "$Destination\record.json" -Encoding ascii
Write-Output "Map gate record: $Destination\record.json"
