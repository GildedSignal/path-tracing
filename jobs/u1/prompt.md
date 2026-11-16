# U1: the kernel (slices C and D, with `count` on every word)

**Your spec** is the integrator's, `C:\Users\PASCAL\Desktop\path tracing\jobs\u1\spec.md`. It adopts section 2 of the draft `jobs\u1-spec\report.md` (read that too) and overrides it in five places; where they disagree, `spec.md` wins. The kernel's judge (`jobs\kernel-judge\last.md`) passed P2 and P4 with findings: P4's food threshold `6` is typed in its assertions; in U1, take every threshold from the law. The plan it serves, approved by Julien (S165, S166, S169): `docs\learning.md` section 0, `docs\representation.md` sections 10–11, `docs\scales.md`, `docs\vocabulary.md` (version 3), `docs\plan.md` (step 1, the core), and the full approved design, `jobs\unified-plan\plan.md` (sections 1, 2 and 10).

**Why:** Julien (S131): "the terrains complexity should be a mirror of compute's complexity here our path should take µs not tens of ms". And (S161): "we can't have two paths like that "easy" or "hard" it should be the same mechanism". U1 builds the kernel that every zone uses, behind the single entry point `solve`.

**You are the builder of the algorithm's core, chosen for it (S174: "opus and astra for important things").** Where the spec is still under-determined, use your judgement under the plan's rules. Record each such decision and its reason in `last.md`, and do not stop on it.

## Decisions already made (do not reopen them)

1. **Base:** your worktree `D:\path-tracing-wt\u1-astra` is branched from `wt/kernel`, not `wt/3.2d-3`. `wt/kernel` already holds slices A and B, P1 (fixed after its judge), P2 (passed: exact binade maps) and P4 (offline food guard). Every path the spec writes under `D:\path-tracing-wt\u1\` is your worktree.
2. **P3 (the two-yaw lattice) is part of U1** (spec section 4, option B). Build its construction from `jobs\spec-fixes\report.md` lines 186–204 into `lattice.rs`, with `t3_two_yaw` as its isolated unit test in `zones_prereqs\t3.rs`.
3. **Frazzoli:** read section 3.1 of `C:\Users\PASCAL\Desktop\path tracing\jobs\frazzoli\report.md` (Argon read his papers in full). Apply what is right for U1, and record each refinement applied or refused, with the reason.
4. **P4's live check waits for U4.** U1's gate is offline.
5. **U1 stays inside `engine\words` and `engine\plan`.** Leave `engine\native\` and `bot\` untouched; the bot's channel is wired at U3–U4.
6. **The goal's touch rule is about to change.** Branch `wt/track-face` (`engine\plan\src\goal.rs`, `engine\plan\tests\face_touch.rs`) replaces "any face of B" with "the clicked face". Build the inverse stop against `Goal::touched` as its one definition, in one place, so that merge changes one function. Do not merge `wt/track-face` yourself.
7. **No terrain-kind branch.** Kill test 8's planted fault (a fast path for uniform ground) must fail your gate. Kill test 17 (the million-block line, thought offline on a generator-backed superflat, Peaceful) is the gate.
8. **Normal priority** (S158). Use `CARGO_TARGET_DIR=D:\path-tracing-target\u1-astra`.
9. **Never run** `scripts\launch.ps1`, `gpt.ps1` or `follow-up.ps1`. No game, server or client.

## Done means

- The spec's gate commands pass, with outputs in `D:\path-tracing-evidence\u1\`.
- **`jobs\u1\last.md`** holds:
  - the files changed;
  - the million-line thinking time as a distribution (min, median, p99, max) in the release build, with what it includes;
  - kill test 8's planted-fault result;
  - each Frazzoli refinement applied or refused;
  - each place you departed from the spec, and why;
  - the replay commands.

A failed premise is a result: write it down and end `STATUS: DONE`. Never add a tolerance or a special case to pass. End with `STATUS: DONE` or `STATUS: BLOCKED: <why>`.
