# Lens: a class with no chosen number, measured on all 18 tracks

**Why it matters.** The class is the key under which a computed move is proposed again (`engine/words/src/class.rs`, `engine/words/README.md`). Every use is replayed from the exact state, so a class that is too coarse only wastes a replay (a "surprise": the move ends otherwise than proposed), while a class that is too fine misses and forces a search, 20 to 100 times dearer. The built class cuts the position inside the block at 0.3 and 0.7 and keeps four velocity bits. Julien wants no chosen number in it (S67). On one flat track and one program, the committed test `stonesimple_hit_rate_from_arbitrary_offsets` printed: built class 241 classes, 98% hits, 92% of hits ending as proposed; no horizontal position 40 classes, 100%, 73%.

**The question.** Extend that measurement, changing only the test harness and the class's grid parameters (no law change):
1. Variants: (a) the built class; (b) no horizontal position; (c) no position and no velocity (discrete fields only); (d) no position, velocity as the sign of each component only.
2. Worlds: each of the 18 track fixtures under `engine/world/tests/fixtures/` that loads (name each that does not, with its refusal).
3. Programs: all 54 tiny programs, not one. Say how classes multiply with programs and with the 64 aims.
4. For each variant and world, over runs 33 to 64 of 64 seeded runs from arbitrary offsets: classes, hit rate, share of hits ending as proposed, and the cost per segment in µs under the stated prices (a miss 2000 µs, a surprise 45 µs, a hit 48 µs). Which variant is cheapest on each world, and by how much?
5. For variant (c): which discrete field or cell read, if added to the class, removes the most surprises? List the top five by surprises removed. These are the law's own comparisons, so name the law function that makes each.

**What to produce.**
- `reproduce.sh` and `work.patch` (test code and grid parameters only), regenerating every number.
- The table of question 4 as a TSV, and the list of question 5.
- `dossier.md`: the claim in one sentence (the cheapest variant and its margin, or that it depends on the world, and on what), and what the measurement cannot say.

**Deliver the first proof early.** Write `dossier.md` with `Claim: ` on its first physical line and a runnable `reproduce.sh` before expanding; save the first output as soon as it exists. Rust: `PATH=/d/rust/cargo/bin`, `RUSTUP_HOME=D:/rust/rustup`, `CARGO_HOME=D:/rust/cargo`, and a `CARGO_TARGET_DIR` inside your run folder. Write only in your run folder. No network, no live game, no production edits, no commits.

**Out of scope:** changing the event rule, the law, or `docs/vocabulary.md`.
