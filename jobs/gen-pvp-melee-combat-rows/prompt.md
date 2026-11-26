# Generator rows: 26.2 Player.attack, sprint knockback, sweep, shield block, and hurtTime i-frames

**Goal:** Extract and execute live 26.2 `Player.attack`, cooldown damage scaling, sprint knockback + `0.6x` XZ damp, sweep, shield block cone, axe stun, and `invulnerableTime` damage override on `server-26.2.jar`.

Cross-checks Vibe's `vibe-gen-pvp-melee-sprint-knockback` for `engine/law/src/combat.rs` and the two-regime PvP melee singularity (`map/detail/representation.md` §10). See `lab/results/local-20261124-130147-11356-1-0-vibe-check-block-place-break/` for clean boot.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`; java/javac: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/`.

**Harness requirements:**
- Mandatory first boot step: `MappedRegistry.prepareTagReload(...).apply()` + `DataComponentInitializers` + live `ServerLevel` + `LevelChunk` + two real `ServerPlayer` instances.
- Inspect `javap -c -p -constants` for `Player.attack`, `Player.getAttackStrengthScale`, `LivingEntity.isDamageSourceBlocked`, and `LivingEntity.hurtServer`.
- Write `dossier.md` (`Claim: ...`), `rows.tsv`, and `witnesses.tsv` early, then extend cleanly.

**Cases & rules to extract and witness:**
1. **Sword/axe cooldown & sprint knockback:** strength scale at `0.0`, `0.5`, `1.0`, attacker `0.6x` velocity damp + sprint clear, crit vs sweep mutex.
2. **Shield block cone & `invulnerableTime = 20`:** exact dot threshold for block vs bypass, axe `100`-tick stun trigger, and higher-damage delta during `1..10` ticks after hit.

**Write in your run folder:**
- `rows.tsv`: `rule_id	method	scale_or_threshold_hex	cooldown_ticks	scars	source`
- `witnesses.tsv`: `case_id	tick	attacker_vx_hex	target_vy_hex	damage_hex	blocked	invuln_ticks	events`
- `dossier.md` (first line `Claim: `), `reproduce.sh`, and `wiki.md`.

**Falsifier:** Any executed damage float bit, attacker velocity bit, shield block cone verdict, or i-frame override that disagrees with `server-26.2.jar`.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
