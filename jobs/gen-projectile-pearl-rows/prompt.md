# Generator rows: ender pearl & throwable wormhole edge

**Goal:** One parameter/scar table for 26.2 ender pearl launch, flight ray-clip, teleport landing, cooldown, and damage mitigation for the `body`, `ray`, `clock`, and `ledger` generators, backed by bit-exact jar witnesses.

An ender pearl is a wormhole edge in the kit (`map/architecture.md` §2; speed `1.5`, drag `0.99`, gravity `0.03`, `5.0` fall-damage toll, `20`-tick cooldown). This job extracts the complete jar order from use-click to landing telemetry.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`.

**From the bytecode** (class, method, offsets):
- **Launch (`EnderpearlItem.use`, `Projectile.shootFromRotation`):** spawn origin `(x, eyeY - 0.10000000149011612, z)` (verify exact `ThrownEnderpearl` constructor), `Mth.sin`/`Mth.cos` direction vector from player `(xRot, yRot)`, speed multiplier `1.5f`, inaccuracy `1.0f` Gaussian spread (`getTriangle` vs `RandomSource.triangle`), shooter velocity inheritance (`shootFromRotation` subtracts or adds `onGround ? 0 : ...`), and `ItemCooldowns` (`20` ticks check order).
- **Flight & collision (`ThrownEnderpearl.tick`, `ThrowableProjectile.tick`, `ProjectileUtil.getHitResultOnMoveVector`):** exact order of ray-clip (`ClipContext.Block.COLLIDER`, `ClipContext.Fluid.NONE`), entity hit check vs block hit check, drag (`0.99f` air / `0.8f` water), gravity (`0.03`), and portal/unloaded chunk scar behavior.
- **Teleport & damage (`ThrownEnderpearl.onHit`):** `player.teleportTo` target coordinates (`hitResult.getLocation()`), `resetFallDistance()`, `hurtServer(level.damageSources().enderPearl(), 5.0f)` mitigation flags (armor bypassed? feather falling? protection EPF? resistance?), Endermite spawn roll order relative to RNG, and pearl despawn when shooter is dead/removed.

**Witnesses:** Run a tiny Java harness against the 26.2 jar covering:
1. Seeded throws at yaws `{0, 45, 90}`, pitches `{-45, 0, -20}`, standing vs sprinting vs jumping, in air and into water, across `60` ticks each until block impact.
2. Teleport destination bits, post-hit `fallDistance` bits, cooldown timer ticks, and health/mitigation bits with bare player vs Feather Falling IV vs Protection IV vs Resistance I.

**Write in your run folder:**
- `rows.tsv`: `phase_or_rule	params_hex	rng_order	damage_flags	cooldown_ticks	scars	source` (`source` = `class.method@offset`).
- `witnesses.tsv`: `case_id	seed	tick	x_hex	y_hex	z_hex	vx_hex	vy_hex	vz_hex	hit_x_hex	hit_y_hex	hit_z_hex	health_hex	cooldown`.
- `dossier.md` (first line `Claim: `), `reproduce.sh` regenerating both tables from the jar, and `wiki.md`: concise `lab/wiki/` note on pearl launch bits, inaccuracy variance, and true wormhole toll.

**Falsifier:** Any witness tick or landing where our replay produces different position/velocity bits, hit location bits, or health/cooldown bits than the jar.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
