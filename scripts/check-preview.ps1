[CmdletBinding()]
param([string]$Jar, [string]$McVersion = $env:PATHTRACING_MC_VERSION)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\common.ps1" -McVersion $McVersion
if (-not $Jar) { $Jar = Get-VerifiedClientJar }
$previewRoot = Split-Path -Parent $PSScriptRoot
Push-Location $previewRoot
try {
    & "$PSScriptRoot\preview.ps1" -Fixture -Jar $Jar -Output jobs/build-preview/samples/fixture
    $fixture = Get-Content jobs/build-preview/samples/fixture/fixture-result.txt
    if ($fixture[0] -ne 'PASS') { throw 'Fixture did not pass.' }
    $records = 'jobs/build-preview/samples/records'
    New-Item -ItemType Directory -Force $records | Out-Null
    $measurements = @()
    foreach ($build in @('pagoda','hearth')) {
        $commands = if ($build -eq 'pagoda') { 'jobs/pagoda/build-placed-x60.txt' } else { 'jobs/build-hearth/place.txt' }
        $box = if ($build -eq 'pagoda') { @(60,-60,-16,92,-10,16) } else { @(2000,-61,-60,2179,120,59) }
        $out = "jobs/build-preview/samples/$build"
        $views = "jobs/build-preview/$build-views.txt"
        foreach ($iteration in 1..3) {
            $watch = [Diagnostics.Stopwatch]::StartNew()
            & "$PSScriptRoot\preview.ps1" -Commands $commands -Box $box -Views $views -Jar $Jar -Output $out
            $watch.Stop()
            $run = Get-Content "$out/run.txt"
            $total = [double](($run | Where-Object { $_ -like 'total seconds=*' }) -replace '^total seconds=','')
            $notes = Get-Content "$out/notes.txt"
            if (($notes | Where-Object { $_ -eq 'Missing textures: []' }).Count -ne 1) { throw "$build has a missing texture." }
            if ((Get-Content "$out/wrapper-run.txt")[0] -ne 'priority=Normal') { throw 'Wrong process priority.' }
            $measurements += [pscustomobject]@{ Build=$build; Run=$iteration; JavaSeconds=$total; WallSeconds=$watch.Elapsed.TotalSeconds }
            Copy-Item -LiteralPath "$out/run.txt" -Destination "$records/$build-$iteration-run.txt" -Force
            Copy-Item -LiteralPath "$out/notes.txt" -Destination "$records/$build-$iteration-notes.txt" -Force
        }
    }
    $measurements | Export-Csv jobs/build-preview/samples/timings.csv -NoTypeInformation -Encoding ASCII
    $report = [Collections.Generic.List[string]]::new()
    $report.Add('# Offline build preview')
    $report.Add('')
    $report.Add('Files: `tools/preview/Preview.java`, `scripts/preview.ps1`, `scripts/check-preview.ps1`, the two `*-views.txt` presets, and `jobs/build-preview/samples/`. The Java source is the complete renderer and fixture; it uses the JDK only.')
    $report.Add('')
    $report.Add('Run from the worktree root (PowerShell):')
    $report.Add('```powershell')
    $report.Add('.\scripts\preview.ps1 -Commands jobs/pagoda/build-placed-x60.txt -Box @(60,-60,-16,92,-10,16) -Views jobs/build-preview/pagoda-views.txt -Output jobs/build-preview/samples/pagoda')
    $report.Add('.\scripts\preview.ps1 -Fixture -Output jobs/build-preview/samples/fixture')
    $report.Add('.\scripts\check-preview.ps1 # regenerates this report, both sample sets and timing distributions')
    $report.Add('```')
    $report.Add('Direct: `& "$Jdk\java.exe" tools/preview/Preview.java <commands> <x0> <y0> <z0> <x1> <y1> <z1> <jar> <views> <output>`. The wrapper reads `scripts/common.ps1`, falls back to the staged rig JDK, and sets the child to Normal priority. No Minecraft process is started.')
    $report.Add('')
    $report.Add('Inputs: the pagoda file in this worktree; the exact hearth `place.txt` copied from `C:\Users\PASCAL\Desktop\path tracing\jobs\build-hearth\place.txt` to its requested worktree path. No generator was substituted. Input, jar, source and view SHA-256 hashes and exact arguments are in each `run.txt`; `records/` retains all timed runs.')
    $report.Add('')
    $report.Add('Geometry/textures come from the jar: `assets/minecraft/blockstates/*.json`, inherited `assets/minecraft/models/block/*.json`, and `assets/minecraft/textures/block/*.png`. Variant and element rotations preserve stair/door/trapdoor/log states; multipart connections are supplied from neighboring blocks unless explicit. Cutout alpha is read per texel; glass/panes and water composite through to the world. Animated strips use their first frame. Plains temperature/downfall and water color come from `data/minecraft/worldgen/biome/plains.json`; grass/foliage colors are sampled from the jar colormaps. Nothing is extracted into the repository.')
    $report.Add('')
    $report.Add('Emission/opacity source: the same jar, disassembled by `$Jdk\javap.exe -p -c -v -classpath <jar> net.minecraft.world.level.block.Blocks`. The renderer follows `Blocks.<clinit>` registrations, collection/property helpers, `Properties.ofLegacyCopy`, `lightLevel` lambdas and `litBlockEmission`; it interprets their state comparisons rather than keeping remembered emission constants. For example this jar gives lantern 15 (`lambda$static$305`), soul lantern 10 (`lambda$static$306`), and copper lantern 15 (`lambda$static$308/309`). `CandleBlock.LIGHT_EMISSION`, `SeaPickleBlock.isDead`, vines/lichen factories, the anchor charge calculation, vault/trial enum constructor values and copper weather-state switch maps are followed into their own class bytecode. State-field aliases follow those classes (for example CHARGE is the serialized charges property). Each block actually used is cited in `notes.txt`. Unknown bytecode is refused and recorded, never silently assigned a light value.')
    $report.Add('Opacity: `BlockBehaviour$Properties.<init>` defaults `canOcclude` to true; `noOcclusion` and `noCollision` clear it, and `ofLegacyCopy` copies it. `BlockBehaviour.getLightDampening` returns 15 for `BlockState.isSolidRender`. This architectural preview uses that jar property plus full model bounds as its binary full-cell occluder; partial shapes pass light through their cell, as the requested non-opaque-cell BFS specifies. Light decreases by one through each cell; a 15-cell halo carries light across the input box boundary. This is an architectural light preview, with faint night ambient, rather than Minecraft lightmap/skylight simulation. Day face shading is fixed at top 1, X sides .6, Z sides .8, bottom .5.')
    $report.Add('')
    $report.Add('Fallbacks are whole cubes with the block texture/face aliases or model particle texture; block-entity graphics are not simulated. Run lists (each id once):')
    foreach ($build in @('pagoda','hearth')) {
        $fallback = (Get-Content "jobs/build-preview/samples/$build/notes.txt" | Where-Object { $_ -like 'Plain-cube fallbacks:*' }) -replace '^Plain-cube fallbacks: ',''
        $report.Add("- ${build}: $fallback")
    }
    $report.Add('')
    $report.Add('Views: eye is 1280 x 720, with Minecraft yaw/pitch and feet + 1.62 eye height, perspective rays and distance haze. Orthographic views use 8 pixels per block; isometrics fit the entire supplied box with a border. `plan y` clips above that block layer; `section x|z value` shows that one-block-thick vertical slice. Weighted model alternatives use their first variant, for deterministic previews. Ray/shape intersections use a voxel traversal; images are rendered on up to eight JDK worker threads.')
    $report.Add('')
    $report.Add('Fixture: **PASS**. A 5 x 5 stone-brick hut with a two-half spruce door, bottom-slab roof and interior lantern is rendered from every view type. All 12 dimensions are asserted (72 x 72 plans, 72 x 56 elevations/cuts, 134 x 137 isos, 1280 x 720 eyes). Lantern cell = 15, all six neighbors = 14, and the opaque wall = 0; worst light margin = 0. Nineteen emitter/state controls cover the named emitters, a lit lamp, lit/unlit candles, vines, lichen, a charged anchor, vault/trial states and all four copper-bulb weather states. The wrong-facing stair mutant changes ' + (($fixture | Where-Object { $_ -like 'wrong-facing image pixels changed=*' }) -replace '.*=','') + ' pixels. Parser controls check command order, clipping and outside superflat. See [fixture result](samples/fixture/fixture-result.txt).')
    $report.Add('')
    $report.Add('Timings in seconds, three separate JDK source-file launches per build. Java includes metadata, world/light preparation, rendering, PNG writes and replay hashes; wall includes source compilation and wrapper overhead. No averages:')
    $report.Add('')
    $report.Add('| Build | Clock | n | min | median | p90 | p99 | max |')
    $report.Add('|---|---|---:|---:|---:|---:|---:|---:|')
    foreach ($build in @('pagoda','hearth')) {
        foreach ($clock in @('JavaSeconds','WallSeconds')) {
            $values = @($measurements | Where-Object Build -eq $build | ForEach-Object { $_.$clock } | Sort-Object)
            $report.Add(('| {0} | {1} | 3 | {2:F3} | {3:F3} | {4:F3} | {4:F3} | {4:F3} |' -f $build,$clock,$values[0],$values[1],$values[2]).Replace(',','.'))
        }
    }
    $report.Add('')
    $report.Add('Sample images: [pagoda views](samples/pagoda/views.md) (all view types, courtyard eye); [hearth views](samples/hearth/views.md) (four day isos, one night iso, plan and trail eye); [hut views](samples/fixture/views.md). `timings.csv` holds every timing; `notes.txt` has per-view sizes/timings, fallbacks, missing-texture list and block metadata. Final sample runs have no missing textures or refusals, and the wrapper records Normal priority. The large blank upper part of hearth isos is the requested box extending to y=120, above the actual mountain peaks.')
    $report.Add('')
    $report.Add('STATUS: DONE')
    [IO.File]::WriteAllLines((Join-Path $previewRoot 'jobs/build-preview/last.md'),$report,[Text.UTF8Encoding]::new($false))
} finally { Pop-Location }
