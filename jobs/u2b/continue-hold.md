# Continue: the pinned exit is a point or a hold (adopted, S203)

Your witness was right, and it found the cause. Opus re-derived the pin from it, and Julien adopted the result (S203: "sure looks good").

**Read, in full:** `C:\Users\PASCAL\Desktop\path tracing\jobs\pin-rederive\proposal.md`. Its experiment crate is `jobs\pin-rederive\pinx\` in the same root.

**The ruling in the docs:** `docs\representation.md:179` (the set: points and holds), `docs\learning.md` kill test 11, and `docs\vocabulary.md` section 5 (aims carry no memory and stay out of entry keys).

**In one breath:** Mojang zeroes a remaining distance below 1e-7 when another collider follows in the visit order (`movement.rs:613`, `shapes.rs:399`). A body that arrives that close to a wall is therefore **held** at its free-flight position, not placed on the wall. A pinned piece's image is either a **point** (the clamp writes the face's exact coordinate, one per value of the declared flag) or a **hold** (the identity on the earlier P2 map, a family at most 2×10⁻⁷ wide, merged by the next clamp). Both are cut by the law's own monotone contact comparisons. Whether a hold appears depends on how many cells the box spans at contact, not on the yaw.

## Do
1. **Section 7 of the proposal, items 1–4:**
   - the law reports the contact outcome per axis pass, under the law hash, with the recurrence unchanged;
   - `face_comparisons` gains the approach comparisons, and the law lists its contact constants;
   - in `families.rs`: the signature, `learn`, the pin check, the image and the merge;
   - `Piece`'s exit becomes a map or points.

   Keep the three old tests as recorded reds of the old object, never weakened.
2. **The new gate `u2_pins_point_and_hold`**, exactly as specified in section 7, frozen before its first run. Your witness pair is case (e).
   - Before freezing, run the native oracle on `glm-ice` and `hardtrack-lane-wall` and write their expected counts from it.
   - Expected: gap 270 → 5, gap 275.625 → 7, gap 270 with z 1.0 → 7, step 270 → 8, step 275.625 → 12.
   - If the oracle disagrees with these, the oracle wins: record it, and don't adjust the gate to match.
3. **Check whether yaw and pitch are in any entry key** (section 6, last point). The aim carries no memory, so remove it from keys if it's there, and say what changed.
4. **Then resume what was waiting on this object** (your `last.md`, item 8, and the original `jobs\u2b\prompt.md` Part 2):
   - the family-and-piece wave;
   - the stop on B, stored once as a joint and looked up;
   - zero player-state copies and zero allocations in the warm full solve, counted.

   Also from the proposal, section 6: the oblique extra should be about 1–2 µs (the second axis's binade crossings). The bearing stage's count search (`lattice::counts`, 58.7 against 15.5 µs) wants a closed-form two-aim count: a 2×2 solve, then an exact check of the neighbouring integer pairs.
5. **Re-run every gate** in the original prompt's list and report the warm million line (axis and oblique) as min, median, p99 and max against the ~5 µs floor, with a profile of what remains.

The base and rules of `jobs\u2b\prompt.md` still hold. A failed premise stops only what depends on it. Update `last.md`, and end with `STATUS: DONE` or `STATUS: BLOCKED: <why>`.
