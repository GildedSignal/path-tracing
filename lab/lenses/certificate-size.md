# Lens: how big should a certificate be? (one parabola, or one block)

**Why it matters.** A certified segment ends at the first tick where any discrete field of the state changes, or where the body's block changes (`docs/vocabulary.md`, section 4). Job 1.6 measured segments of 1.5–2.2 ticks, so a sprint-jump's 12-tick arc is cut into several certificates, each paying a fixed price (the certificate, the sorted footprint, segment machinery: about 6 µs) beside the law's about 2 µs per tick. Julien (11 November): "of course a parabola should be one certificate … the question is more of whether it is more efficient to have bigger or smaller certificates … maybe certificates should be rougher since they won't be broken often … we need to fermi this too." The block-change cut exists for recognition: a longer piece reads more cells, so its key recurs less (vocabulary.md, section 1: about 0.9 per column). This lens measures both sides on his tracks.

**Inputs.** `engine/words` (segments, the event rule in `segment.rs`, the 54 programs in `program.rs`, the proposal table), `engine/words/tests/read_cost.rs` (job 1.6's harness: its timing layers and its walk protocol), `engine/world/tests/fixtures/` (his 18 tracks), `law::pattern` (the footprint key).

**The question.** On every track the harness can stand a bot on, walking the 54 programs from standable starts as `read_cost.rs` does:
1. **Today's rule.** The distribution of segment length in ticks (n, min, p50, p90, p99, max), and for each airborne arc (takeoff to landing) how many segments cut it and which event cut each (body block, or which discrete field).
2. **One arc, one certificate.** The same walks with the body-block cut removed (a copy of the rule in your run folder only): segment lengths; footprint cells per certificate and per tick; replay µs per certificate and per tick on the harness's layers (p50, p90, n ≥ 2000, the machine's load stated).
3. **Recognition, both rules.** The fraction of segments, weighted by ticks, whose `law::pattern` key occurs again elsewhere on the same track and on another track (exact key, translation in scope). Report the class hit rate too if the proposal table runs unchanged.
4. **The inputs of a breakage Fermi, both rules.** Footprint cells × ticks per certificate (the volume an edit or a contact must touch to break it), and the swept cells grown by 1 and by 2 as a rough-certificate bound, with its ratio to the exact footprint.

**What to produce.** `reproduce.sh` regenerating every table; `dossier.md` with the tables and the claim in one sentence, saying which rule costs less per tick of motion at equal recognition, by how much, and where that reverses. No production edits: rule variants live in a copy inside your run folder. Every number from a run, none chosen.

**Deliver the first proof early.** Write `dossier.md` with `Claim: ` on its first physical line and a runnable `reproduce.sh` before expanding. Rust: `PATH=/d/rust/cargo/bin`, `RUSTUP_HOME=D:/rust/rustup`, `CARGO_HOME=D:/rust/cargo`, and a `CARGO_TARGET_DIR` inside your run folder. Write only in your run folder. No network, no live game, no commits.

**Out of scope:** changing the word record, the class, operators and tools.
