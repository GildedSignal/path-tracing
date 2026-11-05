# Job 1.6c: the second judge's findings on the fixture recorder

You built job 1.6b on `wt/fixtures` (`684946e`). A judge from another model family found it **ready with fixes**: `jobs/1.6b-fixture-fixes/judge.md` in the main repository (`C:\Users\PASCAL\Desktop\path tracing`), findings numbered there. Make exactly these changes. Where something is ambiguous, take the most literal reading and write the assumption in your final message. Do not commit; the integrator commits. You don't run the game. A parallel job (1.5e, on `wt/law`) changes the reader to match the field names below, so use them exactly.

1. **The corner course must tell the two axis orders apart** (findings 1, 6, 7).
   - Replace the inside corner with an outer corner: a single stone column, 2 blocks high, at local (20, 8), with nothing adjoining it. The diagonal from the start is unchanged.
   - The witness requires that on the tie tick exactly one of x and z is clipped. An inside corner, which clips both, fails it.
   - A `corner` run without a witness makes the course FAIL with a "no witness" note. Never count it as a pass.
   - Unit-test the predicate with rows for unequal magnitudes, sign bits, equal request and result, both axes clipped, and one axis clipped. An always-true predicate and an always-false predicate must each fail a test.
2. **End-of-run server reads become comparable** (finding 2). `/tick freeze` doesn't freeze players, so don't use it.
   - Every run's input sequence ends with 20 zero-input ticks.
   - `serverEnd` gains `atRest`: true when the last recorded row is on the ground with zero horizontal velocity.
   - `serverTickOffset` stays as you defined it, but is signed: a negative offset is negative, not wrapped modulo 2^31.
   - The report states how many `serverEnd` snapshots have `atRest` true and a non-null offset. The reader replays the offset as zero-input ticks only for those.
3. **A stable start state** (finding 3).
   - Before each run, the bot is at maximum health and its `foodTickTimer` is 0. If not, heal it before the run (`effect give <bot> instant_health`) and wait until both hold. Cleanup after an abort does the same.
   - After the settle, read the server snapshot a second time and require every resource field to be bit-identical to the first read, including exhaustion and the timer. Otherwise reject the run.
4. **Server-driven state is recorded, not assumed** (finding 4). In the 26.2 jar, freezing (`setTicksFrozen` decay, `removeFrost`, `tryAddFrost`) runs only when the level is a `ServerLevel`; the client receives the speed modifier by attribute packet and the thaw by entity data.
   - Every tick row gains `speedModifiers`: every modifier of the client player's `MOVEMENT_SPEED` attribute except the sprinting one, as `[id, amount, operation]`, read at the end of that tick. It also gains `ticksFrozen`, the client's value at the end of that tick.
   - The `player` line exports `movementSpeedBase`, the attribute's base value, instead of the effective `movementSpeed`, which is removed. Its modifiers go in the same `speedModifiers` form.
   - In `bot/recorder.md`, replace "the law must model it" with this: server-driven state is an input the law reads from the recording.
5. **Delete** `jobs/1.6-l0-fixtures/law-reader.patch` and `lab/gates/tests/test-law-reader.ps1` (finding 5).
6. **An exporter check** (finding 7): an offline check that lists, for every block class the courses use, which come out as `unmodeled`, and fails if one that overrides `entityInside` or `fallOn` comes out as `none`.
7. **Minor** (finding 8).
   - `collisionSteps` stores bits only.
   - In the water course, place the rim before the source.

## Done means

- `bot\build.ps1` builds. Every script you touch parses under Windows PowerShell 5.1. ASCII only.
- The offline tests pass, including the new ones from items 1, 2, 3 and 6.
- `bot/recorder.md` documents every new or renamed field.
- The final message lists every change and every assumption.
