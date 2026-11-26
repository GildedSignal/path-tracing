# Generator rows: 26.2 ThrownEnderpearl throw origin, drag, gravity, 20t cooldown, 5.0 damage, and landing offset

**Goal:** Extract and execute live 26.2 `EnderpearlItem.use`, `ThrowableProjectile.shootFromRotation` + thrower velocity, `ThrownEnderpearl.tick` (`0.99` air / `0.8` water drag, `0.03` gravity), `20`-tick cooldown, `5.0f` teleport damage, and block/entity impact resolution on `server-26.2.jar`.

Cross-checks Vibe's earlier pearl probe and validates `engine/law/src/pearl.rs` wormhole modeling (`map/detail/representation.md` §10). See `lab/results/local-20261124-130147-11356-1-0-vibe-check-block-place-break/` for clean boot.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`; java/javac: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/`.

**Harness requirements:**
- Mandatory first boot step: `MappedRegistry.prepareTagReload(...).apply()` + `DataComponentInitializers` + live `ServerLevel` + `LevelChunk` + real `ThrownEnderpearl` and `ServerPlayer`.
- Inspect `javap -c -p -constants` for `EnderpearlItem.use`, `Projectile.shootFromRotation`, `ThrowableProjectile.tick`, and `ThrownEnderpearl.onHit`.
- Write `dossier.md` (`Claim: ...`), `rows.tsv`, and `witnesses.tsv` early, then extend cleanly.

**Cases & rules to extract and witness:**
1. **Launch & seeded dispersion:** `45°` standing vs sprinting throw (`1.5f` speed, thrower `deltaMovement` inheritance on ground vs mid-air), `20`-tick cooldown.
2. **Flight & water transition:** ticks `0..40` through air (`0.99` drag, `-0.03` gravity) and into water (`0.8` drag, bubble column), plus exact landing teleport position, fall-distance reset (`fallDistance = 0`), and `5.0f` fall damage.

**Write in your run folder:**
- `rows.tsv`: `rule_id	method	drag_hex	gravity_hex	damage_or_cooldown	scars	source`
- `witnesses.tsv`: `case_id	tick	x_hex	y_hex	z_hex	vx_hex	vy_hex	vz_hex	events`
- `dossier.md` (first line `Claim: `), `reproduce.sh`, and `wiki.md`.

**Falsifier:** Any executed pearl position/velocity IEEE bit, landing teleport coordinate, cooldown, or `5.0f` damage verdict that disagrees with `server-26.2.jar`.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
