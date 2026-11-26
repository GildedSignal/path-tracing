# Generator rows: 26.2 Repeater, Comparator, Observer, Torch, Target, and Piston redstone signal timing

**Goal:** Extract and execute live 26.2 `RepeaterBlock`, `ComparatorBlock`, `ObserverBlock`, `RedstoneTorchBlock`, `TargetBlock`, and `PistonBaseBlock` tick delays, `TickPriority` order, quasi-connectivity, and 0-tick drops on `server-26.2.jar`.

Cross-checks Vibe's `vibe-gen-redstone-delay-chains-live` and anchors `engine/law/src/redstone.rs` + 4D multi-chain trigger timing (`map/detail/representation.md` §5). See `lab/results/local-20261124-130147-11356-1-0-vibe-check-block-place-break/` for clean boot.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`; java/javac: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/`.

**Harness requirements:**
- Mandatory first boot step: `MappedRegistry.prepareTagReload(...).apply()` + `DataComponentInitializers` + live `ServerLevel` + `LevelChunk` + real `ServerLevel.tick(...)` scheduler.
- Inspect `javap -c -p -constants` for `RepeaterBlock`, `ComparatorBlock`, `ObserverBlock`, `RedstoneTorchBlock`, `TargetBlock`, and `PistonBaseBlock`.
- Write `dossier.md` (`Claim: ...`), `rows.tsv`, and `witnesses.tsv` early, then extend cleanly.

**Cases & rules to extract and witness:**
1. **Priority & sub-tick ordering:** repeater lock vs delay `1..4` (`HIGH` / `VERY_HIGH` / `EXTREMELY_HIGH`), comparator compare vs subtract, torch burnout (`8` toggles in `60` ticks).
2. **Arrow-triggered `TargetBlock` & chained slime-piston timing:** arrow hitting `TargetBlock` -> observer / repeater -> sticky slime piston extension (`T=0..30`), plus quasi-connectivity and 1-tick sticky drop.

**Write in your run folder:**
- `rows.tsv`: `component_id	method	delay_ticks	priority	analog_or_burnout	scars	source`
- `witnesses.tsv`: `case_id	tick	pos	power_or_lit	piston_moving	events`
- `dossier.md` (first line `Claim: `), `reproduce.sh`, and `wiki.md`.

**Falsifier:** Any executed tick delay, `TickPriority`, torch burnout limit, or piston extension tick that disagrees with `server-26.2.jar`.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
