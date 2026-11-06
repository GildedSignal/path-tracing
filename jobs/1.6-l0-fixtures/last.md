# Job 1.6b handoff

Changes are uncommitted. No game or live rig command was run.

## Changes

- Abort replies clear the driver's active-recorder state. No rec stop follows an abort.
  Cancellation, each artifact copy, readiness deletion, publication rollback, game-mode restore,
  process cleanup and lock release have independent catches. Later runs/courses are attempted.
- Stub aborts close their output, retain an incomplete .tmp and send no later recorder reply.
  Tests cover an abort in the middle run, later runs and another course, and cancellation failure.
- Water sources move farther from the start; a full stone rim separates pool/lip from the plain
  strip. Course tests trace water downward and horizontally with vanilla range/slope rules over
  these full-cube courses, reject fluid within three blocks, and catch a missing-rim mutant.
- server-state.ps1 reads all entity data in one data get entity, including health, food,
  saturation, exhaustion, foodTickTimer, game mode and TicksFrozen. Its SNBT splitter respects
  nested arrays/compounds and quoted strings. Game-time queries bracket the atomic entity read.
- Both player and serverEnd resource fields use the server snapshot, including foodTickTimer.
  Tick rows capture gameTime. serverEnd has serverTickOffset and Min/Max bounds relative to the
  final row; a bracket spanning ticks has a null offset. Absolute gameTime is excluded from
  deterministic recording comparisons, as tickCount already was; movement fields remain checked.
- PlayerSnapshot captures getTicksFrozen(). Every run moves to the dry start and waits for zero
  server freezing; client replay settling also requires zero freezing. recorder.md states the
  changing -0.05 * percentFrozen MOVEMENT_SPEED modifier and the law's obligation to model it.
- corner is appended to the course list, preserving existing course seed indices. Its separate
  deterministic input script holds forward+left at yaw zero without jump/sprint. A read-only
  Entity.collide RETURN mixin records requested/result vectors. The report checks nonzero raw-bit
  magnitude equality and horizontal clipping in one step, and reports presence or absence.
- The report states that no vanilla geometry produces 0.6f + 1 ulp as the step candidate, so
  that mutant is equivalent on the game domain and must be replaced by the integrator at freeze.
  No geometry was built for it.
- Edges now has wider islands, connecting strips and an unobstructed return staircase from the
  catch floor. Only this course passes SneakPercent=90; new-inputs defaults remain unchanged.
- Exporter: bed=true and fallOnMultiplier=1; unmodeled pointed-dripstone fallOn; inherited/declared
  unknown entityInside/fallOn overrides become inside=unmodeled. suffocating is a live state query;
  EndPortalBlock/EndGatewayBlock supply endPortalOrGateway. recorder.md lists all derivations.
- The original game mode is restored on exit, including failure paths. Full-course readiness
  exports are deleted and never copied into results.
- law-reader.patch supplies the separate wt/law reader's zero-offset-only resource comparison,
  skip explanation, optional server food-timer comparison, end-trailer validation and tests.
  test-law-reader.ps1 applies it to an isolated archive and tests it without modifying wt/law.

## Integration

After merging the law, the integrator applies:

```powershell
git apply jobs/1.6-l0-fixtures/law-reader.patch
```

The patch also updates lab/fixtures/README.md on that branch. The law branch/worktree itself was
not changed. There are no live fixtures; the live replay test remains explicitly ignored.

## Assumptions and limits

- Existing job 1.6 rig assumptions remain: intended lab superflat, reserved patches replaceable,
  op bot, command feedback, loaded chunks, supported player profile and no interfering entities.
  Resources/equipment/effects are not reset. Post-run exports assume static relevant terrain.
- Start resources come from the atomic pre-replay read and are assumed unchanged until input
  zero; initial health/food/saturation disagreement still rejects the run. This does not prove
  start tick alignment, especially for the server-only exhaustion/timer fields.
- Client-level gameTime is the server-synchronized client clock, not packet-level alignment
  proof. The offset uses vanilla time-query modulo Integer.MAX_VALUE. A straddling read is
  represented as null with bounds rather than an invented exact tick. Missing/nonzero/null
  offsets are skipped by the supplied reader patch; only explicit zero is compared.
- Vanilla omits TicksFrozen at zero; missing TicksFrozen in entity NBT therefore means zero.
- Corner means yaw-zero two-key diagonal motion from settled rest into symmetric stone walls.
  A short run may not reach the corner. Absence is reported, not silently converted to coverage;
  a witness is not a claim that the axis-order mutant was killed. Live evidence belongs to L0.
- Pointed dripstone uses the explicitly allowed unmodeled option. Its distance+2.5, multiplier-2
  landing hook is documented, not implemented in the exporter or law.
- Powder-snow speed evolution belongs in the law. Exporting initial freezing does not implement it.
- Water tracing is for the current courses' air/water/full-cube domain; bubble columns remain
  omitted because the schema/law lacks their acceleration hook. No live fluid settling was tested.
- The reader is absent from this branch; its changes are delivered as a tested integration patch
  to respect the original instruction not to modify wt/law. The integrator must apply it.
- The step-up mutant equivalence is the judge's supplied finding; no substitute mutant is chosen.

## Offline validation (regenerate with these committed scripts)

```powershell
.\bot\build.ps1
.\lab\gates\tests\test-record-fixtures.ps1
.\lab\gates\tests\test-courses.ps1
.\lab\gates\tests\test-server-state.ps1
.\lab\gates\tests\test-fixture-json.ps1
.\lab\gates\tests\test-law-reader.ps1
.\lab\gates\tests\test-run-gate-1.2.ps1 -Case pass,different
```

Build, course/flow checks, snapshot cases, parser smoke check and isolated reader tests pass.
Driver cases cover continuation, thawing, timing brackets, corner witness presence/absence,
restoration, readiness disposal and the original job 1.6 failures/defaults. Gate 1.2's positive
and differing-movement controls still pass with deliberately different absolute gameTime values.
Changed scripts parse under Windows PowerShell 5.1 and are ASCII. No live L0 claim is made.
