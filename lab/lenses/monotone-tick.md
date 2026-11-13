# Lens: is the law's tick monotone per axis, so that corners bound a box of states?

**Why it matters.** The next planner carries sets of states, not points. If the law's per-axis update is monotone in (position, velocity), a box of states maps to the box between its corners' images, bit for bit: a set is checked by a few replays of its corners and needs no geometry code. If it is not, checking a set is a search in disguise and the design changes (`jobs/3.2d-sets/prompt.md`; the Opus red-team's claim C3, read from the law, not measured).

**The question.** With the Rust law (`engine/law`, built in this worktree) on Julien's 18 track fixtures and the 54 programs of `engine/words`:
1. Take states the programs actually reach. Around each, build a box: the state and neighbours that differ only in one horizontal position coordinate or one horizontal velocity component by 1, 2^10 and 2^20 ulps, and in both. Tick every member once with the same input. Does every interior member land inside the box spanned by the corners' images, on every field compared as ordered bits? Count ticks tested, ticks that hold, ticks that fail.
2. For each failing tick, name the operation responsible, by the law's own line: the axis order switching on |vx| against |vz|, a step-up, the edge back-off loop, sprint stopping on a horizontal collision, a clamp to zero, a binade crossing, a wall contact beginning, anything else.
3. Over k = 2, 5, 12 ticks of one program: does the corner property still hold end to end, and how fast does the failing share grow?

**What to produce.**
- `reproduce.sh`: builds and runs the probe offline.
- `dossier.md`: `Claim: ` on its first physical line with the counts; a table of failures by responsible operation; three failing witnesses with their bits and the law line; what was left out.

**Deliver the first proof early.** Write `dossier.md` and a runnable `reproduce.sh` before expanding. Rust: `PATH=/d/rust/cargo/bin`, `RUSTUP_HOME=/d/rust/rustup`, `CARGO_HOME=/d/rust/cargo`, a `CARGO_TARGET_DIR` under your run folder's drive. Write only in your run folder. No network, no live game, no production edits, no commits.

**Out of scope:** vertical motion alone, fluids, non-player entities, proving monotonicity in general (this lens measures it).
