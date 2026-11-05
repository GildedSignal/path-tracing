# Job 1.6b: the judge's findings on the fixture recorder

You built job 1.6 on `wt/fixtures` (`b3d3297`). A judge from another model family found it not ready to record. Make exactly these changes. Where something is ambiguous, take the most literal reading and write the assumption in your final message. Stop only for something that blocks the build. Do not commit; the integrator commits. You don't run the game. Context as in job 1.6, plus `git show wt/law:lab/fixtures/README.md` for the format.

1. **A failed replay must not end the session.** `record-fixtures.ps1` (about lines 238 to 241): after the bot reports an abort, don't send `rec stop` (the recorder has already closed the output and replies nothing). Wrap every cleanup step in its own try/catch, so one failure is reported and the next run and the next course still proceed. Make the test stub behave like the real client: silent after an abort. Add a test where one run aborts mid-course and the following runs and courses complete.
2. **Water.** No water may reach the start tile or the plain strip. Rim the pool and the source row with stone on the side facing the start, or use a stone-rimmed basin. `test-courses.ps1` rejects any course whose fluids can reach within 3 blocks of its start, by tracing vanilla flow from each source over the course's blocks.
3. **The server's view, read in one tick.**
   - Replace the seven RCON calls with one `data get entity <bot>`, which the server answers within a single tick. Take health, food, saturation, exhaustion and `foodTickTimer` from it, for both the start (`player` line) and the end (`serverEnd`).
   - The end is read after the client stops, so it is not tick-aligned with the recording's last tick. Mark `serverEnd` with the number of server ticks between the last recorded tick and the read (from the game time). The reader compares it only when that number is 0, and says so otherwise.
4. **Powder snow.**
   - Capture `getTicksFrozen()` in the `player` line.
   - Before each run, wait until the bot's `ticksFrozen` is 0, moving it off the snow if needed.
   - In `bot/recorder.md`, state that `tryAddFrost` adds a `MOVEMENT_SPEED` modifier of −0.05 × percentFrozen while `ticksFrozen > 0`, so speed changes during a powder-snow recording. The law must model it, and the export doesn't fix it.
5. **The two hard planted faults.**
   - **Axis tie:** add a scripted course, `corner`, with deterministic inputs (not `new-inputs.ps1`). From rest, the bot moves diagonally at a yaw where |dx| = |dz| bit for bit, into an inside corner, so the collision step meets the tie. The script checks in the recording that a tick with |dx| = |dz| and a collision exists, and reports whether it does.
   - **Step-up at 0.6f + 1 ulp:** don't build anything. Say in the report that no vanilla geometry produces that candidate, so the mutant is equivalent on the game's domain. The integrator will have it replaced at the gate's freeze.
6. **Edges.** Make the edge course useful for the sneak back-off: wider islands, a stair or ladder back up from the catch floor, and inputs on this course that hold sneak most of the time (a course-specific setting of `new-inputs.ps1`, not a change to its defaults).
7. **Exporter correctness** (`WorldExporter.java`), even for blocks no course uses yet:
   - beds: `bed: true` and `fallOnMultiplier` 1, as the README says (the halving is in the law);
   - pointed dripstone: its `fallOn` (distance + 2.5, multiplier 2), or `unmodeled`;
   - any block whose class overrides `entityInside` or `fallOn` and isn't modelled gets `inside: "unmodeled"` (or the named hook), never `"none"`;
   - write `suffocating` and `endPortalOrGateway`.
8. **Minor.** Restore the bot's game mode at the end. Don't keep the full-course readiness export in the results.

## Done means

- `bot\build.ps1` builds. Every script you touch parses under Windows PowerShell 5.1. ASCII only.
- The offline tests pass, including the new ones from items 1 and 2.
- The final message lists every change and every assumption.
