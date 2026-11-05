Implemented and left uncommitted.

- Replay initializes the sprint latch exclusively from required `player.isSprinting`. Missing values fail with `missing isSprinting`.
- Removed tick-zero state seeding.
- Updated synthetic fixtures and regression coverage for key-release persistence, missing-field rejection, and post-tick seed rejection.
- Updated `lab/fixtures/README.md`, its example, and the stale engine README wording.

Validation passed with 39 tests passing, all four mutant builds succeeding, and `git diff --check` clean. Restoring the old seed assignment makes the regression fail.

The live-recording replay remains explicitly ignored because no recordings exist.