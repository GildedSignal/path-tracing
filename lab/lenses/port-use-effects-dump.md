# Lens: each item's use effects, dumped from the jar, and a bit-exact slowdown witness (jar reader)

**Why it matters.** The law panics on any tick with `using_item: true`. The rule is pinned (`lab/wiki/port-action-resources.md`: `LocalPlayer.modifyInput` offsets 12–41, `isSlowDueToUsingItem` offsets 0–34), but job 1.4b stopped, rightly: nobody extracted the per-item values, and the only witnesses are rounded. Bots eat, so the first flight needs it.

**The question.** In the 26.2 jars:
1. For every item in the registry, its `UseEffects` component (`speedMultiplier`, `canSprint`, and any other field), read by running the game's own registries, as raw bits. Which items carry a non-default value? What is the default, as raw bits?
2. One movement witness by direct execution of the client's own code: a player on flat stone holding forward while using an item, for 20 ticks: the input vector after `modifyInput` and the position and velocity after each tick, as raw bits. One with the default multiplier, one with a non-default item if any exists.

**What to produce.**
- `reproduce.sh`: regenerates the dump and the witnesses from the jars.
- `use-effects.json`: item id to its fields as raw bits, in the format of the existing block and attribute dumps the law reads (find them under `engine/` and say which you matched).
- `witness.json`: the per-tick bits of question 2.
- `dossier.md`: the claim in one sentence, the evidence type of each part (direct jar execution, bytecode, reconstructed arithmetic), and what was left out.

**Deliver the first proof early.** Write `dossier.md` with `Claim: ` on its first physical line and a runnable `reproduce.sh` before expanding; save the first output as soon as it exists. Use the jars under `D:/path-tracing-lab` and the JDK in `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin`. Write only in your run folder. No network, no live game, no production edits, no commits.

**Out of scope:** eating's food and saturation, cooldowns, the use animation, shields' blocking.
