# Port delta: cauldrons, WitherRoseBlock, BigDripleafBlock and End portal/gateway entityInside hooks

**Goal:** the jar's exact rule for this, with a reproduction that runs against Mojang's 26.2 jar: `dossier.md` (first line `Claim: `), `reproduce.sh`, and the Rust port delta as `work.patch`.

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`.

**Consumer:** `engine/world/tools/ModelTable.java:36-41`, `engine/world/data/census.tsv` (remaining `entityInside` refusals: `LayeredCauldronBlock`, `LavaCauldronBlock`, `WitherRoseBlock`, `BigDripleafBlock`, `EndPortalBlock`, `EndGatewayBlock`), `engine/law/src/player.rs`, `engine/law/src/interact.rs`.

**From the 26.2 server bytecode**, state with class, method and offsets:
- `AbstractCauldronBlock.getEntityInsideCollisionShape` / `isEntityInsideContent` (to locate), `LayeredCauldronBlock.entityInside` (`water_cauldron` vs `powder_snow_cauldron`: `EXTINGUISH` + `handleEntityOnFireExtinguishedWithWater` -> `lowerFillLevel`?), and `LavaCauldronBlock.entityInside` (`LAVA_IGNITE` + `lavaHurt`?);
- `WitherRoseBlock.entityInside`: `isClientSide` / `Difficulty.PEACEFUL` gate, `hurtServer(wither(), 1.0f)`, and `addEffect(MobEffects.WITHER, 40, 0)` (only when `hurtServer` returns `true`?);
- `BigDripleafBlock.entityInside`, `getCollisionShape` (per `Tilt`: `NONE`, `UNSTABLE`, `PARTIAL`, `FULL` -> empty!), and `setTiltAndScheduleTick` delays (`UNSTABLE -> 10 -> PARTIAL -> 10 -> FULL -> 100 -> NONE`?);
- `EndPortalBlock.entityInside` and `EndGatewayBlock.entityInside`: exact `getEntityInsideCollisionShape` (`[0, 6/16, 0]..[1, 12/16, 1]` for `EndPortalBlock` vs `Shapes.block()` for `EndGatewayBlock`?), `canUsePortal(false)`, and immediate `setAsInsidePortal(this, pos)` (`getPortalWaitTime = 0`).

**Build a jar-backed reproduction:** run `reproduce.sh` disassembling all 6 classes above and running a jar-backed harness printing: (1) cauldron inside-content Y thresholds for levels 1..=3 and lava, (2) `BigDripleafBlock` collision shapes across all 4 `Tilt` values and tick delays, and (3) `WitherRoseBlock` Peaceful vs Easy damage/effect application. Write `dossier.md` starting with `Claim: ` on its first physical line, and produce the Rust port delta as `work.patch`.

**Falsifier:** any cauldron content height, dripleaf tilt delay/collision shape, or wither-rose Peaceful/cooldown gate that disagrees with the 26.2 bytecode.
