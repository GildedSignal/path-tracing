# Hard track 2: the same course, with machine-checked reach so no shortcut exists

**Why:** Julien beat the witness (579) with 428 by a route the report said was impossible (S149–S152). Read `C:\Users\PASCAL\Desktop\path tracing\jobs\hard-track\shortcut.md` first: his route is decoy lane → off its east end → the alignment wall's top (y −32) → a jump to the final ladder. His recording is `jobs\hard-track\julien-428.jsonl` (per-tick `position`).

The defect is not this one hole. `jobs\hard-track\report.md:32` claims "the only combination of committed arms that completes" from argument. `reach_and_decoy_certificates` (`engine\plan\tests\hard_track.rs:151`) checks only the inside of each pocket. Plugging his hole alone would leave others. **No patches** (AGENTS.md): the fix is a reach certificate over the whole box.

**Read:** the previous job in `C:\Users\PASCAL\Desktop\path tracing\jobs\hard-track\` (`prompt.md`, `report.md`, `bounds.md`, `build.md`, `witness.md`), `engine\plan\tests\hard_track.rs`, `lab\tracks\hardtrack.build.txt`, `lab\tracks\hardtrack.witness.txt`, `lab\tracks\hardtrack.json`.

## Decisions already made (do not reopen them)

1. **Same box, same A and B:** x and z −110…−60, y −60…−20; A the gold block at (−107, −60, −107), B the diamond at (−84, −25, −88). `lab\tracks\hardtrack.json` does not change.
2. **Same mechanics and the same three forks** (initial ladder, blue-ice head-hitter long jump, slime-well rebound, suspended-ladder catch; decoys at each fork). You may move, resize or add blocks anywhere inside the box.
3. **The reach certificate is a new test, `whole_box_reach_certificate`,** in `engine\plan\tests\hard_track.rs`. It must hold for all controls under the law:
   - **Supports:** every top face a body can stand on, every climbable cell, and every slime face in the box.
   - **Edges:** an edge from support P to support Q exists unless a conservative bound excludes it. The bounds over-approximate reach, so "no edge" is a proof:
     - rise ≤ the law's jump apex (1.252203340253724 from `bounds.md`), plus climbing while on a climbable cell, plus a slime rebound no higher than the impact drop;
     - horizontal travel ≤ the box's maximum horizontal speed × the airtime for that rise or drop. The speed is the law's bound on blue ice with head-hitter jumps, derived in the test from the law's constants, not typed in.
   - **Use the existing box geometry and law.** Write no search over inputs; the graph has a few hundred nodes.
   - **It asserts** that every path from A's support to B's support in this graph passes through the intended elements in racing order, and that each decoy's supports have no edge back out.
4. **His route is a regression:** the test also asserts that no graph path uses the alignment wall's top (or whatever replaces it) as an intermediate support from the fork-1 decoy lane.
5. **The witness is rebuilt** for the new geometry: `hard_track_witness`, `timing_certificates` and `the_required_mechanics_cannot_be_removed_from_the_witness` pass on the new world, with the same scope as before (±1 tick, all later boundaries fixed). Report its T on the race clock. It need not beat 428: the old track's 428 was a shortcut.
6. **Offline only.** No game, server or client is launched; RCON is not touched. The integrator places the build in Julien's world after the judge. Write `lab\tracks\hardtrack.build.txt` as before: the first command clears the box, with the gamerule note.
7. **If no geometry passes the certificate** with these mechanics, that is a result: write which edge cannot be closed and why in `last.md`, and end `STATUS: DONE`. Never add a tolerance or a special case to make it pass.

## Must not touch

`engine\*\src\*` (the law and planner are read-only), `bot\*`, `lab\tracks\hardtrack.json`, `jobs\hard-track\julien-428.jsonl`, `jobs\hard-track\shortcut.md`.

## Done means

- `cargo test --offline -p plan --test hard_track` passes (the ignored planner diagnostic stays ignored), and `cargo test --offline --workspace` passes. Use `CARGO_TARGET_DIR=D:\path-tracing-target\hard-track-2`.
- **`jobs\hard-track-2\report.md`** holds:
  - the changed geometry and why each change closes an edge;
  - the support graph's size;
  - every edge the certificate excludes near the old shortcut, with its bound and margin;
  - the new witness T;
  - the replay commands.
- `lab\tracks\hardtrack.build.txt` and `lab\tracks\hardtrack.witness.txt` are updated.
- End with `STATUS: DONE` or `STATUS: BLOCKED: <why>`.
