# Vibe live verification: 26.2 zero-epsilon AABB face touch on walls & ceilings

**Goal:** Execute live 26.2 player AABB vs block shape boundary intersections across walls and ceilings (full blocks, bottom/top slabs, stairs, open/closed trapdoors, ladders, crouched `1.5` height) on `server-26.2.jar` to pin wall/ceiling B face-touch predicates.

See `lab/results/local-20261124-130147-11356-1-0-vibe-check-block-place-break/` for clean `DataComponentInitializers` / tag boot.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`; java/javac: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/`.

**Harness requirements (mandatory executed witnesses):**
- Boot tags (`MappedRegistry.prepareTagReload(...).apply()`) + `DataComponentInitializers` + Unsafe `ServerLevel` + real `LevelChunk` + real `ServerPlayer`.
- Inspect bytecode with `javap -c -p -constants` for `BlockStateBase.getCollisionShape`, `Entity.collide`, `AABB.intersects`, `TrapDoorBlock`, `SlabBlock`, `StairBlock`.
- Write `dossier.md` (`Claim: ...`), `rows.tsv`, and `witnesses.tsv` early, then extend cleanly.

**Live scenes to execute:**
1. **Wall face touch (East/West/North/South):** sprint `ServerPlayer` against stone wall `[10, -60, 0]` vs slab `[10, -60, 0] (type=bottom)` vs closed trapdoor; record exact stopped `(x, y, z)` IEEE bits, `horizontalCollision`, and block-face plane coordinates.
2. **Ceiling & jump touch (`Face::Down`):** jump from standing (`0.42`) and crouching (`1.5` eye/height vs `1.8`) directly beneath ceiling `[0, -58, 0]` (`stone`, `top slab`, `top closed trapdoor`); record apex `y` IEEE bits, `verticalCollision`, and `verticalCollisionAbove` flags.
3. **Tangent-open boundary:** position player at `z = 1.0 - 0.3` and `z = 1.0 + 0.3` edge against `[10, -60, 0]` verifying strict vs non-strict tangent intersection against `ShapeView` / `VoxelShape`.

**Write in your run folder:**
- `rows.tsv`: `case_id	block_id	face	pose	x_hex	y_hex	z_hex	h_col	v_col	scars	source`
- `witnesses.tsv`: `case_id	tick	x_hex	y_hex	z_hex	vy_hex	flags`
- `dossier.md` (first line `Claim: `), `reproduce.sh`, and `wiki.md`.

**Falsifier:** Any executed boundary coordinate or collision flag on wall or ceiling faces that disagrees with `server-26.2.jar`.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
