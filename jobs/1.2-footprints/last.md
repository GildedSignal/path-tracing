# Job 1.2 result

Finished by Claude Opus 5.5 from GPT Sol's WIP (7b740ec), then changed after the
Designer's red-team (FREEZE WITH CHANGES). Spec: [footprints.md](../../docs/footprints.md).

## Round 1 (commits 66b2213, c765abe, 8801299)

- Merged `main`; restored `BoxWorld`'s collision halo (the WIP held `verify.ps1`'s
  planted mutant); keyed cells by physical model; `world::model_table()` and the
  table test (519 models, 519 digests, 506 closed under the eight frames).

## Round 2: the red-team's changes

1. The key reads `reads` only, each cell projected onto its queries (`cell_digest`).
   `swept` is worldtube data. Gate: 3,001 edits that keep a cell's answers (the digest
   decides) leave the tick bit-identical; an unkeyed fluid or sneak shape is caught.
2. A hit needs bit-equal start bits (position and supporting block relative, `t` out).
   `PlayerState::bits()` moved into the law from the test support.
3. `Entry::scope` and `in_scope`: an integer shift is exact while each axis keeps its
   binade (k <= 23 for the f32 casts). Gate: 6 in-scope shifts bit-exact; 7 of 8
   out-of-scope shifts (1024, 2048, 2^20, y = 64, y = 128) differ.
4. Empty extra-collider queries: their union box, origin-relative, re-queried on a hit.
5. `PatternTable`: u16 ids over a persistent digest table, shared footprint shapes and
   start states, std seeded hash maps over ids. Bytes and µs in the table below.
6. Frames: 62 runs on the 18 tracks; every non-identity frame fails (first gaps 1e-17
   to 3e-5, Mojang's sine table). The key keeps the identity only.
7. MinY relative, read only beside unloaded chunks.

| Word | v1 bytes | v2 bytes own + shared | v1 µs | v2 µs key, lookup |
|---|---|---|---|---|
| 1 tick (90 cells) | 30,028 | 292 + 1,108 | 161 | 25, 20 |
| 10 ticks (174 cells) | 57,736 | 471 + 2,813 | 230 | 30, 25 |

## Evidence

`verify.ps1`: workspace 115 passed, 6 ignored (as on main); 12 mutants caught; audit
passes. `retained-replay.ps1`: identical on `main` and the branch, both at the known
server-saturation red (1100480512 vs 1101004800), job 1.3's.

## Round 3: the judge's FIX

1. `update_fluid`'s chunk check asked `loaded` at absolute chunk corners; it now asks each
   cell of the same columns (option a): a chunk is loaded exactly when its cells are, so the
   law is bit-identical and the read is relative. The old reads fail
   `a_shift_across_a_chunk_edge_cannot_hit_when_the_law_sees_it` (mutant `chunk-corners`).
2. The outside-footprint gate now edits cells outside `reads`; it passes unchanged.
3. `World::model_id` and a dense per-(model, mask, loaded) cache in the table. 1-tick words:
   key 25 -> 3.2 µs warm (18 cold), lookup 20 -> 2.2 µs; 10-tick: 30 -> 6.0, 25 -> 4.4.
