# Generator rows: water and lava travel, climbables, bubble columns, and falling columns

**Goal:** One bit-exact parameter/scar table for 26.2 fluid locomotion (`travelInWater`, `travelInLava`, swim pose, water-exit `0.3` jump, depth strider / dolphin's grace / soul speed), climbables (`ladder`/`vine`/`scaffolding`), bubble columns, and falling fluid drop columns.

Closes open limits from `gen-fluid-bucket-rows` (falling liquid columns) and supplies groups 2, 6, and 9 (`body`, `contact`, fluids) of `jobs/law-grouping-audit/table.md`.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`.

**Harness requirements:**
- Bind real jar tags via `MappedRegistry.prepareTagReload(...).apply()` (`#climbable`, `#impermeable`, fluid tags).
- Use a real `LevelChunk` (`hasFluid()` intact!) with `LevelTicks<Fluid>` for fluid spread and real `LocalPlayer` (or `isLocalPlayer() => true` + `isLocalClientAuthoritative() => true`) for movement ticks.
- Write `dossier.md` (`Claim: ...`), `rows.tsv`, and `witnesses.tsv` early after core bytecode + initial runs succeed, then extend cleanly.

**From the bytecode** (`class.method@offset`):
1. **Fluid travel & exit jump:** `LivingEntity.travelInWater` / `travelInLava` (gravity `0.02 / 4.0` / `-0.003` snap, drag `0.8` / `0.9` / `0.5`, depth strider lerp, dolphin's grace `0.96` vs normal, liquid current push `0.014`/`0.007`/`0.002333...`, jump-out-of-water `+0.3` when lifted box fits).
2. **Climbables:** `LivingEntity.handleRelativeFrictionAndCalculateMovement` + `handleOnClimbable` + `LivingEntity.onClimbable` — horizontal clamp `[-0.15, 0.15]`, `-0.15` descent floor vs `0.0` hold while sneaking on ladder/scaffolding, auto-climb `+0.2` on horizontal collision or jump (`trapdoor` paired with ladder rule), fall-distance reset (`LivingEntity.onClimbable` vs `FALLDAMAGE_RESETTING` discrepancy caught in `gen-fluid-bucket-rows`).
3. **Bubble columns & falling fluid column:** `BubbleColumnBlock` upward/downward `drag` column vs surface open-top acceleration (`1.8` / `0.7` / `-0.9` / `-0.3`), plus `FlowingFluid.spread` downward falling-column scene (`getNewLiquid@185-254` witnessed over a 1-block ledge drop).

**Write in your run folder:**
- `rows.tsv`: `rule_id	medium_or_block	gravity_hex	drag_hex	clamp_hex	scars	source`
- `witnesses.tsv`: `case_id	tick	x_hex	y_hex	z_hex	vx_hex	vy_hex	vz_hex	fluid_or_climb_state	events`
- `dossier.md` (first line `Claim: `), `reproduce.sh`, and `wiki.md`.

**Falsifier:** Any tick where velocity hex bits, water-exit jump y-delta, climb speed clamp, or falling-column `FluidState` level disagrees with the 26.2 jar.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
