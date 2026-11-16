# maps-stick

Consumer: Julien racing downloaded maps, and PathBot racing the same tracks.

## Files changed

- `.gitignore`: ignores the maps registry, its lock and its temporary writes; imported worlds remain under ignored `server/`.
- `bot/src/pathtracing/TrackStore.java`, `scripts/play-track.ps1`: track names accept underscores.
- `bot/src/pathtracing/TrackInventory.java`, `TrackMode.java`, `TrackDetector.java`, `Tracks.java`, `Pointer.java`: race stick, reset/exit, clicked-A dispatch and cross-dimension bot start.
- `bot/src/pathtracing/PathTracingMod.java`, `Maps.java`, `MapSpawn.java`, `MapRegistry.java`: client map command, clickable choices, Mojang NBT spawn reader and registry parser shared with the gate.
- `scripts/import-map.ps1`, `scripts/common.ps1`: offline folder/zip import, flat registry serialization, shared rig server-jar path and isolated `-TestRig`/`-TestRoot` gate mode.
- `bot/tracks.md`: name rule includes underscores.
- `bot/src/pathtracing/TrackWorldView.java`, `scripts/track-world-view.ps1`, `engine/plan/src/bin/plan-track.rs`, `engine/plan/src/flight.rs`: selected-dimension world view and planner loading.
- `bot/tests/TrackTests.java`, `TrackModeTests.java`, `MapTests.java`, `test-maps.ps1`: name, kit, rearm, importer and dimension checks.
- `jobs/maps-stick/verify.ps1`, `last.md`: replayable gates and this handoff.

## Each part's behavior

1. Names use `[a-z0-9_-]+` in the store, listing, playback script and planner; existing names remain valid.
2. Human race kits add/select one stick in the first empty hotbar slot, refuse a full hotbar by track name, compare kits without sticks and preserve the saved kit hash; right-click discards the attempt and teleports to A with the current view, then the existing on-A REARM handler refills; a reset before readiness explains why in chat; sneak-right-click exits/restores.
3. Outside human track mode and creation, clicking a stored A in the current dimension whispers `track run <name>` to PathBot; other pointed blocks keep `goto`.
4. Imports copy the source overworld into `pathtracing:<name>` without replacing an existing map, derive datapack format from the server jar, record source spawn/path/time, and leave the server stopped; `/map` lists clickable choices and teleports both players through the human command channel; bot race preparation and the planner select the track dimension.

## Gate results

Replay from the worktree: `./jobs/maps-stick/verify.ps1`.

Record after the judge fixes: `D:\path-tracing-target\maps-stick\gate-e667ed7fbd7b4eb69904d007df7037be\record.json`.
Full output: the sibling `output.txt`; importer replay details and planner output are under `import/`.

- PASS: `bot/build.ps1`.
- PASS: `cargo test --offline --workspace` in `engine/`, with `CARGO_TARGET_DIR=D:\path-tracing-target\maps-stick`; existing ignored tests stay ignored.
- PASS: `bot/tests/test-tracks.ps1`, including acceptance of `hard_track_2` and `a-b`, refusal of `A`, `a b`, `a/b`, race kit/hash/restore checks and the existing REARM event for stick reset.
- PASS: `bot/tests/test-maps.ps1`, using a copied fixture `level.dat` and one region file under the isolated gate directory: old folder layout and wrapped zip/new layout, duplicate preservation, name refusals, both vanilla spawn layouts, selected region links and rejection of a datapack with extra resources. The gate checks the empty registry and then three sequential imports, verifying every name/spawn through the same `MapRegistry.read` method called by `/map`; it also reads the final flat array back in PowerShell.
- PASS: the map gate also runs outside the old maps-stick-only test root without a `-ServerJar` override. Record: `D:\path-tracing-target\maps-stick-review\maps-a0f8a8a86c1145f9be7e81e635c33dc1\record.json`.
- PASS: planner smoke check with the imported `hard_track_2` reaches search; it uses a capped search and makes no claim to have found a route.
- PASS: PowerShell parsing/ASCII, generated-data ignore checks and `git diff --check`.

No game, server, client or live RCON was launched. All importer writes in the gates went into the temporary rig.

## Judge findings resolved

- HIGH: PowerShell now enumerates each parsed map into a list and serializes its explicit object array, preventing the second import's `value`/`Count` wrapper. The regression uses `hard_track_2` at `(0,-60,0)`, `a-b` at `(12,80,-7)` and `third_map` at `(-8,72,9)`; the shared client reader also rejects the reproduced wrapper with a clear shape error.
- Test destinations have a configurable `-TestRoot`; standalone map tests can derive it from `-Destination` or the script's build folder. `verify.ps1` accepts the same parameter.
- `common.ps1` resolves `VanillaServerJar` from the shared rig before the importer substitutes its synthetic `TestRig`, so worktree gates use `PATHTRACING_ROOT` without a jar override.
- The track-name documentation includes `_`; blocked reset clicks explain restoration, missing selection, stopped mode or pending kit readiness in chat.
- Bot race preparation checks the server-advertised dimension set before any RCON commands and names an unavailable dimension in its refusal.

## Julien's checks after the judge

1. Create/list a track named `hard_track_2`; confirm an existing hyphenated track still lists and starts, and uppercase/space/slash names refuse.
2. Walk onto A with a kit that has a free hotbar slot: check the selected stick, leave A, right-click while looking away from A, and verify `Reset.`, unchanged view, return to A, refill/rearm and no saved attempt. Click before the kit is ready and check the chat explanation. Repeat with a full hotbar to check the named refusal. Sneak-right-click and verify original inventory/game mode restoration; test the ordinary stick outside track mode too.
3. Exit human track mode and stand away from A; point at its stored block and right-click. Verify PathBot receives the race command and starts. Point at another block and verify ordinary `goto`.
4. Import three downloaded folders/zips in succession with `scripts/import-map.ps1 -Source <path> -Name <name>`, then start with Play. Use `/map` and check all names plus overworld; click each and verify both players arrive at their saved spawns without a server restart. Create a track there, place PathBot in another dimension and click A to check its cross-dimension race start. A track naming an unavailable dimension should report that dimension before preparation starts.

## Review notes

The protected Rust world reader was left unchanged. Its overworld-only layout is supplied through a temporary view beside the source world: a copied `level.dat` read/written with `NbtIo`, and hard links to the selected dimension's regions and the unchanged server settings. Only the exact dimension-only maps pack is projected out of the copied enabled-pack list; other packs/resources retain their refusal. The snapshot owns its cells before the view is deleted. This requires the built bot jar and the staged Java classpath; preparation is included in reported thinking time.

Known fixture track bounds remain available. Newly named tracks use an endpoint bounding box with the existing flight reader's pad/halo; the existing finite snapshot/search limitations still apply.

The pre-existing modifications to `engine/world/tests/fixtures/world/dimensions/minecraft/overworld/region/r.0.-1.mca` and `r.0.0.mca`, and their `.bak-20261114` files, were not changed by this job and are not part of its patch. No protected engine source, game jar, saved track/kit or run was edited. No commit or push was made.

STATUS: DONE
