# Job 1.2: the recorder and the dumper

You are the builder for a well-formed task. Design choices are not yours. If the spec below is ambiguous or wrong, stop and write the question in your final message instead of guessing.

## Context (read these first)

- `AGENTS.md`: the team and its rules.
- `docs/engine.md`: the law, the exactness classes, the facts verified so far.
- `README.md`: how the lab works.
- `bot/`: the Fabric mod. Read it all.
  - `bot/build.ps1` compiles it with plain `javac` against the unobfuscated 26.2 client jar. There is no Gradle and no Loom. The classpath comes from `Get-GameClasspath` in `scripts/common.ps1`, and the Fabric API jars from `bot/lib`.
  - The classpath already includes `sponge-mixin`. The game is unobfuscated, so a Mixin needs no refmap.
- Only the bot's client matters here. `PathTracingMod` picks its side from `-Dpathtracing.role`.

## What to build

### A. The recorder

It runs in the bot's client only. It is the judge that the Rust movement law (job 1.5) will be compared against tick by tick, so it must be exact and complete.

1. **Replay a file of tick-stamped inputs.**
   - **Trigger:** a whisper from the owner, `replay <name>`. It reads `<gameDir>/pathtracing/inputs/<name>.jsonl`.
   - **Input format:** one JSON object per tick: `{"t":0,"forward":true,"back":false,"left":false,"right":false,"jump":false,"sneak":false,"sprint":true,"yaw":0.0,"pitch":0.0}`.
   - **Start:** before tick 0, the bot sends the command `tp @s <x> <y> <z> <yaw> <pitch>`, taken from a first line `{"start":[x,y,z,yaw,pitch]}`. It then waits until its position equals the start and its velocity has been zero for 5 ticks.
   - **Applying inputs:** inputs are applied at `START_CLIENT_TICK` through `KeyMapping.setDown`, the same way `Bot.java` drives keys today. Yaw and pitch are set directly on the player.
   - **Recording:** state is recorded at `END_CLIENT_TICK`.
   - **While a replay runs:** the bot's planner and follow loop must not touch any key.
2. **Record a run.**
   - Commands: `rec start <name>` and `rec stop`, sent by whisper.
   - The recording mode logs whatever inputs are applied, so a planner run can be recorded too.
3. **Output.** Write `<gameDir>/pathtracing/recordings/<name>.jsonl`, one JSON object per client tick:
   - `t`;
   - the inputs actually applied;
   - position x, y and z, and velocity x, y and z (`getDeltaMovement()`), each as a decimal *and* as the 16-hex-digit `Double.doubleToRawLongBits`, so comparisons are bit-exact;
   - `yRot` and `xRot` as floats with their `Float.floatToRawIntBits`;
   - `onGround`, `horizontalCollision`, `verticalCollision`, `isSprinting`, `isShiftKeyDown`, `getPose()`, `fallDistance`, `isInWater`, `onClimbable`;
   - client-side `getHealth()`, `getFoodData().getFoodLevel()` and `getSaturationLevel()`.
4. **Anomalies.** Flag server corrections and pushes with a Mixin on `ClientPacketListener`.
   - A `ClientboundPlayerPositionPacket`, or a `ClientboundSetEntityMotionPacket` for the bot's own entity id, adds an `"anomaly"` field to that tick's line: the packet type and its values.
   - The Mixin goes in `bot/src/pathtracing/mixin/`, with a `pathtracing.mixins.json` declared in `fabric.mod.json`.
   - `bot/build.ps1` must package the mixin config.
5. **Server-side state.** Health, food, saturation and exhaustion are computed on the server, and the client never sees exhaustion.
   - Add `scripts/server-state.ps1 <playerName>`. It uses `scripts/rcon.ps1` to run `data get entity <name>` for `Health`, `foodLevel`, `foodSaturationLevel`, `foodExhaustionLevel`, `Pos`, `Motion` and `FallDistance`, and prints them as one JSON object.
   - The integrator runs it at the start and end of each recording.

### B. The dumper

1. **Trigger:** a whisper `dump blocks`. It writes `<gameDir>/pathtracing/dumps/blocks.jsonl`, one line per block **state** for every block in `BuiltInRegistries.BLOCK`, using `getStateDefinition().getPossibleStates()`.
2. **Each line holds:**
   - the block id and the state's properties;
   - `getFriction()`, `getSpeedFactor()`, `getJumpFactor()`;
   - `defaultDestroyTime()`, `requiresCorrectToolForDrops()`;
   - every block tag the state has;
   - `isCollisionShapeFullBlock`;
   - its collision shape as a list of AABBs (doubles and raw bits, as in the recorder).
3. **Collision contexts.** The shape is evaluated over an otherwise empty getter at the origin, in each of these contexts:
   - `CollisionContext.empty()`;
   - the bot's own entity (`CollisionContext.of(player)`) while standing above the block;
   - the bot's own entity while below the block;
   - sneaking above the block.

   Write a context only when it gives a different shape. That way scaffolding, powder snow and anything else whose shape depends on the entity come out right.
4. **Offsets.** For blocks with a position-dependent offset (`state.hasOffsetFunction()`, for example bamboo and pointed dripstone), add the offset at 8 fixed positions, and the method needed to reproduce it.
5. **Drops are not needed.** The bot never breaks blocks.

## Answers to the first run's questions

- **"Settled" before tick 0:** the position is bit-identical on 5 consecutive ticks, `onGround` is true, and the horizontal velocity is exactly zero. On the ground the vertical velocity keeps a small constant from gravity, so it is not required to be zero.
- **Build dependencies:** the sandbox cannot read `%LOCALAPPDATA%`, so a copy with the same layout is staged at `D:\path-tracing-deps\data`. The runner sets `PATHTRACING_DEPS` to it, so `scripts/common.ps1` and `bot/build.ps1` use it unchanged, and `javap` is at `D:\path-tracing-deps\data\java\java-runtime-epsilon\bin\javap.exe`. If anything is still denied, say exactly which path.

## Constraints

- Do not touch `Pointer.java`, the planner, or the scripts other than `server-state.ps1` and `bot/build.ps1`.
- Keep `Bot.java`'s existing behaviour. Only add the replay, recording and dump commands, and make the follow loop stand down while a replay runs.
- Match the existing code style: comment density, names, idioms.
- You cannot run the game here: the rig is outside this worktree. Compile with `.\bot\build.ps1` (Windows PowerShell) and make it pass with no warnings you introduced. The integrator runs everything live.
- Do not commit: the sandbox cannot write the repo's git metadata. Leave every change in the worktree; the integrator commits it.

## Done means

- `.\bot\build.ps1` passes.
- The final message lists:
  - every file changed;
  - the exact in-game commands;
  - the format of each file;
  - anything in this spec you could not do, with the reason.

## The gate the judge will apply (frozen now)

1. Replaying the same input file twice from the same start gives byte-identical recordings, ignoring wall-clock fields; there should be none.
2. On flat ground with no other player near, a recording of 400 random input ticks has zero anomaly lines.
3. The dump covers every block state in the registry: its line count equals the sum of `getPossibleStates().size()`. Every line parses as JSON. Ice has friction 0.98, blue ice 0.989, slime 0.8, soul sand speed 0.4, and honey speed 0.4 and jump 0.5.
4. The negative control: the recorder must flag an anomaly when the integrator teleports the bot during a recording.
