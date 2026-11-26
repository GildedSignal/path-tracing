# Kit Edits in Field (`kit-edges-in-field`): Report

Branch `wt/kit-edges-in-field` over `6c310826` (`CARGO_TARGET_DIR=D:/path-tracing-target/job-kit-edges-in-field`). Completed and merged into local `main` (unpushed).

## What Changed

1. **Field-Edge Interface (`engine/plan/src/kit.rs`):**
   - Replaced per-verb search enumeration with `FieldEdit { Clear, Floor, Cushion }`, `HandStep`, and `KitView::{makes, edge, hand_step, closures, instantiate_step, click_block_target, book, window}`.
   - `KitView::edge` prices parallel body+hand marginals strictly from `law` + `BreakProjection` + `spec.restore_required`: single-tick overlapped interacts/placements cost `0` when overlapped inside `4.5` reach (plus break/restore ticks only where physically forced), irreversible transitions return `None`, and `FieldEdit::Floor` returns `(cost, anchored)` using `place_faces` so floating unanchored floors require a supported predecessor (`rn = 0`).
2. **One Backward `(cell, run)` Wave & Descent Search (`engine/plan/src/defect.rs`):**
   - Deleted `expand_kit_ops` (`5×5×2` box enumeration) and `KitCandidate` hand scores; net delta across `engine/plan/src/` is **-119 lines** (`+988 / -1,107` vs `6c310826`).
   - `Legs::of` derives `(h, up, down, reach)` from `law` gaits (`SPRINT_JUMP` / `WALK`). `TopoField::build` propagates backward over `(cell, run)` states using `KitView::edge` hyperedges (with grounded/supported `gy - 2` seeds and anchored-floor causality).
   - Phase B offers kit actions only from `kv.closures` + `topo.hands`, factored cleanly through `PhaseB::push_or_win` and `ArenaNode::seed`.
   - `exec_kit_chain` passes `Hand::next_aim` so `click_block_target` evaluates `KitTemplate::cell(&s, goal)` live per entered cell.
3. **Warm-Store Bracket Guard (`engine/plan/src/word_store.rs`, `defect.rs`):**
   - `WordStore::propose` returns `Vec<(WordId, bool)>` flagging exact `fine` matches vs coarse/head guesses (`best_guessed`). Phase B searches under Phase A's `T` bound whenever the best proposal is a guess.
4. **`predictedT` Telemetry (`bot/src/pathtracing/{ResidentRunner,Tracks}.java`):**
   - `ResidentRunner` stores `predictedT` (`takePredictedT()`) from the planner's `T`; `Tracks` resets it on run start and logs it on every finished `track run` row.
5. **Executable Kill Test & Regression Gates (`engine/plan/tests/kit_field_edges.rs`):**
   - `kill_test_no_tool_verbs_in_search_code` verifies `solve.rs`, `cut.rs`, `defect.rs` contain zero references to `KitOp::*`, `VerbKind::*`, `KitVerb::*`, `WRITE_SHAPE_*`, `WRITE_FALL_RESET`, `is_hand_toggleable`, `water_source_state`, `powder_snow_state`, `toggle_open_state`, `can_place_at`.
   - Guards `doorparkourblue` (`0` edits, `T <= 90`), `doorcorridor` (`< 132_000` learn ticks, `T <= 36`), and `grassjumpeasy` warm stability (`warm <= cold <= 40`).

## Verification (Offline, After Last Edit)

| Gate | Result |
|---|---|
| `git diff --shortstat 6c310826 -- engine/plan/src/` | `3 files changed, 988 insertions(+), 1107 deletions(-)` (**-119 net lines**) |
| `cargo test --offline -p plan --lib` | **5/5** PASS |
| `cargo test --offline --release -p plan --test kit_field_edges -- --nocapture` | **4/4** PASS (`1.01 s`, kill test + survival/warm regression gates) |
| `cargo test --offline --release -p plan --test kit_unfold -- --test-threads=1 --nocapture` | **6/6** PASS (`1.62 s`; `R_LEGO_1` `gap=5 T=49 / gap=6 T=58`, `step_up=55`, `midair=58 vs 159`, `wall=36`, `door=24`, `water=46`, `snow=42`; `G_KIT` `door 24==24 / breach 36==36` at `0` learn ticks; `doorcorridor` `30` open / `28` restore; `amertystcoral 24`, `ametwoodbox 48`, `mud` `31` open / `28` restored; `10,000` kits `561,922 B`) |
| `cargo test --offline --release -p plan --test generalize` | **5/5** PASS (`1.84 s`, incl. `staircase_compound_emergence_and_self_chain`) |
| `cargo test --offline -p plan --test hard_track` | **5/5** PASS (`0.73 s`) |
| `cargo test --offline -p plan --test face_touch` | **2/2** PASS |
| `cargo test --offline --release -p plan --test real_time -- --nocapture` | PASS (`57.84 s`, `0` refused, `0` warm `allocated_bytes`, `0` warm `law_ticks`; standing `16.75 µs` / p90 `25.8 µs`, moving `14.7 µs` / p90 `18.3 µs`, repeat `5.2 µs` / p90 `5.3 µs`) |
| `cargo test --offline -p pathtracing_native` | **9/9** PASS (`7/7` lib + `2/2` schema) |
| `bot\build.ps1` | Built `pathtracing.jar` (exit 0) |
| `bot\tests\test-native.ps1` | `PASS: every field/mask Java -> Rust -> Java; every unknown refused; pinned recording preserved` |

STATUS: DONE
