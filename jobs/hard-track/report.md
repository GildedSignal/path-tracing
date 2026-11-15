# Hard track: offline build and certificate

Consumer: Julien's race and the hard extreme for `jobs/3.2d-3/spec.md`. The current planner is a diagnostic only. No game, server or RCON was run. The integrator still has to construct the course in-game before Julien can race it.

## Build and replay

Use [the ordered vanilla commands](../../lab/tracks/hardtrack.build.txt), [A/B metadata](../../lab/tracks/hardtrack.json), and [the staged tracks.txt line](../../lab/tracks/hardtrack.tracks.txt). The line is deliberately staged beside the build, rather than registering an unbuilt region snapshot in `engine/world/tests/fixtures/tracks.txt`. Register it there after the in-game build is captured.

The first command clears the entire specified box above the grass floor. **Before executing it, query and save `minecraft:max_block_modifications`, set `/gamerule minecraft:max_block_modifications 106641`, then restore the saved value after construction.** This is a vanilla build prerequisite, not a live action performed by this job. The default is 32768, verified in the staged 26.2 `GameRules` bytecode; all later fills fit that default. [build.md](build.md) regenerates these values and pins the jar. The box is x/z -110 through -60, y -60 through -20; grass at -61 is preserved. These are build inputs, checked by the parser.

`engine/plan/tests/hard_track.rs` parses these exact commands in order over superflat bedrock/dirt/grass. The offline fixture is Peaceful, starts with the law's default healthy body settled on A, and contains no entities. Every block is obtained through the existing generated `world::model` registry, including exact ladder states and blue-ice/slime properties. There is no separate hand-authored collision world, search implementation, law modification, item use or new crate.

The law inspected here implements movement/collision/step-up, jump and sprint impulse; friction/speed/jump factors (ice variants, slime, soul sand, honey); climbable ladders/vines/scaffolding and the open-trapdoor ladder hook; water height/current/travel; cobweb and powder-snow stuck hooks, powder collision/equipment handling; honey sliding; bounce/fall-reset/landing multipliers and grounded contact requests. Its generated shapes include slabs, stairs, fences, walls, doors and trapdoors. Unsupported hooks refuse. **This course uses ordinary full shapes, blue ice, slime and fixed ladders only.** Water, fragile ice, powder snow, world-hook blocks and interactive shapes are unnecessary for this certificate.

## 1. Elements, chosen backward from B

B's isolated high deck requires the suspended ladder. That ladder's low catch requires a jump from the rebound-exit deck. That deck is above the slime floor's ordinary-jump reach; its upper baffle prevents a direct falling transfer from the preceding landing. The bounce needs the tall drop. That drop's entrance is the landing beyond the ice gap, which needs carried ice momentum. Finally, the accelerator's elevation requires the initial ladder. This is the backward dependency order; the table below is the racing order.

| Element from A | Required state / certificate | Tested timing slack |
|---|---|---|
| Initial vertical climb | Settled on the gold A block; climb the backed ladder to the accelerator. Its rise exceeds the law's ordinary jump apex by many times. No other floor-level ascent reaches the course. | Climb-to-accelerator switch survives both -1 and +1 tick. This is a reach element, not a narrow timing claim. |
| Blue-ice head-hitter into the long jump | Ground-only jump feedback builds momentum under the low ceiling; release jump before the ceiling ends, then launch. Seven empty cells and a lower landing require at least 6.4 blocks of center displacement even granting full body overhang at both ends. The conservative ordinary-stone/head-hitter bound is 5.966731392155813 blocks. The witness carries the exact launch state in [witness.md](witness.md). | Moving the takeoff pulse -1 tick fails; +1 passes. Tested slack: 0 ticks early, at least 1 late. |
| Landing turn, drop, slime rebound and west exit | Turn south after the long jump, fall down the constrained chute, then turn west during the fall. Slime carries the body to a high lateral aperture; the rebound clears the exit support by 0.379788496764789 blocks. An ordinary jump from that floor cannot reach it. | West-turn -1 still touches the exit deck but misses the next ladder with all later inputs fixed; +1 reaches B. This is the **composed entry-state/phase** certificate; it is not a claim that the isolated bounce has a one-tick window. Tested sentence slack: 0 early, at least 1 late. |
| Exit-deck alignment, gap and suspended ladder catch | The west stop fixes x, then a sprint jump across three empty cells catches the bottom of the ladder. Walking off falls below its lowest climbable cell before reaching it. The ladder then supplies the final ascent to B. | Moving the takeoff pulse -1 tick fails; +1 passes. Final climb release survives both shifts. Tested takeoff slack: 0 early, at least 1 late. |

The timing claim is deliberately scoped to ±1 tick with all later boundaries fixed, rather than an exhaustive maximal window. [timing.md](timing.md) gives every replay, including failed/dead states and the distribution of successful arrivals. [controls.md](controls.md) removes blue ice, the ceiling, slime or the final ladder independently: the frozen sentence fails in every case.

The falling-transfer exclusion follows the geometry: the south opening is east of the tall separator. The body cannot cross the upper west baffle until its head is below the baffle. Even in crouching pose, the remaining fall before the exit ledge permits fewer horizontal blocks than the separation between baffle and ledge; there is no intermediate support. The baffle's south end meets the chute's south wall and its north end overlaps the separator, so it cannot be skirted at entry height. The replay passes underneath it, rebounds and exits through the lower opening.

## 2. Branch points and decoys

There are three spatial forks with similar initial forward progress. Each false arm becomes dead **after commitment into its pocket**; turning around before commitment is allowed and is not called a dead state. The only combination of committed arms that completes is north accelerator, south rebound well, west suspended ladder. This does not claim a globally fastest input sequence within the successful combination.

| Fork | True arm | Decoy and bound |
|---|---|---|
| Accelerator deck | Northern blue-ice lane | Southern lane has the same accelerator length/ceiling profile and apparent gap progress. Its landing drops into a closed stone pocket. The lowest return sill is four blocks above the floor; the law's jump apex is 1.252203340253724. The pocket has no climbable cell, fluid, inside hook or intermediate support. |
| Long-jump landing | South drop into the well with a west exit | North drop also has a slime pad, but no high exit. Its conservative rebound bound is y=-33.458564655338250, below the return sill at -29. The calculation starts above every possible entry jump and adds a full gravity tick to the uncollided impact bound. Later bounces dissipate; a normal jump cannot improve the first rebound. |
| Rebound-exit deck | West ladder with a low catch | East has another high ladder aligned toward the finish, but its bottom is above every ordinary launch apex. Missing it drops into a closed stone pocket with a seven-block return sill. Its high floating backing offers no reachable intermediate support. |

`reach_and_decoy_certificates` verifies each pocket's floor, every wall up to the minimum return sill, and every interior cell below that sill. These are geometric bounds for **all controls** under the used law, not merely failure of one attempted word. The jump and rebound recurrences, numerical margins and refusal list are regenerated in [bounds.md](bounds.md).

## 3. Witness T

**583 ticks from settled A to first touch of B; 579 on the existing detector clock** (first sample off A subtracted). The witness is [hardtrack.witness.txt](../../lab/tracks/hardtrack.witness.txt), a compact input sentence; `G` means jump only while on ground, and `J` means held jump. All other inputs and segment durations are explicit. It restores everything by making no world edits and using no items; health remains full.

The full law replays every tick, compares a second replay's entire state bits, and checks a frozen digest over the start and every subsequent state. [witness.md](witness.md) records the digest, entry/exit states and worst vertical margin. This is an existence certificate, not a proof of global optimality or an independent-search optimum. Some measured shifted sentences are faster; they remain visible in the timing evidence. Julien's human time is pending.

## 4. Current planner result

**No sentence** in the capped existing mode-5 diagnostic. It uses the current backward field and forward planner over the command-built world, with the flight's mode-5 options. The production `plan-track` binary cannot load a not-yet-built region fixture, so its existing APIs are called directly in the ignored integration test. No planner code was changed.

[planner.md](planner.md) records preparation, field/search time, node counts, stop reason, every refusal and any uncertified candidate. The budget and exact command are there. This result is diagnostic and does not gate the track or prove it impossible for that planner with a larger budget.

## 5. Later item-use extensions

- Pearls could replace either long horizontal transfer or the suspended-ladder catch; the later certificate must include throw motion, cooldown, damage and restoration obligations.
- A bucket could create a different descent, a water ascent around the exit baffle, or a controlled landing. Source retrieval and all downstream water changes must be restored before B.
- Boats could introduce a different ice-speed entry and a mounting/dismounting or fall-clutch branch. The existing separate boat law does not make these executable by this course's player/planner interface.
- Temporary blocks could introduce a recoverable foothold around a pocket sill. Placement, identical-block recovery and all inventory/world debt belong inside T.

None of those capabilities is assumed in any present bound.

## Reproduce

From this worktree run `powershell -NoProfile -File jobs/hard-track/check.ps1 -Diagnostic`. It regenerates evidence/report, records source/input SHA-256 hashes and commands, runs the full offline suite, and optionally reruns the capped diagnostic. [last.md](last.md) inventories every delivered file and command. No live audit was performed, as required by the job.
