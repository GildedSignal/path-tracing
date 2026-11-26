# Kit edits as parallel-overlapped edges in the field

**Goal:** Put kit verbs into `TopoField` strictly as parallel-overlapped edges in `T` (deleting `expand_kit_ops` enumeration and hand scores), guard warm `WordStore` hits against bracket gaps, log `predictedT` on `track run` rows, and keep every `plan`/`native`/`bot` gate green.

## Specs & Post-Mortem (Read First)
- `map/detail/representation.md` section 4 (*No new search code per tool* / *Killed if: adding a tool needs search or geometry code*), section 5 (Causal Hypergraph: `(min, +)` choices over `(max, +)` causal edges), section 10 (*The kit, folded in*, parallel-overlap marginal cost & `UB - LB_kit` bracket gap), section 11.
- `map/principles.md` 1, 4, 6, 7, 8, 9, 22 (no bandaids), 27 (no made-up numbers).
- `jobs/postmortem-kit-edits/report.md`.

## What Fails Today
1. `engine/plan/src/defect.rs::expand_kit_ops` enumerates every door, gate, trapdoor, breakable forward block, and placement inside a `5×5×2` box per Phase B node with hand weights (`score: 3`, `5 + stand_d.min(200)`), burning `240,067` learn ticks into `NeedsJoints` on `doorparkourblue` (229 door cells, bare legs `T = 90`) and `142,638 > 132,000` on `doorcorridor` with 1 door open.
2. `kit_step_cost` carries made-up weights (`3`, `7`, `2`, `+5`, `+4`, `11`, `min(200)`), and serial toll pricing (`+5t` click) violates parallel body+hand execution where inside `4.5` reach a single-tick click costs `0` marginal `T`.
3. Warm `WordStore` hits on `grassjumpeasy` drifted `40 -> 48 -> 59` (vs cold `38`) because cached arrivals close the search even when `T_cached - LB_kit` stays wide, and `track run` rows omit `predictedT`.

## Architecture & Mechanism to Build

1. **One Field with Parallel-Overlapped Edges (Zero Magic Weights):**
   - Derive every cell edge cost from `law` + `KitView` + `ObligationLedger`:
     - Legs movement cost comes from the kit's gait speed in `law` (no `3/7/2/+5/+4`).
     - Because the body channel (`WASD/jump/yaw`) and hand channel (`look/use/attack` inside `4.5` reach) execute in parallel (`ΔT = max(T_move, T_hand) - T_move + coupling`), a single-tick overlapped toggle or placement edge carries **`0` marginal toll** (undo debt `0` while overlapped inside reach, non-zero only when a wait or detour is physically forced by `law`).
     - Multi-tick break edges carry `max(0, break_ticks - reach_overlap)` plus restore wait if physically forced; irreversible transitions cost `∞`.
   - Model the field output cleanly as an edge-interface action (`Move` vs `Edit(field_edge)`) aligned with our causal hyperedge contract (`causes -> effect, parallel_T`).

2. **Descent Search Only (Delete Enumeration):**
   - Phase B offers an edit **only** where the backward `min` field's descent requires that transition (or an open `ObligationLedger` entry must close inside reach). Bare legs is literally the empty kit — the exact same loop with zero verb edges.
   - **Delete** `expand_kit_ops`'s `5×5×2` per-verb loops and hand-scored `KitCandidate` ranking. Net lines in `engine/plan` **must fall**.

3. **Executable Kill Test:**
   - Add an automated test (or `tests/` check in `engine/plan`) that fails if `solve.rs`, `cut.rs`, or the search portion of `defect.rs` names a specific tool verb or `KitOp` variant outside the abstract field-edge interface (`representation.md` §4 guard).

4. **Warm-Store Bracket Guard & `predictedT` Telemetry:**
   - Ensure a warm `WordStore` / `CachedRoute` proposal in `solve.rs` / `word_store.rs` does not close when a wide bracket gap vs `LB_kit` lets a shorter route win, so repeated cross-track runs never degrade `T` (`grassjumpeasy` stays tight instead of drifting `40 -> 59`).
   - Record `predictedT` cleanly on `track run` rows in `bot/src/pathtracing/{ResidentRunner,Tracks}.java` (wiring the planner's `T` cleanly over existing resident output without breaking `NativeChannel` tests).

## Verification Gates (Clone Only — Do NOT Run Live Game Servers)
- Environment: `CARGO_TARGET_DIR=D:/path-tracing-target/job-kit-edges-in-field`, `PATHTRACING_DEPS=D:\path-tracing-deps\data`, `RUSTUP_HOME=D:\rust\rustup`, `CARGO_HOME=D:\rust\cargo`, `cargo ... --offline`.
- All must pass after final edits:
  - `cargo test --offline -p plan --lib` (plus new `doorparkourblue` live survival kit `Ready` with `0` edits & `T <= 90`, and `doorcorridor` restored from both 9 Nov fixture and 1-door-open `< 132,000` learn ticks without raising the cap)
  - Kill-test lint green
  - `cargo test --offline --release -p plan --test kit_unfold -- --test-threads=1`
  - `cargo test --offline -p plan --test generalize`
  - `cargo test --offline -p plan --test hard_track`
  - `cargo test --offline -p plan --test face_touch`
  - `cargo test --offline --release -p plan --test real_time -- --nocapture` (`0 B` warm alloc; medians not worse than `12.75 / 13.90 / 5.10 us`)
  - `cargo test --offline -p pathtracing_native`
  - `.\bot\build.ps1` and `.\bot\tests\test-native.ps1`
- Commit locally on `wt/kit-edges-in-field` (UTF-8 no BOM `-F`, valid `Map:` / `Bought:` trailers; never stage `archive/turns/inbox.md` or `.mca` files; never push).
- Write `jobs/kit-edges-in-field/report.md` and `last.md` inside clone ending with `STATUS: DONE` (or `STATUS: BLOCKED: <reason>`).
