Implemented and left uncommitted.

- Enforced a final, complete `end` trailer with a matching tick count. Other typed rows after the header fail.
- Added `serverTickOffset` support and the interim `ticksAfterEnd` alias. Resources compare only at explicit zero. Delayed, null, or missing timing prints a skip reason.
- Seeded `ticksFrozen`, tested the existing exporter flags, and added named failures for unmodeled inside and landing hooks.
- Ported frost accumulation, decay, support gating, float arithmetic, and modifier ordering.
- Added synthetic fixtures covering items 1–3 and frost regressions.

Validation passed with **49 tests passed, 1 replay ignored**, all four mutant builds successful, and `git diff --check` clean.

Live replay awaits fixtures. A separate contract mismatch remains for integration review. The exporter supplies effective `movementSpeed`, while the law treats it as the base before applying modifiers.