# Judge: track-ux (Sonnet, native sub-agent, 17 November)

**Verdict: PASS WITH FINDINGS.** Saved by the integrator from the judge's final message.

## Tests, from the worktree under PowerShell 5.1
- `bot\build.ps1`: exit 0.
- `bot\tests\test-tracks.ps1`: exit 0. It passed 417 inventory and track-mode checks, 36 mixin injections against 26.2 and 110 headless track checks.
- `bot\tests\test-maps.ps1`: exit 1 as written, because the worktree has no `server\.fabric\server\26.2-server.jar`. With `-ServerJar` pointing at the rig's jar (read-only): exit 0, 22 offline map checks. Pass `-ServerJar` when re-running it.

## Items
1. **Autocomplete:** PASS. Suggestions on delete, best, kit, run and play; none on `new`; `/map` lists the maps plus overworld; prefix filtering in `NameSuggestions.java`.
2. **No /track restore:** PASS. The root cause is real in the old code: after a failed kit, the selection was cleared while `TrackMode` stayed FAILED with its session, and the early return blocked every later selection. `TrackHumanRace.java:56-60` rolls the failed kit back once and latches the refusal. `TrackHumanRace.java:33` recovers a stale session on the first tick. The stuck case is reproduced by a test and now arms.
3. **Arming time:** PASS WITH FINDINGS. The waits removed were not server confirmations, and no delay, tolerance or retry was added.
   - Fresh arming: median 44.5 → 26 ticks.
   - After a crash: median 64.5 → 46 ticks.
   - Repeated stick resets: 9 ticks.
   - *Finding (low):* the distributions come from a deterministic headless packet simulation. Its samples repeat in cycles, so they count round trips, not live latency.
   - *Finding (low to medium):* the track list is now re-read from disk on every client tick (`Tracks.java:261`), where it used to be every 20. That's correct, but it adds disk reads to the tick. A file watcher, or reloading on contact with A, would be cleaner. Not measured.
4. **Action bar and the chat line on leaving A unarmed:** PASS (`TrackHumanRace.java:76-79`, `:41-45`).
5. **Spawns:** PASS. Only two spawn fields change:
   - `diversity_parkour`: y −64 → 0. A spawn at −64 has nothing below it. Julien should glance at it in game.
   - `frosted_ices`: y 84 → 83.

   `the_drop_2`'s 0,0,0 is explicit legacy data, and that cell is standable.
6. **Must-not-touch:** PASS.

**Runs that could be lost:** none found. A refused kit restores his inventory and says why. One friction remains, by design: after a refusal he steps off A and back on before the next try.
