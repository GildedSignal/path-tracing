# Recorder and block dump

Run these whispers from the owner (Brain_Forge by default):

```text
/msg PathBot replay <name> [out]
/msg PathBot rec start <name>
/msg PathBot rec stop
/msg PathBot dump blocks
```

Names contain only letters, digits, `_` and `-`. Replay reads input `name` and writes recording `out`
(default: `name`). Replay and manual recording refuse an existing final or temporary output, naming
the file in the reply. A blocking `.tmp` reply says to delete that file before reusing the name.
Choose another output name or move the old final file before starting again.
Only the bot role registers the recorder. Replay owns all seven movement keys, rejects other movement
commands and suspends the follow loop. `rec stop` cancels a replay too; `stop` also clears the existing
follow path. Starting a replay clears the old follow path, so it cannot resume afterwards.
Both replay and recording require `toggleSprint`, `toggleCrouch` and `autoJump` off, and
`sprintWindow` zero; otherwise the command refuses and reports those values.

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
Vertical velocity is left to vanilla. A start that has not settled by the 200th END client tick
after sending `tp` aborts with a reply. Teleport coordinates and angles use plain decimal notation;
angles are first rounded to their actual float values.

## Recordings

`<gameDir>/pathtracing/recordings/<out or name>.jsonl` is UTF-8 JSONL with LF endings and no clock
fields. Its first line records the required input settings:

```json
{"toggleSprint":false,"toggleCrouch":false,"autoJump":false,"sprintWindow":0,"type":"recording"}
```

Every closed recording ends with a trailer, where `ticks` is the number of tick rows.

```json
{"type":"end","ticks":400,"complete":true}
```

`complete` is true only after all replay inputs ran or a manual recording ended with `rec stop`.
An early replay `rec stop` or `stop` writes `complete:false`. Aborts also write a false trailer
when the output remains writable. A write or close failure can leave a missing or malformed trailer.
Output is written to `.jsonl.tmp` and renamed only after a successful close at replay completion
or an explicit `rec stop` / replay `stop` (including early cancellation). A disconnect, missing
player/world, write failure, settling timeout or tickCount gap aborts and retains only the temporary
output. Existing final recordings are never replaced. Replay opens this output automatically. Manual
recording samples after the follower's START callback and writes at END, without driving the keys.
Each tick row has:

- `t`, the player's absolute `tickCount`, and `inputs` (the seven key booleans, `yaw`, `pitch`);
- `position` and `velocity`, each with `x`, `y`, `z`;
- `yRot`, `xRot`, `onGround`, `horizontalCollision`, `verticalCollision`, `isSprinting`,
  `isShiftKeyDown`, `pose`, `fallDistance`, `isInWater`, `onClimbable`;
- client `health`, `foodLevel`, `saturationLevel`;
- optional `anomaly`: an array of packets, in handler order, preserving multiple anomalies per tick.

After the first row, `tickCount` must advance by exactly one per row. An offending row is retained
in the temporary output and the run aborts, replying with `t` and the previous/current tickCount.

Every double has a sibling `<field>Bits` with sixteen lowercase hex digits. Floats (input angles,
rotations, health and saturation) use eight. `fallDistance` is a double in 26.2. Raw bits are unsigned
hex strings without `0x`. The decimal fields remain JSON numbers.

Position anomalies include `type`, teleport `id`, `position`, `velocity`, `yRot`, `xRot` and sorted
`relatives`. Motion anomalies include `type`, entity `id` and `velocity`. Packet numbers use the
same bit fields. Rotation anomalies include `type`, `yRot`, `xRot`, `relativeY`, `relativeX`.
Explosion anomalies include `type`, `center`, `playerKnockback`, and are emitted only when the
packet carries knockback for this client player. Rotation packets also target this client player;
entity-motion packets are filtered by the bot's id. Packets handled between ticks are included in the next END row; replay's initial
teleport and settling phase precede recorded tick zero.

## Blocks

`dump blocks` refuses during either a recording or a replay (including settling). It writes
`<gameDir>/pathtracing/dumps/blocks.jsonl.tmp`, then atomically replaces `blocks.jsonl` only on
success. Failure leaves the previous final dump intact and retains the temporary output.

The first line has `type: "dump"`, `mainHand` and `offHand`, each an object with the namespaced
`item` id and stack `count` from the bot before sampling. The following lines have one JSON object
per registry block state. Each contains
`block` (namespaced id), `properties` (string values), `friction`, `speedFactor`, `jumpFactor`,
`defaultDestroyTime`, `requiresCorrectToolForDrops`, sorted `tags`, `isCollisionShapeFullBlock`
and `collisionShapes`. Numeric float properties also have eight-digit bit fields.

Each `collisionShapes` entry has `contexts` and `aabbs`. Contexts giving identical AABB lists share
one entry. Every AABB has `minX`, `minY`, `minZ`, `maxX`, `maxY`, `maxZ` and sixteen-digit bit fields.
The getter contains the tested state at the origin and air elsewhere, with no block entity.
Contexts are `empty`, `above` (feet at 0.5,1,0.5), `below` (0.5,-2,0.5), `sneakingAbove`, and
`holdingScaffolding` (standing above with one scaffolding item in the main hand).
Entity contexts use the bot's current equipment, zero fall distance and standing/crouching pose.
The scaffolding context uses `CollisionContext.of(player)`. The original main-hand stack, input,
pose, position, bounds and fall distance are restored in `finally` before returning to the game.

The final line has `type: "summary"`, `writtenCount` (state rows actually written) and
`registryCount` (a separate sum of possible-state counts across the block registry). A mismatch
fails the dump. Both counts appear in the reply. Header and summary are not state rows; total
physical lines are `writtenCount + 2`. The frozen 1.2 gate's line count means state lines, excluding
the header and summary. `check-dump.ps1` is the gate's check of this count against the registry.

States with offsets also have `offset`: `method`, `type` (`XZ` or `XYZ`), maximum offsets, Java
expressions for the seed and components, and eight fixed `samples` containing `pos` and `offset`.
Expressions preserve Java int/long overflow, signed shifts and float-to-double conversions.
Each emitted generator is checked against all its samples. Shapes already include the offset at
the origin; subtract that offset before applying the offset at another position.

## Gate helpers

All five helpers in `lab/gates/` use Windows PowerShell 5.1 and contain only ASCII. The gate driver
holds the rig lock with `Enter-Rig` / `Exit-Rig` from `scripts/common.ps1`. The helpers run inside
that lock. Offline checks accept literal paths (including a retained `.tmp`).

```powershell
.\lab\gates\new-inputs.ps1 -Name random400 -Ticks 400 -Seed 7 -Start '0.5,-60,0.5,0,0'
.\lab\gates\compare-recordings.ps1 <a.jsonl> <b.jsonl>
.\lab\gates\check-recording.ps1 <recording.jsonl>
.\lab\gates\check-dump.ps1 <blocks.jsonl>
.\lab\gates\bot-command.ps1 -Text 'replay random400 run1' -WaitFor 'Saved run1:' -TimeoutSeconds 60
```

`new-inputs` writes `$Rig\clients\bot\pathtracing\inputs\<Name>.jsonl`. A seeded `System.Random`
selects independent pressed/released runs of 1-20 ticks for each key (the final run can be shorter),
and nonzero integer yaw steps from -90 through 90 degrees every 1-20 ticks. Pitch remains the
start pitch. The same arguments on Windows PowerShell 5.1 produce the same bytes; an existing input
file is replaced. The start line is built with invariant-culture round-trip (`R`) formatting for
the three doubles and the two float-rounded angles.

`compare-recordings` compares the header's four settings and every tick-row field except absolute
`tickCount`, including decimals, raw bits, inputs, on-ground and collision flags, pose, food values
and anomalies. Fields are visited in sorted name order and arrays in index order. It reports the
first differing tick and field, or `identical`, and compares the end trailers too. Different lengths,
invalid tick stamps, missing headers/trailers and malformed raw fields fail.
`check-recording` reports every anomaly line, nonconsecutive `t`, and missing/invalid or
nonconsecutive tickCount. It also rejects a missing trailer, a wrong trailer count or `complete:false`.
`check-dump` checks state-row count against both summary counts and checks
every state of ice (friction 0.98), blue ice (0.989), slime (0.8), soul sand (speed 0.4), and honey
(speed 0.4, jump 0.5), comparing as Mojang floats. Both checkers reject empty/malformed files or
missing metadata. All checks exit nonzero on a failure.

### Commands over RCON

Gates drive the bot while Julien's client is out of the game. `bot-command.ps1` sends
`msg PathBot <command>` through `scripts/rcon.ps1`. The bot accepts disguised CHAT whispers only
when both sender and signature are null and the decorated sender name is exactly `Rcon` or `Server`.
The console's stdin can send the same `msg PathBot <command>`. Owner command handling is preserved.

Every reply is still whispered to the owner and also logged on one line with `[pathtracing] `.
With `-WaitFor`, the helper opens `$Rig\clients\bot\logs\latest.log` at its end immediately before
sending, then waits for a newly appended prefixed reply whose full log line matches the regex. It
prints the full log line and exits 0, or exits nonzero on timeout (30 seconds by default). Without
`-WaitFor` it sends the command and returns the RCON response. The gate driver holds the rig lock
through sending and waiting.

## Server endpoints

Run `.\scripts\server-state.ps1 PathBot` at each recording's start and end. It prints one JSON object
with `Health`, `foodLevel`, `foodSaturationLevel`, `foodExhaustionLevel`, `Pos`, `Motion`, `FallDistance`.
Vectors are numeric arrays. The fields come from separate sequential RCON queries, not an atomic
snapshot. Restore fixture state, equipment, effects and server resources before comparing replays;
the replay command only teleports and waits for the specified movement settling condition.
