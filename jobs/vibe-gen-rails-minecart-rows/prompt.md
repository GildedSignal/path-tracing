# Vibe generator rows: rails kernel and minecart physics

**Goal:** One bit- and behaviour-exact parameter/scar table for the 26.2 `Rails` kernel (`jobs/law-grouping-audit/table.md` group 19) covering `AbstractMinecart`, `NewMinecartBehavior` / `OldMinecartBehavior`, rail shape polylines, slopes, powered/activator/detector rails, and minecart water gravity.

Feeds the `Rails` kernel (`jobs/law-grouping-audit/table.md` group 19) so `wt/port-fold`'s rail projection rules match 26.2 bit-exact.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`.

**Harness requirements:**
- Bind real jar tags via `MappedRegistry.prepareTagReload(...).apply()` (`#rails`, `#prevent_mob_spawning_inside`).
- Drive real `Minecart` / `ChestMinecart` / `ServerLevel` + `LevelChunk` with real `Rail` / `PoweredRailBlock` / `DetectorRailBlock` states.
- Write `dossier.md` (`Claim: ...`), `rows.tsv`, and `witnesses.tsv` early after core rail trajectories pass, then extend cleanly.

**From the bytecode** (`class.method@offset`):
1. **`BaseRailBlock` / `RailState` / `RailShape` (all 10 shapes):** polyline exit tables, slope acceleration/deceleration constants, curved corner redirection, off-rail ground/air friction (`0.5` / `0.95`).
2. **`AbstractMinecart` + `OldMinecartBehavior` / `NewMinecartBehavior`:** check whether 26.2 uses old vs new behavior by default (`minecart_improvements` feature flag?), `0.005` water gravity, speed cap per rail/world, fueled boost (`PoweredRailBlock` powered push `+0.06` and standstill nudge `+0.02`, unpowered brake `0.5x`), `DetectorRailBlock` signal/schedule (`20` ticks), and entity push/mount collision.

**Write in your run folder:**
- `rows.tsv`: `rule_id	rail_shape_or_mode	constants_hex	speed_cap_hex	scars	source`
- `witnesses.tsv`: `case_id	tick	x_hex	y_hex	z_hex	vx_hex	vy_hex	vz_hex	rail_shape	events`
- `dossier.md` (first line `Claim: `), `reproduce.sh`, and `wiki.md`.

**Falsifier:** Any tick where minecart position/velocity hex bits, curve exit direction, or booster acceleration differs from the 26.2 jar.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
