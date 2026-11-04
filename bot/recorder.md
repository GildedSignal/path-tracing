# Recorder and block dump

Run these whispers from the owner (Brain_Forge by default):

```text
/msg PathBot replay <name>
/msg PathBot rec start <name>
/msg PathBot rec stop
/msg PathBot dump blocks
```

Names contain only letters, digits, `_` and `-`. Outputs replace an existing file of the same name.
Only the bot role registers the recorder. Replay owns all seven movement keys, rejects other movement
commands and suspends the follow loop. `rec stop` cancels a replay too; `stop` also clears the existing
follow path. A suspended path resumes after replay completion or `rec stop`.

## Inputs

`<gameDir>/pathtracing/inputs/<name>.jsonl` is UTF-8 JSONL. Its first line is:

```json
{"start":[0.5,-60.0,0.5,0.0,0.0]}
```

Every subsequent line has consecutive `t` starting at zero and all of these fields:

```json
{"t":0,"forward":true,"back":false,"left":false,"right":false,"jump":false,"sneak":false,"sprint":true,"yaw":0.0,"pitch":0.0}
```

Replay sends the specified teleport, waits for its position packet and five consecutive END ticks at
the exact start coordinates, on ground with zero horizontal velocity, then starts input tick zero.
Vertical velocity is left to vanilla. A start that cannot settle waits until cancelled.

## Recordings

`<gameDir>/pathtracing/recordings/<name>.jsonl` is UTF-8 JSONL with LF endings and no clock fields or
header. Replay opens this output automatically and closes it after the last input tick. Manual
recording samples after the follower's START callback and writes at END, without driving the keys.
Each row has:

- `t` and `inputs` (the seven key booleans, `yaw`, `pitch`);
- `position` and `velocity`, each with `x`, `y`, `z`;
- `yRot`, `xRot`, `onGround`, `horizontalCollision`, `verticalCollision`, `isSprinting`,
  `isShiftKeyDown`, `pose`, `fallDistance`, `isInWater`, `onClimbable`;
- client `health`, `foodLevel`, `saturationLevel`;
- optional `anomaly`: an array of packets, in handler order, preserving multiple anomalies per tick.

Every double has a sibling `<field>Bits` with sixteen lowercase hex digits. Floats (input angles,
rotations, health and saturation) use eight. `fallDistance` is a double in 26.2. Raw bits are unsigned
hex strings without `0x`. The decimal fields remain JSON numbers.

Position anomalies include `type`, teleport `id`, `position`, `velocity`, `yRot`, `xRot` and sorted
`relatives`. Motion anomalies include `type`, entity `id` and `velocity`. Packet numbers use the
same bit fields. Packets handled between ticks are included in the next END row; replay's initial
teleport and settling phase precede recorded tick zero.

## Blocks

`<gameDir>/pathtracing/dumps/blocks.jsonl` has one JSON object per registry block state. Each contains
`block` (namespaced id), `properties` (string values), `friction`, `speedFactor`, `jumpFactor`,
`defaultDestroyTime`, `requiresCorrectToolForDrops`, sorted `tags`, `isCollisionShapeFullBlock`
and `collisionShapes`. Numeric float properties also have eight-digit bit fields.

Each `collisionShapes` entry has `contexts` and `aabbs`. Contexts giving identical AABB lists share
one entry. Every AABB has `minX`, `minY`, `minZ`, `maxX`, `maxY`, `maxZ` and sixteen-digit bit fields.
The getter contains the tested state at the origin and air elsewhere, with no block entity.
Contexts are `empty`, `above` (feet at 0.5,1,0.5), `below` (0.5,-2,0.5) and `sneakingAbove`.
Entity contexts use the bot's current equipment, zero fall distance and standing/crouching pose.
The input, pose, position, bounds and fall distance are restored before returning to the game.

States with offsets also have `offset`: `method`, `type` (`XZ` or `XYZ`), maximum offsets, Java
expressions for the seed and components, and eight fixed `samples` containing `pos` and `offset`.
Expressions preserve Java int/long overflow, signed shifts and float-to-double conversions.
Each emitted generator is checked against all its samples. Shapes already include the offset at
the origin; subtract that offset before applying the offset at another position.

## Server endpoints

Run `.\scripts\server-state.ps1 PathBot` at each recording's start and end. It prints one JSON object
with `Health`, `foodLevel`, `foodSaturationLevel`, `foodExhaustionLevel`, `Pos`, `Motion`, `FallDistance`.
Vectors are numeric arrays. The fields come from separate sequential RCON queries, not an atomic
snapshot. Restore fixture state, equipment, effects and server resources before comparing replays;
the replay command only teleports and waits for the specified movement settling condition.
