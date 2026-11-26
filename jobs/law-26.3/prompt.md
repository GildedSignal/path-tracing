# The law on Minecraft 26.3

**Goal:** Move the law, the world tables, the bot mod and the server mod from Minecraft 26.2 to 26.3 by diffing Mojang's code, so that Hielke's maps (DataVersion 5023, 26.3 only) load and every jar witness, replay and track test stays green.

## Why

Hielke's 11 parkour maps carry elite, video-verified human records on the speedrun.com `26.2+` board (`map/detail/benchmarks.md`). He publishes only 26.3 builds; the law is 26.2 (`engine/world/data/version.txt` = 4903), and a 26.2 game cannot open a newer world. Bring the law to the judge's version before any flight.

## Read first

- `map/` (loaded whole), `map/detail/engine.md`, `map/detail/benchmarks.md`.
- `map/principles.md` 3, 4, 6, 7, 17, 21, 22 (no bandaids), 27.
- How the tables are made: `engine/world/tools/models.ps1`, `engine/world/tools/ModelTable.java`, `engine/world/data/*.tsv`; the jar path conventions in `scripts/common.ps1` (`$McVersion`), `scripts/check-preview.ps1`, `scripts/preview.ps1`, `scripts/import-map.ps1`, `bot/build.ps1`, `bot/fabric.mod.json`, `server-mod/`.

## The work

1. **Fermi first: the diff.** Get the 26.3 client and server jars from Mojang's official piston-meta version manifest into `D:\path-tracing-deps\` (`minecraft-26.3-client.jar`, never into git). 26.x ships unobfuscated: diff the 26.2 and 26.3 sources of every class the law ports (player and living-entity movement, collision and shapes, fluids, block behaviours the cell rules read, effects, attributes, food, damage). Write `jobs/law-26.3/diff.md` first: each changed rule, the file of the law it touches, and the ones that change no behaviour (deleted from consideration, with why).
2. **Regenerate the tables** with the existing generator against 26.3 (`version.txt` becomes 26.3's DataVersion). New blocks are new rows; no hand-written row.
3. **Port only what changed** in the law, in the law's own structure (generators, data tables, thin scars). A rule 26.3 did not change is not touched. A change you cannot port exactly is a named refusal, not a guess.
4. **Mods:** `$McVersion`, `fabric.mod.json` and every 26.2 path move to 26.3; rebuild the bot and server mod against the 26.3 jar and a 26.3 Fabric API and Loader. Nothing in his launcher, his world or his server is changed: write the steps he runs to move his own instance into `jobs/law-26.3/report.md`.
5. **Delete what 26.3 made dead.** Net lines in `engine/` fall or each growth line carries its `Bought:` reason.
6. **Map:** change `map/` in the same commit wherever it names 26.2 (present tense, no history).

## Gates (in your worktree only; no live game while he plays)

- Environment: `CARGO_TARGET_DIR=D:/path-tracing-target/job-law-26.3`, `PATHTRACING_DEPS=D:\path-tracing-deps\data`, `RUSTUP_HOME=D:\rust\rustup`, `CARGO_HOME=D:\rust\cargo`, `PATH` with `D:\rust\cargo\bin`.
- `cargo test --release --no-fail-fast -p world -p law -p plan -p pathtracing_native -p mind -p build -- --test-threads=1` all green (timing tests are judged serially). Any jar witness whose expected value changes is regenerated from the 26.3 jar, never edited by hand, and listed in the report with the 26.2 and 26.3 values.
- `WorldSnapshot::load` opens `D:\path-tracing-maps\hielke\spiral-3.0.5\Parkour Spiral` (a copy; never modify it) without a DataVersion refusal. Report every unmodeled state it meets, and say whether its datapack needs anything beyond the vanilla rule (the loader refuses non-vanilla datapacks today: name the gap, do not patch it).
- `bot/build.ps1` and `server-mod/build.ps1` build against 26.3.
- `engine/world/src/bin/bench-course.rs` (`exact`, `floored`, `straight`) runs; report its medians beside the 26.2 ones in `map/detail/benchmarks.md`.
- The gate after yours is a flight in his game on 26.3: the 11 tracks in `lab/tracks/` replayed with `T` against `predictedT`. Leave the exact commands for it in the report.

## Closed

Pushing, his world and server, his launcher instance, git history, other branches, secrets. Game jars never enter git.

## Out

`jobs/law-26.3/diff.md`, `jobs/law-26.3/report.md` (what changed, net lines, every gate's raw output, open gaps), commits on `wt/law-26.3`.
