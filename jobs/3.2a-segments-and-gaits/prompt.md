# Job 3.2a: segments and gaits, the first piece of the word compiler

**Why.** S55: about 5 bots with pathfinding working very well on Julien's 18 tracks. The word record is frozen (`docs/vocabulary.md`). This job builds its smallest working core: segments cut by the event rule, gaits found as attractors, and the class-keyed proposal table certified by replay. The planner (3.2b) and the executor come after.

**Read.** `docs/vocabulary.md`, all of it: this job implements its sections 1–7 for movement only, with no operators. Also `docs/footprints.md` (the certificate key, `law::pattern`, `law::footprint`), `docs/representation.md` sections 2, 3 and 5, and `engine/world` (`WorldSnapshot`; the tracks' fixtures in `engine/world/tests/fixtures/`).

**Branch.** `wt/segments`, worktree `D:\path-tracing-wt\segments`, cut from `main`. Toolchain as in earlier jobs.

## Build: a crate `engine/words`

1. **Segments.** `run_segment(world, state, program, args) -> Segment` ticks the law from a state until the event rule fires (vocabulary.md, section 4): any discrete field of the state changes, or the body's block changes. It records `ticks`, the exit kind (which field changed), the exit state, and the footprint.
2. **The class** (section 3):
   - The state fields the law reads before writing, recorded through the footprint adapter by the same induction as cell reads. If the law has no state-read hook yet, add one the way cell reads are tagged: no arithmetic change, bit-identical replays.
   - Resources become guards and deltas.
   - Position relative to the block, quantized to 1/16.
   - Velocity exact where it repeats, otherwise on a grid. Choose the grid, and say why.
3. **Programs** (section 5): a small table from predicates (including "support ends this tick" and "ticks since start") to inputs. Aims come from 64 sine-table yaws.
4. **Gaits** (section 6): enumerate tiny programs per regime (start from the 9 November probe's 54; find it in git history if needed), run each to its attractor on flat ground of each type, and keep the cycles of classes. Report how many gaits per ground type.
5. **The proposal table** (section 2): class → proposals (program, arguments, exit kind, tick range). Every use replays from the concrete state, and the replay's `law::pattern` key certifies it.

## Test, fast

- **Gaits recur:** a sprint-jump gait on stone and on blue ice under a 2-block ceiling (the head-hitter) is found as a cycle of classes. Its steady speed matches the GLM speed-of-light result: 19.810930 blocks/s on blue ice under a 2.0-block ceiling (`lab/wiki/speed-of-light.md`).
- **Events:** each kind in section 4's rule fires on a minimal fixture: stone to ice, a carpet step-up, entering water, a ceiling bonk, losing sprint.
- **Proposals hit:** on one of Julien's flat tracks (`stonesimple`), replaying a gait twice from different starting columns gives the same class sequence, and the second run's segments come from table hits.
- **Bench:** the proposal lookup plus its certifying replay, in µs per segment.

## Must not

- Change the law's arithmetic. The replay and jar suites stay bit-identical, and `jobs/1.2-footprints/verify.ps1` still catches every mutant.
- Touch `bot/` or the gates.
- Push or merge.

## Done means

The crate, the tests green, and a final message giving the gaits per ground type, the head-hitter's speed, the hit rate on the stonesimple test, and the µs per segment.
