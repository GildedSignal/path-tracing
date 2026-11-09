# Job 1.2: footprints and determinism

**Why.** Phase 1 (HANDOFF). The word is the one object (representation.md, section 3): its footprint makes it a certificate, a cache entry and a worldtube. The law must say what it read.

**Build.**
1. A `World` adapter that records every block position read, and through which query (shape, fluid, hooks), for one tick and for a run of ticks. Plus the cells the body's box swept. No changes to the law's arithmetic.
2. Randomness as an input: list every place the ported law draws server randomness or reads the clock. Make each one an explicit input with a recorded value, so the same inputs give the same bits.
3. A draft canonical pattern key: hash the footprint's contents relative to the start position. Detect the 4 rotations and the mirror by hashing, never declare them. Show it on two identical patterns placed far apart (same key) and on a one-block difference (different key). This key is a one-way door: the Designer red-teams it before it freezes, so keep it small and documented.

**Gate.** All existing tests unchanged. New tests: a footprint is a superset of the cells whose change alters the tick, so flipping any cell outside it changes nothing (randomized, cheap); same inputs give the same bits.

**Must not.** Change movement results. Touch `bot/`.

**Done means.** The final message gives the adapter's API, the randomness inputs, the key's definition and its test results.

**Also (the law-speed judge's follow-ups).** The fast paths from job 1.1 are tested only indirectly. Add three cheap differential tests, each answering one question fast (AGENTS.md, "Tests answer fast"):
- On small random `BoxWorld`s (regions with precedence, overlapping overrides, unloaded chunks, shapes reaching outside their cell), do `visit_collisions`, `no_collision` and the suffocation query equal the default full-scan `collisions`?
- Does `tick` equal the same tick on a world whose `has_fluid` and `has_inside_effects` are forced true?
- Does `ShapeView::collide`'s single-box path equal the general path?

Each must fail when you plant the obvious mutant in its fast path; say so in the report.
