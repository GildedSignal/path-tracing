# Port delta: creative-flight horizontal speed, friction and sprint gate

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`, the client jar at `D:/path-tracing-deps/minecraft-26.2-client.jar`, and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`. `port-creative-flight-toggle` already settled vertical flight (`±flyingSpeed * 3.0f` and `vy = d0 * 0.6d`) and `port-movement-modes` settled `handlePlayerAbilities`; do not re-ask those.

**Consumer:** `parkour_race` (`16821` ticks, `jobs/flight-corpus/report.md` §4.1.4), `engine/law/src/movement.rs:133-135,309-315`, and batch `P4e`.

**From the 26.2 client and server bytecode**, state with class, method and offsets:
- `Player.getFlyingSpeed` (to locate): exact float constants and conditions for `abilities.flying && !isPassenger()` (`flyingSpeed` vs `flyingSpeed * 2.0f` when `isSprinting()`), vs non-flying (`0.025999999f` vs `0.02f`);
- `LivingEntity.travel` / `travelInAir` / `getFrictionInfluencedSpeed` (to locate) when `abilities.flying && onGround`: whether ground block friction (`0.6` / `0.98` ice) and `speedFactor` still apply while `abilities.flying` is true, or whether `getFlyingSpeed` is bypassed on ground;
- `LocalPlayer.aiStep` / `canStartSprinting` / `shouldStopRunSprinting` (to locate): whether sprint can start and stay active in mid-air while `abilities.flying == true` (and whether `flying` interacts with `slowly`, `underwater`, or `inWater` sprinting).

**Build a jar-backed reproduction:** run `reproduce.sh` disassembling `Player.getFlyingSpeed` and `LivingEntity.travelInAir` / `getFrictionInfluencedSpeed`, and drive the jar methods for (1) `flying=true, onGround=false` sprint vs walk, (2) `flying=true, onGround=true` on blue ice (`0.989f`) vs stone (`0.6f`), printing raw `f32`/`f64` horizontal velocity bits after 1 tick. Write `dossier.md` starting with `Claim: ` on its first physical line.

**Falsifier:** any `getFlyingSpeed` constant, ground-friction branch while `flying=true`, or mid-air flight sprint verdict that disagrees with the 26.2 bytecode.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
