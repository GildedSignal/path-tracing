# Vibe live verification: 26.2 bow arrow flight, slime-piston bounce, and player knockback impulse

**Goal:** Execute live 26.2 `BowItem` charge release, `AbstractArrow` ballistic flight (`0.99` drag, `-0.05` gravity), `PistonMovingBlockEntity` slime-block bounce transfer, and `Player` arrow knockback `Δv` + `hurtTime`/`invulnerableTime` on `server-26.2.jar`.

Supplies exact 4D rendezvous ground truth for multi-chain causal stunts (shooting arrow + triggering redstone piston bounce + catching arrow knockback mid sprint-jump; `map/detail/representation.md` §5). See `lab/results/local-20261124-130147-11356-1-0-vibe-check-block-place-break/` for tag/registry boot.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`; java/javac: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/`.

**Harness requirements (mandatory executed witnesses):**
- Boot tags (`MappedRegistry.prepareTagReload(...).apply()`) + `DataComponentInitializers` + live `ServerLevel` + `LevelChunk` + real `Arrow`, `PistonMovingBlockEntity`, and `ServerPlayer`.
- Inspect `javap -c -p -constants` for `BowItem.releaseUsing`, `Projectile.shootFromRotation`, `AbstractArrow.tick`, `SlimeBlock.updateEntityMovementAfterFallOn`, `PistonMovingBlockEntity.moveCollidedEntities`, and `LivingEntity.knockback` / `hurtServer`.
- Write `dossier.md` (`Claim: ...`), `rows.tsv`, and `witnesses.tsv` early, then extend cleanly.

**Live scenes to execute:**
1. **Bow draw charge curve:** ticks `3`, `10`, `20` (`power` formula, exact base speed, seeded zero-inaccuracy launch `(vx, vy, vz)` with thrower movement coupling).
2. **Moving slime-block piston bounce:** extending slime piston colliding with airborne `Arrow` and falling `ServerPlayer` across progress `0.0 -> 0.5 -> 1.0` — record bounced velocity `(vx, vy, vz)` and exact impact tick.
3. **Player arrow catch `Δv` & hurt-time window:** arrow hitting sprinting airborne player (`punch` `0` vs `1` vs `2`, health delta, exact knockback `(vx, vy, vz)` double bits, `invulnerableTime = 20`, same/higher-damage override during ticks `1..10`).

**Write in your run folder:**
- `rows.tsv`: `rule_id	method	coeff_or_impulse_hex	tick_delay	hurt_ticks	scars	source`
- `witnesses.tsv`: `case_id	tick	entity_kind	x_hex	y_hex	z_hex	vx_hex	vy_hex	vz_hex	events`
- `dossier.md` (first line `Claim: `), `reproduce.sh`, and `wiki.md`.

**Falsifier:** Any arrow velocity bit, piston slime bounce tick, or player knockback `Δv` vector that disagrees with `server-26.2.jar`.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
