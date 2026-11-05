# Job 1.2c: commands from the server console, and the judge's findings on 1.2b

You built jobs 1.2 and 1.2b on this branch, `wt/recorder`. This job has two parts. Design choices are not yours: where something below is ambiguous, take the most literal reading and write the assumption in your final message. Stop only for something that blocks the build. Do not commit; the integrator commits your changes.

Build dependencies are staged at `D:\path-tracing-deps\data`, and `PATHTRACING_DEPS` points there. Run the build with `powershell -ExecutionPolicy Bypass -File bot\build.ps1`.

## A. Commands from the server console (approved by Julien)

Gates must run while Julien's client is out of the game. Today the bot obeys only whispers from its owner. It must also obey a whisper sent from the server console or over RCON.

Facts, checked in the 26.2 jar:
- `/msg PathBot <text>` from a non-player source builds `PlayerChatMessage.system(...)`. `OutgoingChatMessage.create` turns a system message into `OutgoingChatMessage$Disguised`, so the client receives disguised chat.
- On the client, Fabric's `ClientReceiveMessageEvents.CHAT` fires for it with `sender == null` and `signed == null`. The decorated text is `<name> whispers to you: <text>`.
- The name is `Rcon` over RCON (`RconConsoleSource.RCON = "Rcon"`) and `Server` from the console's stdin.
- Player chat always carries a non-null `sender`.

What to change, in `Bot.java`:
1. In the `CHAT` handler, when `sender == null` and `signed == null`, match the existing `WHISPER` pattern against `message.getString()`. Run the command only if the name is exactly `Rcon` or `Server`.
2. Leave the owner's path unchanged.
3. Replies to a console command must still be readable without Julien's client. Keep the whisper reply to the owner as it is, and also write every reply to the client log with a fixed prefix, `[pathtracing] `, one line per reply.
4. Add `lab/gates/bot-command.ps1 -Text <command> [-WaitFor <regex>] [-TimeoutSeconds <n>]`. It sends `msg PathBot <command>` through `scripts\rcon.ps1`. With `-WaitFor`, it tails the bot client's log, `$Rig\clients\bot\logs\latest.log` (`$Rig` from `scripts/common.ps1`), for a `[pathtracing] ` line matching the regex, printed after the command was sent. It prints that line and exits 0, or exits non-zero on timeout. Same rules as the other gate scripts: Windows PowerShell 5.1, ASCII only.

Update `bot/recorder.md` to say how a gate drives the bot over RCON.

## B. The judge's findings on 1.2b

A judge from another model family reviewed commit `a6e3d7a` and passed it with fixes. Make these changes:

1. **The rig lock (required).** None of the scripts in `lab/gates/` takes the rig lock (`Enter-Rig` / `Exit-Rig`); the gate driver that calls them holds it. Remove the lock from all four existing scripts, and don't add it to `bot-command.ps1`.
2. **The dump's line count.** The frozen 1.2 gate says the dump's line count equals the registry's state count. That means state lines, excluding the header and summary lines. `check-dump.ps1` is the gate's check; say both in `bot/recorder.md`.
3. **A truncated recording.** Every recording ends with a trailer line `{"type":"end","ticks":N,"complete":true|false}`, where N is the number of tick rows. `complete` is true only when a replay ran its whole input file, or a recording was stopped by `rec stop`. An early replay cancel writes `false`. `check-recording.ps1` exits non-zero on a missing trailer or `complete:false`, and `compare-recordings.ps1` compares the trailers too.
4. **A stale `.tmp`.** When a `.tmp` blocks a name, the reply names the file and says to delete it.
5. **What a comparison covers.** `compare-recordings.ps1` compares every field of each tick row except `tickCount`, which is absolute: the raw bits, the inputs, `onGround`, the collision flags, `pose` and the food values. It reports the first tick and the first field that differ.
6. **Exact start doubles.** `new-inputs.ps1` writes the start's doubles with `.ToString('R', [Globalization.CultureInfo]::InvariantCulture)` and its floats as `[single]` with the same format, building that JSON line by hand instead of `ConvertTo-Json`. Check that a start of `0.30000000000000004` round-trips.

## Done means

- The build passes.
- Every script in `lab/gates/` parses under Windows PowerShell 5.1: run `powershell -NoProfile -Command "[ScriptBlock]::Create((Get-Content <file> -Raw)) | Out-Null"` for each.
- The final message lists every change and every assumption you made.
