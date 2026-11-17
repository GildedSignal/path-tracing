# Port delta: LevelTicks.tick drain cap and ServerLevel.tick phase order

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`. `port-scheduled-tick-dedup` and `port-schedule-order-tie` already settled `LevelChunkTicks` first-wins dedup and `ScheduledTick.DRAIN_ORDER`; do not re-ask those.

**Consumer:** `docs/plan.md` step 5 & step 8 (`P2a`, exact phase order across block ticks, fluid ticks, block events, and player ticks in a server tick).

**From the 26.2 server bytecode**, state with class, method and offsets:
- `ServerLevel.tick` (to locate): exact call order within one server tick among `blockTicks.tick(...)`, `fluidTicks.tick(...)`, `runBlockEvents()` (pistons), entity ticks, and `tickConnection()` (`MinecraftServer.tickChildren`);
- `LevelTicks.tick`, `collectTicks`, `drainContainers`, and `runCollectedTicks` (to locate): exact per-tick execution cap (`65536` `MAX_TICK_PER_TICK`?), whether `canScheduleMoreTicks` drops excess or defers them to the next tick, whether `nextTick` (`subTickOrder`) is reset or monotonic across ticks, and the chunk-ticking predicate (`tickThisTick` / `canPositionTick` at ticket level).

**Build a jar-backed reproduction:** run `reproduce.sh` disassembling `ServerLevel.tick` and `LevelTicks` (`tick`, `collectTicks`, `runCollectedTicks`, `drainFromCurrentContainer`), and execute a `LevelTicks` run with `>65536` ticks (or verifying the exact cap constant) plus same-tick `blockTicks` vs `fluidTicks` ordering. Write `dossier.md` starting with `Claim: ` on its first physical line.

**Falsifier:** any phase ordering among `blockTicks`/`fluidTicks`/`blockEvents`/players, per-tick cap constant, or excess-deferral rule that disagrees with the 26.2 bytecode.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
