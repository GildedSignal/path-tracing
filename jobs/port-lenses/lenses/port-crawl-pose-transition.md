# Port delta: dry-land crawl pose transition (Pose.SWIMMING under low ceilings)

Use the staged Mojang 26.2 client jar in `D:/path-tracing-deps/data`, the server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`, and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`. Anchor to `engine/law/src/movement.rs:147-149, 207`.

**Consumer:** Julien's track `amertystcoral` (closing a copper trapdoor onto the player to force `Pose::Swimming` `0.6h` crawl under a `1.0`-block ceiling, `jobs/flight-corpus/report.md`) and `docs/plan.md` step 5.

**From the 26.2 client and server bytecode**, state with class, method and offsets:
- `Player.updatePlayerPose` (or the 26.2 pose-selection method, to locate) and `canPlayerFitWithinBlocksAndEntitiesWhen` (to locate): the exact ordered `Pose` cascade (`SWIMMING`, `CROUCHING`, `STANDING`, etc.) and when a dry-land player (`inWater == false`, `isSwimming() == false`) transitions into and stays in `Pose.SWIMMING` (`0.6` height) vs `Pose.CROUCHING` (`1.5` height);
- `LocalPlayer.aiStep` / `LivingEntity.aiStep` (to locate) ordering relative to movement (`travel`) and `setPose`, plus how `Pose.SWIMMING` on dry land sets the `0.3` slow-input scale (`isVisuallyCrawling`, `movement.rs:157`).

**Build a jar-backed reproduction:** run `reproduce.sh` against the 26.2 jar verifying the pose transitions tick-by-tick for: (1) closing a trapdoor onto a standing player, (2) moving from a `1.0`-block gap into a `1.5`-block gap (does pose become `CROUCHING` or stay `SWIMMING`?), and (3) moving into a `>= 1.8`-block opening. Write `dossier.md` starting with `Claim: ` on its first physical line.

**Falsifier:** any pose prediction on the `1.0` -> `1.5` -> `1.8` clearance transitions that differs from the jar's pose-selection method.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
