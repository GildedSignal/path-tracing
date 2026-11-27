# Vibe live verification: 26.2 water pose transitions, seabed startup, and shore exit

**Goal:** Execute independent live 26.2 `ServerPlayer` swimming transitions, water-surface bobbing, seabed startup, and shore exit ticks on `server-26.2.jar` with `MappedRegistry.prepareTagReload` + `DataComponentInitializers` + `ServerLevel` to cross-check water-joint physics.

See `lab/results/local-20261124-130147-11356-1-0-vibe-check-block-place-break/` for clean `DataComponentInitializers` / tag boot.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`; java/javac: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/`.

**Harness requirements (mandatory executed witnesses):**
- Boot tags (`MappedRegistry.prepareTagReload(...).apply()`) + `DataComponentInitializers` + Unsafe `ServerLevel` + real `LevelChunk` + real `ServerPlayer`.
- Inspect bytecode with `javap -c -p -constants` for `Player.updateSwimming`, `LivingEntity.travelInFluid`, `Entity.updateFluidHeightAndDoFluidPushing`, and `LivingEntity.jumpInLiquid`.
- Write `dossier.md` (`Claim: ...`), `rows.tsv`, and `witnesses.tsv` early, then extend cleanly.

**Live scenes to execute:**
1. **Surface sprint-swim entry & steady cycle:** enter a 3-deep × 16-long water basin at sprint-jump speed, record `(x, y, z, vx, vy, vz)` IEEE hex bits, `pose`, `isSwimming`, `eyeInWater`, `waterHeight`, and verify exact horizontal ticks per block of the steady sprint-swim repeat.
2. **Seabed-to-swim vs 1-deep wading:** compare starting on the 3-deep seabed (`y = -2`) vs 1-deep shallow water (`y = 0`), pinning the exact ticks where `isSwimming` toggles and how `jump` vs no-jump affects seabed lift-off.
3. **Shore exit (`dy = 0` and `dy = +1` stone bank):** exit the basin onto a level stone shore and a 1-block-higher stone ledge; pin exact collision flags and swimming pose deactivation ticks.

**Write in your run folder:**
- `rows.tsv`: `case_id	tick	pose	swimming	eye_in_water	x_hex	y_hex	z_hex	vx_hex	vy_hex	vz_hex	scars	source`
- `witnesses.tsv`: `case_id	tick	x_hex	y_hex	z_hex	vy_hex	flags`
- `dossier.md` (first line `Claim: `), `reproduce.sh`, and `wiki.md`.

**Falsifier:** Any executed `pose`, `isSwimming`, `waterHeight`, or position/velocity bit in `server-26.2.jar` that diverges from the documented 26.2 fluid rules.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
