# Hard track handoff

STATUS: DONE (offline job; in-game construction and Julien's human bound remain with the integrator).

The command-built world reaches B with full health through the frozen input sentence. The per-tick full-state digest is pinned; a second replay compares every bit, and the equivalent existing tiny-program sentence also passes plan::certify with the same endpoint bits. The default workspace test run passes; the capped mode-5 diagnostic returns no sentence, which is diagnostic only. Generated witness, timing, reach, decoy, control and build records hold the exact values and margins.

## Integrator action

Read report.md. Before the first clear, save the actual minecraft:max_block_modifications rule, temporarily raise it to the first fill volume recorded in build.md, execute the build list, and restore the prior rule. Build near the site with its chunks loaded. No live game, server or RCON was opened by this job. The fixture registration line is staged in lab/tracks/hardtrack.tracks.txt; append it to engine/world/tests/fixtures/tracks.txt after the built course is captured in the fixture world. The current region files do not contain this build.

## Commands run

Final verification from engine/, with CARGO_TARGET_DIR=D:\path-tracing-target\hard-track and HARD_RECORD=1:

- cargo test --offline (passed, including all four non-ignored hard_track checks; existing ignored tests remain ignored).
- cargo test --offline --release -p plan --test hard_track current_planner_mode_5 -- --ignored --nocapture (passed as a diagnostic; no sentence within the stated budget; see planner.md and planner.log).

Reproduction from the worktree root:

- powershell -NoProfile -File jobs/hard-track/check.ps1 -Diagnostic (full suite, optional release diagnostic, build facts, report and SHA-256 manifest).
- powershell -NoProfile -File jobs/hard-track/check.ps1 (final full suite after sentence certification was added; passed).
- powershell -NoProfile -File jobs/hard-track/check.ps1 -ReportOnly (refresh prose/metrics/hash manifest from those run records; does not rerun tests).

Earlier iterations used cargo test --offline -p plan --test hard_track -- --nocapture, with HARD_TRACE=1 for exploratory trajectories and HARD_RECORD=1 for certificate records. One early pocket-geometry assertion exposed overlap between the upper chute baffle and the false ice pocket. The baffle was re-derived as one vertical plane and the pocket boundary moved to that plane; the complete air-interior/wall certificate then passed. No law arithmetic or tolerance was changed. The successful shifted sentences that beat the frozen witness remain visible in timing.md; no fastest-route claim is made.

The inherited CARGO_TARGET_DIR was invalid (it contained the prompt); all successful runs explicitly used the path above. An optional rustfmt invocation was unavailable in the installed toolchain; no component was installed. The initial failed probe write used an extra engine/ path and created no file.

## Scope and cleanup

No existing tracked code was modified. All new code is the requested integration test and offline evidence/report scripts. No new search, crate, commit or push. The two pre-existing modified region files and their .bak-20261114 backups were left untouched. Superseded exploratory probe/certificate/aggregate logs were removed as the final job cleanup; their results are now the frozen regression checks and generated records. The dedicated build cache remains outside the worktree for the integrator's judge/replay.

## Every delivered file

- engine/plan/tests/hard_track.rs
- jobs/hard-track/bounds.md
- jobs/hard-track/build.md
- jobs/hard-track/build-facts.ps1
- jobs/hard-track/cargo-test.log
- jobs/hard-track/check.ps1
- jobs/hard-track/commands.txt
- jobs/hard-track/controls.md
- jobs/hard-track/inputs.sha256
- jobs/hard-track/last.md
- jobs/hard-track/planner.log
- jobs/hard-track/planner.md
- jobs/hard-track/report.md
- jobs/hard-track/report.template.md
- jobs/hard-track/timing.md
- jobs/hard-track/witness.md
- lab/tracks/hardtrack.build.txt
- lab/tracks/hardtrack.json
- lab/tracks/hardtrack.tracks.txt
- lab/tracks/hardtrack.witness.txt

Base commit: 3a6e066580a6a10b5f4096fc5cdd95eddcb350a9

Final read-only audit recomputed every SHA-256 in inputs.sha256, checked the workspace log for a passing witness and no failed test result, and checked this file inventory. It passed.
