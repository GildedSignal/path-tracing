# Vibe live verification: 26.2 PvP melee attack, sprint-hit knockback, crits, shield stun, and mace smash

**Goal:** Execute live 26.2 `Player.attack`, attack strength cooldown scale, sprint-knockback `0.6x` XZ damp, crit multiplier, sweeping edge, `ShieldBlock`, mace smash bonus + fall reset, and `invulnerableTime` override on `server-26.2.jar`.

Supplies independent jar truth for `engine/law/src/combat.rs` and the two-regime PvP melee singularity (`map/detail/representation.md` §10). See `lab/results/local-20261124-130147-11356-1-0-vibe-check-block-place-break/` for clean boot.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`; java/javac: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/`.

**Harness requirements (mandatory executed witnesses):**
- Boot tags (`MappedRegistry.prepareTagReload(...).apply()`) + `DataComponentInitializers` + live `ServerLevel` + `LevelChunk` + two real `ServerPlayer` instances.
- Inspect `javap -c -p -constants` for `Player.attack`, `Player.getAttackStrengthScale`, `MaceItem.getAttackDamageBonus`, `LivingEntity.isDamageSourceBlocked`, and `LivingEntity.hurtServer`.
- Write `dossier.md` (`Claim: ...`), `rows.tsv`, and `witnesses.tsv` early, then extend cleanly.

**Live scenes to execute:**
1. **Sprint-hit & cooldown scaling:** full-charge vs half-charge diamond sword hit while sprinting (attacker velocity multiplied by `0.6`, sprint cleared!) vs walking vs mid-air crit (`!onGround && fallDistance > 0`).
2. **Invulnerable window (`hurtTime = 10 / invulnerableTime = 20`):** hit at `dt = 5` with lower damage (rejected) vs higher damage (applies difference `new - lastHurt`).
3. **Shield block angle + axe disable & mace smash:** facing shield block (`dot` angle threshold) vs 5s axe stun (`0.25` base + efficiency?), plus mace smash from `fallDistance = 10.0` (armor/density/fall reset).

**Write in your run folder:**
- `rows.tsv`: `rule_id	method	scale_or_threshold_hex	cooldown_ticks	scars	source`
- `witnesses.tsv`: `case_id	tick	attacker_vx_hex	target_vy_hex	damage_hex	blocked	invuln_ticks	events`
- `dossier.md` (first line `Claim: `), `reproduce.sh`, and `wiki.md`.

**Falsifier:** Any executed damage float bit, attacker `0.6x` velocity bit, knockback vector, shield block angle, or i-frame verdict that disagrees with `server-26.2.jar`.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
