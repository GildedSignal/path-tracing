# Job 1.4f: the law stops panicking when the bot lands on, or stands in, a block it does not model

**Why.** Two `unimplemented!` calls in `engine/law/src/movement.rs` panic **by block name**: `BlockState.fallOn` at line 826 and `BlockState.entityInside` at line 1078. A bot on his parkour tracks lands on something every few ticks and walks through things constantly, so an unmodelled block is not an edge case — it is a panic in the middle of a run, and it takes the whole flight down.

**Consumer.** The first flight (HANDOFF roadmap item 2): one bot racing his 18 tracks. These are the walls most likely to stop it.

**Branch.** `D:\path-tracing-wt\fallon` on `wt/1.4f-fallon-entityinside`, cut from `main`. Anything outside your branch is in the main checkout at `C:\Users\PASCAL\Desktop\path tracing` and must be opened by absolute path.

**Read.** `AGENTS.md` (especially "No patches" and "No job without a consumer"), `docs/engine.md` for the law's contract, `docs/footprints.md`, and `engine/world`'s block-table and exporter code — the same path job 1.4e used when it added a fifth collision context.

**First, measure what is actually missing. Do this before writing any port.**
Enumerate which block states reach each panic. The world reader models 30,951 of 32,366 states and every block in his 18 tracks is already modelled, so the question is not "which blocks exist" but **which of them take an unmodelled `fallOn` or `entityInside` path**. Report:
- the list of block names that reach each panic, and how many states each covers;
- which of them appear in his 18 tracks (`engine/world` has the track block data; job 2.1b produced it);
- how many are reachable at all by a player body.
**If the measurement shows the panics are unreachable on his tracks, say so and stop** — that is a complete and valuable answer, and it moves this job out of the first flight's way instead of spending hours on it.

**Then port what the measurement says matters**, in consumer order: the blocks on his tracks first, then the rest.
- Each rule comes from the jar by class, method and bytecode offset, cited in your report.
- `fallOn` and `entityInside` are **generators**, like the block table already is: the effect belongs to the block's data, read from the game, not to a `match` on block names written by you. If the natural port seems to be a per-block branch, that is the signal the representation is wrong — **stop and say so** rather than writing the branch (ES 19, AGENTS "No patches").
- Where a rule needs state the law does not carry, name the missing state and refuse that block, the way job 1.4e refused the spectator branch. A loud refusal is correct; a guess is not.

**Must not.** Add a tolerance, an epsilon, a retry, or a `match` on block names. Replace a panic with a silent default, a no-op, or "do nothing for unknown blocks" — an unmodelled block must still refuse loudly, just by a named refusal rather than a panic, and your report must say which blocks now refuse. Touch `engine/plan`, `engine/words`, `bot/`, the planner, the gates or the fixtures. Run anything live. Commit, push or merge.

**Done means.** The measurement above, reported in full even if you port nothing; for anything ported, a witness red before and green after, bit for bit against the jar-derived values; `cargo test --offline` from `engine/` passing; the jar suite still 6 of 6; `jobs/1.2-footprints/verify.ps1` still 12 of 12 mutants; and a final message with the block census, the rules as ported with their offsets, which blocks still refuse and why, the net lines, and anything left out.
