# Joints and sets (step 2 of docs/plan.md): the integrator's spec

**Inputs:**
- the draft, `jobs\u2-set-record\report.md` (Hydrogen; a draft, not a spec);
- the Opus red-team, `jobs\u2-redteam\report.md` (FREEZE AFTER FIXES), with its offline check in `jobs\u2-redteam\kt10\`;
- Julien's ruling S182: a set is keyed by the exact arriving state, plus plateaus of one coordinate (`docs\representation.md:179`).

**This file is the spec.** It adopts the draft's structure and makes every red-team fix binding. Where the draft and this file disagree, this file wins.

**Consumer:** step 3, the loop on real tracks. Gates: kill tests 10 and 11 (`docs\learning.md` section 6).
- **Kill test 10:** a reach map built with no knowledge of his run contains his 428 route over `hardtrack`'s old build. The red-team's offline check already finds it, so this spec turns that check into a test.
- **Kill test 11:** plateaus per joint pattern stay few as bisection refines.

## Adopted from the draft
- The split between a **goal-free pattern record** (what a joint does from each arriving state) and a **goal-bound instance** (T-to-go and prices for one target).
- The four statuses per joint: **found**, **forbidden**, **searched without result**, **refused by the law**.
- Invalidation by footprint, through `ReadCertificates`.
- Reach between surfaces derived from mirrored reach tables, never from a search over inputs in acting.

## Binding fixes (red-team finding → rule)
1. **The key** (C1, H3, S182): the **exact entry template**, which is every state field's bits except the one free coordinate s: velocity, flags, gait phase, and the fractional bits of the perpendicular coordinate relative to the joint. Add the per-axis binade of position. s is stored relative to the joint's origin. Arrivals at rest have no line, so they are keyed exactly in both coordinates.
   - No speed bands, no comparison classes for velocity.
2. **Certificates** (C2): bind them to (JointPatternId, plateau index, law_hash), never to one state's bits. Acting checks three things: the key is equal, s lies in [lo, hi], and the certificate is not invalidated.
3. **Plateaus** (H2): split a plateau at any change of the law's discrete event signature (`docs\vocabulary.md:39-43`; a body-block change is not a cut). Within a binade, motion inside a plateau is then a pure translation. The footprint is the per-tick hull of its two endpoints' query boxes, with no sampling.
4. **T-to-go** (H1): each instance node has its own breakpoints: the pattern's breakpoints plus the preimages of the downstream breakpoints, found by bisection on "lands in downstream plateau k". The pattern stores the exit coordinate's bits at lo and at hi.
5. **Forbidden** (H4) needs an envelope proof that carries its premises and the cells of its separating component in `ReadCertificates`. Terrain off its premises (slabs, stairs, fluids) is **searched** or **refused**, never forbidden. An edit inside a forbidden footprint turns it into **searched**.
6. **No replay in acting** (H5): the backward wave reads certified edges only. An uncertified arc becomes a learning-queue item (`docs\learning.md` section 2), and the wave uses the arc's envelope lower bound as its bracket. Mirrored tables are keyed by the exact entry template. There is no per-cell dedup.
7. **Domain and inventory** (M2, M5): the takeoff domain is the set of s values the law reports as grounded, not geometry. The face inventory registers its reads, and an edit enqueues the joints it creates.
8. **IDs** (M3): `ClassId(u16)` for cell answers only; `StateClassId(u32)` and `JointPatternId(u32)` separately. The cold entry-template arena is budgeted apart from the hot ids.
9. **The law's numbers** (M1, L3): body dimensions come from the law (`(0.6f32/2.0f32) as f64`), never a literal. `law_hash` is an automatic build hash covering `sin.bin`, `models.tsv`, the registry and the law sources. It replaces `Binding::version` and the static `SCHEMA`.
10. **The goal** (L2): the target region is `Goal::touched`, with no epsilon. When `wt/track-face` merges, the clicked face is the target.
11. **Prices** (L4): until two witnessed T values exist, a price is the difference between the kit's light-cone lower bounds.
12. **The witness** (decision 7): per plateau, the s bits plus the end digest. Never re-derive a witness from a class.
13. **Frames** (decision 8): the identity frame for certifying. Turned frames only seed brackets.
14. **Stairs** (decision 9): undecided by hand. A stair run is a gait whose count the corridor query finds, only if the state recurs bit for bit per step. Measure that, and record it.

## Tests
- **Kill test 11:** plateau counts per joint pattern under refined bisection, on the patterns of the GLM `trial-offset-monotone` result and of `hardtrack`'s joints. Killed if a count grows as bisection refines.
- **Kill test 10:** the red-team's `kt10` check, rewritten as a test under `engine\plan\tests\`, building the reach relation over the old build's faces through the new record. It must contain the decoy-lane end → wall-top → final-ladder route. Report the best T after the ladder exit against his 169 (the red-team's offline best is 158).
- **Soundness:** the red-team's two-speeds witness (vx 0.24 and 0.36 from (−86.5, −28, −101.5), FS yaw 0) must produce two different keys.
- **One mechanism:** the same record and wave serve a straight run (one gait, no joints) and `hardtrack`. Kill test 8's planted shortcut still fails.

## Builder and order
**Astra**, after the core (step 1) lands, because this builds on the core's store and `solve`. Astra works on a branch off the core's branch, at normal priority, offline only.
