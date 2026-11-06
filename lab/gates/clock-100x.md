# 1.8b frozen comparison

Run from wt/clock with Windows PowerShell 5.1. The reference rig must be stopped.

```powershell
$env:PATHTRACING_ROOT = 'D:\path-tracing-lab'
$env:PATHTRACING_SERVER_PORT = '25566'
$env:PATHTRACING_DEPS = 'D:\path-tracing-deps\data'
.\bot\build.ps1
.\lab\gates\clock-100x.ps1
```

- Use the 1.8 full-blocks, ice and edges course builder and input generator verbatim.
  Each course has 600 active and 200 coasting inputs. Preserve the generated inputs.
- For each course, replay at 20, 2000 and 4000 TPS in that order. Each replay starts a
  fresh bot JVM at server rate 20 and uses the 1.8 creative fixture setup. Both accelerated
  runs use that course's single fresh 20 TPS reference. All sessions use the same temporary
  bot setting `maxFps:260` (26.2's limiter-off value), vsync disabled and the existing
  follow-rate property enabled.
- Use the named lab rig as the source and hold its lock throughout. Run disposable copies of
  the lab server and bot from the evidence directory on the lab ports. Restore the copied server
  rate to 20 and stop owned processes before releasing the lock. Keep the original rig files
  and the play server out of the experiment.
- In each replay, bracket RCON game time with the last flushed client tick before and after
  the query. Use the 1.8 clock calculation verbatim, including at least three inside-replay
  samples, monotonicity and a nonempty intersection of client-minus-server offset intervals.
  Poll in process from command submission, pausing 5 ms after each pass. Use the first
  eligible bracket per consecutive observation of the same server tick. Keep repeated
  ticks and unusable brackets separately with their reason. A backwards sample is a refusal.
- Each run requires 800 consecutive checked rows and zero correction or impulse packets.
  Compare each accelerated recording against the 20 TPS reference bit-for-bit in all fields
  except the top-level absolute tickCount and gameTime. Require a valid lockstep 20 TPS
  reference for each course. Qualify 2000 only if all three pairs satisfy every check.
  Apply the same checks to 4000 and report it separately as stress. Refused and incomplete
  measurements are unqualified. Continue other cells when cleanup succeeds. No retries.
- Report requested rate, observed server/client TPS bounds, offset intersection, replay wall
  time from submission to recording publication as seen by the driver, and full-run wall time.
  Keep each raw recording including partial files, clock samples, comparator output and adverse
  result. Report the per-rate distributions alongside the individual cells.

The original 1.8 gate and its property-off control stay in `clock-gate.ps1`. Any diagnosis
after the matrix is separate from qualification.

To recompute the verdict of a completed matrix from its raw evidence, use
`.\lab\gates\clock-100x.ps1 -Review <evidence-directory>`. This writes a new report
without contacting the rig or editing the original evidence.

## Archived phase evidence

Run the read-only phase probe on the earlier sustained flat replay.

```powershell
$archive = 'D:\path-tracing-evidence\clock-100x\lab\gates\results\20261106T1348131110738Z-clock'
$samples = (Get-ChildItem -LiteralPath $archive -Filter '*.clocks.json').FullName
$recording = (Get-ChildItem -LiteralPath $archive -Filter '*true_2000.jsonl').FullName
.\lab\gates\probes\clock-phase.ps1 -Samples $samples -Recording $recording
```

The probe reports a pair of incompatible brackets, the direction and minimum size of
the phase change, and the separate distribution of steps in recorded client game time.
Client game time includes time-packet re-anchors and local extrapolation.

To inspect the client scheduler in the staged 26.2 jar, run `javap -p -c` on
`net.minecraft.client.Minecraft` and `net.minecraft.client.DeltaTracker$Timer`.
`advanceGameTime` computes whole tick demand using elapsed milliseconds and keeps the
fractional residue. `Minecraft` executes at most ten of those ticks per loop, discarding
any demand above that ceiling. The archived recording does not observe demand and
execution per loop, so it cannot attribute the phase loss to that mechanism.

## Focused fixture candidate

Run the same frozen matrix with -Focused to set the copied server view and
simulation distances to four chunks, and the copied bot's render distance to
four with a 480 by 270 window. The client's local simulation-distance option
uses the vanilla default of 12. The stock-speed and accelerated runs
use the same profile. The courses, input rows, comparison, clock calculation
and qualification stay the same. Compare each focused 20 TPS recording against
the standard-profile 20 TPS recording too. This profile is scoped to the static
32-by-32-block fixtures on the disposable rig.
