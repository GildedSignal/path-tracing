# kit-unfold-slice-2: last

Three local commits on `wt/kit-unfold-slice-2` (2A wire, 2B words, 2C tracks + map). Nothing pushed, nothing flown (server closed). Full detail: [report.md](report.md).

- **Wire:** `meta[7..]` → `Kit::Carried`; no edits → `WORD_PLAN` byte-identical; edits → `KIT_PLAN = 4` = `WORD_PLAN` + `[a, a × 10]` (tick, slot, opcode 1 Interact / 2 Place / 3 BreakHold / 4 BucketEmpty / 5 BucketPickup, x, y, z, face, aim yaw, aim pitch, hold) before the hull. Java performs each record with `useItemOn` → `useItem` / `startDestroyBlock` + `continueDestroyBlock`, look to the face and back, and holds a break's last tick up to 8 ticks. An echo is tolerated only at a cell an issued action edits (door half ± 1).
- **One landing rule** (`KitOp::lands_at`): a click before its step's first tick, a break before its last.
- **Kit words:** `AimRule::Click(KitTemplate)` (verb, `(fwd, up, right)`, restore flag, goal-offset yaw), promoted as one motif. Door 8,191 → 0 and breach 9,548 → 0 learn ticks at a site translated (+25, 0, +17) and turned 90°, with the same T (24, 38). `edit()` outside the footprint keeps the route; inside, it drops it.
- **Tracks:** `amertystcoral` 33 restored (his 63); `ametwoodbox` 45 restored (his 124, needs an axe); `mud` 47 unrestored (his 97; restored it stays `NeedsJoints`, the pillar base is out of reach). Fixes: vanilla placement faces (`place_faces`); the pillar candidate.
- **Gates:** native 9/9 · jar builds · test-native PASS · test-clock PASS · kit_unfold 6/6 · generalize 5/5 (identical) · hard_track 5/5 · real_time 13.75–15.6 / 13.65–15.85 / 5.2 µs (baseline 11.8–14.05 / 13.0–13.1 / 5.2), 0 refused; warm allocation 168 B, the same at the baseline · world 21/21 · law 95/95. test-tracks and test-resident-path fail on the baseline too.
- **Lines:** engine code +879, tests +495 (`engine-lines.sh`); bot +118 / −9.
- **Moved:** R-lego-2 54 → 60 (the pillar reorders its search), R-lego-3 48 → 50 (Phase A takes its first hit).
- **Gaps:** item drops (a restore needs the block in the hotbar), the client's 5-tick destroy delay, use-block place supports, no live flight.

STATUS: DONE
