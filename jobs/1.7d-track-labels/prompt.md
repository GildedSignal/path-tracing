# Job 1.7d: track names floating above A

Julien (S36): "should have a tag or something to see the name of the tracks".

On `wt/tracks`, after job 1.7c. Do not commit; the queue runner commits. You don't run the game. If the worktree has uncommitted changes from an interrupted run of this job, continue from them.

1. In the human client, draw each track's name as floating text about 1.5 blocks above the top of its A block. Show the human's best time under it (ticks and seconds) if one exists. Draw it for tracks within render distance, the same set that gets the particles.
2. **Client-side only.** Nothing is added to the server's world: no entities, armour stands or text displays on the server, and no blocks. Use client rendering, for example a world-render callback, or text displays that exist only in the client's level.
3. The text faces the player, scales so it stays readable from about 32 blocks, and is visible through nothing (normal depth).
4. `/track labels on|off` toggles it, on by default, and the setting is remembered.
5. Update `bot/tracks.md` (at most 37 lines).

## Done means

- `bot\build.ps1` builds. The headless tests and `TrackMixinAudit` pass, including any new injection.
- The final message lists every change.
