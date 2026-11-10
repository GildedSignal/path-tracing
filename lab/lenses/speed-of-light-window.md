# Lens: is the terrain-only c 19.81 or 19.90? (anomaly)

**Why it matters.** The planner's lower bound on T divides distance by the top speed c. `docs/HANDOFF.md` lists an open anomaly: run `local-20261109-1554-1-6-speed-of-light` reports a steady 19.810930 blocks per second for sprint-jumping on blue ice under a 2.0-block ceiling, and its window is said to count 199 displacements over a divisor of 200. `lab/numbers/Speeds.java` gives 19.903 for the same motion. A c that is too low makes the bound invalid. Settle which number is right.

**The question.**
1. Read that run's `reproduce.sh`, `speeds.tsv` and probe source, read-only, at `C:/Users/PASCAL/Desktop/path tracing/lab/results/local-20261109-1554-1-6-speed-of-light/`. Does its steady-speed window divide the displacement over N−1 ticks by N? Cite the line.
2. Recompute with the law on this checkout (`engine/law`, on `main` now): after a warm-up long enough for the cycle to repeat bit for bit, the displacement over a whole number of cycles divided by the ticks it took, as exact bits and in blocks per second. Do the same for the peak single tick.
3. Explain any gap left between the law's value and `Speeds.java`'s closed form: name the term that differs.

**What to produce.**
- `reproduce.sh`: builds the law and prints both computations.
- `dossier.md`: the claim in one sentence (the corrected steady speed, and whether the old window was off by 1/200), then the evidence and what could still be wrong.

**Deliver the first proof early.** Write `dossier.md` with `Claim: ` on its first physical line and a runnable `reproduce.sh` before expanding. Rust is at `D:/rust/cargo/bin` (`RUSTUP_HOME=D:/rust/rustup`, `CARGO_HOME=D:/rust/cargo`); keep the build inside your run's target folder. Write only in your run folder. No network, no live game, no production edits, no commits.
