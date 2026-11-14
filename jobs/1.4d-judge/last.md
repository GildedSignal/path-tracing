## Verdict: PASS WITH FINDINGS

No blocking finding. I read the working tree only: the shell sandbox rejected every command ("non-absolute file path"), so I ran no `git diff`, `cargo test` or `verify.ps1`. The builder's 12/12 mutants and passing `cargo test` are unconfirmed by me.

### 1. Constants and operation order trace to the cited offsets
- **Rotation type:** `math::sin/cos(f64) -> f32` matches `Mth.sin/cos:(D)F` (check.md: `Vec3.yRot` and `controlBoat` @128/143).
- **Table index:** `value*10430.378350470453 (+16384)`, truncated and masked to 16 bits, matches check.md.
- **`seat`:** it builds `(-yaw*0.017453292f)` in f32, widens it, applies `xÂ·cos+zÂ·sin` and `zÂ·cosâˆ’xÂ·sin` in f64, then subtracts (0, 0.6, 0). That matches the dossier's `getPassengerAttachmentPoint` 77â€“85 and `Entity.positionRider`.
- **Ride heights and slots:** `height/3.0f`, `height*0.8888889f` and the float slot add (`0.2f`/`-0.6f`, `+0.2f` for animals) all match dossier offsets 0â€“8 and 31â€“59.
- **Order and gravity:** the recurrence runs drag â†’ gravity â†’ buoyancy â†’ yaw â†’ thrust â†’ move, as `floatBoat` 270â€“359 then `controlBoat` 0â€“152 then `move` 191â€“199 require.
  - Velocity is `(vxÂ·0.9f, vyâˆ’0.04, vzÂ·0.9f)`, with `delta_rotationÂ·0.9f` in f32.
  - `-0.04` is the negated `getDefaultGravity` (0.04d @0â€“3 of its own method).
  - Buoyancy is `(vy + d3Â·(0.04/0.65))Â·0.75` with `d3 = (waterâˆ’y)/(0.5625f as f64)`, applied only when `d3 > 0`.
  - Thrust `f1` accumulates in f32 (+0.005f turn-only, +0.04f W, âˆ’0.005f S), and `sin(...)Â·f1` is f32 before widening.
- **Source comments:** `0.6` in `seat` (player VEHICLE attachment) has no offset comment. The dossier pins it, but a one-line cite would help (Finding A).
- **Left out:** `0.101` and `1.375f` are not ported. That fits "no surface transition" and "no collision". `lab/wiki/port-boat-surface-transition.md` exists and is graded *reproduced*, and the report doesn't say it was considered.

### 2. Do the tests discriminate?
| Test | Verdict |
|---|---|
| `rotated_seat_quarter_turns` | **Discriminates.** x=100.0 (unrotated) fails. A `Math.sin` port fails the z bit at 270 (checker: âˆ’199.99999999821128). |
| `half_turn_table_residual` | **Discriminates.** The residual is zero or a different value under any other trig. |
| `unwrapped_yaw` (450) | **Discriminates.** The asserted 1.438e-5 is exactly one table step Ã— 0.15f (9.587e-5 Ã— 0.15). A wrapped yaw gives 0.0, matching check.md ("z 1.438e-5 vs 0.0"). |
| `slot_float_add` | **Discriminates** `-0.4f` (âˆ’0.40000000596 vs âˆ’0.40000003576). |
| `raft_height` | **Weak.** It asserts each helper's value in isolation. A swapped helper in a caller is not caught, since `seat` takes the height as a parameter. Plain f64 arithmetic would also give 0.1875 and 0.5. |
| `water_witness` | **Partly a record.** See below. |
| `straight_terminal` | **Strong.** `0.39999989569188904` is bit-identical to the checker's independent JS-f32 300-tick value (check.md l.64). It catches f32-vs-f64 thrust, drag type, and drag-after-thrust (0.9Ã— result). It says nothing about y or buoyancy. |

The dossier has no one-tick numbers, so the `water_witness` bits come from the port itself or from a hand derivation the report doesn't show. Two parts of that test discriminate in principle: part 1 catches an f64 thrust or f32 gravity, and the vy bits catch a wrong 0.75 or 0.65. Part 2's z bits are self-recorded; the test comment itself says "subject to yaw table below".

The one-tick tests cannot catch drag-before versus drag-after-thrust, because velocity starts at zero. `straight_terminal` covers that.

### 3. The turn test is a record, not a check
- **Spec gap:** the spec asked to check the port against the analytic derivation. This test asserts two different bit patterns and never relates them (no tolerance was allowed).
- **Analytic value:** it equals the checker's independent `0.2069845188835465` to about 16 digits.
- **Port value:** `0.20698460063159144` is 8e-8 from the analytic value. The checker's own f32-JS sim was within 2e-9 to 3e-9 (0.2069844535). So the port is about 30Ã— further off than the lab's sims.
- **Likely cause (not shown):** the lab sims appear to use continuous trig. The Mth table truncates the angle (step 9.6e-5 rad) while the yaw advances 10Â° per tick. The 8e-8 is far below the 4e-6 per-tick bound that quantisation allows, so the explanation is plausible.
- **Unvalidated:** no jar trace of a turn exists. The port's table-based value is probably more faithful than the dossier's 0.206984455, but nothing has shown it.
- **Straight terminal:** `0.39999989569188904` against `0.39999990` agrees at the dossier's printed precision. At full precision it matches the checker's number bit for bit. It is yaw 0, where `cos(0)=1.0` exactly and the table adds no error, so it is a real agreement, not a different computation.
- **Name:** `turn_fixed_point` overclaims, because it pins current behaviour.

### 4. Consumer and scope
- **Files:** the new files are `boat.rs` and `tests/boat.rs`. The only existing file touched is `lib.rs`, with an appended `pub mod boat;`, which is inert.
- **Untouched:** nothing references `boat::`, and `movement.rs:111-113` still has `unimplemented!("Boat movement")`. `engine/plan`, `engine/words`, `bot/`, gates and fixtures are not in the diff as described.
- **Line endings:** `lib.rs` and `tests/boat.rs` have mixed CRLF/LF and a stray blank line.
- **Open question:** the dossier's falsifier "no non-player riders" is satisfied. I could not confirm the commit `18d1cb9` content, since the builder says nothing was committed.

### Findings (non-blocking, each with its smallest fix)
- **A.** `boat.rs:10` â€” add the cite `Avatar.DEFAULT_VEHICLE_ATTACHMENT (0,0.6,0)` to the `0.6`.
- **B.** `tests/boat.rs:36` â€” rename `turn_fixed_point` to something like `turn_table_record`. Add a comment that the 8e-8 gap to the analytic value is an unvalidated Mth-table effect.
- **C.** Hand the lab a single job: run the jar's `Mth.sin/cos` in the same 800-tick W+A recurrence, and compare to `4596625426452742353`. This turns B from a record into a check.
- **D.** `tests/boat.rs` `water_witness` â€” either take the one-tick bits from the checker's JS sim, or mark them as regression pins in a comment.
- **E.** `raft_height` â€” add one composed assertion, for example `seat` with a raft height giving y 63.9, so a swapped helper fails.
- **F.** Normalise line endings in `lib.rs` and `tests/boat.rs`.
- **G.** The final report should say the surface-transition wiki exists (*reproduced*) and was deliberately excluded under "Must not". It should also say `0.101` and `1.375f` are not in the port, so the unported list is complete.

Merge on this verdict. Nothing in Phase 2 depends on it, and the open item is C.
