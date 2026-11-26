# Generator rows: 26.2 Bow, AbstractArrow, slime piston bounce, and arrow knockback impulse

**Goal:** Extract and execute live 26.2 `BowItem.releaseUsing`, `AbstractArrow` flight (`0.99` drag, `-0.05` gravity), `PistonMovingBlockEntity` slime-block bounce, and `Player` arrow knockback `Δv` + `invulnerableTime` on `server-26.2.jar`.

Cross-checks Vibe's `vibe-gen-arrow-slime-piston-rendezvous` from a second model family for multi-chain 4D rendezvous stunts (`map/detail/representation.md` §5). See `lab/results/local-20261124-130147-11356-1-0-vibe-check-block-place-break/` for clean boot.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`; java/javac: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/`.

**Harness requirements:**
- Mandatory first boot step: `MappedRegistry.prepareTagReload(...).apply()` + `DataComponentInitializers` + live `ServerLevel` + `LevelChunk` + real `Arrow`, `PistonMovingBlockEntity`, and `ServerPlayer`.
- Inspect `javap -c -p -constants` for `BowItem.releaseUsing`, `AbstractArrow.tick`, `SlimeBlock.updateEntityMovementAfterFallOn`, `PistonMovingBlockEntity.moveCollidedEntities`, and `LivingEntity.knockback` / `hurtServer`.
- Write `dossier.md` (`Claim: ...`), `rows.tsv`, and `witnesses.tsv` early, then extend cleanly.

**Cases & rules to extract and witness:**
1. **`BowItem` draw power & thrower coupling:** exact `float` power curve at draw ticks `3..20+`, base `3.0f` multiplier, arrow tick gravity `-0.05` & air/water drag (`0.99` / `0.6`).
2. **Slime piston bounce vs sticky drag:** `Arrow` and `ServerPlayer` colliding with extending slime-block piston across progress `0.0 -> 0.5 -> 1.0`.
3. **Player hit knockback & i-frames:** arrow (`punch 0/1/2`) hitting airborne/sprinting `ServerPlayer` -> exact IEEE `(vx, vy, vz)` knockback bits, health loss, and `invulnerableTime = 20` override window.

**Write in your run folder:**
- `rows.tsv`: `rule_id	method	coeff_or_impulse_hex	tick_delay	hurt_ticks	scars	source`
- `witnesses.tsv`: `case_id	tick	entity_kind	x_hex	y_hex	z_hex	vx_hex	vy_hex	vz_hex	events`
- `dossier.md` (first line `Claim: `), `reproduce.sh`, and `wiki.md`.

**Falsifier:** Any arrow velocity bit, piston slime bounce tick, or player knockback `Δv` bit pattern that disagrees with `server-26.2.jar`.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
