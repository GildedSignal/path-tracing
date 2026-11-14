**The loader is not implemented, and job 3.3d is incomplete.** No source files were changed; net source lines are +0 / −0.

The baseline `cargo test --offline` passed using a fresh target directory, with some existing tests ignored. The footprint verification stopped before running its checks because Windows denied access to its fixed `workspace.log`; 12 of 12 is unverified.

The archived client snapshots I found predate `isUsingItem`, and I did not find saved `server-state.ps1` JSON sample pairs in the evidence directory. Those observations do not prevent implementing the loader, but they leave the requested real-input acceptance tests unresolved. The test run created an untracked `target-loader/` build directory.