VERDICT: FAIL

One high finding (registry corruption on the second import, reproduced); everything else passes. The fix is a few lines.

## Findings

### HIGH
1. `scripts/import-map.ps1:46` and `:91` (`$maps = @(Get-Content ... | ConvertFrom-Json)` then `ConvertTo-Json -InputObject @($maps)`). Windows PowerShell 5.1 does not unroll the parsed array, so a second import nests the old array and writes `{"value":[...],"Count":1}` as the first element of `maps.json`. Reproduced: judge-run2 (`hard_track_2` then `a-b`) gave exactly that file. `Maps.java:30` then calls `entry.getAsJsonObject().get("name").getAsString()` on that element (null, NPE), so `/map` answers "Map: null" and no map can be reached after the second import. The first import writes a correct file (judge-run3). The builder's gate ran two imports but never reads `maps.json` back (`MapTests.java` has no registry check), so it passed.
   Smallest fix: read with `$maps = @(); foreach ($m in (Get-Content -Raw ... | ConvertFrom-Json)) { $maps += $m }`, write with a form verified for 1, 2 and 3 entries, and add a gate that imports twice and parses `maps.json` with Gson through the same code path as `Maps.java`.

### LOW
2. `scripts/import-map.ps1:15` and `bot/tests/test-maps.ps1:9` hard-code `D:\path-tracing-target\maps-stick\` as the only allowed test root, so a judge cannot use its own folder (I used `...\maps-stick\judge-run*`). Derive it from a parameter.
3. `bot/tests/test-maps.ps1:5` and `scripts/import-map.ps1:40` default the server jar to `<rig>\server\.fabric\...`; in a worktree that path does not exist, so the gate fails unless `-ServerJar` is passed. Resolve through `PATHTRACING_ROOT`.
4. `bot/tracks.md:3` still says names are `[a-z0-9-]+`.
5. `Tracks.java:487`: the dimension check is a regex only; a nonexistent dimension fails inside RCON without a clear message. The bot teleport at `:513` uses `execute in <track dimension>` as the prompt asked.
6. `Tracks.java:79`: a reset click is swallowed silently when no track is selected or human mode is not ready. A short chat line would help.

## Each part against the prompt
1. Names: `TrackStore.java:52` and `:96`, `play-track.ps1:4`, `plan-track.rs`, `flight.rs:20` use `[a-z0-9_-]+`; tests accept `hard_track_2` and `a-b`, refuse `A`, `a b`, `a/b`. PASS (doc, finding 4).
2. Stick: `TrackInventory.java:34` `withRaceStick` takes the first empty hotbar slot (36-44), selects it, and throws naming the track when full. `TrackMode.java:54`, `:98`, `:136` compare with `withoutSticks` on both sides, refuse before saving a session, and the saved kit stays stick-free (hash unchanged). Reset (`Tracks.java:68-82`) abandons without recording (`discardRecording`, `run = null`), says "Reset." only when a run is timed, teleports to (x+0.5, y+1.0, z+0.5) with current yaw and pitch via `humanClient.command`, and re-arms through the one existing on-A path (`TrackDetector.requestRearm`, `TrackDetector.java:35`, emitting the existing REARM). No second arming path. Sneak+click runs `command(mc, "exit")`. PASS.
3. `Tracks.java:86-95` `startAtClickedA` whispers exactly `msg PathBot track run <name>` for a stored A in the current dimension; other blocks keep `goto` (`Pointer.java:48`). Reached only outside human mode and creation. PASS.
4. Maps: `Maps.java` lists clickable `/map <name>` lines and sends `execute in <dim> run tp @s` and the same for PathBot, spawn read with `NbtIo` (`MapSpawn.java`); blocked in practice by finding 1. The datapack `pack.mcmeta` reads 107.1 from the jar's `version.json` (`import-map.ps1:41-43`, `:80`). The bot teleport uses the track dimension. The planner (`TrackWorldView.region`) maps `minecraft:overworld` and `pathtracing:<name>` and refuses others by name.
5. Protected paths untouched: `engine/law`, `words`, `native`, `engine/world/src`, saved tracks, runs, jars. The builder did not edit `engine/world/src`; `plan-track` builds a temporary view (`TrackWorldView.java`) beside the world with hard links to the region files, a copied `level.dat`, and a datapack check (`:29-48`) accepting only `data/pathtracing/dimension/<name>.json` equal to the exact flat/void/overworld definition before dropping `file/pathtracing-maps` from the copy. Minimal and principled; cost: planning needs the built bot jar and `powershell.exe` (`plan-track.rs:48`).
6. `main...0590772` touches 22 files, all in scope (the larger `main..` stat is main having moved on). No secret or jar committed; `.gitignore` adds `maps.json`, its lock and temp files.

## Danger review of the importer
- Writes only `dimensions/pathtracing/<name>`, the maps datapack and `maps.json`. It refuses an existing name, target folder or dimension file (`:34`); the dimension file is `CreateNew`; data is staged then moved. Nothing in the world is deleted or overwritten (`pack.mcmeta` is rewritten with identical content). The only recursive delete is the temp folder, guarded by name pattern and prefix (`:93-96`).
- Name is `\A[a-z0-9_-]+\z`, so no separators; `overworld` reserved.
- Zip-slip: I built a zip with a `../` entry; `ExtractToDirectory` on this runtime (PS 5.1.26100) refuses it. A source folder copy may follow a junction inside the source; low risk.
- Stop: `rcon.ps1 'save-all flush' 'stop'`, waits up to 60 s, as `Stop server.cmd`; refuses if the server runs without RCON. Safe. Test mode only prints "Would stop" and is limited to the test root.

## In-game checks for Julien (after the fix)
1. Import two maps in a row, start with Play, `/map`: both maps and overworld list; click each: you and PathBot arrive at the saved spawns, no restart.
2. Create `hard_track_2`; old hyphen names still list; `A`, `a b`, `a/b` refuse.
3. In track mode: stick in the first free slot and selected; a full hotbar refuses naming the track; right-click resets to A with the same view and "Reset." only when timed, no saved attempt; sneak+click exits and restores inventory and mode.
4. Outside track mode click a track's A: PathBot starts; click another block: `goto`. With PathBot in another dimension, it is brought into the track's dimension.

## Commands run and results
- `bot\build.ps1` via `powershell -File`, CARGO_TARGET_DIR=D:\path-tracing-target\maps-stick-judge: built `bot\build\pathtracing.jar` (inside the tool's PowerShell the javac "Note:" on stderr trips ErrorActionPreference; not a build fault).
- `cargo test --offline --workspace` in `engine\`: exit 0, 42 "test result: ok", no failures.
- `bot\tests\test-tracks.ps1`: 421 inventory/track-mode checks and 109 headless checks passed.
- `bot\tests\test-maps.ps1 -Destination ...\maps-stick\judge-run2 -ServerJar <real jar, read only> -Planner ...`: "Passed 10 offline map checks"; the planner reached search (NO SENTENCE, as designed). Needs `-ServerJar` in a worktree and the `maps-stick\` root (findings 2, 3).
- Manual importer runs in synthetic rigs judge-run2 and judge-run3: first import correct, second corrupt (finding 1). Real world never touched.
