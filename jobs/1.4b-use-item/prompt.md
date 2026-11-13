# Job 1.4b: the use-item slowdown enters the law

**Why.** The law panics today (`engine/law/src/movement.rs`: `unimplemented!("LocalPlayer.itemUseSpeedMultiplier")`) on any tick with `using_item: true`. Bots eat, so the planner will meet it.

**Consumer.** The executor and the first flight (HANDOFF roadmap item 2); food as a constraint.

**Read.** `AGENTS.md`, `docs/engine.md`, `lab/wiki/port-action-resources.md`, and in the main checkout `C:\Users\PASCAL\Desktop\path tracing\lab\results\` the run folder `local-20261109-2251-1-4-port-action-resources` and the later run named in that wiki page whose claim begins "In 26.2 the use-item movement slowdown is a client-side, per-item data-driven law" (dossier, `reproduce.sh`, witnesses, any `candidate.patch`).

**Branch.** You are in `D:\path-tracing-wt\use-item` on `wt/use-item`, cut from `main`.

**Build.** Port exactly what the dossiers pin from the jar: `LocalPlayer.modifyInput` scaling the horizontal input by the used item's `UseEffects.speedMultiplier` after the 0.98 smoothing, and `isSlowDueToUsingItem` for sprint. The multiplier and `canSprint` are data of the item, read from the game's own dump, never constants of ours. Witness first: a test that is red on main (it panics) and green after, bit for bit against the dossier's jar-derived values. Where the dossier does not pin something you need, stop and say so; do not guess.

**Must not.** Add a tolerance, a special case per item, or a rule the jar evidence does not show. Touch `bot/`, `docs/`, `engine/plan`, or `engine/words` beyond what compiles the new field. Run anything live. Commit, push or merge.

**Done means.** The jar suite still 6 of 6; `jobs/1.2-footprints/verify.ps1` still 12 of 12 mutants; `cargo test` from `engine/` passes; a final message with the rule as ported (class, method, offsets cited from the dossier), the witness red then green, the net lines, and anything left out.

---

## Continuation, 13 November: the dump exists now

Your earlier run stopped, rightly, because no per-item `speedMultiplier` / `canSprint` data path existed. It does now, reproduced by an independent checker.

**The new input.** `C:\Users\PASCAL\Desktop\path tracing\lab\results\local-20261113-1727-1-1-port-use-effects-dump\` — read `dossier.md`, `use-effects.json` (the dump: every registered item with its `UseEffects` component), `UseEffectsProbe.java` (how the dump is produced from the jar) and `reproduce.sh`. The reproduced claim: in 26.2 all 1,537 registered items bind an explicit `UseEffects` component and exactly seven deviate from `UseEffects.DEFAULT`. The jar sha1s are in the claim; check them against the jar the law already reads before you trust a single value.

**What changes.** The multiplier and `canSprint` are now read from the game's own dump, as the job always required. The dump enters the repo the way block data already does — as the generator, keyed by item, produced by a committed exporter step, not as a hand-written table and not as a constant. Follow the pattern `engine/world` already uses for block data; if that pattern does not fit items, stop and say why rather than inventing a second one.

**Still in force, unchanged:** the witness red-then-green, no tolerance, no per-item special case, no guessing anything the dossier does not pin, no live run, no commit. If the seven deviating items make the general path look like overkill, say so with the Fermi number — but still read the dump, because a constant of ours is exactly what this job exists to avoid.

**Done means,** additionally: your final message names the seven deviating items, says where the dump now lives in the repo and what regenerates it, and states the net lines.
