# Lens: what a change of the law costs (does the vocabulary age well?)

**Why it matters.** Julien (12 November, S89): the system must age well and be antifragile. Every word is exact under one version of the law. A port fix, or a Mojang update, changes the law and so breaks some words. The design claims the durable asset is the solved puzzle (start state, world, goal, and the inputs of its solution), and that the vocabulary is a cache rebuilt from it: after a law change, replay every stored solution, keep what still arrives, and send only the failures back to search. That is cheap only if a law change breaks few solutions and the replay is fast. Not measured.

**Inputs.** `engine/law` with its mutant features (`mutant-axis-tie`, `mutant-step-up`, `mutant-edge-backoff`, `mutant-ceiling`, `mutant-fluid-wall-above` in `engine/law/Cargo.toml`; `jobs/1.2-footprints/verify.ps1` plants seven more by source edit: read it and reuse them in a scratch copy), `engine/words`, his 18 tracks, `engine/words/tests/vocabulary.rs` (the walk protocol).

**The question.**
1. Under the true law, record a corpus: for each standable track, the walks of the protocol as (start state bits, inputs per tick, the piece boundaries, each piece's end state bits and footprint).
2. For each mutant law (each is a small, real change of one rule): replay every recorded input sequence. Report the share of pieces whose end state changes, the share of walks whose final platform changes, the first tick of divergence (distribution), and the wall time to re-prove the whole corpus.
3. Is breakage local? For a changed rule, could the pieces to re-prove be found without replaying everything, from what each piece's footprint recorded (state reads, cell masks)? Measure: of the pieces that changed, how many had read the rule's inputs; of the pieces that did not read them, how many changed (must be 0, or say why).
4. From these numbers: the cost of a law change for a corpus of 10^4, 10^6 and 10^8 pieces, with and without the footprint filter.

**What to produce.** `reproduce.sh` regenerating every table; `dossier.md` with the claim in one sentence. Code only inside your run folder: copy the crates to a scratch overlay to build the mutant laws. Every number from a run.

**Deliver the first proof early.** Write `dossier.md` with `Claim: ` on its first physical line and a runnable `reproduce.sh` before expanding. Rust: `PATH=/d/rust/cargo/bin`, `RUSTUP_HOME=D:/rust/rustup`, `CARGO_HOME=D:/rust/cargo`, and a `CARGO_TARGET_DIR` inside your run folder. Write only in your run folder. No network, no live game, no commits.

**Out of scope:** changing the word record, the class or the law.
