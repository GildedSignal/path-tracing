# Job 1.7c: a track kit, survival on A, and his own inventory back at B

Julien (S34), after creating his first track: "when i stand on A it should switch to survival and look at what i have in my inventory besides the stick that way it is way easier to use tools and it should save it so that when i go onto it it restore that inventory and when i go to B i get back to creative and with my inventory before A".

On `wt/tracks`, now equal to `main` (`7e2a9a6`). Read `bot/tracks.md`, `bot/src/pathtracing/Tracks.java`, `TrackStore.java`, `TrackDetector.java`, `TrackUsage.java`. Make exactly these changes. Where something is ambiguous, take the most literal reading and write the assumption in your final message. Do not commit; the integrator commits. You don't run the game.

**What must not change.** The detector's start, re-arm and stop rules, and T. The bot's side: its toolkit stays "the items he used" (S28) and its preparation is unchanged. All of the below applies to human players only, in the client that runs the mod for its own player.

1. **The kit.**
   - A track's kit is a full inventory snapshot (main inventory, hotbar, armour and offhand, each slot with its complete item data), with every stick removed. It is stored in `lab/tracks/<name>.kit.json`.
   - The first time a human stands on A of a track that has no kit, his current inventory, minus sticks, becomes the kit. Chat says so.
   - `/track kit <name>` replaces the kit with his current inventory, minus sticks. It is refused during a run or while in track mode.
2. **Entering track mode** (standing on A while not in track mode):
   - Save his whole current inventory, including sticks, and his game mode to `lab/tracks/.session-<player>.json`. Write it before anything changes.
   - Set his inventory to the kit, then switch him to survival.
   - Restore his health, food and saturation to full.
3. **Each arming on A while in track mode** (including re-arming after a failed try): reset the inventory to the kit, and restore health, food and saturation to full. Every attempt then starts the same. Time on A is free (S27), so a short mode switch on A to apply the kit is acceptable. The run must start in survival with the kit.
4. **Leaving track mode** (touching B, abandonment such as death, a dimension change or falling 64 below, `/track cancel`, or a new `/track exit`): switch back to his saved game mode, restore his saved inventory exactly, then delete the session file. A finished run is recorded before the restore.
5. **Crash safety.** When he joins the server and a session file exists, restore it as in item 4, and say so in chat.
6. **How the inventory is set.** The client may set slots only through what the server accepts:
   - Creative-mode slot packets work only in creative. So apply a kit or a restore while in creative, and only then switch modes. On leaving, switch to creative, wait until the client sees creative, then restore.
   - Mode changes go through the `gamemode` command, which he can run as an op.
   - If a step fails (no permission, a refused packet), stop, leave the session file in place, and report in chat.
7. **Records.** Each human run line records the kit's SHA-256 (over its canonical JSON) and `gameMode`. `/track best` already separates modes.
8. **Docs.** Update `bot/tracks.md`: the kit, track mode, `/track kit`, `/track exit`, crash recovery. Keep it to at most 35 lines.

## Done means

- `bot\build.ps1` builds. Every script you touch parses under Windows PowerShell 5.1. ASCII only.
- The headless tests pass, with new tests for:
  - stick removal from a kit;
  - kit and session JSON round trips, including enchanted and damaged items, and items with custom names;
  - the track-mode state machine: enter, re-arm, finish, abandon, cancel, exit, crash recovery;
  - refusing `/track kit` during a run.
- `TrackMixinAudit` passes for every injection, new or changed.
- The final message lists every change and every assumption.

**Resumed run.** A previous run stopped mid-way ("model at capacity"). Its partial work is committed on `wt/tracks` as `91dcf5b`, and its tests fail. Continue from it and finish every item.
