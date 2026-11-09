# 1.8c frozen throughput and movement comparison

Run from the clock worktree in Windows PowerShell 5.1. The integrator runs
the live command after stopping the lab server and lab bot.

```powershell
$env:PATHTRACING_ROOT = 'D:\path-tracing-lab'
$env:PATHTRACING_SERVER_PORT = '25566'
$env:PATHTRACING_DEPS = 'D:\path-tracing-deps\data'
.\bot\build.ps1
.\bot\tests\test-full-tick-demand.ps1 -UseRigMods
.\lab\gates\tests\test-clock-full-demand.ps1
.\lab\gates\clock-full-demand.ps1
```

- Take the lab rig lock and reuse the 1.8b focused profile and disposable rig
  on the lab ports. Keep the copied server at 20 TPS between cells. Restore
  the rate, stop the owned processes and release the lock on exit. Launch a
  fresh bot JVM per cell, with the existing rate follower and full tick demand
  both enabled. The normal bot launcher opts in via `-FullTickDemand true`.
- Use the exact 1.8 course builder and 800 controls for full-blocks, ice and
  edges. Run each at 20, 2000 and 4000 in order. Check 800 consecutive,
  complete rows and zero correction or impulse packets in every cell.
  Compare both accelerated cells against that course's fresh reference with
  the existing comparator and 1.8b's byte comparison of tick rows after
  excluding the two absolute counters `tickCount` and `gameTime`.
- Measure each cell with the 1.8b bracketing and 1.8 clock calculations.
  Save observed server and client TPS bounds separately. Report the old offset
  intersection, including empty intersections, as a diagnostic at all rates.
  A missing or refused rate measurement remains a failure of evidence.
  Report replay wall time from dispatch through observed publication, full
  session time and per-rate distributions. Also classify each rate interval
  against its setting as below, includes target, or reaches target.
- After the matrix, use the same full-blocks start and 800 controls followed by
  11,200 copies of the last idle control, at 2000 TPS in a fresh bot JVM.
  Check all 12,000 rows and compare the first 800 to the fresh 20 TPS reference.
  The scheduler audit covers whole frames intersecting the replay. Count timer
  demand, completed calls to the vanilla tick and their difference. Measure
  active time from before application of the first control through the flush
  of the last row. Report 12,000 divided by active seconds and by dispatch-to-
  publication seconds, as TPS and multiples of 20. Record a client JFR profile
  for this sample and keep its raw file for bottleneck analysis. Report rate
  distributions over successive non-overlapping observation windows that span
  at least 1000 client ticks, omitting an incomplete trailing window.
- The 2000 movement verdict requires all three pairs to qualify. The 4000
  stress verdict applies the same rules independently. A measured 100x
  client result requires the 2000 movement verdict, a complete valid sustained
  sample, zero dropped demand and at least 2000 executed replay ticks per
  active wall second. Report whether the end-to-end rate also reaches 2000.
  An observed server interval below its setting is reported separately.
  The runner returns failure if the 2000 result or driver fails, and reports
  the stress verdict separately. Keep raw samples, recordings including any
  partials, inputs, comparator logs and failures. Continue after a red cell
  if cleanup succeeds. No retries.
- The offline control applies Fabric's actual Mixin transformation and forces
  a vanilla timer to return above ten. The transformed loop must execute ten
  ticks with the opt-in off and the whole demand with it on. Audit the
  option-off, player, pause, freeze and missing-level paths.

## Diagnosing a failed result

The result directory includes the client JFR, its replay start time and active
duration, clock brackets, raw recordings and server log. Check the scheduled,
executed and dropped totals first. A throughput miss with zero dropped demand
needs evidence from inside the replay window, including the client render
thread's sampled stacks, GC or file stalls and the observed server rate.

```powershell
. .\scripts\common.ps1
& "$Jdk\jfr.exe" summary "<evidence-directory>\sustained-client.jfr"
& "$Jdk\jfr.exe" print --stack-depth 40 --events jdk.ExecutionSample,jdk.NativeMethodSample,jdk.GarbageCollection,jdk.FileWrite,jdk.ThreadPark "<evidence-directory>\sustained-client.jfr"
```
