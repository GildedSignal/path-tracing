# Generator rows: fluids, buckets, and fall-damage resets

**Goal:** One parameter/scar table covering 26.2 bucket place/pickup, fluid spread cadence, entity fluid push/buoyancy, and fall-damage reset surfaces for the `cell`, `clock`, and `body` generators, backed by bit-exact jar witnesses.

Water/lava buckets and landing clutches are tools that enter the law without special search code (`map/architecture.md` §2). This job extracts every exact parameter, fluid current vector, and clutch reset rule from the jar.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`.

**From the bytecode** (class, method, offsets):
- **Bucket use (`BucketItem.use`, `emptyContents`, `pickupBlock`):** item ray-clip reach (`ClipContext.Fluid.NONE` vs `SOURCE_ONLY`), waterlogging when placing into slabs/stairs/fences vs replacing air/replaceable blocks, Nether vaporization scar.
- **Fluid spread & flow (`FlowingFluid`, `WaterFluid`, `LavaFluid`):** tick delay (`5` water, `30` overworld lava, `10` nether lava), source creation rule (`canConvertToSource`: water yes, lava no unless game rule), slope-search drop-off (`getSlopeFindDistance`: water `4`, overworld lava `2`, nether `4`), falling fluid height quantization (`getHeight`).
- **Fluid force on entities (`Entity.updateFluidHeightAndDoFluidPushing`, `Flowable` flow vector):** flow acceleration (`0.014` water, `0.0023333333...` / `0.007` lava) and buoyancy / immersion thresholds.
- **Fall-damage reset & reduction surfaces (`Block.fallOn`, `Entity.checkFallDamage`, `LivingEntity.causeFallDamage`):** exact reset/multiplier rules for water (source + flowing + 1-tick depth vs ankle), lava, powder snow, cobweb, sweet berry bush, hay (`0.2f`), bed (`0.5 * dist`), slime (`0.0f` unless suppressing bounce), honey (`0.2f`), boat dismount/in-boat, and wind charge/ladder landing.

**Witnesses:** Run a tiny Java harness against the 26.2 jar covering:
1. Bucket empty/pickup reach & ray clip against solid/waterlogged cells.
2. Water and lava flow propagation over 40 ticks on flat floor and toward a 1-block drop.
3. Fall impacts onto every reset/mitigation surface from `10.0` and `25.0` block drops recording exact `fallDistance`, post-landing `vy_hex`, and damage bits.

**Write in your run folder:**
- `rows.tsv`: `surface_or_rule	kind	delay_or_spread	push_scale_hex	fall_mult_hex	resets_fall	scars	source` (`source` = `class.method@offset`).
- `witnesses.tsv`: `case_id	tick	fluid_level	fall_dist_hex	y_hex	vy_hex	damage_hex	in_water	on_ground`.
- `dossier.md` (first line `Claim: `), `reproduce.sh` regenerating both tables from the jar, and `wiki.md`: concise `lab/wiki/` summary of bucket reach, fluid tick cadences, and exact MLG clutch requirements.

**Falsifier:** Any witness tick or drop where the rows give different spread levels, flow velocity bits, or fall-damage / `fallDistance` bits than the jar.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
