# Recorder and block dump

Run these whispers from the owner (Brain_Forge by default):

```text
/msg PathBot replay <name> [out]
/msg PathBot replay <name> <out> wait
/msg PathBot rec continue
/msg PathBot rec start <name>
/msg PathBot rec stop
/msg PathBot dump blocks
/msg PathBot export <name> <x0> <y0> <z0> <x1> <y1> <z1>
```

Names contain only letters, digits, `_` and `-`. Replay reads input `name` and writes recording `out`
(default: `name`). Replay and manual recording refuse an existing final or temporary output, naming
the file in the reply. A blocking `.tmp` reply says to delete that file before reusing the name.
Choose another output name or move the old final file before starting again.
Both roles register the recorder for automatic track recordings (see tracks.md). Replay owns all seven movement keys, rejects other movement
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

With `wait`, the recorder replies `Settled <out>; waiting for rec continue.` after settling and
holds all keys released until `rec continue`. No input row or initialPlayer snapshot is captured
while waiting. The fixture driver uses this boundary for its second server start read. Ordinary
replay retains its automatic start. `rec continue` requires a settled, active replay.

## Recordings

`<gameDir>/pathtracing/recordings/<out or name>.jsonl` is UTF-8 JSONL with LF endings and no wall-clock
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

- `t`, the player's absolute `tickCount`, client-level `gameTime`, and `inputs` (the seven key booleans, `yaw`, `pitch`);
- `position` and `velocity`, each with `x`, `y`, `z`;
- `yRot`, `xRot`, `onGround`, `horizontalCollision`, `verticalCollision`, `isSprinting`,
  `isShiftKeyDown`, `pose`, `fallDistance`, `isInWater`, `onClimbable`;
- client `health`, `foodLevel`, `saturationLevel`;
- `ticksFrozen`, the client's frozen-tick count at END;
- `speedModifiers`, every client MOVEMENT_SPEED modifier except `minecraft:sprinting`, read at END;
- optional `anomaly`: an array of packets, in handler order, preserving multiple anomalies per tick.

`collisionSteps` observes each vanilla `Entity.collide` request and result, with raw vector bits only,
after sneak back-off and stuck processing. The read-only RETURN injection changes no movement.
Each `requested` and `result` has only `xBits`, `yBits`, `zBits`, all sixteen-digit hex strings.
`speedModifiers` is an array of `[id, amount, operation]` tuples. The namespaced id is a string,
amount is the attribute modifier's double value, and operation is its numeric vanilla id
(0 ADD_VALUE, 1 ADD_MULTIPLIED_BASE, 2 ADD_MULTIPLIED_TOTAL). Amounts use round-trip JSON numbers.

After the first row, `tickCount` must advance by exactly one per row. An offending row is retained
in the temporary output and the run aborts, replying with `t` and the previous/current tickCount.

Scalar doubles and ordinary vectors have sibling `<field>Bits` with sixteen lowercase hex digits. Floats (input angles,
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
`tickCount` and `gameTime`, including decimals, raw bits, inputs, on-ground and collision flags, pose, food values
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
with `Health`, `foodLevel`, `foodSaturationLevel`, `foodExhaustionLevel`, `foodTickTimer`, `TicksFrozen`,
`playerGameType`, `Pos`, `Motion`, `FallDistance`, `gameTimeBefore` and `gameTimeAfter`.
Vectors are numeric arrays. All entity fields come from one `data get entity` in one server tick.
Two `time query gametime` calls bracket that read; the SNBT reader respects nested compounds,
arrays and quoted strings. Vanilla omits `TicksFrozen` at zero. Restore fixture state, equipment,
effects and server resources before comparing replays;
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
- `inside` classifies `entityInside` alone. The base hook does nothing and exports `none`.
  An unmodeled override, such as sweet berry bush, exports `unmodeled`.
- `unsupportedFallOn` independently marks a landing override the law cannot replay. The exporter
  walks each hook to its declaring class. Slime, hay and beds export `inside=none` and
  `unsupportedFallOn=false`. Pointed dripstone exports `inside=none` and
  `unsupportedFallOn=true`. Its upward tip adds 2.5 to fall distance and uses multiplier 2.
- Old `inside=unmodeled` rows without `unsupportedFallOn` still refuse at either affected contact.
  Regenerate the world companions that hit that refusal, including 1.6c exports with affected
  slime, hay or bed contacts. The old flag combines the two hooks.
- `HayBlock` and `HoneyBlock` supply `fallOnMultiplier=0.2F`; beds and the default supply `1.0F`.
  `BedBlock` supplies `bed=true`; the law performs its halving. These hooks have no numeric getter. Slime and powder
  snow retain their separate law hooks. Bounce itself is read from the game, including beds/slime.
- `EndPortalBlock` and `EndGatewayBlock` supply `endPortalOrGateway`.
  `suffocating` comes from `state.isSuffocating(level,pos)` at the actual position.
- `LadderBlock` reads `FACING` as a vanilla 3D direction id. An open `TrapDoorBlock` reads its
  `FACING` into `openTrapdoorFacing`. Closed trapdoors omit that field.

The recording header now also contains `initialPlayer`, captured at START after replay settling
and before applying input zero. Header writing is deferred to this point. Stopping an empty or
aborted replay still writes its required metadata/trailer. Existing metadata and tick rows retain
their format. The snapshot has every player field accepted by the fixture README, including
attributes, fluid/eye flags, support position, stuck multiplier, jump delay and client food timer.
`movementSpeedBase` and `movementSpeedBaseBits` contain the MOVEMENT_SPEED attribute's double base
value, replacing `movementSpeed` and `movementSpeedBits`. The player line also has `speedModifiers`
in the same tuple format as tick rows. `speed` retains the observed float `getSpeed()` value.
Read-only mixin accessors expose private/protected state. Its tickCount is the pre-tick count.
Local FoodData exhaustion and timer are client observations, not server-authoritative values.
`ticksFrozen` is captured with `getTicksFrozen()`. Before every run the driver teleports onto the
dry start and waits for server TicksFrozen=0; replay settling also requires client ticksFrozen=0.
Vanilla `tryAddFrost` adds a MOVEMENT_SPEED ADD_VALUE modifier of -0.05 * percentFrozen while
ticksFrozen > 0 (subject to its vanilla supporting-block condition). Speed therefore changes
during a powder-snow recording. Server-driven state is an input the law reads from the recording.
The server runs frost removal, frost speed updates and thaw. The client receives attribute and
entity-data updates. Tick rows capture the resulting client modifiers and ticksFrozen at END,
so the reader consumes those observations at the recorded boundary.

Track recordings include the departure input: row `t=0` is the physics tick whose input left A,
observed by the detector at the next START. Its header is the state on A before that input.
The timer's departure sample is still T=0; a zero-T finish therefore has one recorded input row.
While armed, the recorder retains one candidate tick in memory. During a track run all rows stay
in memory until finish; discarded starts close and delete their reserved `.tmp` without writing rows.
Track filenames include the player's name so the two clients cache independent next indices.
Other control settings still allow timing; the run row explains a missing recording.

### Fixture driver

```powershell
.\lab\gates\record-fixtures.ps1 -Jar .\bot\build\pathtracing.jar
.\lab\gates\record-fixtures.ps1 -Courses ice,edges -Runs 8 -Ticks 400 -Seed 1
.\lab\gates\tests\test-record-fixtures.ps1
.\lab\gates\tests\test-courses.ps1
.\lab\gates\tests\test-fixture-json.ps1
.\lab\gates\tests\test-fixture-state.ps1
.\lab\gates\tests\test-exporter.ps1
```

The optional Jar installation has the same running-client hash guard as gate 1.2. An installed
job 1.6 jar is otherwise required. The driver takes the rig lock, refuses Julien's exact online
name, validates hold-style options and stops only its own PID/start-time-matched processes.
`fixture-rig.ps1` carries gate 1.2's bounded child-process transport and ownership handling.
Courses accepts an array or comma-separated names. Omitting it selects every course. Defaults
for runs, ticks and seed are in the driver. Course seeds use declaration index, so selecting a
subset preserves each course's input sequence for the same Runs/Seed.
`Ticks` counts the generated movement sequence. The driver appends 20 zero-input ticks, retaining
the final yaw and pitch, so each completed recording has `Ticks + 20` rows.

`courses.ps1` constructs enclosed patches on the superflat. Solid floors, walls and roofs contain
any walk and separate the classes. The start pad is plain stone. Edges use wider raised islands,
connecting strips and a return staircase from the safe catch floor. Only edges passes SneakPercent=90
to new-inputs; its default is unchanged. The water pool and flowing lip share a start-facing stone
rim placed before any water, with sources more than three blocks from the start. Offline flow tracing rejects unsafe starts.
Bubble columns are omitted
because the frozen fixture schema/law has no acceleration hook for them. The driver teleports
to load chunks, builds, waits for delivery, and verifies the last gold-block marker through a
full client export before replaying. Each run exports the inclusive floored position extrema of
both initialPlayer and post-tick rows, expanded by three blocks on each axis.
The full readiness export is deleted after checking and is not retained in results.
`corner` uses corner-inputs.ps1, never the random generator: yaw zero, forward+left, no jump/sprint,
from rest toward an isolated two-block-high stone column at local x=20,z=8, y=-60..-59.
The start remains local x=16.5,z=4.5. The report requires nonzero bit-equal |dx|=|dz| and exactly
one clipped horizontal axis in the same observed collision step. A missing witness rejects that
run with a `no witness` reason and makes the course fail. Both axes clipped fails the predicate.
No vanilla geometry produces the step candidate 0.6f + 1 ulp; that mutant is equivalent on the
game domain and the integrator replaces it at gate freeze. Nothing is built for that mutant.

Kept recordings are byte-for-byte copies. Their world companions contain one player row, the
unchanged client export and one serverEnd row. Both files are staged before publication. Existing
fixtures or staging files are refused. Rejections retain inputs, recordings, exports and command
logs under `lab/gates/results`, with per-course reasons and kept/rejected/not-run counts.

Before every run the driver teleports to the dry start, queries the bot's actual maximum-health
attribute, and waits for that health, foodTickTimer=0 and TicksFrozen=0. When health or the timer
is wrong it sends `effect give <bot> instant_health` and waits again. Run cleanup does the same,
including after aborts. The first resource snapshot precedes replay teleport and settling.
Replay pauses after settling. A second server snapshot must have bit-identical Health,
foodSaturationLevel and foodExhaustionLevel floats, and identical foodLevel and foodTickTimer.
Any difference rejects the run before `rec continue`. Initial health, food and saturation must
also match the settled client's snapshot. The world player's resources use the verified server
sample, including exhaustion and foodTickTimer.

The end sample is the first operation after Saved. `serverEnd.atRest` is true exactly when the
last recorded row is on ground with zero x and z velocity. `serverTickOffset` is the signed tick
distance from the last recorded gameTime to the server snapshot. Game-time queries use modulo
Integer.MAX_VALUE. The signed representative nearest zero preserves short negative offsets and
handles wraparound. Equal bracketing times give an exact offset. Different times give null with
`serverTickOffsetMin` and `serverTickOffsetMax` preserving the signed bracket.
The report counts kept serverEnd snapshots with atRest true and a non-null offset. The separate
reader advances a nonnegative offset as zero-input ticks only for those snapshots before comparing
resources. Negative offsets remain negative and cannot be replayed forward. Client gameTime is
synchronized from server packets, so equal clocks alone do not prove packet-level alignment.

The driver assumes the configured rig is the intended lab superflat, the bot is op, and the client
can load each entire patch. Existing courses at these reserved coordinates are replaced.
World exports describe the state after a run and assume movement-relevant terrain stayed static
during it. Readiness verifies loaded chunks and the final construction marker. Coverage of each
law hook and mutant is still judged by L0 against the recorded paths.

Reader integration belongs to parallel job 1.5e. The JSON smoke check copies the law parser and
checks the block, collisionSteps and serverEnd field shapes. The fixture-state check exercises
the actual driver predicates, including constant true/false witness mutants and each resource
field. The exporter check bootstraps the staged jar's registries offline and calls the actual
exporter for every block class named in course commands. It checks inside and landing
classifications separately, passes the slime, hay, bed, sweet berry bush and pointed dripstone
rows to Rust, and checks landing damage, bounce and explicit refusal. It also checks bits-only
collision vectors and all modifier operation tuples.
The same offline jar check applies both axis orders to the outer-corner column and requires
different horizontal results, using vanilla Shapes.collide for both orders.
An abort already closes the recorder and is followed by no rec stop. Each cleanup operation has
its own catch so failures are reported and later runs/courses are attempted. Original game mode
is restored before stopping owned processes and releasing the lock, including failure paths.
