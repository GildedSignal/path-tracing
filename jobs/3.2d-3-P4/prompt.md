# 3.2d-3 premise job P2 (test T2): binary64 binade integer maps and the inverse stop

**Your spec** is section 3's "P4 (T4)" of `C:\Users\PASCAL\Desktop\path tracing\jobs\spec-fixes\report.md` (lines 208–235). Read all of it, and the parallelism schedule (lines 151–154). The 3.2d-3 spec it serves is `jobs\3.2d-3\spec.md`; the design is `jobs\zones-design\design.md` and its red-team `redteam-opus.md`.

**Why:** Julien (S131): "our path should take µs not tens of ms". T4 tests that a long sprint stays a closed form across food: the food guard (food > 6) is an event computed from the exhaustion recurrence, and an observed resource packet re-anchors it without re-simulating.

## Decisions already made (do not reopen them)
1. **Your worktree** is the one this launch gives you, branched from `wt/3.2d-3-P1`: slices A and B, plus P1's `engine\plan\tests\zones_prereqs\` harness with an empty `t4.rs`. Every path the spec writes under `D:\path-tracing-wt\3.2d-3\` is the same path in your worktree. Use `CARGO_TARGET_DIR=D:\path-tracing-target\3.2d-3-P4`.
2. **You edit only** `engine\plan\tests\zones_prereqs\t2.rs` and `jobs\3.2d-3-P4\*`. P2 runs at the same time on `t2.rs`; you may also edit `jobs/3.2d-3/verify.ps1` for `-Case T4Food` only.
3. **Use P1's measured values** where the spec needs the transient and cycle: 263 transient ticks and a 12-tick cycle. They are in `jobs\3.2d-3-P1\last.md` in your worktree.
4. **Peaceful only** (S136). **No game, server or client** is launched. Everything is `cargo --offline`.
5. **A failed premise is the result:** if the guard event is not exact, or a premise of the spec is false (a file, function or line that is not there), write what you found in `last.md` and end `STATUS: DONE`. Never loosen an assertion, add a tolerance, or change engine source.
6. **You are the Terra builder.** Never run `scripts\launch.ps1`, `gpt.ps1` or `follow-up.ps1`.

## Done means
- The spec's gate ran, with outputs in `D:\path-tracing-evidence\3.2d-3-P4\`.
- **`jobs\3.2d-3-P4\last.md`** holds:
  - the files changed;
  - each assertion's result;
  - the food-guard crossing tick from the two spawn bars on Peaceful, compared with GLM trial-food-crossing in lab/results;
  - the guard-event solve's timing distribution (min, median, max);
  - the replay commands.
- End with `STATUS: DONE` or `STATUS: BLOCKED: <why>`.
