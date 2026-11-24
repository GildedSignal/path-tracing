# Generator rows: entity push, cramming, cell hooks, and mounts

**Goal:** One parameter/scar table for 26.2 body–body `push` / cramming, body–cell `cells_along` hook dispatch order, and boat/minecart mount rules for the `contact` generator and ES `Battle` transitions, backed by bit-exact jar witnesses.

Multi-bot movement and `wt/port-fold`'s missing `contact` generator (`map/now.md`, `jobs/law-grouping-audit/table.md` groups 6 & 11) depend on exact pair pushes (`Entity.push`), entity cramming caps, canonical swept `cells_along` ordering, and boat/minecart mounts.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`.

**From the bytecode** (class, method, offsets):
- **Body–body push (`LivingEntity.pushEntities`, `Entity.push(Entity)`, `Entity.isPushable`):** AABB inflation `(0.2, 0.0, 0.2)`, `maxEntityCramming` (`24` default) check & `cramming` `DamageSource` (`6.0f`/tick? verify!), `push(Entity)` axis delta math (`Mth.absMax`, scale factor `0.05` + min threshold `0.009999999776482582`, division by max vs double arithmetic, `hasImpulse = true`), scoreboard collision rules, and passenger/vehicle exemption.
- **Body–cell sweep hooks (`Entity.applyEffectsFromBlocks`, `Entity.checkInsideBlocks`):** canonical voxel traversal order (`cells_along` / AABB step ordering), order of `stepOn`, `fallOn`, and `entityInside` per move tick, and repeat-suppression when crossing shared faces.
- **Mounts (`AbstractBoat.interact`, `AbstractMinecart.interact`, `LivingEntity.dismountVehicle`):** interact reach, mounting position/velocity reset, boat passenger fall-damage immunity, and candidate safe-dismount offset search order around the vehicle.

**Witnesses:** Run a tiny Java harness against the 26.2 jar covering:
1. Overlapping `ServerPlayer` / `LivingEntity` pairs at exact offsets (`0.0`, `0.005`, `0.1`, `0.5`) recording pre/post `vx/vz` hex bits, plus a `25`-entity cramming cluster verifying exact cramming damage tick and amount.
2. Multi-cell oblique moves crossing magma + soul sand + cobweb / sweet berry / pressure plate recording exact hook fire order and velocity/health bits.
3. Boat mount & blocked-side dismount searches recording exact passenger coordinates.

**Write in your run folder:**
- `rows.tsv`: `rule_id	kind	aabb_inflate_hex	thresholds_hex	cramming_limit	damage_hex	order_rule	scars	source`.
- `witnesses.tsv`: `case_id	tick	entity_id	x_hex	y_hex	z_hex	vx_hex	vy_hex	vz_hex	health_hex	events`.
- `dossier.md` (first line `Claim: `), `reproduce.sh` regenerating both tables from the jar, and `wiki.md`: concise `lab/wiki/` summary of `Entity.push` determinism, cramming bounds, and safe dismount offsets.

**Falsifier:** Any witness tick where the row model emits different post-push velocity bits, hook invocation order, cramming damage, or dismount coordinates than the jar.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
