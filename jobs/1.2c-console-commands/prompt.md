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

(added by the integrator after the review)

## Done means

- The build passes.
- Every script in `lab/gates/` parses under Windows PowerShell 5.1: run `powershell -NoProfile -Command "[ScriptBlock]::Create((Get-Content <file> -Raw)) | Out-Null"` for each.
- The final message lists every change and every assumption you made.
