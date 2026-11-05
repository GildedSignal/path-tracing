# Job 1.2e: one command that runs the frozen 1.2 gate live

You built the recorder (jobs 1.2 to 1.2d) on this branch, `wt/recorder`. Now write the driver that runs its frozen gate in the live rig, with Julien's client out of the game, driven over RCON. A runner will start it with one command and read only its report. Where something is ambiguous, take the most literal reading and write the assumption in your final message. Stop only for something that blocks the build. Do not commit; the integrator commits your changes. Do not run the game yourself: the rig is not yours to start in this job.

Read first: `bot/recorder.md`, the scripts in `lab/gates/`, `scripts/common.ps1` (`$Rig`, `Enter-Rig`, `Exit-Rig`, `Test-Port`, the ports and names), `scripts/start-server.ps1`, `scripts/launch-client.ps1`, `scripts/rcon.ps1`, and the frozen gate in `jobs/1.2-recorder/prompt.md` ("The gate the judge will apply").

## The script: `lab/gates/run-gate-1.2.ps1 [-Jar <path>] [-Runs <n>] [-Ticks <k>] [-Seed <s>]`

Windows PowerShell 5.1, ASCII only. Defaults: `-Runs 8`, `-Ticks 400`, `-Seed 1`.

1. **Take the rig lock** (`Enter-Rig 'gate-1.2'`) and release it in `finally`. It is the only gate script that holds the lock.
2. **Refuse to disturb Julien.** If his player (`$PlayerName`) is online (RCON `list`), stop with exit 3 and say so. Never touch his client.
3. **The bot's jar.** With `-Jar`, copy it into `$Rig\clients\bot\mods\pathtracing.jar`, but only while the bot client is not running; if it is running and the file differs, stop with exit 3 and say to close the bot client. Check that the bot's `options.txt` has `autoJump:false`, `toggleSprint:false`, `toggleCrouch:false` and `sprintWindow:0`, and stop if not (`scripts\setup.ps1` sets them).
4. **Bring the rig up.** Start the server with `scripts\start-server.ps1` if its port is closed, and wait for RCON. Start the bot client with `scripts\launch-client.ps1 -Role bot` if `PathBot` is not in `list`. Wait until it is listed and answers a command through `bot-command.ps1` (an unknown word returns its usage line; use that as the ping). Time out with exit 4.
5. **The course.** Teleport the bot over RCON to one fixed flat start, and put it in survival mode. The world is the lab's superflat world, with no other player near. Write the start into the report.
6. **Gate 1 (identical replays).** For each run i in 1..Runs, generate an input file with `new-inputs.ps1`, seed `Seed + i`, and `Ticks` ticks from the fixed start. Replay it twice, to two output names, each through `bot-command.ps1 -WaitFor` with a timeout scaled to the ticks. Then `compare-recordings.ps1` on the pair must say identical.
7. **Gate 2 (no anomalies).** `check-recording.ps1` passes on every recording from gate 1.
8. **Gate 3 (the dump).** Run `dump blocks` through `bot-command.ps1`, then `check-dump.ps1` on the result.
9. **Gate 4 (the negative control).** Start a recording, teleport the bot over RCON a few blocks away in mid-recording, stop it, and check that `check-recording.ps1` fails, naming an anomaly. Gate 4 passes only if the check fails for that reason.
10. **The report.** Write `lab/gates/results/<UTC timestamp>-gate-1.2/report.md`, and copy the recordings, the dump's summary line and every command's output beside it. One line per gate: `Gate N: PASS` or `Gate N: FAIL`, the reason, and the file that shows it. Then `Overall: PASS` only if all four pass. Print the report's path and the overall line last. Exit 0 on PASS, 1 on FAIL.
11. **Leave the rig as found:** stop the bot client and the server only if this script started them.

Add `lab/gates/results/` to `.gitignore` except a `README.md` you write there in three lines.

## Done means

- `bot\build.ps1` still builds.
- Every script in `lab/gates/` parses under Windows PowerShell 5.1 (`powershell -NoProfile -Command "[ScriptBlock]::Create((Get-Content <file> -Raw)) | Out-Null"`). ASCII only.
- Whatever you can test without the game, you test with stubs, and you say what remains for the live run.
- The final message lists every change and every assumption.
