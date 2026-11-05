Implemented, uncommitted. No game or live rig commands were run.

Changes:

- Aborts no longer trigger `rec stop`; cleanup steps fail independently. Added silent-abort and continuation tests.
- Rimmed and moved water away from the start; added flow tracing and a missing-rim negative control.
- Replaced resource queries with one entity snapshot, including `foodTickTimer`; added game-time offset evidence.
- Captured `ticksFrozen`, waited for thaw before each run, and documented the changing powder-snow speed modifier.
- Added deterministic `corner` inputs, collision-request observations and a tie-witness report.
- Documented the step-up mutant’s equivalence; built nothing for it.
- Widened edge islands, added return stairs and enabled course-specific 90% sneak selection.
- Corrected beds, marked unknown hooks and dripstone unmodeled, and exported suffocation/portal flags.
- Restored game mode, discarded readiness exports and updated documentation.

Validation passed: bot build, 34 driver cases, course/flow checks, snapshot tests, parser smoke test, reader tests and comparison controls. All 11 changed scripts parse under Windows PowerShell 5.1 and are ASCII.

Assumptions and limits:

- Existing rig requirements remain: replaceable lab patches, op bot, feedback, loaded chunks, supported player profile and no interference.
- Resources/equipment/effects remain unchanged by the harness. Initial server resources must remain constant until input zero; exported terrain must remain static during recording.
- Client game time supplies alignment evidence, not packet-level proof. Straddling reads get a null offset; only explicit zero is compared. Missing vanilla `TicksFrozen` means zero.
- Short corner runs may lack a witness; absence is reported. A witness does not establish a mutant kill.
- Dripstone remains unmodeled; powder-snow speed evolution belongs in the law. Flow tracing covers the current course domain; bubble columns remain omitted.
- Step-up equivalence follows the supplied judge finding.
- The reader exists only on `wt/law`, so its changes are delivered as a tested [integration patch](D:/path-tracing-wt/fixtures/jobs/1.6-l0-fixtures/law-reader.patch), requiring integrator application.

Full handoff: [last.md](D:/path-tracing-wt/fixtures/jobs/1.6-l0-fixtures/last.md).