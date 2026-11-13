Implemented the frozen-counter server events for L0.

- The recorder captures packets for the bot’s own `DATA_TICKS_FROZEN`, including their value and whether they arrived between ticks, before the block-effects pass, or after it.
- The L0 verifier applies those overwrites and checks the recorded START and END counters. The law’s arithmetic, comparison fields, pass line, and mutants were left unchanged.
- The offline synthetic sequence `1, 2, 4, 4, 6, 7` passed; changing the tick 111 packet value by one was refused.

From `engine/`, `cargo test --offline` passed, with six tests ignored because their external recordings or oracle inputs were unavailable. The mod build passed. The standalone L0 verifier also compiled and passed both frozen-counter controls.

The integrator must run the live gate on a quiet machine. Use a **fresh target** so preparation captures the changed recorder and verifier:

```powershell
$env:PATHTRACING_DEPS = 'D:/path-tracing-deps/data'
$job = '.\oracles\1.6g-terrain-corpus'
$target = 'D:/path-tracing-target/1.6g-freeze'
$reference = 'D:/path-tracing-lab/server'

& "$job/prepare.ps1" -Target $target -ReferenceServer $reference
& "$job/run.ps1" -Mode Preflight -Target $target -Rig 'D:/path-tracing-evidence/1.6g/fast' -Port 25567
& "$job/run.ps1" -Mode Audit -Target $target -Rig 'D:/path-tracing-evidence/1.6g/stock' -Port 25568
& "$job/run.ps1" -Mode Full -Target $target -Rig 'D:/path-tracing-evidence/1.6g/full' -Port 25569 `
  -PreflightCapture '<preflight report directory>' -AuditCapture '<audit report directory>'
```

Live packet capture and mixin execution remain unverified until that run. I did not run the live gate, take the rig lock, modify `docs/`, or commit.