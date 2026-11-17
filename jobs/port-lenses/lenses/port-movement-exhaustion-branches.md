# Port delta: ServerPlayer.checkMovementStatistics exhaustion branches

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`. Anchor to `engine/law/src/movement.rs:1103-1121` (`movement_exhaustion`).

**Consumer:** `parkour_race` (`20.0 -> 13.0` saturation drain, `jobs/flight-corpus/report.md` §4.1.5), `jobs/port-backlog/report.md` #89 (`trial-food-crossing`), and batch `P1a`.

**From the 26.2 server bytecode**, state with class, method and offsets:
- every branch and guard of `ServerPlayer.checkMovementStatistics(double, double, double)` (to locate exact signature and offsets): passenger/vehicle early return or branch, `isSwimming()`, `isEyeInFluid(FluidTags.WATER)` (`walkUnderWater`), `isInWater()` (`walkOnWater`), `onClimbable()`, `onGround()` + `isSprinting()` vs walking vs crouching, and `isFallFlying()`;
- the exact `Math.round(sqrt(...) * 100.0f)` cast/rounding order and exhaustion multiplier (`0.01f`, `0.1f`, `0.0f`, etc.) for each branch, noting any difference from `engine/law/src/movement.rs:1104-1121`.

**Build a jar-backed reproduction:** run `reproduce.sh` disassembling `ServerPlayer.checkMovementStatistics` and exercising representative `(dx, dy, dz)` deltas in each branch (swimming, underwater, water surface, climbable, ground sprint, ground walk, crouch) against `movement.rs:1104-1121`. Write `dossier.md` starting with `Claim: ` on its first physical line.

**Falsifier:** any branch condition, multiplier, or float/double rounding step in `checkMovementStatistics` omitted or misordered vs the 26.2 bytecode.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
