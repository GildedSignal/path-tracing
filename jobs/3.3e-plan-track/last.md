Implemented [plan-track](/D:/path-tracing-wt/plan-track/engine/plan/src/bin/plan-track.rs) and moved the flight harness’s track loading, planning and export code into [the shared library](/D:/path-tracing-wt/plan-track/engine/plan/src/flight.rs).

- **Validation:** `grassjumpeasy`, mode 5 produced byte-identical harness and binary JSONL; the repeatable release comparison passed. `cargo test --offline` passed, and the footprint verification caught **12 of 12** mutants.
- **Strict choice:** this branch has no `PlayerState::from_bits`, so the command accepts only `--start settled` and refuses bits files with exit 3.
- The footprint script’s fixed log directory denied access. The same checks passed with only the log destination redirected to `D:\path-tracing-evidence\footprints-job33e`.
- Net change: **+72 lines**, including the moved harness code. No doctest fix was needed. Build directories remain untracked; nothing was committed, pushed or merged.