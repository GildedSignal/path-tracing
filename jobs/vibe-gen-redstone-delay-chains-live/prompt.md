# Vibe live verification: 26.2 multi-hop redstone delay chains, priorities, quasi-connectivity, and 0-tick pistons

**Goal:** Execute live 26.2 multi-hop `RepeaterBlock`, `ComparatorBlock`, `ObserverBlock`, `RedstoneTorchBlock`, and `PistonBaseBlock` (delays, `TickPriority`, quasi-connectivity diagonally above pistons, and 1-tick sticky piston block drop) on `server-26.2.jar`.

Supplies bit-exact multi-chain timer truth for synchronizing redstone + piston events in 4D rendezvous (`map/detail/representation.md` §5). See `lab/results/local-20261124-130147-11356-1-0-vibe-check-block-place-break/` for clean boot.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`; java/javac: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/`.

**Harness requirements (mandatory executed witnesses):**
- Boot tags (`MappedRegistry.prepareTagReload(...).apply()`) + `DataComponentInitializers` + live `ServerLevel` + `LevelChunk` + real `LevelTicks` scheduler (`ServerLevel.tick(...)`).
- Inspect `javap -c -p -constants` for `RepeaterBlock`, `ComparatorBlock`, `ObserverBlock`, `RedstoneTorchBlock`, and `PistonBaseBlock.hasNeighborSignal` / `checkIfExtend` / `triggerEvent`.
- Write `dossier.md` (`Claim: ...`), `rows.tsv`, and `witnesses.tsv` early, then extend cleanly.

**Live scenes to execute:**
1. **Priority race:** repeater side-locking another repeater, comparator subtracting signal vs repeater `HIGH`/`VERY_HIGH`/`EXTREMELY_HIGH` priority tie-break in the same tick.
2. **Multi-hop chain (`button -> wire -> observer -> 2-tick repeater -> sticky piston + slime`):** tick-by-tick state trace (`T=0..30`) recording exact `TickPriority`, scheduled delay, block events (`blockEvent`), and slime extension tick.
3. **Quasi-connectivity & 1-tick pulse sticky drop:** power cell `y+1` diagonally above piston + neighbor update, plus 1-tick/2-gt observer pulse dropping carried block cleanly.

**Write in your run folder:**
- `rows.tsv`: `component_id	method	delay_ticks	priority	quasiconn_offset	scars	source`
- `witnesses.tsv`: `case_id	tick	pos	state_before	state_after	piston_moving	events`
- `dossier.md` (first line `Claim: `), `reproduce.sh`, and `wiki.md`.

**Falsifier:** Any executed tick delay, `TickPriority` order, quasi-connectivity activation, or sticky drop state that disagrees with `server-26.2.jar`.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
