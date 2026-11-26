# Beat the outside references: session handoff (2026-11-26)

**Goal:** gather the metrics others publish for Minecraft pathfinding and beat the ones that matter to T (Julien's words in `archive/turns/inbox.md`, 2026-11-26 evening).

Read first: `map/detail/benchmarks.md` (every reference, checked at its source, with our measured rows) and `map/detail/representation.md` section 10, "Ways of moving are modes".

## Done

| What | Where | State |
|---|---|---|
| Every pasted reference checked at its primary source; corrections recorded (Hielke Dimension is 11:30; SpectraPhysics' tick course has no cobwebs or honey; BlockStep ran on 1.16.1, seeds unpublished) | `map/detail/benchmarks.md` | on `main` |
| SpectraPhysics run here beside the law on its own course: their exact `tick.js` 5.9–7.2 µs (1.21.4) and 11.0 µs (26.1) per player-tick, ours 1.06–1.6 µs; floored (nobody falls off) 21.7–34.8 µs against ours 2.5–3.3 µs. Their "~3 µs" is mostly free fall (49 of 50 players fall off) | `engine/world/src/bin/bench-course.rs` (`exact`, `floored`, `straight`), `WorldSnapshot::from_fn` | on `main` |
| Section `collide` mask: a sweep through 4³ subcells that hold nothing collidable is proven empty in one bit test (exact course 3.6 → 1.06–1.6 µs) | `engine/world/src/snapshot.rs` | on `main` |
| Delta found: the law tick is 0.85 µs on `BoxWorld` but 1.8–2.7 µs on the 16³ sections the planner reads, ~1 order above the 0.1–0.3 µs floor, fixed per tick | `map/goal.md` floor table | on `main`; not fixed |
| BlockStep's protocol rerun on 26.2 (one seed, 1162026; six biome interiors found by `locate biome` grids; goals at ±20/50/100 on both axes; bare legs; cold store per task; every plan replayed through the law): solved 24/24, 21/24, 20/24, mean 4.79 / 11.94 / 25.27 s against BlockStep 6.6 / 16.2 / 35.1 and Baritone 8.4 / 18.8 / 37.0. Predicted, not flown | `engine/plan/src/bin/blockstep.rs`; evidence `D:\path-tracing-evidence\blockstep-20261126\` | on `main` |
| Why 7 of 72 fail: learning spends its cap (~98k or 240k raw ticks). Strips separate the causes: deep water and a sunk powder-snow floor fail; leaf litter, snow layers and drops do not | `map/detail/benchmarks.md` | on `main` |
| Design, agreed with Julien: ways of moving are modes (walking, swimming, climbing, boat, elytra); each mode is a generator, the law ticked under its inputs until its motion repeats; that repeat is its gait and a gait is a zone | `map/detail/representation.md` section 10 | on `main` |
| Modes built, half working: field edges priced by the class of the cell left (`Legs::learn`); water and lava travel report their affine row (`observe_air_travel`); zone label carries the wet eyes cell; `Gait::learn` closes a cycle on a landing or, for a body that never lands, on its vertical state; one `mode_gait` (fastest level gait) for both extend paths; zones in another mode's class checked column by column (`uniform_corridor`). Open sea is a zone once swimming (`sea5`: Extend, Cut, Compose) | branch `wt/bench-dig` (`2e6a40f3`, `34927c46`), worktree `D:\path-tracing-wt\bench-dig` | **not on main** |
| Sol's `law-26.3`: region reader decodes Spiral's 26.3 palettes; `WorldSnapshot::load` reaches the datapack refusal; 5 shulker-box states refused; suites 268 / 1 failed (server-mind fence at Y = −65, unchanged) / 11 ignored | branch `wt/law-26.3` (`bf7d5c6f`), `jobs/law-26.3/report.md` | **not on main**; blocked |

## Open, in order

1. **`certify:touch-mismatch` on `wt/bench-dig`.** With modes, BlockStep falls from 65/72 to 62/72; `forest-50-se` and `swamp-100-se` end in a touch-mismatch (the certifying replay disagrees with the plan about when B is touched). A wrong prediction: find it before anything else on that branch. Suspects: the level-gait end check in `Crossing::run` (`ground_body` for a mode without a floor) and the wet layers in `Sample`. Rerun: `blockstep.exe 'D:\path-tracing-run\blockstep\server\world' <tasks> <out>` with the task list in the evidence folder; compare with `results.txt`.
2. **The water joints are learned cold** at 100k-tick budgets (into the stroke from the seabed 159 ms, the shore 135–1,000 ms, the final touch on the seabed 213 ms), above the ~12 ms new-joint floor. Uniform water is already a zone; the joints at its edges are what fail `ice-100-nw`, `swamp-100-sw` and the original water tasks. Strip probe: `engine/plan/src/bin/strip-probe.rs` on the branch (`CASE=sea5 STAGES=1`).
3. **Unseparated BlockStep failures:** `hills-50-sw`, `forest-50-sw`, `forest-100-se` (thousands of exceptions among trees and cliffs).
4. **Powder snow:** the law gives a sunk body no rise under jump (witnessed in part in `law/tests/jar.rs`, never flown), so a sunk floor may have no exit without leather boots; the planner should fail fast instead of after ~0.7 s.
5. **26.3:** Sol needs the 26.3 client and server jars (Mojang piston-meta) and a 26.3 Fabric API in `D:\path-tracing-deps\`, never in git; Julien had not yet approved the integrator downloading them. Then: the source diff, regenerated tables, mod builds, and the flight commands in `jobs/law-26.3/report.md`. Hielke's maps also need their datapacks (food, damage, position, terrain) to enter the law as world events.
6. **Flights in his game:** nothing this session was flown. BlockStep against Baritone v1.19.0 (26.2, downloaded) on the same worlds; MineOdyssey (30 worlds downloaded, DataVersion 4671, upgrade copies with a scratch server's `--forceUpgrade`); ParkourCalculator single jumps (not downloaded).
7. **Law tick on sections** (1.8–2.7 µs against 0.85 on `BoxWorld`): a law-speed job.

## Where things are (never in git)

- Opponents: `D:\path-tracing-bench\opponents\` (SpectraPhysics at `1ea12c8`, Baritone v1.19.0 26.2 fabric jar). Node 22.23.5: `D:\path-tracing-deps\node22`.
- Maps: `D:\path-tracing-maps\hielke\` (Parkour Spiral 3.0.5, 26.3), `D:\path-tracing-maps\mineodyssey\` (30 zips, `tasks.json`).
- BlockStep worlds: scratch vanilla 26.2 server `D:\path-tracing-run\blockstep\server` (port 25600, RCON 25610, stopped; never his server). Biome centres: desert (−1040, −568), windswept forest (6996, −5300), snowy plains (2256, 176), snowy slopes (−4636, −6284), swamp (−2456, 4600), forest (184, 176).
- Evidence: `D:\path-tracing-evidence\blockstep-20261126\` (`results.txt` first run, `results-class-pricing.txt`, `results-modes.txt`, `tasks.txt`, `locate-*.txt`, replay scripts).
