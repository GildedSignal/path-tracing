# Judge job: the plain path (S120)

You are the **judge** (AGENTS.md: Claude's work is judged by GPT Terra, in a fresh context, from the other family). You read and run checks; you **edit nothing** in the source tree and commit nothing. Your verdict decides whether branch `wt/plain-path` (commit `eef0cd3`, three commits on `09e1b52`) merges into `main`. Your worktree `D:\path-tracing-wt\plain-judge` is a checkout of that branch; the builder's own worktree is `D:\path-tracing-wt\plain` (do not touch it).

**What was asked (S120, Julien):** a straight line on plain ground should be seen as plain first (presence of anything that matters, not where things are), then answered as the best tool's held word repeated plus a remainder, with almost no thinking: "the bot should move instantly unless things are really hard". Ticks are execution, never thinking (S117/S118): the world model is not re-tested tick by tick.

**The spec it should agree with** is Astra's revised `D:\path-tracing-wt\sets\jobs\3.2d-3\last.md` (sections 2 and 3: one word and a small resource test; presence by four law-read roles). The red-team it answers is `C:\Users\PASCAL\Desktop\path tracing\jobs\3.2d-3-design\redteam-opus.md`.

**The builder's claims** (verify each; a claim you cannot reproduce is a finding):
- `engine/plan/src/plain.rs` (new, 404 lines): plain context (A's column extruded), the held word run from the settled start until its state recurs (`words::recurrence`, resources excluded), then positions folded from the cycle's displacements; stop = hold to B, or run from the last landing; food = one sufficient inequality; presence = cells in body/motion/support reads must match by `law_digest`, halo cells must be inert along an axis they lie outside.
- `engine/law/src/pattern.rs` `law_digest`; `engine/plan/src/flight.rs` split; `plan-track` tries the plain path first and logs `PLAIN fell back: <reason>`.
- `engine/world/src` reader: one region read per run of chunks, canonical Huffman decode.
- Gates: `cargo test --offline` from `engine/` passes (192 passed, 19 ignored); `jobs\1.2-footprints\verify.ps1` catches 13 of 13 mutants; `plan/tests/plain.rs` replays each sentence through the full law in the real snapshot bit for bit on stonesimple, straight100/300/800 and flowers.
- Cold `plan-track` wall time, median of 9: straight800 26.8 ms, T 2248 (main 2316); the other 16 tracks fall back, and the 6 that produce a sentence on main give byte-identical scripts.
- It writes the script by running the chosen word through the law tick by tick on the plain context (no world reads), because the script file needs every tick's full state; the decision itself is made from the recurrence and the fold.

Build with your own target dir (already set). The track tests need the fixture region files, which the launcher copied into your worktree. Nothing runs live: no server, no client, no RCON.

**What to decide, each with a line of evidence (a command and its output, or file:line):**
1. **The bit-for-bit gate is real.** Does `plain.rs`'s test compare against the full law over the real snapshot, or against the plain context itself? Mutate one line of the fold or the stop (in a scratch copy outside the tree, or by `git stash`-free means such as a copied worktree) and show the test goes red.
2. **Presence is a proof, not a patch.** Is the halo inertness rule derived from what the law reads (`footprint::Query`, the collision/suffocation reads), or is it a special case that lets stonesimple pass? Find any cell the law reads during the replay that lies outside the certified corridor. Name every constant in `plain.rs` and `pattern.rs` and trace it to `engine/law/src/movement.rs` lines; one without a source is blocking.
3. **No tick-by-tick thinking.** Does any decision (word, yaw, stop, fallback) depend on the tick-by-tick script run? If the run only materializes the script, say so; if anything is decided or re-tested per tick, it is a finding against S117.
4. **Food.** Does the inequality match the 3.2d-3 spec section 2 (`d <= ceil(saturation) + food - 7`, food > 6), with the law's own debits?
5. **Nothing else moved.** Run old (`09e1b52`) and new `plan-track` on every track in `lab\tracks` and diff the scripts. Run the full test suite and the footprint mutants yourself.
6. **The reader.** Is the canonical Huffman decode correct for every code length the format allows (not only the ones the fixture uses)? Does the per-run region cache ever return a chunk from a stale or different file?
7. **Departures from the 3.2d-3 spec.** List each place the build differs (for example full `law_digest` instead of four role masks; flowers on the plain path; the tick-run script) and whether each is sound, a later slice, or wrong.
8. **Net lines:** +611/−27. Is each part bought by a claim above?

**Done means** `jobs\plain-path-judge\verdict.md` holds: verdict `PASS`, `PASS WITH FINDINGS` or `FAIL`; one section per question above with its evidence; findings ranked blocking first. Then end with `STATUS: DONE`.
