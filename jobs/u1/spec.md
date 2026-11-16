# U1 spec, the integrator's (S175)

**Who wrote what.** Specs are the integrator's job (AGENTS.md: "The integrator orchestrates… It writes specs and prompts"). Hydrogen was asked for a premise audit and a draft (`jobs\u1-spec\report.md`). It should not have been the spec, and the integrator should not have launched on it unreviewed (Julien, S175: "why would hydrogen write the specs for astra?"). This file is the spec. It adopts the draft's section 2 where it agrees with the approved plan, and overrides it where it does not. Where this file and the draft disagree, this file wins.

## Adopted from the draft (section 2 of `jobs\u1-spec\report.md`)
- **Files:** the layout (`store.rs`, `events.rs`, `lattice.rs`, `source.rs`, `solve.rs` in `engine\plan\src`; `count` on `words::program::Arguments`).
- **The store:** `GeneratorStore` keyed by `LabelId`, with the inner key `(MotorKeyId, program, yaw_index, pitch_bits)` and its exclusions (no `t`, no x/z, no numeric food values, no `fed`). `MotorKeyId(u32)` stays separate from the codec's 16-bit class table.
- **The binade map:** P2's kernel, with events at binade, sign and subnormal crossings by integer division.
- **The two-yaw lattice:** over the law's 65,536-entry sine table, by Euclidean quotient and floor-sum, with no enumeration of q and no sweep over yaws.
- **`CorridorSource`:** with `GeneratorSuperflatSource`, one presence query per tube, never a per-block scan.
- **`solve`:** its six recorded stages, `Bracket → ClosedCheck → Extend → Cut → Compose → Allocate`. In U1, `Cut` returns `NeedsJoints` cleanly, because joints and sets are U2.
- **The tests:** kill test 17 (axis-aligned, oblique and shifted starts), kill test 8 with the planted fast path that must fail, the 100,000-key `MotorKeyId` test, and the gate commands and must-not-touch list.

## Overridden (the integrator's corrections, from the plan)
1. **The biggest word the kit holds, never a refusal** (`docs\learning.md` section 0; plan section 2, step 3).
   - With observed `food_level <= 6`, sprint is not allowed by the law's own guard (`movement.rs:219,225`). `solve` then extends the next biggest word the kit holds for this displacement: the walk gait, prepared in the store like any other word. It never returns `Unknown`.
   - U1 prepares at least sprint-jump and walk on the superflat label. The food test asserts a walk plan at food 6 and a sprint-jump plan at food 7, through the same stages.
2. **A missing kernel is learning, not a refusal** (plan section 2, "Learning is the same loop reaching a joint that has no set"). A bearing whose two yaws are not yet prepared is prepared by the same function, and recorded as a `Learn` entry in the stage trace, with its ticks and its time counted as thinking. The gate reports the million line **cold** (preparation included) and **warm** (prepared), each as a distribution. Neither may refuse.
3. **No made-up bounds in the tests.** The draft's `block_reads <= 8` is replaced by the property the plan states: thinking mirrors terrain. On the clean superflat, the counts of world reads, corridor queries and `try_tick` calls are **identical** for 10³, 10⁴ and 10⁶ blocks, and `try_tick` is 0 when warm. With k planted scars in the tube, the work grows with k, not with length (kill test 9's shape).
4. **The goal's touch rule is one function.** The inverse stop is written against `Goal::touched`, so the coming face rule (`wt/track-face`) changes one place. Do not re-derive the AABB test inside `events.rs`.
5. **Normal priority** (S158), not the draft's Idle.

## What stays open for U2 and beyond
Joints, sets and plateaus (U2, after the Opus red-team); labels and the far map (U6); the kit's other terms (U5, U8). U1 must not stub any of them as a second code path. `NeedsJoints` is the only hand-off.
