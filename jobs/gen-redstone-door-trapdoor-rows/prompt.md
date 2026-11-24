# Generator rows: redstone doors, trapdoors, buttons, plates, and wire cascades

**Goal:** One parameter/scar table for 26.2 `cell` rewrite rules used by `doorcorridor`, `trapdoorparkour`, and redstone puzzles (doors, trapdoors, fence gates, buttons, pressure plates, levers, repeaters, comparators, observers, torches, tripwires) with multi-tick `LevelTicks` witnesses.

Supports the `cell` generator (`jobs/law-grouping-audit/table.md` groups 1 & 4) and upcoming flights on `doorcorridor` and `trapdoorparkour`.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`.

**Harness requirements:**
- Bind real jar tags via `MappedRegistry.prepareTagReload(...).apply()` (`data/minecraft/tags/**/*.json`).
- Drive server updates over a real `ServerLevel` / `LevelChunk` with `LevelTicks<Block>`, `CollectingNeighborUpdater`, and real `LocalPlayer` (or `isLocalPlayer() => true` + `isLocalClientAuthoritative() => true`) when sensing box overlaps.
- Write `dossier.md` (`Claim: ...`), `rows.tsv`, and `witnesses.tsv` early after core toggles + primary cascades pass, then extend cleanly.

**From the bytecode** (`class.method@offset`):
1. **Interactable barriers:** `DoorBlock`, `TrapDoorBlock`, `FenceGateBlock`, `LeverBlock` — material open rules (wooden vs iron), partner-half propagation on doors, fluid waterlogged tick scheduling on trapdoors (`delay 5`), hitbox collision changes (`OPEN` shape coords hex).
2. **Sensors & switches:** `ButtonBlock` (`stone` 20 ticks vs `wooden` 30 ticks, projectile-arrow sensitivity on wood), `PressurePlateBlock` (`20` ticks, entity sensitivity), `WeightedPressurePlateBlock` (`10` ticks, `ceil(min(count, max) / max * 15)`), `TripWireHookBlock` + `TripWireBlock` (max 42-block line scan, shears disarm scar).
3. **Propagation elements:** `RepeaterBlock` (delays `2,4,6,8`, `PRIORITY -1/-2/-3`, locking from side diode), `ComparatorBlock` (`compare` vs `subtract`, container analog override), `ObserverBlock` (`2` tick delay rising edge `LevelTicks`), `RedstoneTorchBlock` (burnout `8` toggles in `60` ticks).

**Write in your run folder:**
- `rows.tsv`: `rule_id	block_family	delay_ticks	tick_priority	signal_rule	shape_or_collision_note	scars	source`
- `witnesses.tsv`: `case_id	tick	pos	block_state	power_or_open	scheduled_delays	events`
- `dossier.md` (first line `Claim: `), `reproduce.sh`, and `wiki.md`.

**Falsifier:** Any witness tick where `cell` schedules a different delay/priority, toggles an iron barrier without redstone, or emits different open/powered state than `ServerLevel` + `LevelTicks`.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
