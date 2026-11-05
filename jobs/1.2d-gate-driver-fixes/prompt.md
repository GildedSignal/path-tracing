# Job 1.2d: the judge's findings on 1.2c

You built jobs 1.2 to 1.2c on this branch, `wt/recorder`. A judge from another model family reviewed 1.2c (`a6e3d7a..4539beb`) and passed it with fixes. Make exactly these changes. Where something is ambiguous, take the most literal reading and write the assumption in your final message. Stop only for something that blocks the build. Do not commit; the integrator commits your changes.

Build dependencies are staged at `D:\path-tracing-deps\data` (`PATHTRACING_DEPS`). Build with `powershell -ExecutionPolicy Bypass -File bot\build.ps1`.

1. **A failed command must not cost the whole timeout.** Add `-FailOn <regex>` to `lab/gates/bot-command.ps1`.
   - When a `[pathtracing] ` line printed after the command matches it, print that line and exit 2.
   - Its default matches every failure reply the bot can give today (for example "already exists", "Usage", "Aborted", "Refusing", "Recorder/dumper:"). Read `Bot.java`, `Recorder.java`, `TickInput.java` and `BlockDumper.java` for the full list, and list them in `bot/recorder.md`.
   - Success stays exit 0, and a timeout stays its own non-zero code. Document the three exit codes.
2. **Clean output.** `bot-command.ps1` discards what `scripts\rcon.ps1` prints, so its stdout holds only the matched line, or the failure line.
3. **Cancelled runs are never "identical".** `compare-recordings.ps1` exits non-zero unless both trailers say `complete:true`, and says which file was incomplete.
4. **Exact pitch in every row.** `new-inputs.ps1` writes every row's yaw and pitch with the same `R` formatting as the start line, never through `ConvertTo-Json`. Check that a pitch of `12.3456789` (as `[single]`) round-trips.
5. **Accepted risk, documented.** In `bot/recorder.md`, state that an operator, or a command block or entity named `Rcon` or `Server`, can also command the bot through `/msg`. That is accepted because those already require operator rights.

## Done means

- The build passes.
- Every script in `lab/gates/` parses under Windows PowerShell 5.1: run `powershell -NoProfile -Command "[ScriptBlock]::Create((Get-Content <file> -Raw)) | Out-Null"` for each. ASCII only.
- The final message lists every change and every assumption.
