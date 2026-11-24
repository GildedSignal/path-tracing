# Backward state-set spec: the frozen design and why (`flight-ladder-sets`)

Designer: Opus, read-only over code, 24 November 2026, base `main` at `7b7aaaee`. The closed build prompt is [`jobs/flight-ladder-sets/prompt.md`](../flight-ladder-sets/prompt.md). This report holds the reasons, the Fermi estimates, the blockers found in the existing code and the shortcuts that were turned down. The prompt holds the decisions themselves. If the two ever disagree, the prompt is the one Astra builds from.

## 0. What I measured before designing (main, `plan-track --mode solve`, `PLAN_MEASURE_REPEAT=1`)

Release build in `D:\path-tracing-target\backward-sets-spec`. The raw lines are in [`baseline/`](baseline/), one script per track that reached B. "His best" applies the `+1` clock rule from `flight.rs:33`.

| Track | main today | his best | first exception (lexicographic) | exceptions in the straight box | first µs | repeat ns |
|---|---|---|---|---|---|---|
| grassjumpeasy | NeedsJoints | 50 | [-11,-60,6] | 12 | 159 | 28,300 |
| flowers | **48**, replayed onto B | 48 | — | 0 | 92 | 2,700 |
| stonestaircase | NeedsJoints | 113 | [-12,-60,49] | 18 | 125 | 32,800 |
| fences | NeedsJoints | 89 | [16,-60,41] | 87 | 235 | 143,200 |
| fencestraightbridge | NeedsJoints | 47 | [-30,-61,32] | 16 | 125 | 35,400 |
| fencemaze | NeedsJoints | 66 | [-8,-60,-24] | 78 | 232 | 131,300 |
| birchparkourmedium | NeedsJoints | 333 | [21,-57,3] | 6 | 121 | 15,000 |
| mangrovejumpeasy | NeedsJoints | 45 | [-24,-60,13] | 10 | 127 | 24,900 |
| doorcorridor | NeedsJoints | 99 | [27,-60,68] | 20 | 210 | 112,800 |
| trapdoorparkour | NeedsJoints | 184 | [-26,-60,-46] | 75 | 242 | 146,800 |
| stonesimple (control) | **85** | 86 | — | 0 | 88 | 4,700 |

`flowers` already reaches B on `main`: a flower has the same legs answers as air, so the corridor is clear. "A flower is air" is already emergent; nothing in this pass may break it. Practice tracks 1 to 10 (his best 9, 14, 12, 21, 12, 12, 12, 9, 12, 12) live in the `pathtracing:diversity_parkour` dimension. Astra's checkpoint found that the archive's chunks are DataVersion 4790 and the reader requires 4903. I did not touch his server world.

The repeat column says the refusal is not cached either: 28 to 147 µs is spent re-running the corridor query over the scars.

## 1. Verdict

The parts exist on `main` but are not connected:
- `Family::learn` (`families.rs:110`), `learn_family` (`family_wave.rs:15`), the wave (`joints.rs:287`, `family_wave.rs:77`) and `route` (`joints.rs:304`) are all present.
- Nothing calls `learn_family`, `Family::learn`, `JointStore::learn` or `MirroredReach`.
- `solve.rs:269` turns the first exception into `NeedsJoints`.

The wave has no producer. Connecting it takes four moves:
1. find the first exception **along the word**, not in lexicographic order;
2. hand the takeoff states before it to learning as demands;
3. let learning run one operation per demand (`learn_family` over that demand's states, every candidate word), best first, under a budget proportional to the defects;
4. compose prefix, site words and tail in one min-plus wave whose edges are the learned pieces and the extend chains.

Fourteen blockers in the existing code stop this (section 6). Two of them make caching impossible as the code stands:
- **the tick counter `t` is part of every key**;
- **resources are compared bit for bit**.

So nothing learned at one tick or one exhaustion value can ever be found again. In his game every click has a new `t` and a new exhaustion.

## 2. The data flow (one command)

```
solve(from, B):
  bracket  LB = B.lower_bound(from)
  known?   joints.route(from, B): from's key is a node with T in the field of B
           -> walk the field (no law tick)                             [repeat path]
  extend   Crossing::run(from -> B)  (unchanged closed form)
  presence first exception ALONG the word: bisection on the checkpoint
           index c, the corridor box of (prefix through c + next period) [one query each]
           Clear -> compose (today's path, unchanged)
  cut      window W = checkpoints within one period's displacement of the
           first refused period (W = {from} if it is the first)
           file extend edges from -> w (words truncated at w)
           site(W, B): one learning queue, best first by arrival + gap/v_gait
             pop demand s (an exact arriving state, or a family of them)
               already learned for this key? reuse its pieces (no law)
               extend edge from s: Clear to B -> tail edge s -> B (closed form)
                                   exception  -> s's own window = new demands
               learn: learn_family(s, every candidate word) -> pieces -> exits
               each exit -> demand (arrival + piece ticks)
             stop: bracket closed (min sound key >= UB) | budget | queue empty
  compose  wave(B): backward min-plus over pieces + extend edges (+ families)
           walk from `from`: extend edge -> site pieces -> ... -> tail edge
```

A uniform segment is never searched. It is one extend edge (a counted gait), filed with its own words and its corridor proof. `straight -> defect -> straight` therefore costs one extend, one site and one tail. A chain of several defects is one site queue whose demands are linked by extend edges. Momentum carries because every demand is an exact arriving state: velocity bits stay in the key and no cell centre is ever used.

## 3. The frozen interfaces (summary; exact text in the prompt)

| Object | Where | Shape |
|---|---|---|
| state key | `joints.rs` `write_key` | the exact state minus `t`, minus every `is_resource()` field, minus `fall_distance`, `yaw`, `pitch` and `input`; support stored relative to the body (as today); for a family, the free axis's exponent only |
| resource guard | `joints.rs` (replaces `Resources::same_bits`) | the same `codec::resource_guards(food, health)` bytes, plus `health − damage(piece) > 0`, plus equal `fall_distance` bits unless both stay ≤ 3.0 through the piece; valid only at difficulty 0, which `solve` already requires |
| node | `JointStore::node` | interned by the key with no free axis and the guard class; the stored state is canonical, with `t = 0` (as `pattern.rs:317` does) |
| certificate ticks | `ReadCertificates::capture` callers | relative to the entry tick; every edit invalidates over `(0, u64::MAX)` until there is a `World(t)` |
| word stop events | `Family::learn` stop `(before, after, tick)` | goal touch (swept); landing (air to ground); a new contact (`horizontal_collision` rises); a step-up (grounded and y changes); leaving the site box; the word budget (`max_ticks`, then Searched) |
| candidate words | site queue | every `tiny_programs` id whose `held_input` exists (24) × aims: the bracketing table-yaw pair toward B, toward each standable face centre in the site box, and the 4 axis yaw indices 0, 16384, 32768, 49152, de-duplicated |
| demand | site queue | an exact state, or a family (≥ 2 window states sharing a key on the dominant axis); ordered by arrival + ⌈gap / v_gait⌉ (ordering only); pruned only by arrival + `lower_bound` ≥ UB (proof) |
| budget | site queue | 12,000 law ticks (the map's "~10³ verifications × 12 ticks" per new joint pattern) × the number of exceptions the straight corridor reported |
| extend edge | `JointStore.extends` keyed by goal | `(from node, to node or B, ticks, words, corridor box)`; invalidated when an edit falls inside its box |
| site key (measured, not reused) | `JointStore` | a digest of the entry key with each horizontal coordinate as (fraction bits, exponent), the word, and the footprint's answers relative to the entry floor cell and entry tick; this pass logs would-be translation hits only |

## 4. Fermi

Inputs from the map: a law tick 0.6–0.95 µs (0.8 is used here), the floor of ~12 ms per new joint pattern, words of ~12 ticks. My own estimate: certificate capture at ~30 cells × 12 ticks, ~100 ns per cell answer.

| Case | Estimate | Floor | Gap |
|---|---|---|---|
| words per demand | 24 × (2 + 2F + 4), F = 3–10 faces → 290–630 | — | — |
| one word, learned | 1 replay ≈ 10 µs + capture ≈ 36 µs ≈ 46 µs | — | — |
| one demand | 13–29 ms | 12 ms per pattern | ~0.2–0.4 order |
| demands under budget | 12,000 × X / (W × 12): grassjumpeasy (X = 12) ≈ 25–40; fences (87) ≈ 140–300 | — | — |
| cold site, wall time | grassjumpeasy ≈ 0.3–1 s; fences or fencemaze ≈ 2–8 s | J × 12 ms ≈ 60–360 ms | ~0.7–1.3 orders |
| repeat of the same command | a key probe (~1 µs) + the cached field + cloning ~10–30 WordSteps ≈ 5–20 µs, 0 law ticks | ~1 µs | ~0.7–1.3 orders |
| flat ground, new command | unchanged: the probe is skipped while the store has no nodes, ~+1 µs after sites exist | 1–10 µs | median must stay ≤ 15 µs |
| wave after a site | Dijkstra over ≤ 10⁵ edges with the indexed family pass ≈ 1–10 ms, once per goal | — | — |

The cold cost sits about an order above the floor because certificate capture (B6 removes 3 to 6 times the law ticks) and the aim fan-out dominate. Three levers would close that gap. Each is named in the prompt as next and none is built here:
- translation reuse (section 5, would-be hits measured);
- 2D families, or families over the count of the incoming word;
- the arc certificate in learning, once `law::face::advance` reaches `plan`.

On his tracks, almost every arriving state is transient: momentum's memory is about 260 ticks on stone, which is longer than his tracks. So the sets that actually occur there are mostly points (families of width 0). That is the honest object, not a failure. A family's interval only opens on steady approaches along an axis, such as a defect at the end of `straight800`.

## 5. Decisions and their reasons (each is closed in the prompt)

1. **One free coordinate, the dominant axis, as the map's record states** (representation.md section 10, "a one-way door, red-teamed before it freezes"). Table yaws are exact along the axes: index 0 reads `sin = 0`, `cos = 1` from the table, so the perpendicular coordinate stays constant and a family is exact. On a diagonal, the perpendicular bits change on every tick, so the family has width 0. A 2D box family is the principled generalisation, because the law reads x and z only through per-axis monotone comparisons. But it changes the one-way door, so it waits for a measured need: the different-start reuse rate on the rungs.
2. **A point is a family of width 0.** `learn_family` on `[s, s]` is the only learning operation. `JointStore::learn` (the per-point stepper) and `MirroredReach` become dead and are deleted. One operation, one record (learning.md section 2).
3. **`t` leaves every key.** The law reads `t` only to add one to it (`movement.rs:216`). The pattern table already zeroes it (`pattern.rs:317`).
4. **The resource guard is the law's comparison, not the bits.** At difficulty 0 food never falls (`movement.rs:1284-1288`), so the sprint guard `food > 6` cannot flip inside a word. Saturation and exhaustion never reach motion. Health matters only through death. Set-rederive already moved resources out of the key (S182, narrowed); this finishes that move.
5. **Joints only at the law's events.** These are landing, a new contact, a step-up, the goal touch and leaving the site; never a block boundary of the same class (ES audit, finding 2). "A new class entering the footprint" was turned down for this pass: it needs the footprint inside the stop predicate and the bisection predicate. The falsifier is `fencemaze`'s T against his.
6. **Aims come from the law's table and the site's own faces.** The bracketing pair toward B and toward each face centre, plus the 4 exact axis yaws. The centre is an ordering choice, not a premise: aims not tried leave the record Searched, never Forbidden. No 64-aim grid, no 1/16.
7. **The budget is proportional to the defects.** It is the map's per-pattern floor times the corridor's exception count. Compute mirrors the terrain (S131); a plain has 0 exceptions and therefore a budget of 0.
8. **The prefix and the tail are extend edges** (closed form, filed with their words and corridor proof). That is how `straight -> defect -> straight` composes without searching the uniform parts. A tail is the same mechanism as the first extend (the presence query), so there is no second path.
9. **Extends run only from the source's own floor layer.** `GeneratorSuperflatSource::support_cell` always answers `floor_y` (`source.rs:199`). From a raised support (stair tops, fence tops, platforms) a demand only learns site words. `birchparkourmedium` and `stonestaircase` will show the cost.
10. **Ticks for a pinned point are resolved by the actual endpoint.** The minor-collision flag (`movement.rs:818`) is not monotone. For a width-0 family the actual variant is known. Inside a wider pinned piece, acting resolves it with one law replay of that single word, and those law ticks are reported.
11. **The site key is measured before it is used.** On transient arrivals, a translated hit needs equal velocity bits, which should be rare on his tracks. Measure first (principle 3).
12. **The bot's corridor grows to what the law read.** The resident source records the first cell a certificate could not cover. `resident.rs` widens its box by its existing margin of 4 around that cell, clipped to the received chunks, and solves again until nothing is missing. The store keeps everything learned in between.
13. **The plan's corridor goes on the wire.** It is the hull of every word's certificate box, as four words appended to `WORD_PLAN`, so the Java side stops playback on an edit inside the real corridor and not only inside `box(at, goal) ± 2` (`ResidentRunner.java:63`).

## 6. Blockers in the existing code (file:line; all fixed by the prompt)

| # | Where | Blocks |
|---|---|---|
| B1 | `joints.rs:38-53` `write_key` pushes `F::t` (default branch); `families.rs:97` `pinned_key_axis` likewise; `JointStore::node` (`joints.rs:156`) interns `s.bits()` including `t` | no fact learned at one tick is ever found at another; in his game, nothing is reused |
| B2 | `joints.rs:25` `same_bits` as the guard in `consume` (263), `equivalent_points` (161) and `family_wave.rs` (the `entry.resources.same_bits` checks); `health` and `food_tick_timer` stay in the key | every click carries a new exhaustion: zero reuse in game |
| B3 | `joints.rs:295` and `:311`: the point wave skips family patterns, and `route` returns `None` on a family edge | learned families can never be acted on |
| B4 | `family_wave.rs:79` loops up to edges + states times; `:91` does an O(edges) `position` per piece; `:101` compares every interval with every other; `:142` crosses states × rows with an `EntryTemplate` allocation each | O(n²) to O(n³): unusable beyond ~10³ pieces |
| B5 | `families.rs:110, 172, 179, 210`: the stop sees only the after-state; `:90` pins on any tick with `horizontal_collision` | no swept goal touch and no landing event; a word sliding on from a pinned start pins at tick 1, forever |
| B6 | `family_wave.rs:27-38` replays both endpoints × flags again for the certificate; `Family::learn` replays lo and hi even when lo == hi | 3 to 6 times the law ticks per word |
| B7 | `learn_family` builds `step` and `ticks` from `predict(lo)[0]` (the first flag variant) | a word's tick count can be the wrong variant: a tick deviation in his game |
| B8 | `source.rs:223, 239`: `first.get_or_insert` in `BTreeMap` order | the "first" exception is not the first along the word; no prefix handoff |
| B9 | `source.rs:199` `support_cell` always answers `floor_y` | an extend from a raised support is wrong (gated off, decision 9) |
| B10 | `source.rs:156` `block()` returns the base layer for an uncovered or unknown cell; only `cell()` (192) refuses | correct (capture refuses), but nobody records what was missing for the bot's corridor |
| B11 | `resident.rs:140-143` resets only on a world join; block deltas (`:110-117`) never reach the store | stale facts once joints exist |
| B12 | `ResidentRunner.java:63` corridor = `box(at, goal) ± 2`; exact length check on the answer | a detour plan is not stopped by an edit on its real corridor; the wire cannot grow |
| B13 | no caller of `learn_family`, `Family::learn`, `JointStore::learn` or `MirroredReach` on `main` | the wave has no producer |
| B14 | `joints.rs:134` `compose_chain` handles one linear chain; the wave has no terminal offset or extend edge | no tail composition |

Outside the code: the practice world is DataVersion 4790 and the reader requires 4903 (`world/src/region.rs`). Only the game's own upgrade may produce a 4903 copy (decision in the prompt). A hand projection (`D:\path-tracing-evidence\flight-ladder\ConvertPractice.java`) is not the judge.

## 7. Shortcuts turned down

| Shortcut | Why not |
|---|---|
| Greedy forward search, aiming straight at a face and copying `PlayerState` every tick (`wt/flight-ladder`'s `joints/cut.rs`, `DemandAt`) | first hit wins, points only, law ticks in acting, no min-plus, no cache; it cannot turn, compose momentum intervals or scale across sites (c74c4a5c) |
| A branch per terrain kind (stairs, fences, flowers by name) | one mechanism (S161); a kit class is its legs answers |
| A grid (1/16, a 64-aim grid, sampled lattices) | made-up numbers (S67); cuts come from bisection on the law's comparisons, aims from the table bracket |
| A cell-centred backward field started from rest (the old `reach.rs`) | forgets momentum; it stalled `fences`, `fencemaze` and `redfakemaze` |
| A\* in acting | acting reads the field; best-first ordering lives in learning's queue (learning.md section 1) |
| A tolerance on the predicted tick, or retrying with more budget or aims | a patch (ES 19); the record says Searched |
| Weakening the DataVersion check, or a hand-converted world | the game is the judge |
| Translation reuse before its hit count is measured | Fermi first: transient arrivals rarely share velocity bits |
| 2D families now | changes a one-way door (the set record) without a measured need |
| Exact resource bits as the guard | breaks every reuse in game; the law reads them only through comparisons |
| Joints at every block crossing | the ES audit's finding 2: a cut of ours, not the law's |

## 8. ES audit: what lands in this pass and what waits

| Finding | This pass | Later |
|---|---|---|
| 1. slide by replay from tick 0 | no slide is executed; `slide_want` is counted per rung (kill test 14 measured, no replay) | the local slide inside a set |
| 2. cuts at body-block crossings | forbidden by the stop events | — |
| 3. cell-keyed backward field | replaced by exact-state demands and the wave | — |
| 4. per-tick arcs | acting never replays; learning replays through `law::contact::tick` (cold only) | the arc certificate, once `plan` uses `law::face::advance` (`wt/port-fold`) |
| 5. `PlayerState` copies on the hot path | the acting walk clones WordSteps (states behind an `Arc`), with no per-tick state copy | compact hot state (`now.md` port) |
| 6. the settle loop and the tick script | settle is already one tick (`flight.rs:50`); the bot still plays tick inputs, and `inputs()` gains the touch-on-the-predicted-tick check | the direct `(word, count)` runner in `bot/` |
| flat ground ≤ 15 µs median, 0 refused | gated (`real_time.rs`) | — |

## 9. Kill tests and catalog entries this pass can fail

- **Kill test 8 / catalog 1.2:** one entry point and one `cut()` for every terrain. `straight800`'s stages are `Bracket, ClosedCheck, Extend, Cut, Compose, ClosedCheck, Allocate`; a site rung adds `Site{..}` between `Cut` and `Compose`. A planted fast path for uniform ground must turn the gate red.
- **Kill test 9 / catalog 1.2:** work units (site law ticks + corridor queries) are reported against exceptions for every rung and for `straight100/300/800`. The straights show 0 site law ticks and equal query counts.
- **Kill test 11 / catalog 1.3:** the one new witness test (`engine/plan/tests/sets.rs`): computed pieces equal the sampled partition at every resolution, every sampled exit is bit-equal to its prediction, and the only difference at a pin is the declared flag.
- **Catalog 1.4:** a decorated-ground series in `real_time.rs` (flowers, buttons, pressure plates, mixed stone, dirt and planks) gives the same T and words as plain ground, with a median ≤ 15 µs. The unused-item half waits for kits: today the kit is only `Kit::BareLegs`, with nothing to carry.
- **Kill test 14:** the count of joints asking for a slide, and of those that do not, per rung. Measured, not acted on.
- **Kill test 4 / test 12 (honest reds expected):** a second command from a start one ulp off on a diagonal rung relearns its site. The prompt asks for that number, not for a fix.

## 10. What this design does not do

- Turns before contact in mazes: the event set has no "new class entering the footprint".
- Extends from raised supports.
- Translation reuse (measured only).
- 2D families.
- The slide.
- The arc certificate in learning.
- The direct word runner.
- Doors and trapdoors that must be used: bare legs has no use action, so a route that needs one ends Searched, or Refused where the law refuses.
- Edits and kits: step 5.

Each is named in the prompt's report section so that Astra records where it bites.

STATUS: DONE
