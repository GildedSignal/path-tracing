# Job 1.5c: seed the sprint latch from the state before tick zero

On `wt/law` (`45a8d0b`), job 1.5b seeds the sprint latch from the recording's tick-zero `isSprinting`, as the 1.5b spec said. The spec was wrong: recorder rows are post-tick, so tick zero's `isSprinting` is the state after the first tick, not before it.

Make exactly this change. Do not commit; the integrator commits. Cargo settings as in job 1.5b.

1. The replay harness seeds the latch, and every other piece of hidden state, only from the fixture's `player` line (`lab/fixtures/README.md`: "Exactly one `player` line supplies the state before recording tick zero"). Its `isSprinting` field is the latch's initial value.
2. If a fixture's `player` line lacks `isSprinting`, the replay fails for that fixture with a message naming the missing field. Never fall back to tick zero.
3. Update `lab/fixtures/README.md` to say that `isSprinting` is required in the `player` line, and why.
4. Add a test: a synthetic fixture whose `player` line has `isSprinting: true` while its tick-zero row has the key released. The latch must stay on, as the jar would.

Done means `cargo test` passes in `engine/`, and the four mutant builds pass. The final message lists every change.
