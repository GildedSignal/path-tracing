# Lens: the idiot index census (cost paid against the cost the information requires)

**Why it matters.** Julien (12 November, S83): "we need to look at where our idiot index is highest biased by how often the thing is used and fundamental it is … too much work where it doesn't matter with input data that doesn't matter, duplicate work, not leveraging things." The idiot index of a step is what it costs today divided by a Fermi floor for what it has to compute. This lens produces the census the integrator's pass will reason from: every number measured or derived, nothing redesigned.

**Inputs.** `engine/` (law, world, words; `cargo run --release --bin bench`), `engine/words/tests/read_cost.rs`, `jobs/1.6-read-cost/`, `docs/footprints.md`, `docs/vocabulary.md` (sections 2, 10, 11), `docs/scales.md`, his 18 tracks in `engine/world/tests/fixtures/`, `lab/wiki/` (recorder-cost, certificate-size, gait-modulo, class-without-numbers).

**The census.** For one bot walking each standable track with the 54 programs (the class lane's walk protocol), instrument or sample and report, per operation:
1. **Micro (inside one tick and one piece):** law tick, collision queries, block reads by mask kind, footprint recording, pattern key hashing, table lookup, replay. For each: calls per piece, ns per call, share of the total.
2. **Duplicates:** how many block reads per piece hit a cell already read in the same piece, the same tick, the previous piece; how many collision queries repeat a query with identical arguments; how many replays start from a state bit-identical to an earlier replay's start; how many pattern keys are computed and never looked up.
3. **Inputs that do not matter:** per read kind, the share of reads whose answer could change without changing the piece's outcome (test by flipping the answer in a scratch copy on a sample, n ≥ 1000 per kind); state fields written every tick and read by nothing before being overwritten.
4. **The floor:** for each operation, a Fermi floor with its derivation (bits of information actually needed × a stated cost per memory access or float operation), and the index = measured ÷ floor.
5. **Weighted ranking:** index × share of total time, and separately index × calls per bot-second at 20 ticks per second; the top ten of each, with file and line.
6. **Meso and macro, derived only:** from the measured numbers, what one bot costs per second of travel today, and what 5 and 1000 bots cost if nothing is shared against if identical pieces are shared (use the measured recurrence rates). State each assumption.

**What to produce.** `reproduce.sh` regenerating every table; `dossier.md` with the claim in one sentence (the single largest weighted index and its evidence). Code only inside your run folder: copy crates to a scratch overlay to instrument them. Every number with n, p50, p90, max and the machine's load.

**Deliver the first proof early.** Write `dossier.md` with `Claim: ` on its first physical line and a runnable `reproduce.sh` before expanding. Rust: `PATH=/d/rust/cargo/bin`, `RUSTUP_HOME=D:/rust/rustup`, `CARGO_HOME=D:/rust/cargo`, and a `CARGO_TARGET_DIR` inside your run folder. Write only in your run folder. No network, no live game, no commits.

**Out of scope:** fixing anything, proposing a new representation, changing the law.
