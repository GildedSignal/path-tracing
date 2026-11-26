# Kit Edits in Field (`kit-edges-in-field`)

- Replaced `expand_kit_ops` (`5×5×2` box enumeration) and `KitCandidate` hand scores with `TopoField` parallel-overlapped hyperedges priced via `KitView::edge` (anchored floor precondition + `ObligationLedger`) and descent-only `kv.closures` + `topo.hands` (`-119` net lines in `engine/plan/src/` vs `6c310826`).
- Added live `next_aim` cell target checks in `exec_kit_chain`, coarse/guess tracking in `WordStore::propose`, `predictedT` logging in `ResidentRunner.java` / `Tracks.java`, and executable `kit_field_edges.rs` kill/regression gates.
- All offline gates green after last edit: `plan --lib` 5/5, `kit_field_edges` 4/4, `kit_unfold` 6/6 (`doorcorridor` 28t restored, `amertystcoral` 24t, `ametwoodbox` 48t, `mud` 28t restored / 31t open, `G_KIT` `door` 24==24 & `breach` 36==36 at 0 learn), `generalize` 5/5, `hard_track` 5/5, `face_touch` 2/2, `real_time` 0 B warm alloc (`16.75 / 14.7 / 5.2 µs`), `pathtracing_native` 9/9, `bot/build.ps1` + `bot/tests/test-native.ps1` PASS.

STATUS: DONE
