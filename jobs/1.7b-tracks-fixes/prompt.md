# Job 1.7b: the judge's findings on the parkour tracks

You built job 1.7 on `wt/tracks` (`00fc6d8`). A judge from another model family found it **ready with fixes**: `jobs/1.7-tracks/judge.md` in the main repository (`C:\Users\PASCAL\Desktop\path tracing`), finding numbers below. The timing core was judged correct; don't change the detector's start, re-arm and stop rules except where item 8 says so. Make exactly these changes. Where something is ambiguous, take the most literal reading and write the assumption in your final message. Do not commit; the integrator commits. You don't run the game.

1. **His line is his, not the bot's** (finding 1). The finish message and `/track best` show the best time per player, and `/track best` prints the human best and the bot best on separate lines (the bot is the player named by the bot's client, the human every other player). The ladder's rule is that his best time is the line the bot must beat, so the bot's times must never appear as his. Group by game mode as well, and show the mode next to each best.
2. **Discarded starts leave nothing behind** (finding 2). On a discard (re-arm, cancel, abandonment), close the recorder and delete its `.tmp`. Don't scan the recordings folder from index 1 on every start: keep the next index in memory after the first scan. A false start should cost no file I/O on the game thread beyond opening the recording.
3. **The toolkit counts what was needed, not swings** (finding 3, S28: the bot gets the items he used). An item whose stack size is 1 (tools, weapons, buckets, elytra) counts 1 per run, plus 1 each time one breaks. Stackable items count the units consumed, placed, thrown or dropped. A held attack is never counted per tick. Add tests: a 40-tick mining hold counts 1 pickaxe; 3 ender pearls thrown count 3.
4. **A wall clock beside the game-time cross-check** (finding 4). Also record `System.nanoTime()` at start and stop as `wallNanos` in the run line. In `bot/tracks.md`, say that the game-time difference is the client's own counter between time packets, so it rarely shows server drift.
5. **No crash from the tick handler** (finding 5). Wrap the body of `Tracks.tick` in try/catch: log, cancel the run, report once in chat, and keep the client running.
6. **A result is never lost** (finding 6). If appending to the run file fails for any reason, write the run to `runs/<name>.unsaved-<timestamp>.json` and say where in chat.
7. **Temp files never block a name** (finding 7). The temp file for a track definition gets a unique name and is deleted in a `finally`.
8. **Tracks that can finish** (findings 8 and 9). Refuse a B equal to A at creation. Stop is the first tick at which the player's box, swept from its position at the previous sample to its position now (the union of the two boxes), touches B's cell grown by 1e-3. That way a non-solid B can't be skipped at speed. Say this in `bot/tracks.md`.
9. **Smaller points** (finding 10).
   - Fix `suppressedUntil` so that `/track cancel` while standing on A doesn't re-arm until the player leaves A and lands back on it.
   - `/track new` refuses while a run is in progress instead of discarding it.
   - A bot run ends as abandoned after 20 times his best time on that track, or after 6,000 ticks if he has no time yet.
   - Before a bot run, set the bot's game mode to the mode of his best run, and record it.
   - The recording includes the departure tick's input (the tick whose input left A), so a replay from A reproduces the first tick. State in `bot/recorder.md` which row is tick zero.
   - Re-arming shows on the action bar only, never in chat.

## Done means

- `bot\build.ps1` builds. Every script you touch parses under Windows PowerShell 5.1. ASCII only.
- The headless tests pass, including new tests for items 1, 2, 3, 6, 7 and 8 (the swept stop: a 2-block-per-tick fall through a 1-block non-solid B is detected).
- `TrackMixinAudit` passes for every injection, new or changed.
- The final message lists every change and every assumption.
