# Verdict on the uncommitted bench-dig diff (on 8dfb7022)

## In one paragraph

Half of it is the mechanism the spec describes, and half of it patches the benchmark. The mechanism: swimming priced by the class of the cell left (`Legs::learn`, committed), the gait that closes on its vertical state (`Gait::learn`/`entry`), the mode's sticky-sprint program, `eyes_class` in the label and `codec` mask (the law reads the eyes to swim), the exact wave optimisations, and the powder-snow support rule. The patches: the **shrinking box ladder** (`bounds_for`), the **`exhausted` → 600-tick cold cap** that rides on it, and the **polish and far-field thresholds**. The ladder causes the T regressions. It also buys three of the six new solves, because without it no field exists on large boxes and water falls back to raw learning. So the diff has the right goal and the wrong field. A field that fits the corridor, or a far field that carries water, is the missing mechanism. The cap is not it. Other pieces are dead code (the fast path, the `1e-6` stop) or special cases in raw learning that a swimming zone would delete (`in_fluid` aims, `fluid_air_fly`). Each piece carries its own verdict in the table. The diff as a whole should not merge. KEEP the mechanism pieces, DELETE the dead ones, and REWRITE the ladder and the caps.

## Method

I copied the worktree to `D:\path-tracing-run\judge-bench-dig` and added env toggles (`JT` bitmask) there only, so each piece can be switched off alone. `JT=0` reproduces the evidence exactly (hills-20-ne 113, hills-20-se 120, hills-50-nw 293). Toggles:

- 1 = no ladder: main's single box, empty field above 120,000 cells
- 2 = old polish (4,500 / 3,500)
- 4 = old cold cap (8,000)
- 8 = no `1e-6` stop
- 16 = old far-field thresholds (26 / 96,000)
- 32 = no fast path
- 64 = compare the fast path with `way_clear`
- 512 = `MAX_TOPO_CELLS` 600,000

I also ran the whole of BlockStep with `JT=3` and `JT=7` (files in that folder: `full-jt3.txt`, `full-jt7.txt`).

## Table

| # | file:line (worktree) | what it does | law or benchmark | verdict |
|---|---|---|---|---|
| 1 | `defect.rs` `bounds_for` ~359-394 (ladder `(20,depth,8) … (4,2,3)`) | Shrinks the field box until it fits 120,000 cells. Main returned an empty field above the cap. | **Benchmark.** Every rung is hand-picked, and no law fact fixes a margin of 14, 10 or 6. At d=20 a typical box (swamp-20-ne, 119,072 cells) is already on the lowest rungs. Switching it off (`JT=1`) restores main's T exactly on forest-20-nw (107) and nearly on the others (T regressions section). The truncated box also corrupts the 'unreachable' proof (row 2). Without it, 3 of the 6 new solves are lost (icemountain-100-se, swamp-50-se, forest-100-se) and forest-100-ne fails, though main solved it (`JT=3`, 66/72). Reaching 71 needs the ladder, and the ladder costs T. | **REWRITE.** The field is a corridor around the straight line plus the defect's exceptions, or the `FarField` horizon carries the wet and stuck classes. Its extent comes from where the wave can still beat the bracket's UB, never from a cell count. No tuning ladder. |
| 2 | `defect.rs` ~2235-2240 `cold_cap` (600 / 8,000 / 16,000 / 32,000) and `field.exhausted` | When the wave dies without reaching the seeds, gives up learning after 600 raw ticks (no kit). | **Benchmark, and unsound.** `exhausted` is true when the heap empties inside a box the ladder truncated. It does not prove unreachable. hills-50-sw: the ladder box is 107,163 cells, the wave settles 5,521 and exhausts, and the planner gives up at `learn_ticks=605`. With the 600,000-cell box the wave hits the existing 40,000-pop cap instead and learning spends 16,006 ticks. It still fails, and main failed it too (240,016). `JT=7` (old cap) changes no task result. The only effect is that hills-50-sw fails in 195 ms instead of 1,576. | **DELETE** the 600 and the 16,000/32,000 split. Keep `exhausted` as a flag only if the box is the full clamp. Failing fast is a decision for him. A fast fail from a truncated field is a false certificate. |
| 3 | `defect.rs` `polish` ~2250 (1,500 / 400 / 2,000 vs 3,500 / 4,500) | Stops raw learning sooner after an incumbent when there is no kit. | **Benchmark.** Made-up numbers. It costs T: `JT=2` alone gives hills-50-nw 284 against 293 with it. `JT=1` gives hills-20-ne 102, and `JT=3` gives 100 (main 100). | **DELETE** (restore 3,500 / 4,500). Thinking is not saved: `JT=2` costs the same thinking. |
| 4 | `defect.rs` `Legs::learn` second cycle stop ~152-162 (`t >= 32`, `1e-6`) | A second stop when velocity changes by ≤ 1e-6. | **Neither, and redundant.** Principle 11: the Legs price is a field price, not a bit-exact word, so the tolerance does not flip a decision. But `JT=8` (stop removed) gives identical T on all 10 strips (water1x6 124, water2x6 131, water3x10 145, water5x20 169, sea2 189, sea5 189, drop2 105). By the dossier, the sprint-swim gait `vx ← (vx+0.02f)·0.9f` is a contraction (0.9^n), so the exact bit repeat closes in a few hundred ticks, inside `MEMORY` = 4096. The swimmer's exact repeat did close; I did not print the `h` values. The extra stop is dead weight and a made-up number (32, 1e-6, principle 27). | **DELETE.** |
| 5 | `defect.rs` `Legs::of` ~83-89 (`fallback` 2.81 / 6.19 on more paths) | Returns the hard-coded walking constants when either gait lacks a floor and sprint is allowed. | **Benchmark-shaped.** The call prices only the dry class (class 1 = no inside effects), and wet classes go through `learn`. So a non-walking mode is not priced as walking. But a dry cell is priced by a copy of one number (2.81 against the law's 2.80, and `up` 6.19, `reach` 4). The `else` branch already does the right thing: `Legs::learn([full, air, air])`. The two branches disagree. | **REWRITE.** Both branches use `Legs::learn([full, air, air], from)`: the dry class from the law, whatever the entry label. Delete the constant. |
| 6 | `defect.rs` `has_step_support_with` ~269-284 | Powder snow is support only for a boots-wearer. | **Law.** `law/src/world.rs:502-516`: powder snow collides only for `fall_distance > 2.5` (0.9 high) or for a boots-wearer on top and not descending. It reads `Inside::PowderSnow`, a data field, not a name. A smaller law fact is better asked of `collision_shape` itself. | **KEEP**, then fold. Ask the law's `collision_shape` under the kit's `CollisionContext` instead of matching `Inside::PowderSnow`. |
| 7 | `defect.rs` `cell_flags` / `all_unit_xz` / `visit_extra_collisions` fast path ~524-586 | A second way to decide `way_clear` from cached per-cell flags. | **A second path.** It hard-codes powder snow in `empty_clean`, `no_overhang_up` and `full_solid`, and its context fixes `leather_boots: false`, so it would be wrong for a booted bot (not exercised, bare legs). **It never fires:** with the comparer on (`JT=64`), `fast_hits=0` on 9 tasks and the strip probe. `JT=32` changes thinking by ~10 %, which is noise (hills-20-ne 192 → 227 ms). | **DELETE.** Spartan on duplication. If way_clear is too slow, make the one function cheaper. |
| 8 | `defect.rs` heap `(bits, e, si_u32)`, early-out `all(dist < cur_d)`, cached `legs_cached`, `sup_and_fluid`, `kit.is_none()` gates | Wave optimisations. | **Law-neutral, exact.** `priced` is `kit.and_then(..)`, so with no kit a closed edge was already dropped. The early-out is valid because every edge cost is ≥ 0. | **KEEP.** Thinking did not halve (JT=1 still ~2× main); it is not where the gain is. |
| 9 | `defect.rs` `fluid_air_fly` ~778-796 | Climbing out of fluid into unsupported air only exits onto adjacent dry ground. | **Law fact, wrong shape.** The shore exit is `jumpOutOfFluid`: +0.3 when `horizontalCollision` and the box lifted 0.6 is free. That is a rule of the mode. The code scans neighbours for a dry floor instead. Not ablated: I could not isolate it. | **REWRITE.** Price the exit as the event "leave water", from the law's lift 0.6, in the class-of-the-cell-left pricing. |
| 10 | `defect.rs` raw learning: `exec_atomic_raw` caps/`hit_wall`/`medium_changed`/`swim_settled`, `programs_for` sticky sprint, `try_tail_extend` in water, `direct_blocked` `slow_thresh 1e-4`, `aims.len().min(5)` in fluid | Teaches raw learning to swim. | **Mixed.** Sticky sprint is law (`updateSwimming`: keep = sprinting ∧ in water). `swim_settled` and the hit-wall in water follow the gait. `1e-4` and `.min(5)` are made-up and are special code for a mode (Killed-if: "its own search code"). | **KEEP** sticky sprint and the event rules. **DELETE** `1e-4` and `.min(5)`. The real answer: swimming should be a zone (the law's travel fires `observe_air_travel`) so raw learning never swims. Not ablated. |
| 11 | `solve.rs:620` far-field switch `> 26 \|\| > 96,000` → `> 40 \|\| > 72,000` | Moves when the planner switches to the far field. | **Benchmark.** No law fact. `JT=16` (old thresholds) changes **no T** on the six regression tasks, and I did not run the full 72 with it. | **DELETE** (restore). Unproven benefit, made-up numbers. |
| 12 | `store.rs:96` `Gait::learn` stop `position.y > top + 2.5 && velocity.y >= 0` | Ends the floor-gait loop early for a body that rises and never lands. | **Coincidence with a law number.** 2.5 is `PowderSnowBlock`'s fall distance, which has nothing to do with a position above the top. A jump rises 1.25, so 2.5 is about twice the apex: a margin, not a derivation. The loop previously ran to 4,096 phases because `position.y` is in the repeat key and never repeats for a swimmer. | **REWRITE.** Take `position.y` out of the key (the second loop already keys on vertical state alone): a mode is floor-bound iff it lands before its vertical state repeats. No height constant. |
| 13 | `store.rs` `Gait::entry` (pose, swimming, in_water, sprinting in the match) | The entry phase must match the body's mode flags. | **Law.** These flags are the law's mode state. | **KEEP.** |
| 14 | `source.rs:792` `support_cell`, `(1..=3).contains(fy - floor_y)` | When airborne, guesses the support is the generator floor if the feet are 1 to 3 above it. | **Heuristic.** A sprint-jump apex gives `fy − floor_y ≤ 2`, so 3 is a margin, not a derivation. On hills (`floor_y` is the start's) it can name a cell that is not under the body. Not ablated. | **REWRITE.** Read the column under the body (first solid cell below) from the source. No window. |
| 15 | `source.rs` `uniform_corridor` / eyes in `corridor_query`, `codec.rs` `Support02` mask `Shape`, `eyes_class` in `world_mirror.rs` | The label reads the eyes' class. | **Law.** The law reads the eyes to swim. This is exactly the missing item in `representation.md` ("The zone label reads support and feet, not the eyes"). | **KEEP.** |
| 16 | `cut.rs:108` `main_supporting_block` kept only if the last row had one | A swimmer has no supporting block. | **Law** (that is what the touch-mismatch was). | **KEEP.** |
| 17 | `solve.rs:176` `gait.floor` in `Crossing::run` | Held rows only for a gait with a floor. | **Law.** | **KEEP.** |
| 18 | `strip-probe.rs` | A replay for starts that are not settled. | A test tool, not engine code. | **KEEP.** It matters for the judge: `powder_snow` stays `NeedsJoints`, which the map's "sunk, no way out without boots" predicts for bare legs. |

## The T regressions: cause shown on raw ticks

Cause: the **box ladder**. Raw ticks, T on main → on the diff → on the diff with only the ladder off (`JT=1`; polish also off, `JT=3`, in brackets):

| Task | main | diff | no ladder (JT=1) | thinking: main / diff / no ladder |
|---|---|---|---|---|
| forest-20-nw | 107 | 125 | **107** | 178 / 371 / 177 ms |
| hills-20-se | 105 | 120 | 105 | 220 / 280 / 225 |
| hills-20-ne | 100 | 113 | 102 (100) | 152 / 436 / 316 |
| hills-50-nw | 258 | 293 | 253 (252) | 563 / 785 / 724 |
| ice-100-ne | 417 | 442 | 433 | 633 / 1,981 / 876 |

- On forest-20-nw the no-ladder run has no field at all (no `TOPO` build; the box exceeds 120,000 cells), the same as main, and it gives main's 107 exactly. The ladder replaces 'no field' with a field of 60 × 23 cells wide: in practice the wave sees only part of the terrain and misleads the guided search.
- The residue (hills-20-ne 102 against main 100) is the **polish** numbers: with both off, 100.
- The rest of the +183 ticks on the 65 common tasks is the same two pieces. I ran only the six regression tasks with single toggles. I did not rerun all 65 under each toggle. The full-72 `JT=3` run (ladder and polish both off) gives, on the 63 tasks it solves in common with main and the diff, a sum of −135 ticks against main and −305 against the diff. Its thinking sum is 60,420 ms against 54,093 (diff) and 41,827 (main).
- That run also lost the new solves. Solved: main 65, diff 71, `JT=3` 66 (and `JT=7` the same). Unsolved with `JT=3`: icemountain-100-se, swamp-50-se, hills-50-sw, forest-100-ne, forest-100-se. So the ladder pays: swamp-50-se (one exception, one wet cell) only solves when a field prices water.
- Thinking: the diff roughly doubles cold thinking on many tasks and the ladder explains about half of it. The fast path (row 7) and the wave rewrite do not give back the rest. T is the one currency, so thinking counts. Compare also `ice-100-ne`: 633 → 1,981 ms for +25 ticks.

## hills-50-sw: why it fails at its own start cell

It is not the start cell. `exception: [6994,124,-5300]` is only the first corridor exception of the straight line, 2 blocks from the start, with 3,845 exceptions in the 50 × 50 window: the whole mountain is the defect. The planner then asks the field for T-to-go.

- The goal (6946,105,-5250) is 19 below the start. With the ladder, the box is 62 × 27 × 63 (107,163 cells). The wave from the goal settles 5,521 cells, exhausts, and never reaches the start. So `all_seeds_unreachable && exhausted` sets the cold cap to 600, and the planner returns `NeedsJoints` after `learn_ticks=605`.
- With the full 20-block margin box (430,612 cells, `JT=641`), the wave stops at the existing 40,000-pop cap (`exhausted=false`), the cold cap is 16,000, and learning spends 16,006 ticks and fails. Main failed it at 240,016. No version solves it.
- So I cannot say what the right answer is, only that the diff's failure is a decision made by the box and the cap and not by the terrain. Whether the goal is enclosed or the box is too small, I did not establish (I did not print the wave's frontier).

## What I could not check

- Whether the box ladder (not the cap) is the **only** cause of the T regressions on all 65 common tasks. I ran six with toggles and two full-72 runs with ladder and polish off. Toggle 16 and the individual contribution of rows 9, 10 and 14 were not measured on the full 72.
- The fast path (row 7): it never fired, so I proved neither exactness nor inexactness. Its powder-snow and `leather_boots` assumptions are read from code only.
- The `Legs::of` fallback (row 5): I did not print `h` for the learned dry class against 2.81, nor test the change.
- That the swimmer's exact repeat closes (row 4): shown only as 'identical T on 10 strips', not by printing `h`.
- A jar witness for each class price (the Killed-if line) and the Killed-if 6-wide shallow strip: `water1x6` solves at 90.9 ms thinking against ~12 ms of a joint pattern's floor; `sea5` at 50 ms and `water5x20` at 170 ms. All are above the ~12 ms floor, so the Killed-if line (cold thinking above a new joint's floor) is **still true** for water. `powder_snow` still returns `NeedsJoints`.
- The `cargo test` suites (`kit_unfold`, `kit_field_edges`, `generalize`) were not run: I spent the time on ablations.
- Principle 22 says to ask him. This is the case for it: the ladder and the cap are the 'cap raised until a test passes' it forbids, and the decision is his: which field extent the far map should hold.
