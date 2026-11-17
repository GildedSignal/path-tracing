# What a pinned word's exit really is: Opus's re-derivation after Astra's 1-ULP witness (17 November)

Saved by the integrator from the Opus Designer's final message (the harness blocks sub-agent writes). Its experiment crate is copied to `jobs\pin-rederive\pinx\`; it depends read-only on the `wt/u2b-astra` crates. Build it with `CARGO_TARGET_DIR=D:\path-tracing-target\pin-rederive cargo build --release --offline`. Modes: none, `mono`, `down`, `merge`, `res`, `sweep`, `disc`. Variables: `WORLD=gap|step`, `YAW=`, `Z0=`, `K=` (log2 of the samples), `MINMAP=1` (key on the law's hold outcome), `WALL2=1`.

## Verdict
- **The witness is not rounding dust, and the pin is not "the law forgetting".** It comes from the law's contact rule: Mojang treats any remaining distance below 1e-7 as zero once another collider follows in the visit order. So a body that arrives within 1e-7 of a wall is **held where it is**, not placed on the wall.
- **Every pinned piece therefore has two kinds of image:**
  - a **point**, the wall's exact coordinate, when the clamp writes the remaining distance;
  - a **hold**, the entry's own earlier position, which is an exact family at most 2×10⁻⁷ wide.
- Both images are monotone maps. They are cut by a law comparison (a monotone threshold), so the object stays **families and points, few and exact, one mechanism**.
- My earlier proposal missed this, and the miss is mine. The holds are about 2.5×10⁻⁸ of the entry domain, so none of my 65,537 samples landed in one. Astra found it through exact cut neighbours.

## 1. Where the trace comes from

**Direct trace of the witness** (`pinx` default, tick by tick):
```text
entry 405818d3c15831d9: t=12 x=100.64343472159220028 (free) | t=13 x=40592cccccc00000 moved.x=5.6565e-2 hc=true
entry 40581c7285449b7c: t=12 x=40592cccccbfffff (free, W-1ulp) | t=13 x=40592cccccbfffff moved.x=0e0 hc=true
```

**The law, step by step:**
- The body's half-width is r = 0.3f32 (`shapes.rs:121`), and the wall coordinate is W = 101 − r = 0x40592cccccc00000.
- On the x pass, the wall cell clamps the distance to delta = plane − b.max (`shapes.rs:414-422`). On the lattice at |x| < 2^27, both x + r and plane − b.max are exact: x and r are multiples of the binade's ulp, and the subtraction falls under Sterbenz.
- So a clamp writes x' = x + delta = W **exactly**, the same point for every entry.
- **The exception:** when delta < 1e-7, the next shape visited sets d = 0 (`movement.rs:613`, and `:628` on the streaming path; also `shapes.rs:399-401`). Then x' = x (`movement.rs:750`).
- The second entry reached W − 1 ulp in free flight at tick 12. At tick 13 its remainder was 2⁻⁴⁶, and the second wall cell zeroed it.
- **The overlap side:** a body that comes down past the plane by less than 1e-7 is also held (`shapes.rs:414` ignores an overlap ≥ 1e-7; `:421` accepts delta ≥ −1e-7).
- **Visit order** is z, then y, then x (`world.rs:609-626`, Mojang's Cursor3D).

**The other contact constants the compiler must take from the law:**
- the 1e-5 hit test, `math.rs:35-36` via `movement.rs:752`. A clamp shorter than 1e-5 raises no horizontal collision: it is a silent clamp;
- the 1e-7 gate on the position update, `movement.rs:745`.

**Partition, gap at yaw 275.625** (`MINMAP=1 pinx mono`). Every piece has key_mismatch 0, its exit never decreases, and the hold identity is never violated:

| Piece | Kind | Entry ulps | Pinned x |
|---|---|---|---|
| P02 | pin at tick 13, clamp | 6.2×10¹² | 1 value, W |
| P03 | pin at tick 13, hold (pre-pin x in [W−1e-7, W], with the silent-clamp window) | 710,724,298 | 7,036,875 values, [W−1e-7, W] |
| P04 | pin at tick 12, clamp | 2.2×10¹³ | 1 value, W |
| P05 | pin at tick 12, hold (pre-pin x in [W−1e-7, W+1e-7]) | 14,073,748 | 14,073,748 values |

Astra's witness pair falls across this cut: one entry is in P02 and the other in P03.

**Is the count bounded by a law fact? Yes:**
- A clamp piece gives one x (times the declared flag, which stays carried).
- A hold piece gives one lattice interval of at most 1 + 2×10⁻⁷/ulp(x) values: 14,073,748 at |x| in [64, 128) (measured), about 1,700 at 10⁶ and about 54 near the world border.
- It is stored as two endpoints and a map. At most one hold piece forms per pin tick.

## 2. Does the trace matter downstream?

**Without a further wall** (`pinx merge`): 260 pinned samples give 68 distinct states and 66 distinct x, unchanged from 0 to 80 ticks after the exit.

**The tick-by-tick run** (`pinx down`, 120 ticks): the trace persists as an exact translation, with an offset of up to 1e-7. The other events are identical: horizontal collisions on ticks 1–2 and landings at [9, 22, 34, 46, 58] for every representative. One member 1 ulp off merged when its value was absorbed.

**With a second wall at x = 115, three blocks high** (`WALL2=1 pinx merge`):
```text
MERGE after_exit_ticks=40 pinned_samples=260 distinct_states=68 distinct_x=66
MERGE after_exit_ticks=60 pinned_samples=260 distinct_states=2  distinct_x=1
```
The remaining 2 states are the minor-collision flag's sprint split.

**So a hold reaches T or the next word's choice only as an ordinary family:** a later threshold would have to fall inside its ≤ 2e-7 span, and that is cut exactly like any face. The next clamp from farther than 1e-7 merges it back into a point. Holds never compound.

## 3. The right object

**One object:** a piece is an entry interval with a monotone exit map.
- Free motion: P2's exact maps. Inside the 1e-5 silent window this is min(map, W); it raises no event, so it is no new piece.
- Clamp: a constant, the point (one per flag value).
- Hold: the identity applied to the earlier map, a family.

**The cut stays on the input side, at the law's own contact comparisons.** It is not moved to "where the law forgets": a held entry is never forgotten until the next clamp, so there is no later tick to cut at.

**Counts with the law's hold selector** (`MINMAP=1 pinx res`, stable at every resolution from 2^6 to 2^16):
```text
gap  yaw=270     z0=0.5 -> 5   gap  yaw=275.625 z0=0.5 -> 7   gap yaw=270 z0=1.0 -> 7
step yaw=270     z0=0.5 -> 8   step yaw=275.625 z0=0.5 -> 12
```
Without the outcome in the key, gap 275.625 gives 5, Astra's merged object, whose pins differ.

## 4. Why yaw 270 closes and 275.625 does not

**It is not the yaw. It is how many collider cells the box spans at contact.**
- At yaw 270 with z = 0.5, the body spans only the wall cell z = 0, and that cell is the last shape visited. The sub-1e-7 remainder is written, so x = W.
- At 275.625, z has drifted to 0.95 at the pin, so the box spans z = 0 and z = 1, and the second cell zeroes the remainder.
- **The proof:** yaw 270 with z = 1.0 behaves exactly like 275.625 (7 pieces, 2 holds; the shell is 7,036,875 values wide).

**The grid** (`K=10 pinx sweep`, 13 grid yaws from 236.25 to 303.75, gap and step): holds appear wherever the box spans two cells at a pin, and every other pinned piece is a single x at every yaw (`other_pin_pieces_with_unequal_pin_x=0` throughout). The rule belongs to the box, so it covers every yaw.

**The flag stays as declared:**
- At yaw 270 it alternates inside a clamp piece: 159 sign changes in the step's tick-2 pin, the acos noise.
- At 275.625 it is a single threshold.
- It is carried as two points, never cut.

## 5. Cost at the two extremes

Fermi inputs: a law tick costs 0.75 µs, a word is L = 12 ticks, one bisection cut is about 3 µs, and the hard track has J = 10³ joints with W = 10–30 words per joint.

**The million-block line: no change.** There are no contacts. A stop on B against a wall adds at most one piece.

**The hard track (S166):**
- Hold pieces are measured at 0–1 per pin tick, so pieces per (family, word) go from 5–8 to 7–12.
- Each hold costs one cut plus two endpoint replays, about 21 µs.
- **Cold:** about 4 s, against 3 s before, spread over thinking while moving.
- **Warm:** about 0.1–0.2 µs per joint, unchanged: one more comparison in the sorted lookup.
- **Memory:** about 11 MB cold (7.7 MB before) and about 90 KB hot.

## 6. The disc frame (Julien's addendum, S200 and S202), tested from the law

**For free motion, it holds in the reals:**
- The drag is the same on both axes (`movement.rs:455`, `:799`).
- The input is normalized (`shapes.rs:561-575`).
- The jump boost uses the same sine-table vector (`movement.rs:378-379`).
- The aim carries no memory: it is written from the input before it is read (`movement.rs:140-141`).

**But the disc's radius depends on the yaw, through the sine table** (`pinx disc`, gait speed over 40 cycles on each of the 64 grid yaws):
- the relative speed is ½(sin² + cos² − 1) exactly, between −2.88×10⁻⁸ and +2.96×10⁻⁸;
- over 10⁶ blocks that is ±0.03 block, enough to move T by one tick.

So a lower bound in the disc frame must use the largest table norm, not 1. That is a law fact, not a tolerance.

**What the frame cannot do:**
- Positions are two separately rounded doubles, so the exact readout stays per-axis maps, and binade crossings stay per axis.
- Contacts are per axis in `axisStepOrder` (`shapes.rs:96-103`).
- **It does not change the pin.** The hold depends on the box's cell span and the visit order, which is the box on the grid at contact.

**The core already uses the disc frame where it matters:** `lattice::bracket` picks the two neighbouring sine-table directions, out of 65,536, that bracket the bearing (`lattice.rs:86-93`, `solve.rs:98`). Their heading loss is about 1×10⁻⁹. `vocabulary.md:58` ("64 yaws … loses about 0.1%") does not describe this solve and should be checked.

**The oblique extra cost (261 against 169 µs) is not removed by the frame.** From Astra's record:
- the bearing stage takes 58.7 against 15.5 µs: the count search in `lattice::counts` (`lattice.rs:171-177`);
- the inverse takes 130.5 against 108.8 µs;
- the output takes 58.1 against 35.4 µs.

By Fermi, the necessary oblique extra is about 1–2 µs: the second axis's roughly 19 binade events. The rest is implementation:
- a closed-form two-aim count (a 2×2 solve, in any frame, then an exact check of the neighbouring integer pairs);
- zero copies (86–87 today) and zero allocations (321–454 today).

The disc frame makes the reason visible; it is not the lever.

**One thing to check:** the aim is dead at entry, so yaw and pitch should not be in an entry key. Including them is exact, but it splits keys by aim. I did not check whether Astra's read tracker already drops them.

## 7. The builder's list

**Replacement for `docs/representation.md:179`.** Only the sentences about cuts and points change:
> …A **family** is an interval of the free coordinate's lattice, cut into **pieces** where the word's swept box crosses a face between cells of different class and at the law's own contact comparisons around that face (the 1e-7 approach and overlap shells, the 1e-5 hit test), all monotone thresholds computed from the footprint and the exact integer maps (P2), never found by sampling. Inside a piece the exit is a monotone map of the entry. It is P2's map in free motion, the face's exact coordinate where the word clamps the free coordinate (a **point**, one per value of the law's non-monotone flags, today only the minor-collision test, `movement.rs:818`), or the entry's earlier image where the law's contact rule holds the body (a **hold**: within 1e-7 of the face and followed by another collider in the law's visit order, `shapes.rs:399`, `movement.rs:613`; at most 2×10⁻⁷ wide, merged by the next clamp). Points and holds that become bit-identical merge…

**Kill test 11 (`learning.md:127`):**
> **Pieces are few.** Per joint pattern, the computed pieces equal the native partition by events and contact outcome (free, clamp, hold) at every resolution, and do not grow as it is refined (`u2_pins_point_and_hold`). The first re-cut missed the hold. With it: gap 5/7 and step 8/12 at yaws 270/275.625, and 7 at yaw 270 across two wall cells (`jobs/pin-rederive/proposal.md`).

**In Astra's code on `wt/u2b-astra`:**
1. **The law reports the contact outcome** for each axis pass (free, clamp with its plane, hold) from `collide_in_world` (`movement.rs:596-635`) and `ShapeView::collide` (`shapes.rs:395-465`), for example by extending `tick_with_collision`, under the law hash, with no change to the recurrence.
   - `face_comparisons` (`shapes.rs:89-93`) gains the approach comparisons, computed exactly as the law computes them: `plane - hi < 1e-7` and `lo - plane < 1e-7`.
   - The law lists its contact constants: 1e-7 (`shapes.rs:399`, `:414`, `:417`, `:421`; `movement.rs:613`, `:628`, `:745`) and 1e-5 (`math.rs:36`).
   - `NON_MONOTONE_FIELDS` stays the flag alone.
2. **`families.rs`:**
   - The replay signature (`:46-62`) carries the per-tick outcome on the free axis.
   - `learn` (`:75-`) bisects each hold's boundary on P2's map against the approach comparison. It checks the "another collider follows" condition from the law's outcome at both ends, and cuts at the expanded box's cell planes if the outcomes differ.
   - The pin check (`:149-154`) becomes: for a clamp, the pinned keys are equal at both ends except the flag; for a hold, the pinned x equals the pre-pin x and the other fields are equal.
   - The image (`:171`) is Points for a clamp and a map (hold applied to P2) for a hold.
   - The merge (`:188-193`) uses equal signature and outcome word. This should also bring step yaw 270 from Astra's 9 to the oracle's 8.
3. **`Piece` (`joints.rs:66-73`):** the exit is either a map (P2 compositions with the clamps and holds at their listed ticks) or points.
4. **Recorded reds of the old object, never weakened:**
   - `u2_pieces_computed_and_closed` (frozen hash 5C04…; its 275.625 expectations came from my wrong premise);
   - `u2_pin_erases_coordinate_premise`;
   - `u2_pin_cut_neighbourhood_witness`.
5. The stored stop on B and the zero-copy full solve proceed on this object.

**The new gate, `u2_pins_point_and_hold`, frozen before its first run:**
- **Inputs:**
  - the fixture of `tests/u2b/premises.rs:4-17`: gap and step at yaws 270 and 275.625 with z = 0.5;
  - the gap at yaw 270 with z = 1.0 and the support block at z = 1;
  - `glm-ice` and `hardtrack-lane-wall`, as in `pieces.rs`;
  - entries over [95.5, 99.5].
- **Oracle:** native ticks only, using the law-reported outcome. Keys are the per-tick events (flag masked) plus the free-axis outcome up to the first horizontal collision. Samples run from 2^6 to 2^18, with ±8 ulps around every computed cut and 256 consecutive ulps at both ends of every hold.
- **Checks:**
  - (a) the partition equals the computed pieces;
  - (b) every exit is bit-equal to its piece's prediction, with resources inside the use interval;
  - (c) in a clamp piece, the pinned x equals plane − (0.6f32/2) as the law computes it, and the endpoints are equal except the flag;
  - (d) in a hold piece, the pinned x equals the pre-pin x and lies within the law's 1e-7 of W;
  - (e) the witness: `0x405818d3c15831d9` is in a clamp piece with pinned x `0x40592cccccc00000`, and `0x40581c7285449b7c` is in a hold piece with pinned x `0x40592cccccbfffff`;
  - (f) with the second wall, all pinned gap states at 275.625 reach one x within 60 ticks after the exit.
- **Expected pieces:** gap 270 → 5, gap 275.625 → 7, gap 270 with z 1.0 → 7, step 270 → 8, step 275.625 → 12.
  - `glm-ice` and `hardtrack-lane-wall`: run the native oracle on them before freezing. Astra measured 2 and 2, and I did not re-measure them.

**Anomaly for HANDOFF:** Mojang holds a body at its free-flight position when it arrives within 1e-7 of a wall and another collider follows. The hold persists until the next clamp, and depends on how many cells the box spans, not on the yaw. It is nature's behaviour, ported faithfully.

**Sources:** the law in `wt/u2b-astra`, read at the lines cited, and Astra's record and evidence. Mojang's `Shapes.collide` and `Cursor3D` were not re-read; I relied on the port's comments.
