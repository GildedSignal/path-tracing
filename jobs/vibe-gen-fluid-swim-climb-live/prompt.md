# Vibe live verification: fluid swim travel, climbables, and bubble columns executed headless

**Goal:** Execute 26.2 `travelInWater` / `travelInLava` (water drag `0.8/0.9`, `0.5` vertical, `-0.003` snap, water-exit `+0.3` jump, depth strider / dolphin's grace / soul speed), climbables (`ladder`/`vine`/`scaffolding` `±0.15` clamp, sneak hold), and `BubbleColumnBlock` (`1.8/0.7/-0.9/-0.3`) LIVE on `ServerLevel` + `LevelChunk` + bound tags against `server-26.2.jar`.

Closes GLM's pending `gen-water-swim-climb-rows` lane ahead of schedule using the proven Unsafe `ServerLevel` + `LevelChunk` + `prepareTagReload` + `DataComponentInitializers` recipe.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`.

**Harness requirements (mandatory executed trajectories):**
- Boot jar tags (`MappedRegistry.prepareTagReload`: `#climbable`, `#soul_speed_blocks`, `#water`, `#lava`) + `DataComponentInitializers` + Unsafe `ServerLevel` + real `LevelChunk` (`hasFluid() = true` intact!) + local movement player (`isLocalPlayer() => true` + `isLocalClientAuthoritative() => true`).
- Write `dossier.md` (`Claim: ...`), `rows.tsv`, and `witnesses.tsv` early after first fluid/climb runs pass, then extend cleanly.

**Live scenes to execute:**
1. **Water & Lava travel (`LivingEntity.travelInFluid`):** water walking vs sprint-swimming (`0.8` vs `0.9` horizontal drag, `-0.003` falling snap, surface `0.3` step-out jump check when `horizontalCollision` + lifted box free), Depth Strider `1..3` lerp toward `0.546`, Dolphin's Grace `0.96`, Lava (`0.5` drag, `-0.02` gravity).
2. **Climbables (`handleRelativeFrictionAndCalculateMovement` + `handleOnClimbable`):** ladder / vine / scaffolding horizontal `±0.15` clamp, max `-0.15` descent vs `0.0` sneak-on-ladder hold, `+0.2` climb up against horizontal wall collision or jump.
3. **Bubble columns (`BubbleColumnBlock` `UP` vs `DOWN`):** inside submerged column vs open air above column (`1.8` max / `+0.7` up vs `-0.9` min / `-0.3` down, surface splash behavior).

**Write in your run folder:**
- `rows.tsv`: `rule_id	medium_or_block	gravity_hex	drag_hex	clamp_hex	scars	source`
- `witnesses.tsv`: `case_id	tick	x_hex	y_hex	z_hex	vx_hex	vy_hex	vz_hex	fluid_or_climb_state	events`
- `dossier.md` (first line `Claim: `), `reproduce.sh`, and `wiki.md`.

**Falsifier:** Any executed tick where fluid velocity hex, water-exit `0.3` jump, climb `±0.15` clamp, or bubble column velocity hex differs from `server-26.2.jar`.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
