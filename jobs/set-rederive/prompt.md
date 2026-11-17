# Re-derive how a set is stored, after kill test 11 failed

You are the Opus Designer in a fresh context (AGENTS.md; S174: "opus and astra for important things"; S185: "Opus re-derives it"). You think, read, and run small offline experiments. You do not build, edit code or docs, commit, or launch a game, server or client. **The harness may block your file writes: return your full proposal as your final message**, and the integrator saves it to `jobs\set-rederive\proposal.md`.

## Read first (root `C:\Users\PASCAL\Desktop\path tracing\`)
1. `docs\PRINCIPLES.md` lines 1–47: the goal and Julien's philosophy, always on.
2. `docs\plan.md`, the plan.
3. `docs\learning.md` sections 0, 1, 2 and 6, and `docs\representation.md` sections 10–11. Line 179 is Julien's ruling S182: "a set is keyed by the exact arriving state … plus **plateaus** of the one coordinate a held word leaves free".
4. `jobs\unified-plan\plan.md` sections 1.2, 2 and 11. Section 11 says plateau finiteness was its weakest premise and that kill test 11 should run first.

## What failed
Astra built joints and sets (`D:\path-tracing-wt\u2-astra`, branch `wt/u2-astra`, record `jobs\u2\last.md` there; spec `jobs\u2\spec.md`; red-team `jobs\u2-redteam\report.md`).
- **Kill test 11 is red:** the plateau count per joint pattern grows under refinement on the gap pattern. Evidence: `D:\path-tracing-evidence\u2\kill11.log`.
- **The translation premise is red:** within one binade and one discrete event signature, adjacent input positions can map to outputs two ULPs apart, by rounding parity. Evidence: `D:\path-tracing-evidence\u2\translation-premise.log`. Astra notes that this parity is already represented in the core's exact integer maps (P2: `engine\plan\tests\zones_prereqs\t2.rs`; `engine\plan\src\events.rs` on `wt/u1-astra`).
- **The consequence:** joints currently certify exact single states only. That is sound, but it does not compress.
- **What works:** kill test 10 (the map holds his shortcut; 158 ticks against his 169 after the ladder exit), the exact entry keys, certificates bound to pattern and plateau, and the backward wave over certified edges.
- **Also read:**
  - the GLM `trial-offset-monotone` result: `lab\results\*trial-offset-monotone*\check.md`, which saw clean single-ulp flips and 12 → 14 → 12;
  - `D:\path-tracing-evidence\u2\u2-record.log` and `joints-check.log`.

## The question
**What is the right way to store the set of arriving states at a joint, so that it is exact, compresses where the law forgets, and stays one mechanism for every terrain?**

Think from the law outward. Do not patch the plateau idea.
- Is the free coordinate's structure better described by the exact integer maps P2 already built (residues, parity classes, a finite automaton over the bits) than by intervals?
- Does the growth under refinement come from the representation's choice of coordinate, from the gap pattern itself, or from something else? Use the logs, and run a small offline experiment if you need to.
- What would make the count bounded, and by what? Give a law fact, not a number of ours.
- What does each option cost (Fermi, stated inputs) at the two extremes: the million-block line, and a hard track as Julien defines it (S166)?
- What changes in the docs (`representation.md:179`) and in the joints-and-sets code, and what stays?

## Return
1. **The diagnosis:** why kill test 11 and the translation premise fail, from the evidence, with file:line.
2. **Two or three candidate representations,** each with its exactness argument, its compression where the law forgets, the one-mechanism test, a Fermi cost at both extremes, and the kill test that would refute it.
3. **Your recommendation,** and the exact replacement for `representation.md:179`, for Julien to rule on.
4. **What a builder must change on `wt/u2-astra`,** and the test that must pass first.

No made-up numbers; cite everything. Primary sources where you use mathematics. Be economical with reading: the files above, not the whole repository.
