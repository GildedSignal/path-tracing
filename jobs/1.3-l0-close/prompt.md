# Job 1.3: close L0

**Why.** Phase 1 (HANDOFF). L0 is frozen (ladder.md). Its full run refused at step-ups seed 17, tick 204, on speed: the sprint flag and movement-attribute provenance. Evidence: `D:/path-tracing-target/sprint-diagnostic-source-20261107T231317`. The original pairs are in `D:/path-tracing-evidence/l0-original-pairs`.

**Work.** Find the first divergent field at that tick. Name the rule in Mojang's 26.2 code that explains it (class, method, offset). Port that rule. Run the frozen L0 verdict again. Report any new first divergence the same way, until L0 passes or a refusal is outside the law's contract (then say why).

**Must not.** Weaken L0, its mutants or its comparison fields. Add tolerances. Run live gates without the rig lock, or while Julien drives the bot.

**Done means.** The final message gives each divergence, its rule, the fix, and the final verdict with the distribution it prints.

## Continuation (your diagnosis is accepted)

Proceed with the fix you described. The rig (`D:\path-tracing-lab`) and the evidence folder are now writable for this job.
- In the law, the sprint flag and the sprint speed modifier are independent state. The local sprint setter updates both. A synchronized flag (entity data) changes only the flag. This is the "server-driven" exactness class in engine.md: what the server sends is an input, never simulated.
- Extend the recorder's fixture schema with the pre-tick sprint flag and the sprint modifier's presence, which it doesn't record today, then re-record L0's corpus under the rig lock. Adding an input field isn't weakening the gate: the pass line, mutants and comparison fields stay as frozen.
- Then the frozen L0 verdict. Report each new first divergence with its rule as before, until L0 passes or a refusal falls outside the contract.
