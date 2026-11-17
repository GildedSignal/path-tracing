# Racing without friction: autocomplete, arming that just works, and good map spawns

**Julien (S194, verbatim):** "for the mod for the tracks make sure autocompletes work it's an not great to not have that feature oh yes and another quite frustrating issue is that i often have to type /track restore before it doesn't properly start them and also i have to wait for too long when on the block for it to properly load which is frustrating where you're restarting the track often or even worse finish it to then realize it did not count." And: "just make sure the spawn points are good."

**Consumer:** his racing on his maps. He races "massively in parallel" with the algorithm's build (S194), so every lost run is lost data.

**Read:**
- the mod: `bot\src\pathtracing\Tracks.java`, `TrackMode.java`, `TrackStore.java`, `TrackHumanClient.java`, `TrackDetector.java`, `Pointer.java`, `Maps.java`, `MapRegistry.java`;
- the map tooling: `scripts\import-map.ps1`;
- the rig's map list: `maps.json` at the rig root (`scripts\common.ps1`, `$Rig`).

## Decisions already made (do not reopen them)
1. **Autocomplete everywhere a name is typed.**
   - `/track delete|best|kit|run|play <name>` suggests the stored track names, filtered by what he has typed.
   - `/track new <name>` suggests nothing, since the name is new.
   - `/map <name>` suggests every map in `maps.json` plus `overworld`.
   - Use Brigadier suggestion providers. Add a test that the suggestions for a prefix are exactly the matching names.
2. **No `/track restore` needed, ever.**
   - Find why arming refuses until he types it. Write the root cause in `last.md`, with file:line.
   - Fix the cause. When a stale saved session exists (from a crash, a disconnect or an unfinished exit), the mod recovers it by itself through the existing recovery path, then arms. `/track restore` stays as a manual command, but a race never depends on it.
3. **Arming as fast as the server allows.**
   - Measure, in client ticks, the time from stepping on A to "armed", over at least 20 arming events in the headless track tests. Name each wait.
   - Remove every wait that is not a server confirmation: fixed delays, polling intervals, retries. What remains is the round trips the kit transaction needs. Report the before and after distributions.
   - No tolerance or new delay may be added.
4. **He always knows whether the run will count.**
   - While on A, the action bar shows the state continuously: "Arming…", "Armed: go", or "Not armed: <reason>".
   - If he leaves A while not armed, the chat says at once "This run is not timed: <reason>", at the moment he leaves, not at B.
5. **Good spawns for every map** in `maps.json`.
   - Read the spawn in both level.dat formats (old `SpawnX/Y/Z` and the newer spawn compound). `the_drop_2` currently reads 0,0,0, which suggests one format is missed.
   - Check the spawn in the map's own dimension: the feet and head cells must be passable, and the cell below must support a standing player.
   - If not, use the nearest standable cell in the same column, searching up first, then down. If the column has none, report the map and leave its spawn for Julien.
   - Read the region files read-only: through the engine's world reader for that dimension (`plan-track` already reads a track's dimension), or Minecraft's `NbtIo` in Java.
   - Write the corrected spawns to `maps.json`, and list each map's old and new spawn in `last.md`.
   - Run this against the real rig, read-only except `maps.json`, with the server stopped. Check `scripts\common.ps1`'s `Test-Port` first and refuse if it is running.

## Must not touch
`engine\law\`, `engine\words\`, `engine\native\`, `engine\plan\src\` (except a read-only spawn tool under `engine\plan\src\bin\` if you choose Rust); his tracks and runs; the server world's region files (read only). Launch no game, server or client. Never run `scripts\launch.ps1`, `gpt.ps1` or `follow-up.ps1`.

## Done means
- `bot\build.ps1` builds; `bot\tests\test-tracks.ps1` and `bot\tests\test-maps.ps1` pass; the new suggestion and arming tests pass.
- `maps.json` holds checked spawns.
- **`jobs\track-ux\last.md`** holds:
  - the restore root cause;
  - the arming time before and after, as distributions;
  - the spawn table;
  - what he should try in game.

End with `STATUS: DONE` or `STATUS: BLOCKED: <why>`.
