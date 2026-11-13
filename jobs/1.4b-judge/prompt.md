# Job 1.4b-judge: the use-item slowdown, judged

You are the **judge** of a completed build, in a fresh context, from a different family than the builder (GPT Terra built it). Your verdict decides whether this branch merges into `main`. You are adversarial by role: find the way this could pass while being wrong.

**What is under judgment.** Branch `wt/use-item`, worktree `D:\path-tracing-wt\use-item`, head commit `41972f8`, job `1.4b-use-item`. It ports the use-item movement slowdown into the law.

**Read, in this order:**
1. `AGENTS.md` — "Truth", "No patches", and "Run records, merge door, component state".
2. `docs/engine.md` for the law's contract.
3. `jobs/1.4b-use-item/prompt.md` — the spec, **including the continuation section at the end**, which is the one it was actually built against. Note that the first attempt stopped on missing data and that this is the second.
4. `jobs/1.4b-use-item/last.md` — the builder's own report.
5. The lab dossier that unblocked it: `lab/results/local-20261113-1727-1-1-port-use-effects-dump/` — `dossier.md`, `use-effects.json`, `UseEffectsProbe.java`, `check.md`. The reproduced claim is that all 1,537 registered items bind an explicit `UseEffects` component and exactly seven deviate from `UseEffects.DEFAULT`.
6. The diff: `git -C D:\path-tracing-wt\use-item show --stat 41972f8` then `git -C D:\path-tracing-wt\use-item diff main...wt/use-item`. Read the whole diff.

**Judge each of these, with a verdict line and the evidence:**

- **Does the ported rule equal Mojang's?** The spec pins `LocalPlayer.modifyInput` offsets 12–41, `itemUseSpeedMultiplier` offsets 0–19, `isSlowDueToUsingItem` offsets 0–34, `canStartSprinting` offsets 31–35. Check the code against the cited offsets, not against the report's prose. "Close" is a FAIL. Pay attention to the **order** of the 0.98 smoothing, the item multiplier and the sneak factor: an order swap is bit-visible and easy to miss.
- **Is the item data really the game's, or ours?** This is the whole reason the first attempt was stopped. The multiplier and `canSprint` must be generated from the jar, not written by hand and not defaulted. Find what regenerates `engine/world/data/use-effects.tsv`, confirm it reads the jar, and confirm the jar sha1 matches the one the law already reads. **A hard-coded default anywhere in this path is a blocking FAIL**, however well commented.
- **Is there a patch?** Any epsilon, tolerance, retry, clamp, or per-item special case added so a check would pass. Name it with file and line. The seven deviating items are a particular risk: a branch on those seven by name would be exactly the wrong representation.
- **Is the witness real?** The spec requires red-then-green. Verify the apple witness fails on `main` because of the `unimplemented!("LocalPlayer.itemUseSpeedMultiplier")` panic and passes after, bit for bit against jar-derived values — and that the test was not weakened to pass. The report says "the test was updated to supply the newly required item fields"; check that this is a necessary signature change and not a relaxed assertion.
- **Sprint semantics.** The report claims sprint *start* is refused while existing sprint continuation is unchanged, and that the dossier says retention does not read the predicate. Confirm both against the dossier.
- **Growth.** Report the net lines. The diff is about 1,649 insertions, most of it generated data: say how much is production code, how much is the generated table, and whether the commit's `Bought:` line is honest about it.
- **Gates.** Run `cargo test --offline` from `engine/` on the branch, with `PATH` plus `D:\rust\cargo\bin`, `RUSTUP_HOME=D:\rust\rustup`, `CARGO_HOME=D:\rust\cargo`, `CARGO_TARGET_DIR=D:\path-tracing-target`. The machine is busy, so a link failure on a locked binary is environmental — say so rather than calling it a code failure. If you cannot run it, say so explicitly instead of assuming.

**Rules for you.** Do not fix anything. Do not commit, push, merge or modify any file in the repo or the worktree. Do not start a server or client. Do not soften a finding because the builder explained itself well. If the spec was unclear, say the spec is at fault rather than blaming the build.

**Return** exactly one of `PASS`, `PASS WITH FINDINGS`, `FAIL`, then: the blocking findings each with file:line and why it blocks; the non-blocking ones; the net lines split as asked; the gate results; and one closing sentence on whether this merges as-is, merges after named fixes, or goes back to the builder. At most 60 lines.
