# Lens: how far does a gait stay valid? (the corridor as one query)

**Why it matters.** Lane `gait-modulo` (reproduced) showed that on uniform ground a gait's cycle recurs bit for bit and its block crossings are floor arithmetic, so a long run needs one certified cycle and no replay per piece. What is not built is the other half: knowing how far the ground stays the same. Julien (12 November, S83): "we're not leveraging the generator mental model enough." The ground of a long run is a generator (the same cells repeated); the gait's worldtube is a generator too (one cycle's footprint, shifted by the cycle's displacement). Their agreement should be one cheap query, with an exact replay only where it first fails (`docs/vocabulary.md` sections 6 and 12, `docs/footprints.md`, `docs/PRINCIPLES.md` "From Emergent Space": store the generator, compute only surprises).

**Inputs.** `lab/results/local-20261111-1823-1-0-gait-modulo/` (read its dossier first; reuse its probe and predictor), `engine/words` (`gait.rs`, `segment.rs`), `engine/law` (`footprint.rs`, `pattern.rs`), `engine/world` (snapshots, the model table).

**The experiment.** Uniform ground (stone, blue ice, stone under a 2.0 ceiling: the fastest gait of each), one obstacle at distance d ∈ {8, 32, 128, 512, 2048} blocks along the run. Obstacle kinds: a full block in the body's path, a one-block hole in the floor, a slab step, a block changed beside the path that the body never touches, a block changed one cell outside every cell the footprint reads.
1. **Today:** replay every piece until the run's outcome changes. Report replays, ticks simulated, and time.
2. **Compiled:** certify one cycle; take the union of one cycle's footprint cells and masks; find by arithmetic the first cycle whose shifted footprint reads a cell that differs from the uniform pattern; advance n cycles by the arithmetic; replay exactly from the last cycle start before that cell. Report the same three numbers, the cost of the corridor check per block of travel, and whether the end state (every bit, at the first tick after the obstacle is passed or hit) equals method 1.
3. **The check itself, three ways:** cell by cell over the shifted footprint; over a run-length description of the ground along the travel axis (one compare per run); say what the snapshot would have to store for the second to be available on real terrain, and measure how long the uniform runs are on his 18 tracks along each gait's axis.
4. Does the block that the footprint never reads leave the compiled run untouched, and does every obstacle the footprint reads stop it at or before the right cycle? Any case where the compiled run continues past a cell method 1 reacts to falsifies the claim.

**What to produce.** `reproduce.sh` regenerating every table; `dossier.md` with the claim in one sentence. Code only inside your run folder. Every number from a run, with n, p50, p90, max.

**Deliver the first proof early.** Write `dossier.md` with `Claim: ` on its first physical line and a runnable `reproduce.sh` before expanding. Rust: `PATH=/d/rust/cargo/bin`, `RUSTUP_HOME=D:/rust/rustup`, `CARGO_HOME=D:/rust/cargo`, and a `CARGO_TARGET_DIR` inside your run folder. Write only in your run folder. No network, no live game, no commits.

**Out of scope:** changing the word record, the class or the law.
