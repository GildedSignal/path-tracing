# 1.6g terrain corpus

Frozen inputs are in [manifest.ps1](manifest.ps1), [courses.ps1](../../lab/gates/courses.ps1) and [movement-controls.tsv](movement-controls.tsv). The integrator owns all live commands. Each copied rig starts at stock speed, and only its fenced replay window uses the selected lab rate.

| Stage | Cases | Inputs per case | Target rate |
|---|---|---|---|
| Preflight | Three variants of the mixed route, in order | 64 controls and 20 released-key ticks | 2,000 TPS |
| Audit | The same three variants on a fresh copy | Same inputs | 20 TPS |
| Full | All 14 existing courses, eight runs per course, in declaration order | 400 controls and 20 released-key ticks | 200 TPS |

The mixed run starts on ice, crosses the z=4112 chunk seam onto bottom slabs under a ceiling, then drops from an ice lip into water flowing from the far end of the channel. The reverse pulse lands at tick 22, 23 or 24. Jump is held from tick 28 through 45. Each variant requires observed ice support, an upward step with the body crossing the seam, a center crossing on the slab, an upward collision clipped under the ceiling, dry support on the ice lip and water contact in a cell with observed nonzero flow. The verifier prints the tick of each contact.

Full retains the existing random control generator and seed formula `1 + courseIndex * 8 + run - 1`. The corner uses the existing deterministic tie generator. The archived full-blocks and ice passes, the 1.6f landing captures and all four planted movement faults keep their existing verdicts and criteria. The archived edges recordings and injected teleport still take the strict refusal path. Archive fixtures without the new observations are controls or retained refusals according to their original gate. A missing observation in a new capture refuses that capture.

## Pass line

- Exactly the frozen controls, including the rest tail and starting angles, must be applied to a settled survival player. Every recorded tick must match position, velocity, angles, fall distance, movement flags, pose and every collision request and result bit for bit. Read server-driven speed modifiers and the freeze counter at START and compare speed and the freeze counter at END. A client tick gap, server correction, unsupported state or missing evidence is a refusal.
- Client exports before and after replay must agree over the replay footprint. The footprint includes a three-block margin and is inside the loaded pre-run envelope. A block or chunk change in the footprint during the fenced interval refuses the static-world fixture.
- Each externally applied motion event needs its own accepted landing move, base `fallOn` witness, effective fall damage, same-source mark, consume and dispatch on the witnessed player connection. Wire and client observations must agree in order, identity and decoded bits. The staged 26.2 codec must reproduce each wire payload. The client handler must apply it between ticks, after its own accepted landing, with the expected before and after velocity. Unknown, ambiguous or additional external motion refuses the case.
- A separate server resource timeline uses the same identity and server fences. Apply its ordered exhaustion, effective damage, regeneration and FoodData inputs from the authoritative start. Compare raw health, food, saturation, exhaustion and food timer after every regeneration and FoodData tick and at seal. Match the effective damage order to the verified landing witnesses. A missing event or checkpoint is never filled from the end snapshot.
- The accelerated client must report the configured rate and both lab clock options. It must execute above 1,000 recorded ticks per recorder second at the 2,000 TPS target and above 100 at the 200 TPS target. Report the distributions of client recorder rate and Rust replay rate, the slowest witnesses and the separate gate wall time.
- Full requires the preflight, stock audit, every full case and the original movement controls. Any shorter run has preflight or reduction status.

## Integrator commands

Prepare three stopped disposable copies of the same reference rig as in [1.6f](../1.6f-landing-motion/gate.md), with disjoint port pairs and fresh PathBot player data. Add `terrain-corpus.disposable` beside the two existing marker files. Set both server distances to four chunks. In each copied bot's options set `renderDistance:4`, `simulationDistance:12`, `overrideWidth:480`, `overrideHeight:270`, `maxFps:260` and `enableVsync:false`. Keep the existing hold-option checks and close other bot instances.

Use Windows PowerShell. Preparation writes the frozen hashes, staged lab client, probe and verifier before live use. The lab clock comes from `388a1bc` and is assembled inside the target directory.

```powershell
$env:PATHTRACING_DEPS = 'D:/path-tracing-deps/data'
$job = '.\jobs\1.6g-terrain-corpus'
$target = 'D:/path-tracing-target/1.6g-live'
$reference = 'D:/path-tracing-lab/server'
if (-not (Test-Path -LiteralPath $target)) {
  & "$job/prepare.ps1" -Target $target -ReferenceServer $reference
}
& "$job/run.ps1" -Mode Preflight -Target $target -Rig 'D:/path-tracing-evidence/1.6g/fast' -Port 25567
& "$job/run.ps1" -Mode Audit -Target $target -Rig 'D:/path-tracing-evidence/1.6g/stock' -Port 25568
& "$job/run.ps1" -Mode Full -Target $target -Rig 'D:/path-tracing-evidence/1.6g/full' -Port 25569 `
  -PreflightCapture '<preflight report directory>' -AuditCapture '<audit report directory>'
```

The driver verifies each case before continuing. On refusal, keep the result directory and the first `*-verify.log` or capture diagnostic. For a movement mismatch at tick `t`, rerun the immutable recording with the verified prefix of length `t`, then with the first failing row included. Keep the adjacent rows, requested input, pre/post worlds and raw server and client timelines together. Run the same route at stock speed to separate clock, transport and law failures. Reduce resource and motion failures at the first event sequence or wire ordinal in the diagnostic. Record the boundary before extending coverage.

```powershell
& "$job/verify.ps1" -Mode Preflight -Capture '<preflight report directory>' -Course mixed-seam -Run 1 `
  -Prefix <number-of-ticks> -Target $target -ReferenceServer $reference
```

Without `-Prefix`, the verifier requires the full case. Omit both `-Course` and `-Run` to verify a complete stage from saved evidence. The full verdict also requires the prerequisites and archived controls that `run.ps1` checks.
