# Lens: why recording a segment costs 40 µs when the law costs 6

**Why it matters.** Every use of a cached move is replayed with a recording world so that its footprint certifies it (`docs/vocabulary.md`, line 19, assumes 1 to 6 µs for that replay). Job 3.2a measured, and its judge confirmed: lookup 3.0 µs, the law alone 5.8 µs for about 1.55 ticks, the recorded segment 41 µs, replay with certificate 45 µs. A one-off probe counted about 558 `block()` calls per tick for about 95 distinct cells per segment, each paying a map insert in `RecordingWorld`. The planner's whole cost estimate rests on this number.

**The question.** On `main` (`engine/law/src/footprint.rs`, `pattern.rs`, `world.rs`, `movement.rs`; the bench in `engine/words/tests/compiler.rs` and `cargo run --release -p law --bin bench`):
1. Measure it yourself: calls to each `World` method per tick, distinct cells per tick and per segment, on sprint-jump over flat stone and on `stonesimple`. Which law function asks for the same cell again?
2. Where does the recorded segment's time go: the map, the key construction, the certificate insert, allocation? Give a breakdown that sums to the total.
3. The smallest change that brings the recorded segment near the unrecorded one **without changing what is recorded**: the footprint and the pattern key must stay bit-identical.

**What to produce.**
- `reproduce.sh`: builds a copy of `engine/` in your run folder, runs the counts and the timing before and after.
- `candidate.patch` against `main`. It must not touch the law's arithmetic, must add no cache keyed on anything but the cell, and must keep `cargo test --release` green in `engine/`, including the jar suite. `jobs/1.2-footprints/verify.ps1` (offline) must still catch all 12 mutants: run it and paste its lines.
- A table: µs per recorded segment at p50 and p90, before and after, at least 2000 samples, and the same for the unrecorded law, with the machine's other load stated.
- `dossier.md`: the claim in one sentence with both numbers, and what you did not try.

**Deliver the first proof early.** Write `dossier.md` with `Claim: ` on its first physical line and a runnable `reproduce.sh` before expanding; save the first output as soon as it exists. Rust: `PATH=/d/rust/cargo/bin`, `RUSTUP_HOME=D:/rust/rustup`, `CARGO_HOME=D:/rust/cargo`, and a `CARGO_TARGET_DIR` inside your run folder. Write only in your run folder. No network, no live game, no production edits, no commits.

**Out of scope:** changing the law, the class, or the proposal table's meaning.
