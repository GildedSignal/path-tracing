# Port delta: Creative mining, inactive STOP, and mining reach gate

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`. `port-breaking-threshold`, `port-break-stop-boundary`, `port-mining-abort-target`, and `port-mining-target-switch` already settled survival START `>=1.0f`, matching STOP `>=0.7f`, ABORT, and target switch; do not re-ask those.

**Consumer:** `engine/law/src/mining.rs:99-102` (explicitly defers Creative START, air START, inactive STOP, and external eligibility/reach gates), `docs/plan.md` step 5 (`P2c`).

**From the 26.2 server bytecode**, state with class, method and offsets:
- `ServerGamePacketListenerImpl.handlePlayerAction` (to locate) -> `ServerPlayerGameMode.handleBlockBreakAction`: the exact reach check (`canInteractWithBlock`, `blockInteractionRange` + slack), `blockActionRestricted` (Adventure/Spectator/`mayBuild`), and order before `START`/`STOP`/`ABORT`;
- `handleBlockBreakAction` `START_DESTROY_BLOCK` when `isCreative()` is true (to locate): whether it ignores `getDestroyProgress` (including bedrock hardness `-1.0f`!) and calls `destroyAndAck` immediately;
- `handleBlockBreakAction` `START_DESTROY_BLOCK` when `state.isAir()` is true, and `STOP_DESTROY_BLOCK` when `pos` does NOT equal `destroyPos` (or `hasDelayedDestroy` is active on `pos` or another pos).

**Build a jar-backed reproduction:** run `reproduce.sh` disassembling `ServerGamePacketListenerImpl.handlePlayerAction` and the Creative/air/reach/inactive-STOP branches of `ServerPlayerGameMode.handleBlockBreakAction`, and execute direct jar probes for (1) Creative `START` on stone & bedrock, (2) `STOP` on a non-matching pos while mining or delaying, and (3) `START` at exact `blockInteractionRange + 1.0` boundary. Write `dossier.md` starting with `Claim: ` on its first physical line.

**Falsifier:** any reach threshold, Creative hardness check, or non-matching `STOP` state transition that disagrees with the 26.2 bytecode.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
