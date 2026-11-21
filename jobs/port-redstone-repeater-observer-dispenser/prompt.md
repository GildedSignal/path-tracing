# Port delta: RepeaterBlock, ObserverBlock and DispenserBlock redstone timing and bucket/projectile dispatch

**Goal:** the jar's exact rule for this, with a reproduction that runs against Mojang's 26.2 jar: `dossier.md` (first line `Claim: `), `reproduce.sh`, and the Rust port delta as `work.patch`.

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`. `port-interact-lever-and-redstone-door` and `port-scheduled-tick-drain-and-cap` already settled `CollectingNeighborUpdater` and `LevelTicks` priority ordering.

**Consumer:** `engine/law/src/interact.rs`, `map/detail/plan.md:57-58` (stasis networks and redstone chains), `jobs/port-wave-2/report.md`.

**From the 26.2 server bytecode**, state with class, method and offsets:
- `net.minecraft.world.level.block.DiodeBlock` and `RepeaterBlock` (to locate): `getDelay(BlockState)` (`DELAY * 2` ticks, `2..=8`), `checkTickOnNeighbor` (`isLocked` side-diode gate, `TickPriority.EXTREMELY_HIGH` (`-3`) when pointing into another diode, `VERY_HIGH` (`-2`) on turn-off when already powered, `HIGH` (`-1`) otherwise), `tick` (`POWERED` pulse hold: if `!powered && !shouldTurnOn`, still turns `POWERED=true` and schedules turn-off at `+delay` with `VERY_HIGH`!), and `updateNeighborsInFront` (`pos.relative(facing.getOpposite())`);
- `net.minecraft.world.level.block.ObserverBlock` (`updateShape`, `startSignal`, `tick`, `updateNeighborsInFront`): triggers only on `direction == state.getValue(FACING)` when `!hasScheduledTick`, schedules `delay = 2` (normal priority `0`), and `tick` flips `POWERED = false -> true` (rescheduling `delay = 2`) then `true -> false`, notifying the output cell `pos.relative(FACING)`;
- `net.minecraft.world.level.block.DispenserBlock` (`neighborChanged`, `tick` at `delay = 4`, `dispenseFrom`) and `DispenseItemBehavior` for `WATER_BUCKET` / `LAVA_BUCKET` / `BUCKET` (`DispensibleContainerItem.emptyContents` / `BucketPickup.pickupBlock` at `pos.relative(FACING)`).

**Build a jar-backed reproduction:** run `reproduce.sh` disassembling `DiodeBlock`, `RepeaterBlock`, `ObserverBlock`, and `DispenserBlock`, plus a jar-backed execution stepping: (1) a 1-tick pulse into a delay-1..4 repeater (locked vs unlocked), (2) a block change in front of an `ObserverBlock` across ticks `t..t+5`, and (3) a `DispenserBlock` placing and picking up water at `t+4`, printing all block writes, flags, scheduled tick delays, and `TickPriority` values. Write `dossier.md` starting with `Claim: ` on its first physical line, and produce the Rust port delta as `work.patch`.

**Falsifier:** any repeater delay/priority/pulse-extension rule, observer 2+2 tick timing, or dispenser 4-tick bucket behavior that disagrees with the 26.2 jar execution.
