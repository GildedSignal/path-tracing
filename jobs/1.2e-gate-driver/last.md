Implemented and left uncommitted.

Changed:

- [run-gate-1.2.ps1](D:/path-tracing-wt/recorder/lab/gates/run-gate-1.2.ps1): lock, safety checks, startup, four gates, evidence report and owned-process cleanup.
- [test-run-gate-1.2.ps1](D:/path-tracing-wt/recorder/lab/gates/tests/test-run-gate-1.2.ps1): offline integration stubs.
- [.gitignore](D:/path-tracing-wt/recorder/.gitignore): ignores generated results.
- [README.md](D:/path-tracing-wt/recorder/lab/gates/results/README.md): three lines as requested.

Run:

```powershell
powershell -NoProfile -File .\lab\gates\run-gate-1.2.ps1 -Jar .\bot\build\pathtracing.jar
```

Verified: bot build passes; all seven gate/test scripts parse under Windows PowerShell 5.1 and are ASCII. Stub tests passed, including defaults, refusals, timeouts, failure detection and cleanup.

Assumptions:

- Fixed start: `0.5,-60,0.5,0,0`, on the prepared superflat course.
- Ports and names come from `common.ps1`.
- Ping uses `rec gate-ping`; unknown top-level words are silent.
- Comparison uses the existing helper, which ignores absolute `tickCount`.
- Health, hunger and inventory remain live.
- An already running, disconnected bot is awaited without launching a duplicate.
- Replay timeout is `60 + ceiling(Ticks / 10)` seconds; readiness deadlines are 120 seconds for RCON and 180 for the bot.

No live game was started. The four live gates remain for the runner.