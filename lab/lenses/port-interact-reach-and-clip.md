# Port delta: block interaction range, eye-raycast pick and BlockPlaceContext

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`, the client jar at `D:/path-tracing-deps/minecraft-26.2-client.jar`, and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`. `place-reach-legality` already settled `ServerGamePacketListenerImpl.handleUseItemOn`'s squared-distance server check; do not re-ask that.

**Consumer:** `docs/plan.md` step 5 (deriving legal click targets & `BlockHitResult` from player `(pos, pose, yaw, pitch)`), `engine/law/src/blocks.rs:187` (`traverse_blocks`).

**From the 26.2 client and server bytecode**, state with class, method and offsets:
- `Player.blockInteractionRange()` and `Attributes.BLOCK_INTERACTION_RANGE` (to locate): base value, range, and `ServerPlayer.canInteractWithBlock(BlockPos, double)` (exact `1.0` server tolerance / AABB distance squared);
- `GameRenderer.pick` / `Entity.pick` / `Projectile.getHitResult` (to locate): exact raycast origin (`Entity.getEyePosition` = `(x, y + getEyeHeight(pose), z)` in float->double), direction (`getViewVector(1.0f)` via `Mth.sin/cos`), `ClipContext.Block.OUTLINE` vs `ClipContext.Fluid.NONE`, and `Level.clip` tie-breaking on corner/edge hits;
- `BlockPlaceContext` constructor from `UseOnContext` (to locate): `getNearestLookingDirections()` / `getHorizontalDirection()` (`orderedByNearest`) used by `getStateForPlacement`.

**Build a jar-backed reproduction:** run `reproduce.sh` disassembling `Player.blockInteractionRange`, `ServerPlayer.canInteractWithBlock`, `Entity.pick`, and `BlockPlaceContext.getNearestLookingDirections`, and invoke `Entity.pick` / `Level.clip` and `canInteractWithBlock` directly across standing (`eye=1.62f`), crouching (`1.27f`), and crawling (`0.4f`) poses at the exact `4.5` vs `5.5` reach boundary. Write `dossier.md` starting with `Claim: ` on its first physical line.

**Falsifier:** any eye-height cast, `BLOCK_INTERACTION_RANGE` default/slack, or `orderedByNearest` tie-break that disagrees with the 26.2 bytecode.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
