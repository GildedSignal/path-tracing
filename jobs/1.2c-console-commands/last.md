Implemented on `wt/recorder`. Changes are uncommitted.

**Changes**

- `Bot.java` accepts null-sender/null-signature CHAT whispers from exactly `Rcon` or `Server` and logs every reply with `[pathtracing] `.
- Added `bot-command.ps1` for RCON commands and optional log-reply waiting.
- `Recorder.java` writes completion trailers, counts retained gap rows, and names blocking `.tmp` files with deletion instructions.
- Removed rig-lock acquisition/release from all four existing gate scripts.
- `check-recording.ps1` rejects missing, incomplete, misplaced or incorrectly counted trailers.
- `compare-recordings.ps1` compares trailers and every tick field except `tickCount`, reporting the first differing tick and field.
- `new-inputs.ps1` hand-builds the start JSON using invariant `R` formatting for doubles and float-rounded angles.
- Updated `bot/recorder.md` with these behaviors, RCON usage and the dump’s state-line counting rule.

**Validation**

- Build passed.
- All five gate scripts passed Windows PowerShell 5.1 parsing and ASCII checks.
- Local fixtures passed, including exact `0.30000000000000004` round-tripping, field mutations, trailer failures and caller-lock preservation.
- Log waiting passed with a local RCON stub. Live game behavior awaits integrator validation.

**Assumptions**

- Default reply timeout is 30 seconds. Regex matching covers the full log line, with the cursor captured immediately before sending.
- “First field” uses sorted field names and array-index order.
- Early cancellation retains the existing final-file behavior with `complete:false`. Aborts retain `.tmp` output and write a false trailer when writing remains possible.