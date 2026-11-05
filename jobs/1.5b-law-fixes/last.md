Implemented job 1.5b. Changes are uncommitted.

**Changes**

- Sprint now latches, with shallow-water, sneaking and key-release tests.
- Both epsilons use the widened float constant. Audited other numeric literals.
- Beds halve fall distance before safe-distance subtraction. Added bed, hay and honey checks.
- Added four-corner push-out, suffocation queries and `noPhysics` handling.
- Added block-or-fluid visited-set handling and water extinguishing.
- Ported fall-reset voxel traversal and hit/miss clipping, including end portals and gateways.
- Extended replay comparisons for all requested fields.
- Added jar-generated sprint, epsilon, landing, ray and braking expectations. Both braking tests are labeled regression-only.
- Updated engine and fixture documentation.

**New `unimplemented!` branches**

- Unsupported block `entityInside` hooks, including bubble columns and sweet berry bushes.
- Lava’s deferred freeze-clearing, ignition and damage effects.

**Assumptions**

- Following the literal request, recording tick-zero `isSprinting` seeds the latch before simulating tick zero, although recorder rows are post-tick.
- `Block::full()` represents a suffocating solid. Custom blocks require explicit suffocation and hook metadata.
- Beds require `bed=true` and landing multiplier `1`. Fluid heights must already include world-dependent evaluation.
- Server forecasts with zero-delay Nether portals must mark eligible portals as fall-resetting.
- Braking probe orchestration remains a regression model. Whole-player exactness awaits L0 recordings.

**Validation**

- `cargo test` passed with **38 passed, 1 ignored**. Replay remains ignored without fixtures.
- All four mutant-feature builds passed.
- `git diff --check` passed.