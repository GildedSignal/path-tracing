# Lens: one parabola, one certificate (are an arc's cuts arithmetic?)

**Why it matters.** Julien (11 November, S79): "of course a parabola should be one certificate." Lane `certificate-size` (reproduced) measured that removing the body-block cut still leaves a mid-air arc cut about 3 times by other events; lane `gait-modulo` (reproduced) showed that on uniform ground the block crossings of a gait follow from one cycle's per-tick increments by exact floor arithmetic, provided clamped ticks snap onto their collision plane. If the same holds for an arc (takeoff to landing), a jump is one certificate whose crossings are computed, not replayed, and its worldtube is the swept cells (`docs/vocabulary.md` sections 2, 6 and 12, version 2).

**Inputs.** `engine/words` (`segment.rs`: the event rule; `program.rs`: the 54 programs), `engine/law` (movement), `engine/world/tests/fixtures/` (his 18 tracks), `lab/results/local-20261111-1756-1-1-certificate-size/` and `lab/results/local-20261111-1823-1-0-gait-modulo/` (read both dossiers first; reuse their probes).

**The question.**
1. On the standable tracks, for every arc (the ticks from leaving the ground to the next ground contact): list each event that cuts it today, by kind, with counts. Which are deterministic from the takeoff state alone in free air (no collision on the way), and which read the world?
2. For arcs with no collision before landing: are the per-tick increments a function of the takeoff state alone, bit for bit, wherever the arc is translated by whole blocks (state the binade scope)? Predict every block crossing and the landing tick by floor arithmetic from those increments and compare with the law over every such arc. Report agreement and each disagreement with its tick.
3. What must one arc certificate store (takeoff state bits, increments, the swept cells and their masks), how many cells is its worldtube against the sum of today's pieces, and what does one use cost (measured) against replaying its pieces?
4. How often does an arc recur at another place on the tracks under this key, against 1.2% for today's pieces?

**What to produce.** `reproduce.sh` regenerating every table; `dossier.md` with the claim in one sentence. Code only inside your run folder (a probe binary on the workspace crates). Every number from a run.

**Deliver the first proof early.** Write `dossier.md` with `Claim: ` on its first physical line and a runnable `reproduce.sh` before expanding. Rust: `PATH=/d/rust/cargo/bin`, `RUSTUP_HOME=D:/rust/rustup`, `CARGO_HOME=D:/rust/cargo`, and a `CARGO_TARGET_DIR` inside your run folder. Write only in your run folder. No network, no live game, no commits.

**Out of scope:** changing the word record, the class or the law.
