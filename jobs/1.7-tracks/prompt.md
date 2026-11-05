# Job 1.7: parkour tracks with a timer

Julien (S25): "i could set up parcours in the superflat world by defining A and B for each parkour and when i leave A T start and when i touch B T stops, it should be as easy as possible for me to create multiple tracks, the goal being to gather what a human can do and hopefully surpass it but human is the minimum threashold".

His time on a track becomes the line the bot must reach (`docs/ladder.md`, the human baseline). So the timing must be exact, identical for him and for the bot, and creating a track must take him seconds. Where something is ambiguous, take the most literal reading and write the assumption in your final message. Stop only for something that blocks the build. Do not commit; the integrator commits. You don't run the game.

Read first: `bot/src/pathtracing/` (`Pointer.java`: the stick; `Bot.java`; `Recorder.java`; `PathTracingMod.java`), `bot/recorder.md`, `scripts/launch-client.ps1`, `scripts/common.ps1`. The same mod runs in his client and in the bot's.

## Creating a track (his client)

- A client command `/track new <name>`. The next two right-clicks with the stick set A, then B, as the clicked blocks, instead of sending a goal. Chat confirms each one. Sneak + right-click cancels.
- `/track list`, `/track delete <name>`, `/track best <name>`.
- A and B are shown in his view only, never placed in the world: client-side particles above each block of tracks within render distance, A and B in different colours. Nothing a track does may modify the world.
- Tracks are files: `lab/tracks/<name>.json` under the rig (`$Rig`, passed to both clients by `launch-client.ps1` as a system property), holding A, B, the creation time and the creator. Names are `[a-z0-9-]+`; refuse to overwrite.

## Timing (both clients, one detector)

- **Armed:** the player stands on A (A is its supporting block, on the ground).
- **Start:** the first client tick at which it is no longer standing on A. Landing back on A before reaching B re-arms the timer, so jumping in place never starts a run.
- **Stop:** the first tick at which the player's bounding box touches B's block (intersects its cell grown by 1e-3), whether standing on it or not.
- **T** is the number of ticks from start to stop. Also record the difference in the level's game time between the same two ticks, as a cross-check on client against server ticks. Report both if they differ.
- Show the running time on the action bar every tick. At the stop, chat shows T in ticks and seconds, and the track's best so far.
- Leaving the track without reaching B (for example falling far below, or `/track cancel`) discards the run.

## Records

- Every finished run is appended to `lab/tracks/runs/<name>.jsonl`: the player name, T, the game-time difference, the date, the client's hold-style settings (`autoJump`, `toggleSprint`, `toggleCrouch`, `sprintWindow`), the game mode, and the inventory summary.
- If the recorder can run (it requires the hold-style settings), record the run automatically from start to stop, to `recordings/track-<name>-<n>.jsonl`, and put that file's name in the run line. If it can't, time the run anyway, and note why it wasn't recorded.

## The bot

- A bot command `track run <name>`: teleport to A's top centre over RCON-style `tp` as `replay` does, settle, then `goto` B with the current planner. The same detector times it, and its runs go to the same file with its name. Report its latency from the command to its first non-empty input separately.
- **The same toolkit as Julien** (S28: "it should have in its inventory all the items that i used because if i use them it means that it was meant to test them"):
  - During each of his runs, record every item he *used*: placed, used (right-click), thrown, consumed or swung at a block, with counts. Write the list into the run line as `itemsUsed`.
  - Before `track run`, give the bot exactly the union of `itemsUsed` over his runs on that track, with each item's largest count in any single run. Use vanilla server commands over RCON (`clear`, then `give`); no server mod.
  - Say in the run line which toolkit the bot had, so a later change in his runs can't silently change an old comparison.
- **The same liberty as Julien** (S27): he may start sprinting or moving on A without leaving it, and the clock only starts when he leaves. So may the bot. Nothing in the bot's side may restrict its movement on A or start its clock earlier than the detector does.

## Done means

- `bot\build.ps1` builds. Every script you touch parses under Windows PowerShell 5.1. ASCII only.
- `bot/recorder.md`, or a new `bot/tracks.md`, says how to create, run and read tracks in at most 25 lines, written for Julien.
- Whatever can be tested without the game is tested (the detector's start, re-arm and stop rules on synthetic tick sequences).
- The final message lists every change and every assumption.
