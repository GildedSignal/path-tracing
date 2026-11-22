# Port delta: RedstoneTorch, Comparator, CopperBulb, Target and DaylightDetector rules

**Goal:** the jar's exact rule for this, with a reproduction that runs against Mojang's 26.2 jar: `dossier.md` (first line `Claim: `), `reproduce.sh`, and the Rust port delta as `work.patch`.

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`. `port-interact-lever-and-redstone-door` and `port-redstone-repeater-observer-dispenser` are already settled; do not re-ask lever, door, wire, repeater, observer, or dispenser.

**Consumer:** `engine/law/src/redstone.rs`, `map/now.md` (full redstone circuit logic).

**From the 26.2 server bytecode**, state with class, method and offsets:
- `RedstoneTorchBlock` & `RedstoneWallTorchBlock`: `2`-tick schedule delay (`TOGGLE_DELAY`), `neighborChanged` -> `update`, `TickPriority` (default `NORMAL`?), and burnout toggle history (`8` toggles within `60` ticks -> extinguishes + `160`-tick cool-down schedule? verify exact bytecode);
- `ComparatorBlock`: `2`-tick delay, `TickPriority.HIGH` when facing diode (`shouldPrioritize` vs repeater!), `COMPARE` (`input >= side ? input : 0`) vs `SUBTRACT` (`max(input - side, 0)`), `getInputSignal` (max of behind wire/diode vs power-15 container override via `getBlockState(pos.relative(dir))` + `getAnalogOutputSignal`), and `getAlternateSignal` side inputs;
- `CopperBulbBlock`: `POWERED` edge trigger (`hasNeighborSignal` rising edge flips `LIT`, synchronous `setBlock` flags `2`/`3`, analog output `15` when `LIT`, zero delay!);
- `TargetBlock` (projectile distance-to-center signal `1..=15` and `20`-tick vs `8`-tick pulse) and `DaylightDetectorBlock` (skylight/time-of-day formula and inverted mode).

**Build a jar-backed reproduction:** run `reproduce.sh` disassembling all 5 block classes and executing live 26.2 jar tests covering torch burnout (`8` toggles in `< 60` ticks), comparator compare/subtract/container/priority, copper bulb rising-edge toggle, target signal vs hit offset, and daylight detector light curves. Write `dossier.md` starting with `Claim: ` on its first physical line and `work.patch` extending `engine/law/src/redstone.rs`.

**Falsifier:** any wrong tick delay, priority, burnout threshold (`8` / `60` / `160`), comparator math edge, or copper-bulb edge trigger against the 26.2 jar.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
