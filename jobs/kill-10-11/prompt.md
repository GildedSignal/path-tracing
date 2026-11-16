# Kill tests 10 and 11: do plateaus stay few, and does a reach map hold his shortcut?

**Consumer:** the U2 set record before it freezes (`jobs\u2-set-record\report.md`; red-team `jobs\u2-redteam\`), and kill tests 10 and 11 in `docs\learning.md` section 6. Test 11 "runs first": the whole set record rests on plateaus being few per joint pattern.

**The questions** are already written as four drafts. Read them all, and answer each with an offline experiment against our law (`engine\law`) on main:
- `lab\lenses\trial-plateau-refined.md` and `trial-plateau-entry-speed.md` (kill test 11);
- `lab\lenses\hardtrack-face-reach.md` and `reach-face-overhang.md` (kill test 10).

Where a draft names the jar as the judge, our law stands in for it: it is bit-exact against the jar on the L0 kept runs. Say so in the report.

## Decisions already made
1. **Offline only.** Write a test or an example under `engine\plan\tests\` or `engine\plan\examples\` in your worktree. Never touch `engine\*\src\`. Use `CARGO_TARGET_DIR=D:\path-tracing-target\kill-10-11`.
2. **Kill test 11:** count the plateaus per joint pattern as bisection over takeoff states is refined. Use the patterns the drafts name, plus the two from GLM's `trial-offset-monotone`: `lab\results\*trial-offset-monotone*\check.md`, which saw 12→14→12. Report each pattern's plateau count at every refinement level. The test is killed if a count grows without bound as bisection refines.
3. **Kill test 10:** build the reach relation over the **old** `hardtrack` build's faces with no knowledge of his run:
   - the old build is `D:\path-tracing-wt\hard-track-2\jobs\hard-track-2\old.build.txt`;
   - the faces are top faces, climbable cells and slime faces;
   - the bounds are conservative, from the law.

   Report whether it contains his route: the decoy lane's end → the alignment wall's top → the final ladder (`jobs\hard-track\shortcut.md`, `julien-428.jsonl`). The hard-track-2 certificate's machinery on branch `wt/hard-track-2` may be reused as a reference, but it is not on main: copy only what you need into your test.
4. **A killed premise is the result:** write it down and end `STATUS: DONE`. Never loosen a bound or add a tolerance.
5. **You are the Terra builder.** Never run `scripts\launch.ps1`, `gpt.ps1` or `follow-up.ps1`. No game, server or client.

## Done means
**`jobs\kill-10-11\report.md`:** per test, the verdict (holds or killed), the numbers with their exact inputs, the replay command, and what it means for the set record. End with `STATUS: DONE` or `STATUS: BLOCKED: <why>`.
