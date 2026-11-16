# U1 spec: the kernel (slices C and D, with `count` on every word), decision-free for Terra

**Consumer:** Terra's U1 build on the integration branch, step U1 of the approved plan (S165; `docs\HANDOFF.md` roadmap, U1). Its gate is kill test 17, the million line through `solve` (`docs\learning.md` section 6). Terra halts at its first open question, so leave none open.

You read only, and write only your report. Read every file by its absolute path (root `C:\Users\PASCAL\Desktop\path tracing\`).

## Read
- **The approved plan**, folded today: `docs\learning.md` (section 0 above all), `docs\representation.md` (sections 10–11, "Thinking floors"), `docs\scales.md` (sections 2, 4.3, 4.7), `docs\vocabulary.md` (version 3), `docs\engine.md` ("The contract"), `docs\HANDOFF.md` (U0–U9), and `jobs\unified-plan\plan.md`, `fold.md`, `questions.md`.
- **The old spec's rows C and D:** `jobs\3.2d-3\spec.md`, lines 104–117. Slices E, H and K are replaced; never carry them over.
- **What exists:** slices A and B on `D:\path-tracing-wt\3.2d-3` (`engine\native`, `engine\law\src\codec*.rs`, `bot\src\pathtracing`).
- **The kernel tests:**
  - P1, the transient memo: `D:\path-tracing-wt\3.2d-3-P1\jobs\3.2d-3-P1\last.md`. Fixed after the judge: 48/54/78 µs hot path in debug, including 8 world queries.
  - P2, the binade maps, rerunning now with the law's own half-width: `D:\path-tracing-wt\3.2d-3-P2\jobs\3.2d-3-P2\last.md`.
  - P4, the food guard. Offline it passes, and on Peaceful there is no food crossing: `D:\path-tracing-wt\3.2d-3-P4\jobs\3.2d-3-P4\last.md`.
  - The P1 judge: `jobs\3.2d-3-P1-judge\last.md`.
- **Frazzoli's maneuver automaton**, if `jobs\frazzoli\report.md` exists when you start; Argon is reading it in parallel. Otherwise note it as pending.

## Write `jobs\u1-spec\report.md`
1. **Premises:** every function, file and line the spec relies on, checked to exist on `wt/3.2d-3` or on the P1/P2/P4 branches. Mark each EXISTS, DIFFERS or MISSING, and give the evidence.
2. **One decision-free Terra job spec for U1.** It gives:
   - the base branch;
   - the worktree;
   - the exact files to create and edit;
   - `count` as an argument of every word;
   - the generator store keyed by `LabelId`;
   - the event maps: binade crossings and food as events, with the P2 kernel when it lands, or a marked dependency;
   - `solve` as the single entry point: bracket, extend the biggest word until its certificate breaks, cut, compose. In U1 only the uniform-zone case is exercised, through the same code path the later steps extend. **No terrain-kind branch.** Kill test 8's planted fault, a fast path for uniform ground, must fail the gate.
   - the exact tests, including kill test 17 offline at 10⁶ blocks on a generator-backed superflat, Peaceful;
   - the gate commands;
   - what it must not touch.
3. **What it consumes from P2 and P4** and how it waits for P2 if P2 is not done. Say what can start now.
4. **Open questions you could not close.** Each one with the options and your recommendation. These go to the integrator, not to Terra.

Rules:
- **No made-up numbers.** Constants come from the law, a measurement, or a stated Fermi input.
- **No tolerances, special cases or terrain-kind code paths.**
- **Cite file:line for every claim;** where something is absent, write "not found in" with the files checked.
