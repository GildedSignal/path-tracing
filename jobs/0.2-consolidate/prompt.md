# Job 0.2: finish verifying the consolidated engine line

**Why.** Phase 0 of the approved plan (HANDOFF, 9 November): one line of truth. The integrator has merged `wt/world`, `wt/law`, `wt/raw-fluid-export`, `wt/glm-lab`, `wt/tracks` and `main` into `wt/engine`, on top of `wt/l0`. This worktree is that branch. `wt/clock` is not merged yet; it waits for its review.

**Already green.**
- `cargo test --offline` from `engine/`: 81 passed, 6 ignored (live recordings).
- `bot/build.ps1`, then `bot/tests/test-tracks.ps1`. The mixin audit was taught local capture (commit `f0bcea2`).
- In `lab/gates/tests`: `test-courses`, `test-exporter`, `test-fixture-json`, `test-fixture-state`, `test-record-fixtures` and `test-server-state`.

**Your work.**
1. `lab/gates/tests/test-run-gate-1.2.ps1` exits 1 on this line. Find out whether it also fails on `wt/l0` alone, which would make it a pre-existing failure. If the merge broke it, fix the cause, never the check. If it was already broken, say why, and fix it only if the cause is a stale test rather than a broken gate.
2. Run `lab/tests/test-harness.sh` under Git Bash; it is slow. Report its result, and fix what the merge broke.
3. Run every other self-test you can find that needs no live server or Minecraft client. Don't start the server or clients: the rig may be in use.

**Must not.** Change any law behaviour. Delete or weaken a check. Touch `wt/clock`. Commit: you can't write git metadata, so leave your changes in the worktree and the integrator commits them.

**Done means.** Your final message lists every suite you ran, with its result; each failure and its cause; each change you made, with the reason; and anything you could not run, with the reason.
