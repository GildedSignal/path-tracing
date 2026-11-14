The new footprint test catches the missing key, but **the requested pair of unmodified exported cells remains unmet**. The two scaffolding states also differ in the sneak context. The test therefore clones the capped model and substitutes the inside-cell shape from the uncapped exported model.

- Green: `test inside_cell_shape_is_keyed_in_recorded_footprint ... ok`
- Mutant red: `test inside_cell_shape_is_keyed_in_recorded_footprint ... FAILED`; the two digests were equal.
- `verify.ps1`: **13 of 13 mutants caught**.
- `cargo test --offline`: passed; 11 tests ignored.
- Net change: **+28 lines** across [blocks.rs](D:/path-tracing-wt/inside-cell/engine/world/tests/blocks.rs:28) and [verify.ps1](D:/path-tracing-wt/inside-cell/jobs/1.2-footprints/verify.ps1).

The [mutant output](D:/path-tracing-deps/footprints-verification/inside-cell-unkeyed.log) and [workspace output](D:/path-tracing-deps/footprints-verification/workspace.log) are recorded. No commit, push, or merge was made.