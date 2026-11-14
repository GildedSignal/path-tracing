# Job 3.3h: thinking time down to its floor's order on straight lines

**Why.** T counts thinking (PRINCIPLES, the scoreboard; his S110 and S111). Measured on 14 November with the planner on `wt/3.3g-play`, mode 5, from rest on A, on flat straight tracks (`straight100`, `straight300`, `straight800` in `engine/world/tests/fixtures/tracks.txt`, plus `stonesimple`, 29 blocks):

| Length | T | Search | First sentence | Thinking | Floor compile |
|---:|---:|---:|---:|---:|---:|
| 29 | 85 | 12 ms | 6 ms | 0.08 s | 0.23 s |
| 100 | 300 | 32 ms | 18 ms | 0.56 s | 0.97 s |
| 300 | 876 | 106 ms | 59 ms | 3.57 s | 1.84 s |
| 800 | 2316 | 321 ms | 201 ms | 23.1 s | 3.03 s |

The Fermi floor for 800 straight blocks is about 0.3 to 2.5 ms (certify 2,316 ticks at ~1 µs, or ~190 arcs at 0.7 µs; read ~12,400 tube cells). Thinking is 4 orders above it and grows as length squared. **The quadratic part is `slide`** (`engine/plan/src/flight.rs:152`, the joint polish of S94): on `straight800` it took the found sentence from 2,580 ticks to 2,316, saving 264 ticks, but spent ~22.8 s = 456 ticks to do it, a net loss. It appears to replay the whole sentence once per joint (~430 joints × 2,316 ticks), and it is bounded only by the budget, not by the search's own rule that thinking stops when the ticks it spends exceed the ticks it could still save (line 143). A second cost: the floor's table compile (`Field::build`, ~3 s on 800 blocks) is rebuilt on every call.

**Consumer.** The first flight's scoreboard, where thinking is part of T (`scripts/play-track.ps1` calls `plan-track` per click).

**Worktree** `D:\path-tracing-wt\thinking`, branch `wt/3.3h-thinking`, from `wt/3.3g-play`.

**This job has three deliverables, in order. Each is independent of the next; finish each before the next.**
1. **`slide` obeys the pay rule.** It stops as soon as the ticks spent thinking since the first sentence reach the ticks still possible to save (the same test as line 143, with the same lower bound). Gate: on all four straight tracks and on every one of his 18 tracks that certifies today, `T + thinking ticks` is never worse than before, and on `straight800` it is better.
2. **`slide` replays locally.** A slid joint replays from that joint only until the new trajectory's full state equals the old one's at the same tick, bit for bit (`PlayerState::bits()`), then splices the old remainder; if the states never rejoin, it replays to the end, as now. No tolerance: equality of bits or a full replay. Gate: the final sentence is bit-identical, tick by tick, to the one the full-replay slide produces on all four straight tracks and on every track that certifies today, and the law's ticks spent in `slide` grow linearly with length (report them for 29, 100, 300, 800).
3. **The planner stays loaded.** `plan-track --serve` reads one request per line on stdin (`<track> <mode> <out>`) and answers each with the same line it prints today. It keeps each floor's compiled `Field` in memory, keyed by the track's box and a hash of the snapshot's cells in that box (a changed cell is a new key, never a stale table), and records for each table what built it, when, from which snapshot hash and in how long. Thinking per request is measured from the line read to the file written; a table built inside a request counts in full inside that request's thinking, and a table reused counts its lookup. `scripts/play-track.ps1` starts one `--serve` process per rig and reuses it. Gate: on `stonesimple`, the second request's thinking excludes the compile and its output is byte-identical to the first's.

**Must not.** Change the search's order, the law, a gate, the fixtures (except to read them), or `T` itself; this job changes only what thinking costs. Add a tolerance or a cache keyed by track name. Commit, push or merge.

**Done means.** The three gates passing with their numbers; a table of the four straight tracks before and after (T, search, slide, compile, thinking); `cargo test --offline` from `engine/` passing; `jobs/1.2-footprints/verify.ps1` 13 of 13 on main's gate if merged, or 12 of 12 on this branch; the net lines.
