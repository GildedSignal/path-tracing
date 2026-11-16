Continue the same job: the judge's verdict is PASS WITH FINDINGS. Read `C:\Users\PASCAL\Desktop\path tracing\jobs\3.2d-3-P1-judge\last.md` in full, then fix all three findings in `engine\plan\tests\zones_prereqs\t1.rs`. The integrator's decisions on each:

1. **The zero-hot-ticks check must be able to fail.** Route everything the hot path does through the counting wrapper, and measure it inside the timer: the floor-label lookup at click time, the memo lookup and the application. If a world read is needed to compute the label, count it as a hot read and report how many there are. The judge's surviving mutation (a hot `law::try_tick` through the plain world) must now fail the test. Show that it does, then revert it.
2. **y is translated like x and z.**
   - Set `a.position.y = s.position.y + (states[first].position.y - start.position.y)`, under the same binade condition as x and z.
   - Add starts at a different floor height with the same floor `LabelId`, inside the same y binade. If the fixture tracks have no such floor, build a minimal one in an in-memory world, as `engine\plan\tests\hard_track.rs` does.
   - Assert bit-exactness against the fresh reference replay. If y is not exact, that is the result: record it and do not loosen anything.
3. **Real independent starts:**
   - At least nine starts that are not the preparation start. Vary t, x, z, y and the resource bars independently: no two starts change the same set of variables together.
   - Add a negative control that changes the floor `LabelId` (a different floor block under the same motor state), alongside the existing `on_ground` control.

Rerun the gate in the spec's section 2 and the judge's mutations. Update `jobs\3.2d-3-P1\last.md` with the new hot-path timing distribution, now including the label lookup, and each finding's resolution. You are the Terra builder: never run `launch.ps1`, `gpt.ps1` or `follow-up.ps1`. End with `STATUS: DONE` or `STATUS: BLOCKED: <why>`.
