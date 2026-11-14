# Judge verdicts, 14 November (Sonnet 5.5; read and ran only)

## 1. wt/1.4k-inside-cell-mutant: PASS
- Test `inside_cell_shape_is_keyed_in_recorded_footprint` passes here (green). Builder's mutant log (inside-cell-unkeyed.log) shows it FAILED with the key dropped.
- The synthetic cell is fair. The capped and uncapped scaffolding states differ in sneak too, so the test clones the capped model and swaps in only the uncapped inside_cell_shape. Shapes still come from the exported table. It asserts the four old contexts' coords are equal and the inside-cell coords differ. It then shows the tick really diverges (y bits differ) and the digests differ under the recorded mask. That discriminates the key.
- Mutant line drops exactly `inside_cell` (`let _ = inside_cell;`, list [shape, above, below, sneak]); the search string matches pattern.rs:398 and it runs the new test (`-p world --test blocks`). Nothing existing is edited: +28 lines, two files, additions only.
- Not re-run by me: verify.ps1 13 of 13 (builder's claim).
- Note: the report admits the "pair of unmodified exported cells" is unmet; the deviation is justified.

## 2. wt/3.2g-from-bits (19d88db): PASS WITH FINDINGS (for the part built)
- from_bits is an exact inverse of bits(), compared field by field: same order (t, input yaw/pitch, yaw, pitch, pose, hurt cooldown, food, timers, 34 bools, 10 f32, the two optionals with presence flag, 9 f64, three Vec3, movement_this_tick, supporting block, modifiers). i32 is sign-extended and checked on decode; f32 is checked <= u32::MAX; floats are raw bits, NaN payloads kept; enum tags are range-checked (Pose 0..2, SpeedOperation 0..2).
- Refusals are positional (position, reason) and never default: bad bool, f32, i32, u32, pose, operation, id length/byte/utf8, modifier count, missing word, extra word.
- `cargo test --offline -p law --test from_bits`: 2 passed. The tests cover a NaN-payload state with every optional field set, the default state (optionals unset), and one test per refusal kind.
- Findings (non-blocking, per instruction the witness part is a separate job):
  a. Spec step 2 (round trip over states from existing tests/fixtures) is not done; only two hand-built states are round-tripped. Equality is checked via bits, not field-for-field PartialEq.
  b. Refusal kinds "boolean" in the optional flags and "f32" for later floats are covered by one sample each; fine.
  c. Builder's report says the gate never completed (log dir denied), so 12 of 12 is unverified. state.rs lacks a trailing newline.

## 3. wt/3.3e-plan-track (6c871ec vs HEAD~1): PASS WITH FINDINGS
- It is a move. Diffing the old tests/flight.rs against src/flight.rs (whitespace-insensitive), the only changes are: pub visibility, `use crate::*`, write_script taking a path, and a `fly_with_budget(..., seconds, script)` wrapper with `fly` calling it with None, None. Budget `seconds.unwrap_or_else(old formula)`, PLAN_RECORD path logic unchanged when script is None. The harness test and probe stay in tests/flight.rs. Cargo.toml: world moved from dev-dependency to dependency (needed).
- Bits-file start is refused: any `--start` other than `settled` returns "NO SENTENCE ... only --start settled is supported", exit 3. It never silently uses the settled start.
- Wall time: Instant is taken at the top of run() (before arg parsing, track load and all tables) and read after fly returns, which writes the file. Effectively process start to file written; it excludes only runtime startup before main. Minor, acceptable.
- Verified: release build of plan-track OK; `cargo test --offline -p plan` passes (planner 4 passed; flight tests ignored); the ignored byte-identity test `binary_exports_the_flight_script` passes in release (1.14 s).
- Findings (non-blocking): the identity test compares two calls of the same function, so it checks determinism and the CLI path, not independence; it is #[ignore]d (spec-sanctioned release test). A found-sentence panics on a script write failure and is caught to "NO SENTENCE planner or export failed" (exit 3): a refusal, not a default. The unparsed `--cap` is a budget in seconds (spec said `<s>`), consistent. Footprint gate 12 of 12 was run by the builder with a redirected log dir; not re-run by me.
