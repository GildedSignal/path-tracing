Implemented. Changes remain uncommitted.

### Changes

- Replaced the corner with an isolated two-block column. Missing witnesses fail. Predicate tests catch constant-true and constant-false mutants.
- Appended 20 zero-input ticks. Added `serverEnd.atRest`, signed offsets and snapshot counts.
- Added maximum-health/timer restoration, abort cleanup, and a settle pause for bit-identical resource checks.
- Recorded `speedModifiers` and `ticksFrozen` each tick. Replaced `movementSpeed` with `movementSpeedBase`.
- Made `collisionSteps` bits-only and placed the water rim first.
- Deleted the reader patch and its test.
- Added offline exporter, axis-order, resource and JSON checks. Updated [recorder.md](/D:/path-tracing-wt/fixtures/bot/recorder.md).

### Validation

Build passed. All 39 fixture cases passed across offline runs, alongside 30 existing gate cases and the course, snapshot, predicate, exporter and JSON checks. PowerShell 5.1 parsing, ASCII and diff checks passed.

The existing gate suite required explicit case selection because its default selector fails.

### Assumptions

- `Ticks` counts movement ticks, with 20 additional release ticks.
- Modifier operations use vanilla numeric IDs 0, 1 and 2.
- Offsets use the nearest signed modulo value. Negative offsets remain recorded for reader handling in job 1.5e.
- **Item 6 was applied literally.** Slime, hay and beds now export `inside=unmodeled` despite their modeled landing fields. The current reader consequently rejects those contacts.
- The column occupies y=-60 through -59.