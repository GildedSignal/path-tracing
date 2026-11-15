# 3.2d-3 premise job P1 (test T1): the observed-start transient is prepared once and reused with zero ticks

**Your spec** is section 2 of `C:\Users\PASCAL\Desktop\path tracing\jobs\spec-fixes\report.md` (lines 117–145). Read all of it, and section 3's parallelism schedule (lines 149–154): you create the `zones_prereqs` harness with empty `t2.rs`, `t3.rs` and `t4.rs` so P2 and P4 can later run in parallel. The 3.2d-3 spec it serves is `C:\Users\PASCAL\Desktop\path tracing\jobs\3.2d-3\spec.md`; the design is `jobs\zones-design\design.md` and its red-team `redteam-opus.md` in the same repository.

**Why:** Julien (S131): "the terrains complexity should be a mirror of compute's complexity here our path should take µs not tens of ms". T1 tests the premise that a straight run's transient is computed once per recurrence key and then applied with no tick simulation, whatever the click's tick, position inside the binade, or resource bars.

## Decisions already made (do not reopen them)

1. **Your worktree** is the one this launch gives you, branched from `wt/3.2d-3` (slices A and B merged). Every path the spec writes under `D:\path-tracing-wt\3.2d-3\` is the same path in your worktree. Use `CARGO_TARGET_DIR=D:\path-tracing-target\3.2d-3-P1`.
2. **The plain path** (`D:\path-tracing-wt\plain`) is read-only reference. Never copy or merge it.
3. **Peaceful only** (S136). The fixture world is Peaceful with mobs off.
4. **No game, server or client** is launched (S135: someone else is using the computer). Everything is offline: `cargo --offline`, at Idle priority.
5. **The integrator checked these premises on 15 November at 21:20, on `wt/3.2d-3`:**
   - `jobs\3.2d-3\verify.ps1` exists and accepts `-Case Prerequisites`;
   - `jobs\1.2-footprints\verify.ps1` holds 13 mutants;
   - `bot\tests\fixtures\native-state.hex` and `recorded-state.hex` exist;
   - `engine\law\src\codec.rs` has `StateQuestion` (line 28), `state_key` (30), `LegsRole` (72) and `Interner::label` (233);
   - `tracks.txt` lines 19–21 are `straight100`, `straight300`, `straight800`.
6. **The example values** in the spec (ticks 0, 77, 1543; x 300.5, 304.5, 315.25; the two resource bars) are the values to use.
7. **If `verify.ps1 -Case Prerequisites -Tests T1`** does not accept `-Tests`, add that parameter to it so the case checks only the named tests' manifest entries. This is the one script file outside `engine\plan\tests\` and `jobs\3.2d-3-P1\` you may edit.
8. **If the test fails** (the memo misses, a hot tick is called, or a bit differs), that is the result: write which assertion failed and its values in `last.md` and end `STATUS: DONE`. Never loosen an assertion, add a tolerance, or change engine source to make it pass. A failed premise is data for the integrator.

## Done means

- The gate in the spec's section 2 ran, and each command's output is in `D:\path-tracing-evidence\3.2d-3-P1\`.
- `jobs\3.2d-3-P1\last.md` holds: the files changed, the measured `transient_ticks` and `cycle_period`, the hot lookup's timing distribution (min, median, max over the starts), each assertion's result, and the commands to replay.
- End with `STATUS: DONE` or `STATUS: BLOCKED: <why>`.
