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

## Continuation (after the first pass, committed as `bed67f3`)

The first pass gave 4–8× (sprint-jump 33.8 → 6.0 µs). The target is still ≤ 1 µs, and a bit-identical replay is part of the gate.

- **Recordings are now in this worktree** under `lab/fixtures/` (corner, full-blocks, slime: the original L0 pairs), so the replay test runs. One mismatch is known and pre-existing: the peaceful full-blocks end saturation. Show the replay outcome is identical before (`bed67f3^`) and after your changes, field for field; don't try to fix that mismatch.
- **Next allocations to remove,** as you found: translated shapes (`moved()`) and result vectors in collision queries. Collide in local coordinates or with offsets instead of building shapes. Reuse buffers. Look at `BoxWorld`'s `BTreeMap` and linear region lookup too.
- **Profile without new tools:** add temporary counters or timers around the tick's phases (collisions, suffocation and push-out, inside-block visits, attributes and food) in the bench build, report the share of each, and remove the counters afterwards.
- Stop when the median is ≤ 1 µs on all four cases, or when the profile shows the remaining cost is in the law's arithmetic itself. Then say which, with numbers.
