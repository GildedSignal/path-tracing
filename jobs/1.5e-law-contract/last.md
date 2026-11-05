Implemented in `wt/law`; left uncommitted.

- Removed server-only frost decay and speed-modifier generation. Retained the client’s deferred `FREEZE` increment, capped at 140.
- Added movement-speed base and modifier types, vanilla operation/hash iteration order, clamping, and one law-owned sprint modifier.
- Updated replay to require `movementSpeedBase`, read modifier tuples, and inject each row’s `speedModifiers` and `ticksFrozen`.
- Added resting end-offset replay, exact comparison of all five resource fields, skip reasons, and compared/skipped totals.
- Added synthetic coverage for items 1–3, one-bit resource rejection, the corner mutant test, and a regeneratable jar attribute corpus.
- Updated engine and fixture documentation.

Validation: **56 tests passed**, live replay ignored without recordings, all four mutant builds compiled, and the axis-tie mutant failed the new test at tick 22. `git diff --check` passed.

Assumptions: row state is injected before each tick; modifier lists seed fresh vanilla maps in listed insertion order, since prior map history is unavailable. Missing cached `speed` derives from the initial attribute. Idle replay retains facing and the last supplied attributes.