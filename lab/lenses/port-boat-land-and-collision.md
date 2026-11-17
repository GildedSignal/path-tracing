# Port delta: AbstractBoat ON_LAND, IN_AIR, UNDER_WATER and move collision

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`. `port-boat-delta`, `port-boat-surface-transition`, and `trial-boat-water-surface-bits` already settled `IN_WATER` and `checkInWater`/`getStatus`; anchor to `engine/law/src/boat.rs`.

**Consumer:** `docs/plan.md` step 8 (carried movement: boat), batch `P4a` (`jobs/port-backlog/report.md`), and `engine/law/src/boat.rs`.

**From the 26.2 server bytecode**, state with class, method and offsets:
- `AbstractBoat.floatBoat` (to locate) and `AbstractBoat.getGroundFriction` (to locate): exact drag/friction/gravity/buoyancy constants and writes for every non-`IN_WATER` status (`ON_LAND`, `IN_AIR`, `UNDER_WATER`, `UNDER_FLOWING_WATER`, to locate), including how `landFriction` averages supporting blocks (`0.989` blue ice, `0.98` ice, `0.6` stone) and how `outOfControlTicks` ejects passengers when submerged;
- `AbstractBoat.tick` -> `Entity.move` (and step-up / `maxUpStep` / `checkFallDamage`, to locate): exact boat `maxUpStep` (is a boat `0.0f` or `0.6f`?) and horizontal wall collision behavior.

**Build a jar-backed reproduction:** run `reproduce.sh` disassembling `floatBoat`, `getGroundFriction`, and `tick`, and drive a jar-backed boat over 10 ticks on (1) `ON_LAND` stone vs blue ice (`0.989`), (2) `IN_AIR`, and (3) `UNDER_WATER` (`outOfControlTicks` eject threshold), printing raw velocity/position bits. Write `dossier.md` starting with `Claim: ` on its first physical line.

**Falsifier:** any `landFriction` formula, `IN_AIR` drag/gravity constant, `maxUpStep` value, or `outOfControlTicks` ejection tick that disagrees with the 26.2 jar.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
