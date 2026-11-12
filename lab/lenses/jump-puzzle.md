# Lens: one jump as a puzzle (search from the alphabet, from both ends)

**Why it matters.** Julien (12 November, S87–S89): parkour maps are the curriculum; each jump is a puzzle (from this block, stand on that block); a jump the vocabulary cannot do is the bottleneck, and only then is search spent. "thanks to T the mingled parkour 4D path just is not you just need to throw quite a bit of compute because the search radius is big." The design claims a jump is always solvable from the alphabet (one tick of held keys and an aim) by exact search in the law, and that searching from both ends (which takeoff states land on the target; which run-ups reach one of them) shrinks the radius. Neither is measured. The Fermi for a hard map rests on trials per jump.

**Inputs.** `engine/law`, `engine/world` (his 18 tracks in `engine/world/tests/fixtures/`: each track has a start A and goals B), `engine/words` (`program.rs`, `segment.rs`, `table.rs`: `pieces`, `invent`; `tests/vocabulary.rs` for how a bot is stood on A), `lab/wiki/arc-certificate.md` and `lab/wiki/gait-modulo.md` (arcs and gaits are arithmetic), `lab/wiki/speed-of-light.md`.

**The question.**
1. On each standable track, list the jumps (a takeoff platform and the next platform a route to B needs), with gap, height change and the smallest landing area. Rank them by how hard they look.
2. For the 10 hardest: solve each three ways and report trials (law ticks simulated), wall time, and the T (ticks) of the best solution found: (a) forward only, from the alphabet (every held-key combination, aims on the law's own yaw grid, iterative deepening on ticks); (b) forward with the existing 54 programs as moves; (c) from both ends: first the set of takeoff states (position on the platform, velocity, facing) from which some airborne input sequence lands on the target, then the run-ups that reach that set. State every pruning rule; prune only by a bound that is exact (a state that provably cannot arrive).
3. The envelope of one solution: keep the solution's inputs, slide the target platform (by whole blocks and by the shape grid) until it fails; then re-search at the first failing offset. Where does no solution exist at all, and what bound shows it?
4. Does a solution found on one jump transfer: replay its inputs, translated, on every other jump with the same gap and height. Report how many succeed bit for bit.
5. From these numbers: trials per new jump (n, p50, p90, max), and the cost of a map of 100, 1,000 and 10,000 jumps if a fraction f of them is new, for f measured on his tracks.

**What to produce.** `reproduce.sh` regenerating every table; `dossier.md` with the claim in one sentence. Code only inside your run folder (a probe binary on the workspace crates). Every number from a run.

**Deliver the first proof early.** Write `dossier.md` with `Claim: ` on its first physical line and a runnable `reproduce.sh` before expanding. Rust: `PATH=/d/rust/cargo/bin`, `RUSTUP_HOME=D:/rust/rustup`, `CARGO_HOME=D:/rust/cargo`, and a `CARGO_TARGET_DIR` inside your run folder. Write only in your run folder. No network, no live game, no commits.

**Out of scope:** changing the word record, the class or the law.
