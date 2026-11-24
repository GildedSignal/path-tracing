# Now

What is in flight and what comes next. Rewritten freely; history lives in git.

## In flight

- **Flat ground & real-time hot path on `main` (`Load-bearing`, flown):** `plan-track --mode solve` writes exact tick scripts verified by `law::try_tick`. Straight / flat tracks: `straight100` 288 / 288 (127 µs), `straight300` 845 / 845 (174 µs), `straight800` 2,248 / 2,248 (182 µs), `flowers` 48 / 48 (117 µs, his 48), `stonesimple` 85 / 85 (117 µs, his 86). `tests/real_time.rs` (60 samples each): standing new-goal median 18.25 µs (p90 32.0 µs, max 43.7 µs, 0 alloc, 0 law ticks), moving median 16.4 µs (p90 20.1 µs, max 29.5 µs), repeat median 5.3 µs (p90 8.6 µs, max 11.6 µs); post-site flat solves in `generalize.rs` green alongside 5/5 cross-track checks.
- **Kit unfolding Slice 1 on `main` (`Flying` offline):**
  - Core architecture ([kit.rs](../engine/plan/src/kit.rs), [defect.rs](../engine/plan/src/defect.rs), [certify.rs](../engine/plan/src/certify.rs), [word_store.rs](../engine/plan/src/word_store.rs), [solve.rs](../engine/plan/src/solve.rs), [source.rs](../engine/plan/src/source.rs)): `StateId(u16)` preservation (`scar_states` without adding legs scars on mixed ground), factored `< 1 MB` `KitView` (`StateStatic(u16)` + `LegsProjection` + `BlockBreakClasses` + law-probed `VerbEffect`s via `WorldDelta + RecordingWorld`, sharing `Arc` handles across `64 -> 63` stack decrements and clearing `VERB_PLACE` at `1 -> 0`), admissible `KitCone`, double-entry `ObligationLedger` + backward `shadow_price`, causal obligation-exit windows in `exec_atomic_raw`, `try_tail_extend` crouching guard, `route_for_kit_in_world` `reads` digest verification across stores, and `certify_route` `WorldDelta` restoration + `NoRegen` verification.
  - Spartan verification ([kit_unfold.rs](../engine/plan/tests/kit_unfold.rs), `5/5` green): mixed flat ground zero-tax (`bare_T=60`, `kit_T=60`, median `4.55 µs`), `10,000` combinatorial kits in `356,010 B` cache, `KitCone` KT16 (`500/500`, `0` violations), `R-lego-1` 5/6-block gaps with restore (`T=45` in `14,012` learn ticks / `T=57` in `25,783` ticks), `R-lego-2` step-up gap (`T=54`), `R-lego-3` mid-air place vs stop-and-place (`48` vs `159`), `R-lego-4` Silk Touch wall restore (`T=38`) / oak door restore (`T=24`, 2 interacts) / 25-block water clutch (`T=32`, `1t` pickup latency), `E3-nether-bucket` water evaporation refusal + powder snow 25-block drop (`T=36`), real `doorcorridor` fixture (`T=32` both open and restored vs his `99`), plus companion diagnostics (`amertystcoral` ready `T=33` vs his `63`; `ametwoodbox` and `mud` return `NeedsJoints`).
- **Recursive `Word` / `WordStore` defect crossing on `main` (`Flying` offline):**
  - Core architecture ([defect.rs](../engine/plan/src/defect.rs), [word_store.rs](../engine/plan/src/word_store.rs), [certify.rs](../engine/plan/src/certify.rs)): coordinate-free `Word` (`Atomic` + arbitrary-depth recursive `Compound`), rotation-invariant block-name-free `DefectSig`, raw exploration on `LocalGrid` + `TopoField` (Dial backward BFS, zero `RecordingWorld` during search), Phase A word proposal before Phase B raw miss search (`propose_local` capped at 20% budget), and single-winner `RecordingWorld` certification (`NoRegen` resource guard + footprint-scoped `edit()`).
  - Offline track ladder (`plan-track --mode solve` on `tracks.txt`): **16 / 16 unclipped bare-legs tracks pass** with exact `write_script` replay: `grassjumpeasy` 38 (his 50, 44.3 ms), `stonestaircase` 99 (his 113, 42.7 ms, 8 compounds), `fencestraightbridge` 46 (his 47, 39.1 ms, 3 compounds), `mangrovejumpeasy` 53 (his 45, 44.3 ms), `fences` 111 (his 89, 139.0 ms), `fencemaze` 68 (his 66, 394.8 ms), `redfakemaze` 61 (his 71, 51.4 ms), `carpetwater` 95 (his 85, 120.2 ms), `butoonpressureplates` 79 (his 71, 32.4 ms), `trapdoorparkour` 122 (his 184, 99.2 ms), `doorparkourblue` 86 (his 126, 53.6 ms), plus flat/straight tracks above; repeat solve 2.8–8.6 µs.
  - Five remaining tracks in `tracks.txt` return `NeedsJoints`: `birchparkourmedium` is clipped by `tracks.txt` bounds (`18 -64 1 29 -45 22` cuts off its `z = -11..-7` climb tower); `ametwoodbox`, `mud`, `doorcorridor`, and `amertystcoral` require block place/break or door/trapdoor use (`TopoField` halts bare-legs `doorcorridor`/`mud` in ~2.5–2.9 ms).
  - Generalization & guard verification (`engine/plan/tests/generalize.rs`, 5/5 green): `learn_ticks == 0` on exact repeat, `+1 ULP`, `+0.05` shift, `+25` translation, `90°` rotation, and rotated 12-step staircase from 8-step compounds; `NoRegen` refuses lethal `health=5.0, timer=0` while admitting `6.0, timer=0` and exact `5.0, timer=9`; outside `edit()` leaves cache intact while inside `edit()` invalidates and re-solves cleanly; 11 bare-legs tracks solve sequentially on one shared store in 1.33 s total with `learn_ticks=0` on `stonestaircase`, `fencestraightbridge`, `carpetwater`, and `butoonpressureplates`.
- **Bot mod wire on `main`:** [resident.rs](../engine/native/src/resident.rs) and `ResidentRunner.java` handle `WORD_PLAN` with 4-word corridor hull, `store.words` block-edit invalidation (`edit()` kinds 1/2/3/4), and `first_uncovered` chunk growth.
- **26.2 Law & `16³` `StateId` world spine on `main` (`Flying` offline):**
  - Full 26.2 law (`core.rs`, `face.rs`, `sim.rs`, fluids, mining, placement, redstone, doors, boats, portals, pistons, combat) verified across 9,117+ jar witnesses, replays, and `WorldDelta` rollback (`checkpoint` / `rollback`, ref-counted capability hooks).
  - Canonical 2-byte `StateId(u16)` + `BlockId(u16)` + `ModelId(u16)` backed by `16³` paletted sections, `4³` summary masks (`World::uniform`), and `WorldSnapshot::load_for_plan` partial-coverage support.
  - **GLM 5.4** (`jobs/glm-chain.txt`): parameter rows, Mojang scars, and jar witnesses running (`gen-water-swim-climb-rows`, `gen-ground-tags-rewitness`). **Mistral Vibe** (`jobs/mistral-chain.txt`, 5 jar-truth prompts) running 2 parallel workers (`vibe-gen-block-rules-census`, `vibe-gen-item-entity-drop-pickup`). **Codex Sol** (`yarn-cold-fold`): completed 26.2 jar baseline census (`32,366` states, `180` pass / `5` ignored, 2 boundary diffs on water/oak slab and piston/bedrock), blocked on read-only `.git/worktrees` index lock in Codex's sandbox.
  - **ES 1B-bot probe ([es-1b-probe](../jobs/es-1b-probe/report.md)):** passes 5/5 checks and 3/3 mutants offline (`10⁹` bots / `10⁶` scars in `4.28 MB`, `172 µs` edit split, `129.8 µs` max observer frame, `0.418 ms/tick` 500-bot `Battle`).

## Next, in order

1. **Live flight verification of bare-legs defect crossings** in Julien's game and cleanup of fixture clipping (`birchparkourmedium` corridor bounds when needed).
2. **Wire `plan` onto `World::uniform()` and `law::face::advance`** (replacing `LocalGrid` copies and deleting `events.rs` inverse duplication).
3. **Tools, edits, portals, and dynamic world:** wire Slice 2 bot/JNI hotbar & use-item actions onto Slice 1 `KitOp` plans and close `ametwoodbox` and `mud` replay under live game validation.
4. **Far map & ES-native `10⁹` bots (`scales.md`, `server.md`):** shared goals, `Wheel`, `(max, +)` building, and melee singularity handoff ([pvp-strategy-2026-11-22.txt](../archive/research/pvp-strategy-2026-11-22.txt)).

## Derived component state (ES rule)

| Component | Derived state | Scope |
|---|---|---|
| Core `solve`, `Word` / `WordStore`, `LocalGrid` + `TopoField`, `NoRegen` `certify` | `Flying` | `main`, `16/16` unclipped bare-legs tracks + `generalize` green |
| Kit unfolding Slice 1 (`kit.rs`, `solve`, `defect`, `certify`, `word_store`) | `Flying` offline | `main`, `kit_unfold` `5/5` + `doorcorridor` `T=32` restored |
| 26.2 law + `16³` `StateId` world spine + `WorldDelta` | `Flying` | `main`, jar witnesses + replays green |
| Live JNI resident channel (`resident.rs` + `ResidentRunner.java`) | `Load-bearing` (flat) / `Flying` (words) | `main` |
| Parked `wt/flight-ladder-sets` | `Sketch` (unmerged) | parked unmerged; core recursive `Word` lives on `main` |
| Tools/edits live bot/JNI wire, far map, ES `10⁹` bots | `Sketch` | awaits Slice 2 hotbar/use JNI & residual survival defects |

## Branches and open notes

- Kept until needed: `wt/port-fold` (`origin/wt/port-fold`, 26.2 jar suite with 9,117 witnesses and 32,252 / 32,366 modeled states), `wt/flat-ground` (untracked `lab/fixtures/flat-ground-replay/`), `wt/server-mind`, `wt/build-4d`, `wt/track-face`, `wt/hard-track-2`, `wt/l0-veto`, `wt/build-preview`, `wt/hearth-small`.
- Untrimmed region files `r.0.0.mca` / `r.0.-1.mca` stay uncommitted on `main` (`*.bak-20261114` backups beside them).
