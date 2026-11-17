# Joints and sets (step 2 of the plan)

**Your spec** is the integrator's: `C:\Users\PASCAL\Desktop\path tracing\jobs\u2\spec.md`. Read it all. Behind it:
- the draft `jobs\u2-set-record\report.md`;
- the Opus red-team `jobs\u2-redteam\report.md`, with its offline check `jobs\u2-redteam\kt10\`;
- the plan `docs\plan.md` (step 2) and the full approved design `jobs\unified-plan\plan.md` (sections 1.2, 2 and 3);
- `docs\learning.md` sections 0, 2 and 6; `docs\representation.md` sections 10–11, with line 179 as Julien ruled it (S182: a set is keyed by the exact arriving state, plus plateaus of one coordinate).

**Why:** Julien (S161): "we can't have two paths like that "easy" or "hard" it should be the same mechanism". The core (step 1) handles uniform ground through `solve`, and hands off at the first obstacle with `NeedsJoints`. This step makes joints and sets real, so that the same `solve` goes on past obstacles.

**You are the builder, chosen for it (S174: "opus and astra for important things").** Where the spec is under-determined, use your judgement under the plan's rules, and record each decision and its reason in `last.md`.

## Decisions already made
1. **Base:** your worktree `D:\path-tracing-wt\u2-astra` is branched from `wt/u1-astra`, the core: all its gates pass. Build on its store, events and `solve`. Never fork a second entry point.
2. **A failed premise stops only what depends on it.** Build everything else, and record what is red and why.
3. **Normal priority,** offline only, `CARGO_TARGET_DIR=D:\path-tracing-target\u2`. No game, server or client. Never run `launch.ps1`, `gpt.ps1` or `follow-up.ps1`.

## First: the core's two gates that cannot fail (the judge, `jobs\u1-judge\last.md`)
Fix these before building on the core. Each fix is proved by a mutation that the fixed gate catches.
1. **The planted fast path** (`jobs\u1\verify.ps1:65-70`) only deletes the `Stage::Extend` label, and `solve.rs:87` writes that label before any work, so kill test 8 checks a trace the solver writes itself.
   - Plant a real shortcut instead: a branch that answers uniform ground without the corridor query or the store.
   - Make the gate detect it from what the solve did: counted corridor queries, store reads and the composed words. The stage labels alone are not evidence.
2. **"0 law ticks" is a hand increment** (`store.rs:197`). The judge's 1,000 uncounted `try_tick` calls per solve passed every gate. Count ticks where they happen: wrap the `World` that `law::try_tick` reads, as P1's fixed test does (`engine\plan\tests\zones_prereqs\t1.rs`), so any tick through any path is counted.
3. **The lower findings in the same report:**
   - the typed caps and program indices (`store.rs:8-10`);
   - the production `assert_eq!` that re-folds every map (`solve.rs:69`): move it to the tests;
   - the bundle lookup that only one test guards.
4. **Speed, after the gates:** the judge traced the ~600 µs warm time to `runs()` re-allocating its pieces and `phase_pair` cloning states per probe (`events.rs:237-263`), and gives a five-step path toward the plan's ~5 µs floor. Apply what keeps the result bit-exact, and report the new distribution.

## Done means
- The spec's tests pass or are recorded red, with outputs in `D:\path-tracing-evidence\u2\`. That covers kill test 11, kill test 10 (the reach map holds his route; report the best T after the ladder exit against his 169), the two-speeds soundness test, and the one-mechanism test.
- The core's gates still pass, including the 13 footprint mutants and the planted fast path.
- **`jobs\u2\last.md`** holds the files changed, every decision you made where the spec left room, the timings as distributions, and the replay commands.

End with `STATUS: DONE` or `STATUS: BLOCKED: <why>`.
