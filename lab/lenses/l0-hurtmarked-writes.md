# Lens: velocity writes between a hurt mark and its packet (L0's first divergence on ladders)

**Why it matters.** L0's live gate (11 November) kept 32 of 32 runs on four courses and refused on `ladders-vines` run 1 with `missing hurtMarked consume`. The verifier (`oracles/1.6f-landing-motion/verify.rs` on branch `wt/l0-veto`, function `server`) reads each fall landing as exactly four server rows: damage, mark, consume, dispatch. The refused capture has three extra `write` rows between the mark and the consume of its first landing, all carrying the marked velocity (0, bfb41205c28f5c29, 0); its two later landings have none. The capture is `D:/path-tracing-wt/l0-veto/lab/gates/results/20261111T0926556663607Z-fixtures/l0_20261111T0926556663607Z_ladders-vines_1.*` (read it there; `server-motion.csv` is the file). The source is readable with `git show wt/l0-veto:<path>`.

**The question.**
1. In the 26.2 server jar: what sets `hurtMarked` on a player's fall damage, what clears it, and which velocity does the motion packet carry: the value at the mark, or the value at the moment `ServerEntity.sendChanges` reads it? Bytecode offsets for each.
2. Which server paths can write a player's `deltaMovement` between the mark and the send, within one world tick? In particular, can several movement packets handled in one tick (the capture runs the client at 200 ticks per second) each write it, and can such a write ever carry a different value than the mark?
3. From 1 and 2, the grammar a landing's rows really follow (damage, mark, zero or more writes, consume, dispatch) and the exact equality the consume must satisfy.

**What to produce.**
- `reproduce.sh`: regenerates the bytecode evidence, then builds and runs the check below offline.
- `candidate.patch` against the branch's `oracles/1.6f-landing-motion/verify.rs` implementing the grammar of question 3, with no tolerance and no skipped row: every row is still accounted for, and the emitted velocity is checked bit for bit against the value the jar says the packet carries.
- A check, in a copy of the branch inside your run folder: with the patch, the refused capture's server chain parses (say how far the whole verification then gets, and its next refusal if any); a planted capture in which one `write` carries a different velocity than the consume is refused; a planted capture with a missing consume is still refused.
- `dossier.md`: the claim in one sentence, the evidence type of each part, what was left out.

**Deliver the first proof early.** Write `dossier.md` with `Claim: ` on its first physical line and a runnable `reproduce.sh` before expanding; save the first output as soon as it exists. Rust: `PATH=/d/rust/cargo/bin`, `RUSTUP_HOME=D:/rust/rustup`, `CARGO_HOME=D:/rust/cargo`, and a `CARGO_TARGET_DIR` inside your run folder. The jar is `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`, the JDK is in `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin`. Write only in your run folder. No network, no live game, no production edits, no commits.

**Out of scope:** the client's handling of the motion packet, the other courses, any live run.
