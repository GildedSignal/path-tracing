# Vibe generator rows: velocity kicks, knockback, explosions, wind charge, and mounts

**Goal:** One bit- and server-exact parameter/scar table for 26.2 `Kicks` (`v' = a*v + b*dir` on `body`: melee knockback, sprint-hit extra knockback, punch, wind burst, explosion push, riptide, shulker push) and entity mounts (boat status/paddles/buoyancy, horse, pig, strider, camel).

Supplies group 12 (`Kicks`) and the mount rows of group 2 (`Body`) from `jobs/law-grouping-audit/table.md`.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`.

**Harness requirements:**
- Bind real jar tags via `MappedRegistry.prepareTagReload(...).apply()`.
- Drive real `ServerLevel`, `LevelChunk` (`hasFluid()` intact!), `LocalPlayer` (`isLocalPlayer() => true` + `isLocalClientAuthoritative() => true`), `AbstractBoat`, `Horse`, `Pig`, `Strider`, and `Camel`.
- Write `dossier.md` (`Claim: ...`), `rows.tsv`, and `witnesses.tsv` early after core bytecode + primary witnesses pass, then extend cleanly.

**From the bytecode** (`class.method@offset`):
1. **Kicks (`LivingEntity.knockback`, `Player.attack`, `Explosion`, `WindCharge` / `WindBurst`, `Player.releaseUsing` Riptide, `Shulker` push):** exact `v' = a*v + b*dir`, NaN-safety early exit on knockback, knockback resistance multiplier `1.0 - resistance`, vertical cap `min(0.4, vy/2 + 0.4*strength)`, explosion inverse-distance scaling + eye-ray exposure (`getSeenPercent`), and wind-charge `hasImpulse` / velocity delta.
2. **Boats (`AbstractBoat`, `Boat` / `Raft`):** `getStatus` (`IN_WATER`, `UNDER_WATER`, `UNDER_FLOWING_WATER`, `ON_LAND`, `IN_AIR`), ground friction scan across supporting AABB (`getGroundFriction`), buoyancy vertical force constants (`invFriction`, `waterLevel`, `landFriction`), and paddle turning/forward thrust bits.
3. **Living mounts (`AbstractHorse`, `ItemBasedSteering`, `Pig`, `Strider`, `Camel`):** ridden input transform, attribute speed scaling, horse jump power & `0.4`..`1.0` scale, strider lava boost/cold scar, and camel dash cooldown (`55` ticks).

**Write in your run folder:**
- `rows.tsv`: `rule_id	kind	a_hex	b_hex	cap_or_formula	scars	source`
- `witnesses.tsv`: `case_id	tick	entity	x_hex	y_hex	z_hex	vx_hex	vy_hex	vz_hex	status_or_flags	events`
- `dossier.md` (first line `Claim: `), `reproduce.sh`, and `wiki.md`.

**Falsifier:** Any witness tick where post-kick velocity bits, explosion exposure, boat status/friction, or mount movement bits differ from the 26.2 jar.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
