# Now

What is in flight and what comes next. Rewritten freely; history lives in git.

## In flight

- **Flat ground & real-time hot path on `main` (`Load-bearing`, flown):** `plan-track --mode solve` writes exact tick scripts verified by `law::try_tick`. Straight / flat tracks: `straight100` 288 / 288 (127 µs), `straight300` 845 / 845 (174 µs), `straight800` 2,248 / 2,248 (182 µs), `flowers` 48 / 48 (117 µs, his 48), `stonesimple` 85 / 85 (117 µs, his 86). `tests/real_time.rs` (60 samples each): standing new-goal median 19.6 µs (p90 27.8 µs, 0 alloc, 0 law ticks), moving median 20.85 µs (p90 23.6 µs), repeat median 6.65 µs; post-site flat solves in `generalize.rs` (`MeteredWorld`, 20 goals after learning sites): 6.2 µs median, 8.2 µs max, 0 law ticks.
- **Recursive `Word` / `WordStore` defect crossing on `main` (`Flying` offline):**
  - Core architecture ([defect.rs](../engine/plan/src/defect.rs), [word_store.rs](../engine/plan/src/word_store.rs), [certify.rs](../engine/plan/src/certify.rs)): coordinate-free `Word` (`Atomic` + arbitrary-depth recursive `Compound`), rotation-invariant block-name-free `DefectSig`, raw exploration on `LocalGrid` + `TopoField` (Dial backward BFS, zero `RecordingWorld` during search), Phase A word proposal before Phase B raw miss search (`propose_local` capped at 20% budget), and single-winner `RecordingWorld` certification (`NoRegen` resource guard + footprint-scoped `edit()`).
  - Offline track ladder (`plan-track --mode solve` on `tracks.txt`): **16 / 16 unclipped bare-legs tracks pass** with exact `write_script` replay: `grassjumpeasy` 38 (his 50, 44.3 ms), `stonestaircase` 99 (his 113, 42.7 ms, 8 compounds), `fencestraightbridge` 46 (his 47, 39.1 ms, 3 compounds), `mangrovejumpeasy` 53 (his 45, 44.3 ms), `fences` 111 (his 89, 139.0 ms), `fencemaze` 68 (his 66, 394.8 ms), `redfakemaze` 61 (his 71, 51.4 ms), `carpetwater` 95 (his 85, 120.2 ms), `butoonpressureplates` 79 (his 71, 32.4 ms), `trapdoorparkour` 122 (his 184, 99.2 ms), `doorparkourblue` 86 (his 126, 53.6 ms), plus flat/straight tracks above; repeat solve 2.8–8.6 µs.
  - Five remaining tracks in `tracks.txt` return `NeedsJoints`: `birchparkourmedium` is clipped by `tracks.txt` bounds (`18 -64 1 29 -45 22` cuts off its `z = -11..-7` climb tower); `ametwoodbox`, `mud`, `doorcorridor`, and `amertystcoral` require block place/break or door/trapdoor use (`TopoField` halts bare-legs `doorcorridor`/`mud` in ~2.5–2.9 ms).
  - Generalization & guard verification (`engine/plan/tests/generalize.rs`, 5/5 green): `learn_ticks == 0` on exact repeat, `+1 ULP`, `+0.05` shift, `+25` translation, `90°` rotation, and rotated 12-step staircase from 8-step compounds; `NoRegen` refuses lethal `health=5.0, timer=0` while admitting `6.0, timer=0` and exact `5.0, timer=9`; outside `edit()` leaves cache intact while inside `edit()` invalidates and re-solves cleanly; 11 bare-legs tracks solve sequentially on one shared store in 1.33 s total with `learn_ticks=0` on `stonestaircase`, `fencestraightbridge`, `carpetwater`, and `butoonpressureplates`.
- **Bot mod wire on `main`:** [resident.rs](../engine/native/src/resident.rs) and `ResidentRunner.java` handle `WORD_PLAN` with 4-word corridor hull, `store.words` block-edit invalidation (`edit()` kinds 1/2/3/4), and `first_uncovered` chunk growth.
- **Port & companion lanes in flight (`wt/port-state-spine`, `wt/port-fold`):**
  - Core planner architecture stays directly between Julien and the integrator; porting and data-table lanes run in parallel.
  - **Claude Opus** (`wt/port-state-spine`, branched from `wt/port-fold`): implements the canonical 2-byte `StateId(u16)` + `16³` paletted section + `u64` `4×4×4` summary mask + `O(1)` uniform-box query and reversible `WorldDelta` spine (`jobs/port-state-spine/`), deleting `WorldOverlay`/`SimCell` duplication in `sim.rs`.
  - **GLM 5.4** (`jobs/glm-chain.txt`): parameter rows, Mojang scars, and jar witnesses (6 items queued: `gen-projectile-pearl-rows`, `gen-damage-ledger-rows`, etc.). **Mistral Vibe** (`jobs/mistral-chain.txt`) finished its queue (`0` remaining).
  - **ES 1B-bot probe ([es-1b-probe](../jobs/es-1b-probe/report.md)):** passes 5/5 checks and 3/3 mutants offline (`10⁹` bots / `10⁶` scars in `4.28 MB`, `172 µs` edit split, `129.8 µs` max observer frame, `0.418 ms/tick` 500-bot `Battle`).

## Next, in order

1. **Live flight verification of bare-legs defect crossings** in Julien's game and cleanup of fixture clipping (`birchparkourmedium` corridor bounds when needed).
2. **Land `wt/port-state-spine` onto `wt/port-fold` / `main`**, wire `plan` onto `law::face::advance` and `16³` `O(1)` uniform-box queries (deleting `events.rs` inverse duplication).
3. **Tools, edits, portals, and dynamic world:** carry block place/break, door/trapdoor interaction, bucket/pearl tools, and ledger restoration as `Word` operators under the same planner loop (`ametwoodbox`, `mud`, `doorcorridor`, `amertystcoral`).
4. **Far map & ES-native `10⁹` bots (`scales.md`, `server.md`):** shared goals, `Wheel`, `(max, +)` building, and melee singularity handoff ([pvp-strategy-2026-11-22.txt](../archive/research/pvp-strategy-2026-11-22.txt)).

## Derived component state (ES rule)

| Component | Derived state | Scope |
|---|---|---|
| Core `solve`, `Word` / `WordStore`, `LocalGrid` + `TopoField`, `NoRegen` `certify` | `Flying` | `main`, `16/16` unclipped bare-legs tracks + `generalize` green |
| Live JNI resident channel (`resident.rs` + `ResidentRunner.java`) | `Load-bearing` (flat) / `Flying` (words) | `main` |
| `wt/port-state-spine` (`16³` `StateId` world spine over `wt/port-fold`) | `Sketch` (Opus active) | `D:\path-tracing-wt\port-state-spine` |
| Parked `wt/flight-ladder-sets` | `Sketch` (unmerged) | parked unmerged; core recursive `Word` lives on `main` |
| Kit edits, tools, far map, ES `10⁹` bots | `Sketch` | awaits live defect flight + port spine |

## Branches and open notes

- Kept until needed: `wt/port-fold` (`origin/wt/port-fold`, 26.2 jar suite with 9,117 witnesses and 32,252 / 32,366 modeled states), `wt/flat-ground` (untracked `lab/fixtures/flat-ground-replay/`), `wt/server-mind`, `wt/build-4d`, `wt/track-face`, `wt/hard-track-2`, `wt/l0-veto`, `wt/build-preview`, `wt/hearth-small`.
- Untrimmed region files `r.0.0.mca` / `r.0.-1.mca` stay uncommitted on `main` (`*.bak-20261114` backups beside them).
