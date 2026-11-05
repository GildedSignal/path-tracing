# Recorder and block dump

Run these whispers from the owner (Brain_Forge by default):

```text
/msg PathBot replay <name> [out]
/msg PathBot rec start <name>
/msg PathBot rec stop
/msg PathBot dump blocks
/msg PathBot export <name> <x0> <y0> <z0> <x1> <y1> <z1>
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
the three doubles and the two float-rounded angles. Every tick row uses the same `R` formatting
for its float yaw and pitch, without passing them through `ConvertTo-Json`.

`compare-recordings` compares the header's four settings and every tick-row field except absolute
`tickCount`, including decimals, raw bits, inputs, on-ground and collision flags, pose, food values
and anomalies. Fields are visited in sorted name order and arrays in index order. It reports the
first differing tick and field, or `identical`, and compares the end trailers too. Different lengths,
invalid tick stamps, missing headers/trailers and malformed raw fields fail. Both trailers must have
boolean `complete:true`; an incomplete trailer fails and identifies its file, including both files
when both are incomplete.
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
An operator, or a command block or entity named `Rcon` or `Server`, can also command the bot through
`/msg`. This is an accepted risk because those already require operator rights.

Every reply is still whispered to the owner and also logged on one line with `[pathtracing] `.
With `-WaitFor`, the helper opens `$Rig\clients\bot\logs\latest.log` at its end immediately before
sending, then waits for a newly appended prefixed reply whose full log line matches the regex. It
checks `-FailOn <regex>` first on each new prefixed line, then `-WaitFor`. A failure match prints the
full line and exits 2 immediately. A success match prints the full line and exits 0. Timeout
(30 seconds by default), transport errors and other helper errors print a diagnostic and exit 1.
RCON responses are discarded, so stdout contains only the matched reply or an error diagnostic.
Without `-WaitFor` it sends the command, discards the response and exits 0 without waiting.
The gate driver holds the rig lock through sending and waiting.

The default `-FailOn` covers every current failure reply from `Bot`, `Recorder`, `TickInput` and
`BlockDumper`, plus `Refusing` for refusal replies. The patterns are case-sensitive .NET regexes.

- `Replay is running.` rejects commands during replay.
- `Usage` covers both recorder/dumper syntax and `goto` syntax errors.
- `I can't see you.`, `No path to ` and `Stuck at ` cover movement failures.
- `Recorder/dumper:` covers all synchronous recorder/dumper exceptions, including invalid names,
  an active recording/replay, invalid hold settings, missing/unreadable inputs, malformed JSON,
  missing/invalid start values, nonconsecutive input ticks, missing/invalid key booleans or angles,
  empty replays, output/open/write errors, dump-count mismatches and unknown offset generators.
- `already exists` also explicitly matches existing final or temporary outputs.
- `Aborted` covers disconnects, unavailable player/world, settling timeouts, tickCount gaps and
  recording-write failures, with the reason and temporary path in the reply.
- `Recording finalization failed` covers trailer-write, close and final-rename failures.
- `Refusing` matches refusal replies.

## Server endpoints

Run `.\scripts\server-state.ps1 PathBot` at each recording's start and end. It prints one JSON object
with `Health`, `foodLevel`, `foodSaturationLevel`, `foodExhaustionLevel`, `Pos`, `Motion`, `FallDistance`.
Vectors are numeric arrays. The fields come from separate sequential RCON queries, not an atomic
snapshot. Restore fixture state, equipment, effects and server resources before comparing replays;
the replay command only teleports and waits for the specified movement settling condition.

## L0 world export and initial state (job 1.6)

`export` writes `<gameDir>/pathtracing/fixtures/<name>.world.jsonl` through a sibling `.tmp`.
Both existing final and temporary outputs are refused. Failures retain the temporary output.
The two corners are normalized per axis and bounds are inclusive. Every intersecting client chunk
must be loaded. Export refuses during recording/replay. The completion reply is
`Exported <name>: <count> blocks.`

Every non-air position becomes a `kind=block` row with integer `at`, registry `block` id, diagnostic
`state` string and local numeric AABB arrays. `boxes` uses the empty collision context. `aboveBoxes`,
`belowBoxes`, `sneakBoxes` and `holdingScaffoldingBoxes` use the dumper's contexts translated to that
world position, with the client's actual neighbours. The holding-scaffolding array is extra metadata
that the current Rust reader ignores. `insideBoxes` comes from `getEntityInsideCollisionShape` with
the actual player. Equipment, keys, pose, fall distance, position and bounds are restored in `finally`.

Friction, speed/jump factors and bounce are block getters, including `getBounceRestitution`.
Climbable, fence, wall/gate, bounce suppression and fall-distance reset are the live block tags.
Fluid kind is read through fluid tags, height through `getHeight(level,pos)` and current through
`getFlow(level,pos)`. Floats and vectors retain raw IEEE fields. A world row reads client minY,
difficulty and dimension FAST_LAVA. Remote ClientLevel has no game rules. Export therefore sends
the vanilla natural-health-regeneration query and reads the `commands.gamerule.query` translatable
reply's arguments, independent of client language. Missing operator permission/feedback times out
and refuses export. There is no assumed game-rule value and no player row in an export.

The class-derived hooks follow the 26.2 bytecode. These are the complete export derivations.

- `ScaffoldingBlock` supplies `scaffolding`.
- `SlimeBlock` supplies `slime`, which selects its landing-damage and step hooks in Rust.
- A `LiquidBlock` carrying water, or a `BubbleColumnBlock`, supplies `waterBlock`.
- `WebBlock`, `PowderSnowBlock` and `HoneyBlock` select `inside=cobweb`, `powderSnow` and `honey`.
  Other blocks select `none`. Their hook parameters are implemented by the law.
- `HayBlock` and `HoneyBlock` supply `fallOnMultiplier=0.2F`, `BedBlock` supplies `0.5F`, and
  the default landing hook supplies `1.0F`. These hooks have no numeric getter. Slime and powder
  snow retain their separate law hooks. Bounce itself is read from the game, including beds/slime.
- `LadderBlock` reads `FACING` as a vanilla 3D direction id. An open `TrapDoorBlock` reads its
  `FACING` into `openTrapdoorFacing`. Closed trapdoors omit that field.

The recording header now also contains `initialPlayer`, captured at START after replay settling
and before applying input zero. Header writing is deferred to this point. Stopping an empty or
aborted replay still writes its required metadata/trailer. Existing metadata and tick rows retain
their format. The snapshot has every player field accepted by the fixture README, including
attributes, fluid/eye flags, support position, stuck multiplier, jump delay and client food timer.
Read-only mixin accessors expose private/protected state. Its tickCount is the pre-tick count.
Local FoodData exhaustion and timer are client observations, not server-authoritative values.

### Fixture driver

```powershell
.\lab\gates\record-fixtures.ps1 -Jar .\bot\build\pathtracing.jar
.\lab\gates\record-fixtures.ps1 -Courses ice,edges -Runs 8 -Ticks 400 -Seed 1
.\lab\gates\tests\test-record-fixtures.ps1
.\lab\gates\tests\test-courses.ps1
.\lab\gates\tests\test-fixture-json.ps1
```

The optional Jar installation has the same running-client hash guard as gate 1.2. An installed
job 1.6 jar is otherwise required. The driver takes the rig lock, refuses Julien's exact online
name, validates hold-style options and stops only its own PID/start-time-matched processes.
`fixture-rig.ps1` carries gate 1.2's bounded child-process transport and ownership handling.
Courses accepts an array or comma-separated names. Omitting it selects every course. Defaults
for runs, ticks and seed are in the driver. Course seeds use declaration index, so selecting a
subset preserves each course's input sequence for the same Runs/Seed.

`courses.ps1` constructs enclosed patches on the superflat. Solid floors, walls and roofs contain
any walk and separate the classes. The start pad is plain stone. Edges use a raised grid over a
safe catch floor. The water patch has a source pool and a flowing lip. Bubble columns are omitted
because the frozen fixture schema/law has no acceleration hook for them. The driver teleports
to load chunks, builds, waits for delivery, and verifies the last gold-block marker through a
full client export before replaying. Each run exports the inclusive floored position extrema of
both initialPlayer and post-tick rows, expanded by three blocks on each axis.

Kept recordings are byte-for-byte copies. Their world companions contain one player row, the
unchanged client export and one serverEnd row. Both files are staged before publication. Existing
fixtures or staging files are refused. Rejections retain inputs, recordings, exports and command
logs under `lab/gates/results`, with per-course reasons and kept/rejected/not-run counts.

Server timing remains unaligned. The start sample precedes replay teleport and settling. Initial
health, food and saturation must match the settled client's snapshot or the run is rejected.
The player's resource fields then use that server sample, including exhaustion. This assumes
exhaustion stays constant between sampling and input zero. The end sample is the first operation
after the Saved reply is observed. `server-state.ps1` issues separate queries, spanning an
unbounded number of server ticks. These endpoints are provisional resource evidence. No player
resources, equipment or effects are reset. Live L0 resource exactness requires tick-aligned reads.

The driver assumes the configured rig is the intended lab superflat, the bot is op, and the client
can load each entire patch. Existing courses at these reserved coordinates are replaced.
World exports describe the state after a run and assume movement-relevant terrain stayed static
during it. Readiness verifies loaded chunks and the final construction marker. Coverage of each
law hook and mutant is still judged by L0 against the recorded paths.

The current `wt/law` replay reader accepts the header but rejects the recorder's `type=end` trailer.
It needs trailer support before these unchanged recordings can run through L0. The independent
JSON smoke check compiles a copy of its parser and checks a hand-written block row without
modifying `wt/law`.
