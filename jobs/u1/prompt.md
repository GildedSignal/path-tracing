# U1: the kernel (slices C and D, with `count` on every word)

**Your spec** is section 2 of `C:\Users\PASCAL\Desktop\path tracing\jobs\u1-spec\report.md`: the header, decisions 1â€“6, the files, the components, the tests and the gate. Read it all, and section 3. The plan it serves, approved by Julien (S165, S166, S169): `docs\learning.md` section 0, `docs\representation.md` sections 10â€“11, `docs\scales.md`, `docs\vocabulary.md` (version 3), `docs\HANDOFF.md` (U1).

**Why:** Julien (S131): "the terrains complexity should be a mirror of compute's complexity here our path should take Âµs not tens of ms". And (S161): "we can't have two paths like that "easy" or "hard" it should be the same mechanism". U1 builds the kernel that every zone uses, behind the single entry point `solve`.

## Decisions already made (do not reopen them)

1. **Base:** your worktree is branched from `wt/kernel`, not `wt/3.2d-3`. `wt/kernel` already holds slices A and B, P1 (fixed after its judge), P2 (passed: exact binade maps) and P4 (offline food guard). Every path the spec writes under `D:\path-tracing-wt\u1\` is your worktree.
2. **P3 (the two-yaw lattice) is part of U1** (spec section 4, option B). Build its construction from `jobs\spec-fixes\report.md` lines 186â€“204 into `lattice.rs`, with `t3_two_yaw` as its isolated unit test in `zones_prereqs\t3.rs`.
3. **Frazzoli:** read section 3.1 of `C:\Users\PASCAL\Desktop\path tracing\jobs\frazzoli\report.md` (Argon read his papers in full).
   - Apply its confirmations and refinements for U1 where they agree with the spec or add a law fact.
   - Where one conflicts with the spec, keep the spec and record the conflict in `last.md`. It is never a reason to stop.
4. **P4's live check waits for U4.** U1's gate is offline, so accept P4's offline pass (spec section 4, option B).
5. **U1 stays inside `engine\words` and `engine\plan`** (spec section 4, option A). Leave `engine\native\` and `bot\` untouched; the bot's channel is wired at U3â€“U4.
6. **The goal's touch rule is about to change.** Branch `wt/track-face` (`engine\plan\src\goal.rs`, `engine\plan\tests\face_touch.rs`) replaces "any face of B" with "the clicked face". Build the inverse stop against `Goal::touched` as its one definition, in one place, so that merge changes one function. Do not merge `wt/track-face` yourself.
7. **No terrain-kind branch.** Kill test 8's planted fault (a fast path for uniform ground) must fail your gate. Kill test 17 (the million-block line, thought offline on a generator-backed superflat, Peaceful) is the gate.
8. **Priority:** run at normal priority. Ignore the spec's "Idle" (S158).
9. **You are the builder (Astra, S174: "opus and astra for important things").** Never run `scripts\launch.ps1`, `gpt.ps1` or `follow-up.ps1`.

## Done means

- The spec's gate commands pass, with outputs in `D:\path-tracing-evidence\u1\`.
- **`jobs\u1\last.md`** holds:
  - the files changed;
  - the million-line thinking time as a distribution (min, median, p99, max) in the release build, with what it includes;
  - kill test 8's planted-fault result;
  - each Frazzoli refinement applied or refused;
  - the replay commands.

A failed premise is a result: write it down and end `STATUS: DONE`. Never add a tolerance or a special case to pass. End with `STATUS: DONE` or `STATUS: BLOCKED: <why>`.
