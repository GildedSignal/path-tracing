# Vibe live verification: doors, trapdoors, fence gates, switches, and redstone cascades

**Goal:** Execute 26.2 `DoorBlock`, `TrapDoorBlock`, `FenceGateBlock`, `ButtonBlock`, `PressurePlateBlock`, `WeightedPressurePlateBlock`, `LeverBlock`, `RepeaterBlock`, `ComparatorBlock`, `ObserverBlock`, `RedstoneTorchBlock`, and `TripWireHookBlock` LIVE on `ServerLevel` + `LevelChunk` + real `CollectingNeighborUpdater` + `LevelTicks<Block>`.

Supplies executed multi-tick ground truth for `doorcorridor`, `trapdoorparkour`, and `cell` redstone/toggle rules (`jobs/law-grouping-audit/table.md` groups 1 & 4).

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`.

**Harness requirements (mandatory executed witnesses):**
- Boot jar tags (`MappedRegistry.prepareTagReload`) + `DataComponentInitializers` + Unsafe `ServerLevel` + real `LevelChunk` + real `CollectingNeighborUpdater` + real `LevelTicks<Block>`.
- Write `dossier.md` (`Claim: ...`), `rows.tsv`, and `witnesses.tsv` early after core toggles + schedules pass, then extend cleanly.

**Live scenes to execute:**
1. **Interactable barriers (`useWithoutItem` + collision AABB hex):** wooden vs iron `OakDoorBlock`/`IronDoorBlock` (lower+upper sync), `OakTrapDoorBlock`/`IronTrapDoorBlock` (`OPEN` + `HALF` collision coords, waterlogged fluid `delay 5`), `FenceGateBlock` (`OPEN` -> empty collision vs closed `1.5` high!).
2. **Sensors (`press`, `release`, scheduled tick delays):** `StoneButtonBlock` (`20`) vs `OakButtonBlock` (`30` + arrow pin), `StonePressurePlateBlock` (`20`) vs `HeavyWeightedPressurePlateBlock` (`10`, `ceil(min(count, max)/max * 15)`), `TripWireHookBlock` + string line (valid vs sheared).
3. **Propagation (`tick`, `updateNeighbors`):** `RepeaterBlock` delays `1..4` (`2,4,6,8` game ticks) & side locking, `ComparatorBlock` compare vs subtract mode, `ObserverBlock` `2`-tick rising edge pulse, `RedstoneTorchBlock` burnout (`8` toggles inside `60` ticks).

**Write in your run folder:**
- `rows.tsv`: `rule_id	block_family	delay_ticks	priority	aabb_or_signal	scars	source`
- `witnesses.tsv`: `case_id	tick	pos	block_state	power_or_open	aabb_hex	events`
- `dossier.md` (first line `Claim: `), `reproduce.sh`, and `wiki.md`.

**Falsifier:** Any executed tick where open state, collision shape AABB hex, scheduled delay, signal strength, or torch burnout tick disagrees with `server-26.2.jar`.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
