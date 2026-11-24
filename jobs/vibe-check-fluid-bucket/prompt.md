# Vibe second-family verify & close open fluid/bucket limits

**Goal:** Independently verify `gen-fluid-bucket-rows` against `server-26.2.jar` and close its open limits (1-block drop falling fluid column, `BucketItem.useOn` on bottom slab, `ClipContext$Block.FALLDAMAGE_RESETTING` ladder vs cobweb/berry check, and fluid push velocities).

Second-family (`vibe`) independent verification of `C:/Users/PASCAL/Desktop/path tracing/lab/results/local-20261124-103203-4468-1-0-gen-fluid-bucket-rows/` plus resolution of its documented open cases. Do NOT trust any prior dossier claim without re-running it from `server-26.2.jar`.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`.

**Harness requirements:**
- Bind real jar tags via `MappedRegistry.prepareTagReload(...).apply()` (`data/minecraft/tags/**/*.json`).
- Use real `ServerLevel` + real `LevelChunk` (`hasFluid()` intact!) with `LevelTicks<Fluid>` and `LocalPlayer` (or `isLocalPlayer() => true` + `isLocalClientAuthoritative() => true`).
- Write `dossier.md` (`Claim: ...`), `rows.tsv`, and `witnesses.tsv` early after core witnesses pass, then extend cleanly.

**Verify & close these specific items:**
1. **Cross-check GLM's fluid/bucket numbers** against `server-26.2.jar`: water cadence `5`, Nether fast lava `10`, Overworld lava `30`, bucket reach `4.5`, 25-block MLG clutch same-tick order, hay/honey `0.2`, bed `0.5`, slime `0.0`, cobweb/sweet berry bush/powder snow `0.0`.
2. **1-block drop falling column (`FlowingFluid.spread` / `getNewLiquid@185-254`):** construct a 2-tier terrain (`y=64` stepping down to `y=63`) and witness how water spreads over the ledge and forms the falling `level=8, falling=true` column cell by cell.
3. **`BucketItem.useOn` vs `BucketItem.use` on a bottom slab:** trace both bytecode paths on a half-slab hit to explain why `useOn` answered `PASS` while `emptyContents` waterlogged it.
4. **`FALLDAMAGE_RESETTING` discrepancy:** inspect `ClipContext$Block.FALLDAMAGE_RESETTING` predicate and why `ladder` (`#climbable`) kept `8.0 / 23.0` fall damage when falling past it while cobweb/berry bush reset fall distance.
5. **Fluid current acceleration (`Entity.updateFluidHeightAndDoFluidPushing`):** witness exact pre/post `vx, vy, vz` hex bits across flowing water and lava (`0.014`, `0.007`, `0.0023333333333333335`).

**Write in your run folder:**
- `rows.tsv`: `case_id	topic	verified_status	constants_or_delta_hex	scars	source`
- `witnesses.tsv`: `case_id	tick	x_hex	y_hex	z_hex	vx_hex	vy_hex	vz_hex	fluid_state	damage_hex	events`
- `dossier.md` (first line `Claim: `), `reproduce.sh`, and `wiki.md`.

**Falsifier:** Any divergence between our reproduced jar witness and `rows.tsv`, or any unresolved open item left unexplained without a reproducing jar trace.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
