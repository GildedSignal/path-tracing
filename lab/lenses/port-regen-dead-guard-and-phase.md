# Port delta: server_regeneration health > 0.0 guard and dead-player food/saturation

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`. Anchor to `engine/law/src/movement.rs:1253-1268` (`server_regeneration`) and `jobs/port-backlog/report.md` #57 (`port-peaceful-regen`).

**Consumer:** batch `P3b` (`jobs/port-backlog/report.md`) and `engine/law/src/movement.rs:1257` (missing `health > 0.0` guard).

**From the 26.2 server bytecode**, state with class, method and offsets:
- `ServerPlayer.tickRegeneration` (to locate) -> `Player.isHurt` / `LivingEntity.heal(float)` / `FoodData.setSaturation` / `FoodData.setFoodLevel`:
- when `difficulty == PEACEFUL` and `naturalRegeneration == true` and `health <= 0.0f` (dead player, `tickCount % 20 == 0` and `tickCount % 10 == 0`): does `tickRegeneration` still run on a dead player (`ServerPlayer.doTick` / `Player.aiStep`), and if so, which of `health`, `saturation`, and `foodLevel` actually change vs stay gated by `health > 0.0f` (`isAlive()` / `LivingEntity.heal` `@0-18`)?
- also confirm `FoodData.tick` (`movement.rs:1280, 1288`) when `health <= 0.0f` vs `0.0f < health < 20.0f`.

**Build a jar-backed reproduction:** run `reproduce.sh` disassembling `ServerPlayer.tickRegeneration`, `Player.isHurt`, `LivingEntity.heal`, and `ServerPlayer.doTick`/`Player.aiStep` around the call, and execute the jar logic on `health = 0.0f, saturation = 5.0f, foodLevel = 10` at `tickCount = 20`, printing exact before/after raw bits for `(health, saturation, foodLevel)` vs `movement.rs:1253-1268`. Write `dossier.md` starting with `Claim: ` on its first physical line.

**Falsifier:** any `(health, saturation, foodLevel)` mutation on a `health <= 0.0f` Peaceful player that disagrees with the 26.2 jar.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
