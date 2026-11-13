# Job 3.2b-judge: judge the planner branch (you are the judge, in a fresh context)

You are in the worktree `D:\path-tracing-wt\planner` on branch `wt/planner`; the docs named below are in this worktree, and the job's spec is `jobs/3.2b-planner/prompt.md`. Change no file under version control; write scratch only under `D:/path-tracing-target/planner-judge`. Set `CARGO_TARGET_DIR=D:/path-tracing-target/planner`.

Read `AGENTS.md`, PRINCIPLES (goal, philosophy), `docs/learning.md`, `docs/vocabulary.md` sections 3, 10, 11, and `prompt.md` here. Review `git -C D:\path-tracing-wt\planner diff fa75a08..HEAD`; run `cargo test` from `engine/` in the worktree (target `D:/path-tracing-target/planner`); check the run records under `D:\path-tracing-evidence\planner\` and rerun one or two tracks. Nothing live.

1. Is every returned sentence certified by one end-to-end replay from the exact start state, with no path that returns an uncertified one?
2. Did anything in `engine/law` change arithmetic or add a rule? Jar tests 6 of 6?
3. Are the two dominance rules proofs (the tick counter in a world that never changes; fields every tick writes before reading)? Try a counterexample. Is the heuristic a true lower bound for the goal region (the detector's touch rule)?
4. Does any pruning, merge or closed-set key use the class or anything lossy? Do the sentinel 1e12 or the greedy order ever discard a state?
5. Tolerances, special cases per block, catalogues of jumps, tautological or weakened tests (the pinned 27 became 26; the scheduled-bridge assertion was rewritten).
6. Does the premise check on backward sets do what is claimed (one changed cell breaks exactly the sets that read it)?
7. Do the reported numbers match the run records (spot-check three tracks)? Is T on the same clock as his?
8. Why does A* alone find nothing: a bug, or the branching (64 aims × 24 pieces) with a weak bound? This matters most for the next spec.
9. Anything against the job's "Must not" list.

Final message: PASS, PASS WITH FINDINGS or FAIL; findings by severity with evidence and whether each blocks the merge; what could not be checked.

For question 8, the run records show (mode 3 is A* alone): about 3,000 expansions fill 1.5 M nodes, about 70% of children are dominated, and the bound discards none. Say whether the code confirms that reading: branching with a weak bound, duplicate programs, or a bug.
