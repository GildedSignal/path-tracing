# The port, build A: four settled laws into the Rust law, each bit-exact against the jar's witness

**Julien (18 November, verbatim):** "have hydrogen work on the port"

**Consumers:**
- step 5 of `docs\plan.md`, editing the world: mining time is the price of a break word (Part 1);
- step 8 of `docs\plan.md`, carried movement and wormholes: boat on land, boat mount and dismount, elytra with fireworks, the ender pearl (Parts 2–4). The elytra law is also what the brachistochrone rung (`docs\ladder.md`, R-brach) needs.

The facts are settled: GLM found each one in Mojang's 26.2 jar and a second GLM session reproduced it. Your own digest wrote each port delta: `C:\Users\PASCAL\Desktop\path tracing\jobs\port-wave-2\report.md`, Part 1, items 1, 3, 4, 5, 6 and 8, and section 3, batches A and B. That digest is your spec. Where it and the evidence disagree, the evidence wins: say so in the report.

## Where you work

- Worktree `D:\path-tracing-wt\port-build-a`, branch `wt/port-build-a`. Build and test there, with `--target-dir D:\path-tracing-target\agy-port-build-a`. Rust is at `D:\rust` (`CARGO_HOME=D:\rust\cargo`, `RUSTUP_HOME=D:\rust\rustup`, `D:\rust\cargo\bin` on `PATH`). Run `cargo test --offline -p law` from `engine\`.
- The evidence (read only): the folders in `C:\Users\PASCAL\Desktop\path tracing\lab\results\`:
  - `local-20261117-1340-1-0-port-mining-destroy-speed`
  - `local-20261117-1342-1-0-port-boat-land-and-collision`
  - `local-20261117-1519-1-0-port-boat-mount-dismount`
  - `local-20261117-1520-1-0-port-elytra-firework-and-impact`
  - `local-20261117-1521-1-0-port-pearl-raytrace-and-landing`
  - `local-20261117-1617-1-0-port-boat-trig-index`

  Each folder has `dossier.md` (the author's fact), `check.md` (the checker's grade, which wins) and the raw witness outputs (`evidence.txt`, `check-extra.out`, `witness.txt`, `probe-output.txt`, `jar_bits.txt`, and so on). **Your test values are copied from those raw outputs, never retyped from the dossier's prose.** Name the file and line for every value in a comment.
- Write your report as you go into `C:\Users\PASCAL\Desktop\path tracing\jobs\port-build-a\report.md`. Update it after each part, so a stop loses nothing.

## The four parts (independent: a part that blocks does not stop the others)

1. **Mining speed** (`engine\law\src\mining.rs`): `Player.getDestroySpeed` in Mojang's exact `f32` operation order, the tool rules' first match, feeding the existing `getDestroyProgress`. Tests: all 23 author and checker cases, raw `f32` bits and the break tick count.
2. **The boat on land and in and out of it** (`engine\law\src\boat.rs`): the five statuses, ground friction, per-status drag, gravity and buoyancy, `maxUpStep = 0`, the signed-zero restitution, the 60-tick ejection; mounting, `clampRotation`, the dismount location. Plus the trig-index regression test (item 8: tests only, no production change). Tests: the raw `u64` bits from the evidence files.
3. **Elytra and fireworks** (new `engine\law\src\elytra.rs`): the boost in the bytecode's exact association, the lifetime bounds, the boost landing one tick after its tick, the fly-into-wall damage with its NaN case. The glide itself (`travelFallFlying`) is not in this job: if the boost cannot be tested without it, test the boost as a function of (velocity, look) and say what the glide still needs.
4. **The ender pearl** (new `engine\law\src\pearl.rs`): gravity before drag, the water drag, the collider-only ray with fluids ignored, the entity sweep margin, the teleport to the start-of-tick position, fall reset before the 5-point damage. Use the existing block traversal in the law; add no new search or geometry code.

`engine\law\src\lib.rs` gets only `pub mod` lines for the new modules.

## Rules that bind you

- **One mechanism** (`docs\learning.md` section 0, S161): no block kind or terrain kind gets its own code path. A law that applies to a set of blocks reads it from the block table's data, never from a list of names in code.
- **No patches** (AGENTS.md): no tolerance, epsilon, clamp, retry or special case added to make a test pass. If the port and the witness disagree, write down the disagreement with both values and stop that part. Mojang's own oddities (NaN, signed zeros, unclamped `acos`) are ported faithfully, never fixed.
- **Do not touch** `engine\law\src\movement.rs` or `state.rs` (another builder changes them), any other crate, any doc, or any existing test's expected values. Wiring the boat and elytra refusals in `movement.rs` is a later job: list what it needs.
- The law's hash (`LAW_HASH`, `build.rs`) changes with any edit to the law's source. That is expected. If a test pins a hash value, leave it failing and report it; never update a pinned hash.
- No commits: the integrator commits your work.

## The gate

`cargo test --offline` in the law crate passes with every new test, and every pre-existing test result is unchanged (run the suite before your first edit and record the result). Each new test fails if the law's formula is changed to the mutant the evidence falsified (for example drag before gravity for the pearl, or the folded boost `0.5*v + 0.85*look`). Show one such mutant run per part.

## The report

Per part: done, partly done or blocked; the functions added (file:line); the tests and the evidence file each value came from; the mutant run; what wiring is left for `movement.rs`. Then the full test command and its summary line, before and after. End with `STATUS: DONE` or `STATUS: BLOCKED` and why.
