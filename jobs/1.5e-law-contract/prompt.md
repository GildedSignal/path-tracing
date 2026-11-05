# Job 1.5e: the law reads server-driven state, and the end-of-run check gets coverage

On `wt/law` (`3f6d65c`). Make exactly these changes. Where something is ambiguous, take the most literal reading and write the assumption in your final message. Do not commit; the integrator commits. Cargo settings as in job 1.5b. A parallel job (1.6c, on `wt/fixtures`) changes the recorder to write the fields below. Their names are fixed here, so build the reader against them with synthetic fixtures.

1. **Server-driven state is input, not simulation.**
   - In the 26.2 jar, freezing runs only when the level is a `ServerLevel` (`LivingEntity` tick, the `freezing` section: `setTicksFrozen` decay, `removeFrost`, `tryAddFrost`). The client, whose movement the law reproduces, never computes it; it receives the frost speed modifier by attribute packet.
   - Remove the frost simulation that job 1.5d added. Keep its arithmetic only where the client also runs it (if any; say which).
   - Every tick row carries `speedModifiers`: every non-sprint modifier of `MOVEMENT_SPEED` as `[id, amount, operation]`, as in effect during that tick. It also carries `ticksFrozen`. The law applies these modifiers, plus its own sprint modifier, in vanilla's attribute order.
2. **The movement-speed contract.** The `player` line has `movementSpeedBase` (the attribute's base value) and `speedModifiers`, and no longer the effective `movementSpeed`. Seed the attribute from them, so sprint is never applied twice. If `movementSpeedBase` is missing, the fixture fails with the field named.
3. **The end-of-run check gets coverage.**
   - `serverEnd` carries `atRest` and a signed `serverTickOffset`.
   - When `atRest` is true and the offset is k ≥ 0, replay k more zero-input ticks after the last row, then compare health, food, saturation, exhaustion and `foodTickTimer` bit for bit.
   - Otherwise skip, and print why.
   - At the end, print how many fixtures were compared and how many skipped.
4. **The axis-tie mutant must be catchable.** Add a test: a `BoxWorld` with a single 2-high stone column at the outer corner the `corner` course uses (start at local (16.5, 4.5) on the floor, column at local (20, 8), yaw 0, forward + left held from rest). Assert that on the first contact tick |dx| = |dz| bit for bit, and that exactly one axis is clipped. Also assert that the axis-tie mutant build produces a different position on that tick. If it doesn't, stop and report: the planted fault would be uncatchable.

## Done means

- `cargo test` passes in `engine/` (replay ignored without fixtures).
- The four mutant builds pass, and the axis-tie mutant fails the new test from item 4.
- There is a synthetic fixture for each of items 1 to 3.
- The final message lists every change.
