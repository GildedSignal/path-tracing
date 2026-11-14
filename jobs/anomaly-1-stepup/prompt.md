# Read job anomaly-1: how did L0's control report a mutant caught that cannot be caught on block terrain?

**Why.** HANDOFF anomaly 1. L0 is the gate that checks the Rust law against the real client. Its planted controls include a **step-up mutant** (a one-ulp change in the law's step-up rounding), and L0's controls reported it **caught**. But GLM run `local-20261111-1756-1-2-step-up-reachability` (reproduced by an independent checker) found that of **98,871** exact step-up candidates from block shapes, shulker peeks and vehicles, **none** rounds into the mutant's one-ulp window: on block terrain the mutant is *equivalent* to the law. Until we know how the control reported "caught", that "caught" does not count, and a gate whose control passes for the wrong reason is not a gate.

**Consumer.** L0's merge (branch `wt/l0-veto`, waiting only on its live gate) and the anomaly list.

**Read** (all read-only): the GLM run folder `C:\Users\PASCAL\Desktop\path tracing\lab\results\local-20261111-1756-1-2-step-up-reachability\` (`dossier.md`, `check.md`); the L0 branch at `D:\path-tracing-wt\l0-veto` and `D:\path-tracing-wt\l0-close`: find the step-up mutant (search for `step` in the mutant or control definitions, in `oracles/`, `jobs/1.3*/`, `lab/gates/` and `engine/law/tests/`), the test or replay that is said to catch it, and the run record of the catch; `C:\Users\PASCAL\Desktop\path tracing\jobs\1.3-l0-close\last.md`, `jobs\1.3b-frozen-counter\last.md`, `jobs\1.3c-phase-control\last.md`.

**Answer, each with file and line:**
1. Where the step-up mutant is defined: the exact source edit.
2. What reported it caught: which test, fixture or replay, and its recorded failure output.
3. Why it failed: a real step-up that rounds into the window (then GLM's census missed a case: name it), or a side effect of the edit on something other than step-up rounding (name what), or a test that fails for an unrelated reason.
4. Verdict: **genuine catch**, **wrong-reason catch**, or **undetermined** (say what would decide it).

Do not edit anything. Do not invent a mechanism: if a file you need is unreadable, name it. Under 40 lines.