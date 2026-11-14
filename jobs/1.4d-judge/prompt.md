# Judge job 1.4d: the boat enters the law

You are the **judge** (AGENTS.md: GPT's work is judged by Sonnet, in a fresh context, from the other family). You read; you do not edit anything. Your verdict decides whether branch `wt/boats` (worktree `D:\path-tracing-wt\boats`, commit `18d1cb9`) merges into `main`.

**The spec** is `C:\Users\PASCAL\Desktop\path tracing\jobs\1.4d-boats\prompt.md`. **The builder's report** is `jobs\1.4d-boats\last.md` in the same checkout. **The evidence** the port must match is in `C:\Users\PASCAL\Desktop\path tracing\lab\results\local-20261113-2043-1-1-port-boat-delta\` and `...\local-20261113-2043-1-0-port-boat-rider-attachment\`: read each `dossier.md` and `check.md` (the independent checker's grade, including where it weakened a falsifier). The decompiled jar text the dossiers cite is under `D:\path-tracing-lab\` if you need an offset.

**The whole diff (98 lines), so you need no file read to start:**

```rust
// engine/law/src/boat.rs (new)
//! Isolated occupied-boat law in established IN_WATER, without collision.
use crate::{math, Vec3};
// AbstractBoat.getPassengerAttachmentPoint 31-85; Entity.positionRider.
pub fn seat(position: Vec3, yaw: f32, ride_height: f64, offset: f32) -> Vec3 {
    let angle = (-yaw * 0.017453292f32) as f64;
    let sin = math::sin(angle) as f64;
    let cos = math::cos(angle) as f64;
    position.plus(Vec3::new(0.0 * cos + (offset as f64) * sin, ride_height, (offset as f64) * cos - 0.0 * sin))
        .minus(Vec3::new(0.0, 0.6, 0.0))
}
// Boat.rideHeight 0-8, Raft.rideHeight; AbstractBoat passenger slot 31-59.
pub fn boat_ride_height(height: f32) -> f64 { (height / 3.0f32) as f64 }
pub fn raft_ride_height(height: f32) -> f64 { (height * 0.8888889f32) as f64 }
pub fn slot_offset(first: bool, animal: bool) -> f32 {
    let mut offset = if first { 0.2f32 } else { -0.6f32 };
    if animal { offset += 0.2f32; }
    offset
}
pub struct Boat { pub position: Vec3, pub velocity: Vec3, pub yaw: f32, pub delta_rotation: f32 }
pub struct Controls { pub left: bool, pub right: bool, pub forward: bool, pub backward: bool }
// AbstractBoat.tick 150,164,191-199; floatBoat 138-167,270-359;
// getDefaultGravity 0-3; controlBoat 0-152. Water level is supplied by the caller.
pub fn tick(boat: &mut Boat, water_level: f64, input: Controls) {
    let draft = (water_level - boat.position.y) / (0.5625f32 as f64);
    let drag = 0.9f32;
    boat.velocity.x *= drag as f64;
    boat.velocity.y += -0.04f64;
    boat.velocity.z *= drag as f64;
    boat.delta_rotation *= drag;
    if draft > 0.0 { boat.velocity.y = (boat.velocity.y + draft * (0.04f64 / 0.65f64)) * 0.75f64; }
    if input.left { boat.delta_rotation -= 1.0f32; }
    if input.right { boat.delta_rotation += 1.0f32; }
    let mut thrust = 0.0f32;
    if input.left != input.right && !input.forward && !input.backward { thrust += 0.005f32; }
    boat.yaw += boat.delta_rotation;
    if input.forward { thrust += 0.04f32; }
    if input.backward { thrust -= 0.005f32; }
    boat.velocity.x += (math::sin((-boat.yaw * 0.017453292f32) as f64) * thrust) as f64;
    boat.velocity.z += (math::cos((boat.yaw * 0.017453292f32) as f64) * thrust) as f64;
    boat.position = boat.position.plus(boat.velocity);
}
```

Tests (`engine/law/tests/boat.rs`, new): quarter-turn seats bit-exact at yaw 90 and 270; the half-turn table residual `1.8369703518723924e-17`; ride heights 0.1875 and 0.5; the slot float add `-0.40000003576278687`; unwrapped yaw 450; one-tick water witnesses; a 300-tick straight run to `0.39999989569188904`; an 800-tick turn whose speed is compared to an analytic fixed point by **asserting two different bit patterns** (the port's and the analytic one), not by checking they agree.

**What to decide, each with a line of evidence:**
1. Does every constant and every operation order trace to the cited offset — float versus double arithmetic at each step (Java's `float * float` stays float; `Mth.sin` takes what type in 26.2?), the order drag → gravity → buoyancy → yaw → thrust → move, and the `getDefaultGravity` value? A constant whose source you cannot find is a blocking finding.
2. The builder says the five falsifier tests were written **after** the port, so their red runs are not on record. Do these tests discriminate — would each fail on the wrong port the spec names for it? Say which do and which are tautological.
3. The turn test records the port's own Mth-table result beside a continuous-trig value that differs in its last bits. Is that a check or a record? Is `0.39999989569188904` against the dossier's `0.39999990` an agreement at the dossier's printed precision, or evidence of a different computation?
4. Consumer and scope: nothing in Phase 2 may depend on this (the spec says so). Confirm the diff touches nothing the first flight runs.

**Verdict:** PASS, PASS WITH FINDINGS, or FAIL. A FAIL names each blocking finding with file and line and the smallest fix. Do not invent blocking findings: a judge that fails everything is not a judge. Keep it under 60 lines.
