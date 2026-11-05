# Job 1.8: the bot's client follows a faster server tick rate

**Why.** Principle 26: everything should shorten the feedback loop. Live gates were meant to run under `/tick rate` so each stays under 60 seconds. But in the 26.2 client, `Minecraft.getTickTargetMillis(float)` returns `Math.max(base, level.tickRateManager().millisecondsPerTick())` while the tick rate runs normally (read with `javap -c -p` on `minecraft-26.2-client.jar`). The client follows a slower server, never a faster one. The player's movement is computed by the client, so under `/tick rate 100` the world runs 5× but the bot stays at 20 ticks per second.

**Branch.** Cut from `main` after `wt/fixtures` is merged (the integrator creates the worktree). Do not commit; the integrator commits. Don't run the game; the integrator runs the live gate with the script you write.

## Make exactly these changes

1. **A mixin** in the bot mod, registered in `bot/pathtracing.mixins.json`. On `Minecraft.getTickTargetMillis`, when the property below is on and the level's tick rate runs normally, return the server's `millisecondsPerTick()` even when it is below the base. In every other case, return exactly what vanilla returns.
2. **Off by default.** It is active only when the JVM property `pathtracing.followTickRate=true` is set. `scripts/launch-client.ps1` sets it for the bot role only. Julien's client loads the same mod and must behave exactly as vanilla.
3. **Nothing else changes.** No change to movement, input handling, networking or rendering. If something else turns out to depend on wall time and breaks at a faster rate, report it. Don't patch it.
4. **The live gate script, `lab/gates/clock-gate.ps1`.** The integrator runs it. It takes the rig lock and uses the existing recorder and courses (`git show main:bot/recorder.md`, `scripts/courses.ps1` from `wt/fixtures`).
   - For 3 courses (flat sprint-jumping, the ice course and the edge course), it replays the same input sequence at 20, 100 and 200 ticks per second (`/tick rate`), with the property on.
   - It records each replay and compares every tick row bit for bit against the 20 tick-per-second replay.
   - It counts server corrections.
   - It reports the achieved rate of client ticks against server ticks over each run (from game time and the client's tick count), and the highest tested rate at which the two stay in lockstep.
   - It restores `/tick rate 20` in every exit path, including failures.
   - A second check: with the property off, the client's ticks per second at `/tick rate 100` stay at 20.
5. **An offline test:** the mixin returns the vanilla value whenever the property is off, the level is null, or the tick rate is not running normally.

## Done means

- `bot\build.ps1` builds. Every script you touch parses under Windows PowerShell 5.1. ASCII only.
- The offline test passes.
- The final message lists every change, and every place in the client's code you saw that reads wall time during a tick (for the integrator to watch in the live gate).
- **Pass line for the live gate**, frozen now:
  - bit-identical tick rows at every rate where client and server stay in lockstep;
  - zero server corrections;
  - the highest lockstep rate reported.
- A rate where they fall out of lockstep is a limit to report, not a failure.
