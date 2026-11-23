# The hot state and the event face (Opus, 23 November): report

`wt/port-fold` `f24a6317` → `43f43730`, 5 commits. Rerun by the integrator: workspace builds; lib 95, jar 18, replay 32 + 4 ignored (the new `event_face` judge included); `plan`'s `real_time` passes (Opus: 364 lines identical once timings are stripped).

**The event face** (`law::face::advance`, 155 code lines against the 575 of `events.rs`): it steps the core until the law's branch state repeats (period up to 40), checks that the period is exactly the accumulators' own arithmetic, then jumps without calling the core while every cell within 2 of the body answers like the first of its line. Causes it knows: a cell unlike its line (wall, edge, floor, block effect), the food guard, the world border, the end of the held input. Binade edges need nothing: the native additions cross them exactly.

**Judged:** every held-input stretch of the 31 gated recordings plus 62 extremes (six grounds, five gaits at x = 0.5 and 1,048,575.5 for 3,000 ticks, 20,000-tick falls in air and water), bit for bit on wire words, with stepping's footprints inside the face's. 0 anomalies; a planted 1-ulp mutation was caught (`anomalies.tsv`).

**Coverage:** 22.2% of recorded ticks jumped (2,968 of 13,347). His holds are short (median 1 tick: his yaw moves almost every tick), so recordings understate it: straight800-2 jumps 1,969 of 2,248. Stepped: 10,152 ticks of transient (short holds, and the drag's approach to its orbit, ~275 ticks for sprint-jump on stone, which has no exact closed form), 172 of cells unlike their line (plates, flower offsets), 55 of lanes clipped mid-cycle.

**Speed:** a jumped tick ~0.19 µs against ~1.2 µs stepped (~6×); where nine ticks in ten jump, 424 against 1,443 ns/tick (3.4×). The corridor's per-cell reads dominate.

**Lines:** `engine/` code 24,146 → 24,308, tests 11,748 → 11,880: growth, not progress until `plan` switches.

**The hot state: not shrunk.** Still 296 B. The core now reads each fact once (the aim from the input, the move as its own output), but `yaw`, `pitch` and `movement_this_tick` (40 B) stay because the frozen `plan` reads them (`state_view.rs`, `solve.rs`, `arith.rs`, `families.rs`, `flight.rs`). The wire is unchanged: all 13,378 recorded states round-trip.

## For the switch after Astra's merge

- `plan/arith.rs`'s free arc (125 lines) and `search.rs`'s cycle-observing loops go now.
- `events.rs`'s inverse half (`first_touch*`, `Approach`, `AxisTable`, `horizon`, `advance`, `first_cross`, `PhaseGeometry`) is referenced only inside itself on this branch: recheck on `main`.
- `solve.rs`'s O(binades) `advance_cycles` stays until the world answers "uniform box" in one query; then ~100 lines of `Region`/`Graph` move into the face and `events.rs` goes.
- `plan` reading `input` and `Effect.collision` frees the 40 B; the fluid flags follow.

## Left

The transient as a stored memo (the planner's store); an O(1) uniformity query on the world; a cheaper repeat key; footprints are containment, not equality. `D:\pt-base` (5.3 MB, Opus's scratch copy of `f24a6317`) is still on disk.
