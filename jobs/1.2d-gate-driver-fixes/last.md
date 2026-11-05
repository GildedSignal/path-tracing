Implemented in four files. Changes are uncommitted.

- [bot-command.ps1](/D:/path-tracing-wt/recorder/lab/gates/bot-command.ps1) adds `-FailOn`, suppresses RCON output, and uses exits 0 for success, 1 for timeout/helper errors, and 2 for matched failures.
- [compare-recordings.ps1](/D:/path-tracing-wt/recorder/lab/gates/compare-recordings.ps1) requires boolean `complete:true` and identifies incomplete files.
- [new-inputs.ps1](/D:/path-tracing-wt/recorder/lab/gates/new-inputs.ps1) writes every row’s yaw and pitch using invariant `R` formatting.
- [recorder.md](/D:/path-tracing-wt/recorder/bot/recorder.md) documents failure replies, exit codes, formatting, completion requirements, and the accepted `/msg` operator risk.

Build passed. All five gate scripts passed Windows PowerShell 5.1 parsing and ASCII checks. Fixture tests passed, including bit-exact `[single]12.3456789` pitch round-tripping.

Assumptions made are that failure matching applies when `-WaitFor` is supplied, failure takes precedence over success, and send-only calls return silently. Reply checks used mocked RCON/log fixtures.