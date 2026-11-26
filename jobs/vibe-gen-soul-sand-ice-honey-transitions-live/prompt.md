# Vibe live verification: 26.2 Soul Sand, Ice, Honey, Slime surface transitions

**Goal:** Execute live 26.2 `Block.getSpeedFactor`, `Block.getJumpFactor`, `Block.getFriction`, `HoneyBlock.entityInside`, `SlimeBlock.updateEntityMovementAfterFallOn`, and `SoulSpeed` / `BlockPosBelowThatAffectsMyMovement` lookups on `server-26.2.jar`.

Supplies bit-exact ground-truth transitions across special friction/bounce/wall-slide blocks for 26.2 law certification. See `lab/results/local-20261124-130147-11356-1-0-vibe-check-block-place-break/` for clean `DataComponentInitializers` / tag boot.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`; java/javac: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/`.

**Harness requirements (mandatory executed witnesses):**
- Boot tags (`MappedRegistry.prepareTagReload(...).apply()`) + `DataComponentInitializers` + Unsafe `ServerLevel` + real `LevelChunk` + real `ServerPlayer`.
- Inspect bytecode with `javap -c -p -constants` for `Entity.getBlockPosBelowThatAffectsMyMovement`, `Entity.getBlockSpeedFactor`, `LivingEntity.getBlockJumpFactor`, `HoneyBlock`, `SlimeBlock`, `SoulSandBlock`, and `IceBlock`/`BlueIce`.
- Write `dossier.md` (`Claim: ...`), `rows.tsv`, and `witnesses.tsv` early, then extend cleanly.

**Live scenes to execute:**
1. **Speed & friction table:** sprint 8 ticks onto `soul_sand`, `honey_block`, `ice`, `packed_ice`, `blue_ice`, `slime_block`, recording exact block `friction`, `speedFactor`, `jumpFactor`, and `u64` IEEE bits for `(x, y, z, vx, vy, vz)` on the transition tick.
2. **Honey wall slide & reduced jump:** fall while horizontally pressed against `honey_block` (`vy = -0.13` slide clamp / velocity cap) and jump off `honey_block` (`0.5` jumpFactor).
3. **Slime block bounce & sneak suppress:** fall onto `slime_block` from `y+5` standing vs `shiftKeyDown = true` (bounce reflection vs non-bounce + fall damage).

**Write in your run folder:**
- `rows.tsv`: `block_id	friction_hex	speed_factor_hex	jump_factor_hex	y_affect_offset	scars	source`
- `witnesses.tsv`: `case_id	tick	x_hex	y_hex	z_hex	vx_hex	vy_hex	vz_hex	events`
- `dossier.md` (first line `Claim: `), `reproduce.sh`, and `wiki.md`.

**Falsifier:** Any executed tick where friction, speed/jump factor, honey wall-slide speed, or slime bounce velocity IEEE bits disagrees with `server-26.2.jar`.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
