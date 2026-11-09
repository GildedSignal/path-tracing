# Job 1.1: the law, fast

**Why.** Phase 1 (HANDOFF). The law costs 26–59 µs per tick (engine.md). Compiling the vocabulary in Phase 3 needs ≤ 1 µs. The Fermi floor is about 0.1–0.3 µs.

**The known cause.** `World::collisions()` and the suffocation check clone a `VoxelShape` for each of the ~150 cells around the box, air included, and `moved()` allocates again. `BoxWorld` looks up blocks through a `BTreeMap` and a linear region scan.

**Build.**
- Intern shapes: a shape is an index or a `&'static`/`Arc` reference, never cloned per query.
- Fast paths for air and the full block.
- Collision and suffocation queries with no heap allocation per tick.
- Any other hot spot a profile shows.
- Behaviour must stay bit-identical.
- A committed bench, `engine/law/benches` or a `--bin bench`, offline. It reports ns per tick as min, median and max over at least 5 repetitions, for flat walking, sprint-jumping, a 2-block ceiling and bumpy terrain.

**Gate.** Every existing test passes unchanged: `cargo test --offline` from `engine/`, the jar corpora and the replays. The four mutant features still fail their tests. Report the bench before and after, on the same machine.

**Must not.** Change any movement result. Add a tolerance. Touch `bot/`.

**Done means.** The final message gives the before-and-after distributions, every change, and what the profile says the next hot spot is.
