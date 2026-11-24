# Vibe generator rows: trident riptide, bow/crossbow projectiles, and wind charges

**Goal:** One bit- and server-exact parameter/scar table for 26.2 `TridentItem` riptide (`1/2/3`), `BowItem` / `CrossbowItem` / `AbstractArrow`, and `WindChargeItem` / `AbstractWindCharge` (including the 40-tick `ignoreFallDamageFromCurrentImpulse` window) executed headless against `server-26.2.jar`.

Supplies `Kicks` and `Body` projectile rows (`jobs/law-grouping-audit/table.md` groups 2 & 12) using real `ItemStack` components and executed `ServerLevel` + `LevelChunk` runs.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`.

**Harness requirements (mandatory executed witnesses, never bytecode-only):**
- Boot tags (`MappedRegistry.prepareTagReload(...).apply()`) AND item components (`RegistryDataLoader.load(..., SYNCHRONIZED_REGISTRIES)` + `BuiltInRegistries.DATA_COMPONENT_INITIALIZERS.build(provider).apply()`, see `C:/Users/PASCAL/Desktop/path tracing/lab/results/local-20261124-130147-11356-1-0-vibe-check-block-place-break/`).
- Use Unsafe-allocated `ServerLevel` + real `LevelChunk` (`ChunkPos(0,0)`), real `AbstractArrow`/`WindCharge`/`ThrownTrident`, and `ServerPlayer`/`LivingEntity`.
- Write `dossier.md` (`Claim: ...`), `rows.tsv`, and `witnesses.tsv` early once core executed runs pass, then extend cleanly.

**From bytecode + live execution:**
1. **Riptide (`TridentItem.releaseUsing` / `Entity.setRiptide` / `Player.startAutoSpinAttack`):** velocity kick formula by Riptide level `1,2,3`, `20`-tick spin window, pitch/yaw trig, in-water/rain gate, collision spin damage.
2. **Bow & Crossbow (`BowItem.releaseUsing`, `CrossbowItem`, `AbstractArrow.tick`, `Projectile.shoot`):** draw-tick power curve `(t^2 + 2t)/3`, `20` max charge, `crit` flag, arrow gravity `0.05` vs water drag `0.6` vs air drag `0.99`, zero-randomness `(seed/deterministic)` velocity launch, `inGround` stop, and base/power damage.
3. **Wind Charge & Wind Burst (`WindChargeItem`, `AbstractWindCharge`, `WindCharge.explode`, `LivingEntity` impulse grace):** projectile zero-gravity + fixed acceleration `0.1`, explosion knockback `Explosion` radius `1.2`, upward velocity boost, and exact `ignoreFallDamageFromCurrentImpulse` + `currentExplosionImpactPos` + `40`-tick / landing reset rules.

**Write in your run folder:**
- `rows.tsv`: `rule_id	kind	constants_hex	formula_order	scars	source`
- `witnesses.tsv`: `case_id	tick	entity	x_hex	y_hex	z_hex	vx_hex	vy_hex	vz_hex	fall_dist_or_dmg_hex	events`
- `dossier.md` (first line `Claim: `), `reproduce.sh`, and `wiki.md`.

**Falsifier:** Any divergence in post-riptide velocity bits, arrow trajectory hex bits, wind-charge boost, or fall-damage reset tick against `server-26.2.jar`.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
