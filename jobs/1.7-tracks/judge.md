# Judge: job 1.7 tracks (commit 00fc6d8, branch wt/tracks)

**Verdict: ready with fixes.**

The timing core is correct and identical for both clients. Nothing writes to the world, and the bot's RCON commands address only the bot's own name. The build passes and the 69 headless checks pass (I ran both; I had to add `-ExecutionPolicy Bypass`). The worktree stayed clean. I did not run the game, so everything below that depends on vanilla behaviour is from reading the code and from vanilla knowledge, and is marked where it matters.

The problems are in what surrounds the detector. The "best" figure mixes the bot's times with his. Every false start litters the disk with a `.tmp` file. Tool counts will break the bot's toolkit. The game-time cross-check is not independent of the client clock.

Line numbers refer to `bot/src/pathtracing/` in the worktree.

## Findings, most severe first

### 1. "Best" mixes the bot's runs with Julien's. This corrupts the baseline the whole job exists to produce.
`TrackStore.java:127-136` (append, `best`) and `Tracks.java:111-114` (`/track best`) take the minimum over every run of the track definition. The report says so ("Best time includes human and bot runs").
- Scenario: Julien's best is 100 ticks. The bot runs and finishes in 80. `/track best x` now says 80, and so does the "Best ..." in his next finish message. He or a later agent reads 80 as the human line. The bot's baseline becomes the bot's own earlier result, and the ladder's rule "human is the minimum threshold" is silently broken.
- The file keeps the player name, so nothing is lost on disk. The surfaced number is wrong.
- Fix: `best` and the finish message should filter on `player` equal to the finishing player. For example, `append` returns the min over runs by the same player, and `/track best` prints "human best" and "bot best" on two lines.

### 2. Every discarded start leaves a permanent `.tmp` recording, and the scan grows with them.
`Tracks.java:223-240` (begin starts a recorder on every START), `:193-197` (REARM discards), `Recorder.java:175-206` (`finish(mc,false)` writes header and end line but never moves or deletes the `.tmp`), and `Recorder.java:73-76` (`recordTrack` index scan).
- Scenario: he stands on A and jumps in place, or steps off the edge and back on. That is START, then REARM, many times. Each cycle creates `recordings/track-<name>-N.jsonl.tmp` with CREATE_NEW and files it away forever. Hundreds of attempts give hundreds of files.
- `recordTrack` restarts from index 1 on every call and does two `Files.exists` per existing file. The START tick then does O(n) file-system calls on the game thread, growing with every false start.
- Every cycle also opens a file and writes on the game thread in the departure tick.
- The report discloses the leftovers ("Discarded partial recordings retain .tmp files"). It is a bad call, and the spec asks for the opposite: "jumping in place never starts a run".
- Fix: on a discard, delete the `.tmp` (`discardTrack` should close the writer and `Files.deleteIfExists(temporaryPath)`). Optionally start the recorder lazily or reuse the index across calls.

### 3. Tool and attack counts are per tick, so the bot's toolkit can be absurd or refused.
`mixin/TrackAttackMixin.java:28-36` counts at the `swing` call inside `continueAttack`.
- From vanilla knowledge, not run: `continueAttack` swings every tick that the mouse is held on a block that is being mined, in both survival and creative.
- Scenario: a 2-second hold with a pickaxe is about 40 swing events. `itemsUsed` records `minecraft:iron_pickaxe: 40`. `toolkit()` takes the per-item maximum and `checkCapacity` (`Tracks.java:~333`) needs 40 slots for non-stacking tools. That exceeds 36, so `track run` refuses with "toolkit exceeds 36 slots". If it is under 36, the bot is handed 30 pickaxes.
- The spec says "the same toolkit". For a tool or weapon, "swung" should be one use per contiguous attack burst, or per block broken.
- Fix: count `continueAttack` swings only once per burst. Track "attack key held" and add 1 on the rising edge. Cap tool kit counts at the item's stack size, or mark non-stackables as 1.

### 4. The game-time cross-check is not independent, so it will almost never show the server drift it was asked to catch.
`Tracks.java:189` reads `mc.level.getGameTime()`.
- From vanilla knowledge, not run: the client level advances that value by itself once per client tick and corrects it only when a time packet arrives, about once a second.
- Scenario: the server runs at 30 TPS for a 4-second run. The client's game time still advances one per client tick, so `gameTicks == T` and no discrepancy is reported. Only a resync landing inside the run shows a difference, and then it is a jump of a few ticks, not a measurement.
- This is what the spec asked for ("difference in the level's game time"), so it is not a deviation. It is a weak check and the docs oversell it.
- Fix: additionally record `System.nanoTime` wall-clock at START and STOP. Or have the bot's RCON read `time query gametime` before and after, though that is not the same moment. At least say in `tracks.md` that the figure is client-local between time packets.

### 5. One exception in `Tracks.tick` crashes his client mid-run.
`Tracks.java:164-210`: the whole tick body, including `begin`, `finish`, `DETECTOR.sample` and `markers`, has no try/catch. Fabric tick events are not caught.
- Scenario: any unforeseen RuntimeException on a state path. The client crashes, the run is lost, and the world is not harmed but the session is.
- Fix: wrap the body of `tick` in try/catch, log the error, call `cancel(mc, true)`, and report once.

### 6. A torn or corrupt line in a run file blocks every later save, and the result then exists only in chat.
`TrackStore.java:107-112` (`readRuns` throws on a bad line) and `:127-132` (append calls it). The test "Corrupt history is preserved" confirms the refusal.
- Scenario: power loss or a kill between write and fsync leaves half a JSON line. From then on, every finish prints "Track result could not be saved ... Result {json}" and `/track best` fails. The record exists only in the scrollback.
- Refusing is defensible. Having no fallback is not.
- Fix: on append failure, write the run to `runs/<name>.unsaved-<timestamp>.json`.

### 7. A failed track create can leave a `.json.tmp` that blocks the name forever.
`TrackStore.java:66-68`: `CREATE_NEW` on `<name>.json.tmp` with no cleanup on failure or crash.
- Scenario: the write dies after creating the temp file (disk full, crash). A later `/track new <name>` fails with a bare `FileAlreadyExistsException` path, and `list()` hides the file, so he cannot see why.
- Fix: delete the temp file in a `finally`, or use a unique temp name and `deleteIfExists` before create.

### 8. A track whose A and B are the same cell can never finish.
`Tracks.click` (`~Tracks.java:128-146`) does not refuse `cell == first`. The detector gives `onA` precedence, so it re-arms instead of stopping when B overlaps A. B directly adjacent to A is also unfinishable while the player stands on A.
- Fix: refuse B equal to A at creation. Say in the docs that B is touched only after leaving A.

### 9. Stop detection checks the box only at tick starts. It can skip a non-solid B at speed.
`Tracks.java:218-221`. From vanilla knowledge, not run: a player can move more than one block in a tick (falling, elytra, riptide), and a non-colliding B (flower, rail, water, carpet is solid so not this one) can be crossed between two samples.
- The spec's literal reading is satisfied, since the box is checked "at the tick". The risk is only for unusual B blocks.
- Fix, if wanted: test the union of the previous and current box, or document that B must be a collidable block.

### 10. Smaller points
- `Tracks.java:36,183,288`: `suppressedUntil = clock + 1` is dead. Next tick `clock == suppressedUntil`, so `clock < suppressedUntil` is false. Effect: `/track cancel` while standing on A re-arms on the next tick. Harmless, but the intent is not achieved.
- `Tracks.java:94-104`: `/track new` calls `cancel`, which silently discards a run in progress.
- `Tracks.java:~254-262` (`startBot`) and the bot flow have no timeout. If the planner can't path, or the bot gets stuck, a started run continues until the player falls below 64 blocks or someone sends `stop`. The planner's wall time is also invisible to T, because ticks don't advance during a blocked game thread. This is reported separately as latency, as specified.
- No check that the bot's game mode matches Julien's. The run line records the mode, so a creative-flight human run and a survival bot run would still be compared.
- Recording starts at the START tick, one tick after the tick whose input left A. The departure input (the jump press) is not in the recording, so a replay from A cannot reproduce the first tick. The spec says "from start to stop", which is what it does, but this is worth knowing before replay is built on it.
- Chat on his client: a "re-armed" line prints on every false start. Bouncing on A in front of a track spams his chat.
- `Pointer.java:27`: `keyUse.setDown(false)` runs on every stick right-click. This is harmless and the stick already swallowed all right-clicks before this change (goto). It is not new interception.

## Checked and found sound

1. **Timing semantics.** `TrackDetector.java` implements the spec exactly. Start is the first sample not on A. Landing back on A re-arms and zeroes the clocks. Stop is the first sample touching B, and A takes precedence over B when both are true. T is `tick - startTick`, so the departure tick is 0 and there is no off-by-one at start or stop. Game time is differenced over the same two samples. A and B crossed in one sample give STOP with T=0, and `Tracks.tick` then calls `begin` first, so the run line is complete.
2. **Sampling point.** Both clients sample at START_CLIENT_TICK on the same code path. Departure and arrival are both seen one tick after the physics tick that caused them, which cancels in the difference. Client lag cannot lose or double a tick: `clock` counts executed ticks, so a stall delays them in wall time but does not change T.
3. **Bot and human use the same detector, in the same `tick`.** The bot only differs by how `selected` is chosen (forced after settling) and by the explicit arming sample in `settle`, which requires the exact A top-centre, supporting block A, no horizontal velocity and the exact toolkit. Nothing in the bot restricts movement on A or starts its clock early. The bot's `goTo` is called only after arming.
4. **World safety.** No code places or breaks anything. The mixins only observe. Markers are client particles. RCON runs `clear`, `give` and `tp` against `mc.player.getName()` of the bot's own client, validated by regex, so it cannot address Julien, and the human client refuses `track run`. The toolkit is capped at 36 slots so `give` cannot drop item entities into the world. Credentials stay inside `TrackRcon`, and an auth failure message contains no password (tested).
5. **File handling.** Overwrite is refused under a cross-process lock, and the temp file is moved without REPLACE. Run appends are locked, force-synced and truncated back on a failed write. A two-process race test loses no lines. Track names are validated against path escape.
6. **Mixins.** `TrackMixinAudit` reads the compiled mod jar and the Minecraft 26.2 client jar and checks the method name, parameter list, callback type, static flag and INVOKE call-site for all 18 injections. That makes a mixin crash at startup unlikely. I ran it.
7. **Spec items done as written.** Commands, stick click selection with sneak to cancel, two colours of particles, the file layout with dimension/creator/creation time, run line fields (player, T, game-time difference, date, hold-style settings, game mode, inventory, `itemsUsed`, toolkit for bot runs, recording name or reason), separate latency, the `$Rig` property in `launch-client.ps1`, and `tracks.md` at 25 lines.
8. **Tests (item 6).** The headless run compiles the real `TrackDetector.java`, `TrackUsage.java`, `TrackStore.java` and `TrackRcon.java` from source, not copies. The detector cases assert exact tick counts (2, 7, 0), so a start or stop off-by-one would fail them. The AABB and `onGround` conditions in `Tracks.onA` and `Tracks.touchesB`, the tick ordering and the `begin`-on-STOP path in `Tracks.java` are not covered. They need the game.
9. **Not found.** No chat spam beyond finding 10, no per-tick file I/O (the reload is once per second and tiny; the recorder flushes one small line per tick only while a run is active), and no particle load (one dust particle per marker every 5 ticks).
