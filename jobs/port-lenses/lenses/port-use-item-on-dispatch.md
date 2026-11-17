# Port delta: ServerPlayerGameMode.useItemOn block-vs-item dispatch and client prediction

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`, client jar in `D:/path-tracing-deps/data`, and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`. `place-reach-legality` already settled `ServerGamePacketListenerImpl.handleUseItemOn` reach and border gates; start inside `ServerPlayerGameMode.useItemOn` (to locate).

**Consumer:** `docs/plan.md` step 5 (`P2c`), `ametwoodbox`, `doorcorridor`, `doorparkourblue`, `trapdoorparkour`, `amertystcoral`, and `port-action-resources` (`TryEmptyHandInteraction` open lead).

**From the 26.2 server and client bytecode**, state with class, method and offsets:
- `ServerPlayerGameMode.useItemOn` (to locate): the exact ordered branches between `BlockState.useItemOn`, `tryEmptyHandInteraction` / `BlockState.useWithoutItem`, the sneaking-with-item bypass (`isSecondaryUseActive` / `doesSneakBypassUse`, to locate), and `ItemStack.useOn` (`BlockItem.place`);
- `MultiPlayerGameMode` (client jar, to locate): whether the client runs the same dispatch locally on the click tick (`performUseItemOn`, to locate) so block state / collision shape updates on the client tick immediately or waits for the server packet.

**Build a jar-backed reproduction:** run `reproduce.sh` disassembling `ServerPlayerGameMode.useItemOn` and `MultiPlayerGameMode`'s counterpart, with a minimal witness contrasting: (1) empty hand clicking a trapdoor/door, (2) held block clicking a trapdoor standing vs sneaking, and (3) main-hand fail falling through to off-hand or `useItemOn`. Write `dossier.md` starting with `Claim: ` on its first physical line.

**Falsifier:** any dispatch branch order, sneaking bypass condition, or client-prediction flag that disagrees with the 26.2 bytecode.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
