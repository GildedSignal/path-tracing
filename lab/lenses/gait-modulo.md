# Lens: a gait cycle as a modulus (are block crossings arithmetic?)

**Why it matters.** Julien (11 November): "a cycle of gate is related to modulous no does that help?" A gait is a program's attractor (`docs/vocabulary.md`, section 6, version 2): after one cycle every field but position should repeat its bits, and position advance by the cycle's displacement Δ. Then the bot is (segment, phase) with phase = tick mod period, and the ticks where the body crosses into the next block would follow from Δ by arithmetic modulo the block (a Beatty-like sequence), with no replay. The red-team of 11 November found the class's surprises concentrate on exactly that: which tick the body crosses a block boundary. If the crossings are arithmetic, those surprises disappear and the block-change cut stops mattering on uniform ground.

**Inputs.** `engine/words` (`gait.rs`: the attractors found per ground; `program.rs`: the 54 programs; `segment.rs`: the event rule), `engine/law` (movement, its float order), `engine/world/tests/fixtures/` (his 18 tracks; flat runs of one block type are the cleanest case), `docs/footprints.md` section 3 (translation scope: integer shifts are bit-exact only inside one binade).

**The question.**
1. For each gait `engine/words` finds on open ground (stone, ice, blue ice, soul sand, slime, honey, under a 2.0 ceiling): does every state field but position repeat bit for bit after one period? Report the period, Δ per axis as bits, and the first cycle where anything else differs (food and exhaustion are guards, report them apart).
2. Is position after n cycles exactly start + n·Δ in IEEE arithmetic? Measure where it stops being exact: across a binade (1024, 2048, …), at large n, and say why (rounding of position + per-tick deltas).
3. Predict the block-crossing ticks of a long run from Δ alone (floor arithmetic on start + k·δ per tick inside the cycle), then compare with the law's own crossings over at least 10³ cycles per gait. Report agreement per gait and every disagreement with its tick.
4. Consequence: on uniform ground, how many replays does a 10³-block run cost today (one per segment) against the cycle memo plus the arithmetic (one per cycle, or one in all)?

**What to produce.** `reproduce.sh` regenerating every table; `dossier.md` with the claim in one sentence. Code only inside your run folder (a probe binary on the workspace crates). Every number from a run.

**Deliver the first proof early.** Write `dossier.md` with `Claim: ` on its first physical line and a runnable `reproduce.sh` before expanding. Rust: `PATH=/d/rust/cargo/bin`, `RUSTUP_HOME=D:/rust/rustup`, `CARGO_HOME=D:/rust/cargo`, and a `CARGO_TARGET_DIR` inside your run folder. Write only in your run folder. No network, no live game, no commits.

**Out of scope:** changing the word record, the class or the law.
