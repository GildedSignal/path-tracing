# Lens: what caught L0's step-up mutant?

**Why it matters.** L0's live gate proves the law against the server only if each planted mutant is caught by the evidence it claims. Lane `step-up-reachability` (reproduced twice, `lab/wiki/step-up-reachability.md`) showed that no block geometry in 26.2 puts a step-up candidate within one float ulp of `0.6f`, so the step-up mutant is equivalent on block terrain. Yet L0's controls report it caught (HANDOFF anomaly 1). Either the mutant is not the one-ulp change the lane analysed, or something other than a step-up decision catches it, or the control passes for a wrong reason. A control that passes for a wrong reason makes the gate's other "caught" lines suspect.

**Inputs.** The worktree `D:/path-tracing-wt/l0-veto` (read only): `oracles/` (the 1.6 oracles, their mutants and controls, `verify.rs` files), `lab/gates/` (the gate scripts and `results/`, the latest run `20261111T1453381612953Z-fixtures`), `engine/law/src/movement.rs` (the step-up code). `lab/results/local-20261111-1756-1-2-step-up-reachability/`.

**The question.**
1. Find the step-up mutant: its exact source change, the file that plants it, and the control that reports it caught. Quote the lines.
2. Reproduce the catch offline from the recorded evidence: which run, which tick, which field differs first, by how many bits. Is that tick a step-up decision at the `0.6f` boundary, or something else the mutation also changes?
3. Say which holds, with the evidence: (a) the mutant differs from the one the reachability lane analysed; (b) it is caught through another path (name it); (c) the control reports "caught" without a differing tick.
4. Do the same check, briefly, for every other mutant the gate lists as caught: first differing run, tick and field. Flag each one caught by something other than its own named rule.

**What to produce.** `reproduce.sh` regenerating the table of mutants (source change, first differing run/tick/field); `dossier.md` with the claim in one sentence. Code only inside your run folder; copy what you need out of the worktree, never write in it.

**Deliver the first proof early.** Write `dossier.md` with `Claim: ` on its first physical line and a runnable `reproduce.sh` before expanding. Rust: `PATH=/d/rust/cargo/bin`, `RUSTUP_HOME=D:/rust/rustup`, `CARGO_HOME=D:/rust/cargo`, and a `CARGO_TARGET_DIR` inside your run folder. Write only in your run folder. No network, no live game, no commits.

**Out of scope:** changing the gate, the oracles or the law.
