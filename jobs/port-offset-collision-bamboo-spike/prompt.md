# Port delta: position-dependent XZ offset collision (BambooStalkBlock and SulfurSpikeBlock)

**Goal:** the jar's exact rule for this, with a reproduction that runs against Mojang's 26.2 jar: `dossier.md` (first line `Claim: `), `reproduce.sh`, and the Rust port delta as `work.patch`.

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`.

**Consumer:** `engine/world/tools/ModelTable.java:112` (`offset collision` refusal on `minecraft:bamboo` and `minecraft:sulfur_spike` in `engine/world/data/census.tsv`), `engine/law/src/world.rs`, `engine/law/src/shapes.rs`, `map/now.md` S38.

**From the 26.2 server bytecode**, state with class, method and offsets:
- `net.minecraft.world.level.block.state.BlockBehaviour$BlockStateBase.getOffset(BlockPos)` and `BlockBehaviour$Properties.offsetType(OffsetType)` (to locate): exact hash `Mth.getSeed(x, 0, z)` (`(x * 3129871) ^ (z * 116129781L)` -> `l = l * l * 42317861L + l * 11L; l >> 16`?), `f32`/`f64` conversion order `(((double)((float)(l & 15L) / 15.0F) - 0.5D) * 0.5D)`, and `Mth.clamp(..., -maxHorizontalOffset, maxHorizontalOffset)`;
- `net.minecraft.world.level.block.BambooStalkBlock` and `net.minecraft.world.level.block.SulfurSpikeBlock` (extending `SpeleothemBlock`): exact un-offset collision shapes (`getCollisionShape`), `getMaxHorizontalOffset()` (`0.25f` vs `0.125f`?), and any `SulfurSpikeBlock` / `SpeleothemBlock` `fallOn` / `entityInside` / `updateShape` rules in 26.2.

**Build a jar-backed reproduction:** run `reproduce.sh` disassembling `BlockBehaviour$BlockStateBase.getOffset`, `Mth.getSeed`, `BambooStalkBlock`, `SulfurSpikeBlock`, and `SpeleothemBlock`, plus a jar-backed execution printing raw `u64` `(dx, dy, dz)` and shifted `AABB` bits for `bamboo` and `sulfur_spike` at 10 `(x, z)` coordinates (including negative and large coordinates). Write `dossier.md` starting with `Claim: ` on its first physical line, and produce the Rust port delta (`get_offset(x, z, max_h)` in `engine/law`) as `work.patch`.

**Falsifier:** any 1-ulp disagreement between the Rust `get_offset` formula and the 26.2 jar's `BlockStateBase.getOffset(BlockPos)` across the 10 `(x, z)` test coordinates.
