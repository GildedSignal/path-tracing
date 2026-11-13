The walked-tube check changes the backward field, but **all three maze tracks still stalled** at the cap. The requested follow-up replays and the controlled momentum comparison are **not complete**, so these measurements do not yet decide whether cells are the bottleneck.

Probe 1 used mode 5, cap 1 and audit. Each entry is **set T at A / sentence T / expansions / thinking seconds**. `—` means no chain to A or no sentence, respectively. Sentence T is from the start state, excluding thinking.

| Track | Before | After |
|---|---:|---:|
| butoonpressureplates | 56 / 199 / 28 / 0.523 | 56 / 199 / 28 / 0.526 |
| carpetwater | 61 / 108 / 62 / 0.203 | 61 / 108 / 62 / 0.201 |
| fencemaze | 38 / — / 11,195 / 3.272 | 55 / — / 11,279 / 3.266 |
| fences | 55 / — / 15,627 / 4.414 | 55 / — / 15,791 / 4.414 |
| fencestraightbridge | 31 / 56 / 10 / 0.064 | 39 / 56 / 10 / 0.052 |
| flowers | 30 / 49 / 10 / 0.036 | 30 / 49 / 10 / 0.040 |
| grassjumpeasy | — / 50 / 179 / 0.176 | — / 50 / 179 / 0.169 |
| mangrovejumpeasy | — / 67 / 16 / 0.078 | — / 67 / 16 / 0.076 |
| redfakemaze | 35 / — / 15,572 / 3.513 | — / — / 13,099 / 3.519 |
| stonesimple | 57 / 85 / 16 / 0.064 | 57 / 85 / 16 / 0.070 |
| stonestaircase | 99 / 132 / 78 / 0.113 | 99 / 132 / 78 / 0.113 |

The check raises the set estimate on `fencemaze` and removes the chain to A on `redfakemaze`; it does **not** establish why the forward searches stall. I did not complete the required replay of each stalled joint’s claimed hop, including first departure tick and collision boxes.

For probe 2, the diagnostic found **no search node standing on the reported source cell** `[-11, -58, 4]`. It sampled three nearby airborne search nodes instead. One, at `[-9.7266, -57.8232, 5.2552]`, carried velocity `[-0.00154, -0.15234, -0.21110]`; the table below summarizes whether *any* of 64 aims landed on `[-11, -58, 2]` within 40 ticks from that state or from rest at the source-cell centre.

| Reach-table tools | Rest | Nearby airborne state |
|---|---|---|
| `FLjnsn`, `FLjnsa`, `FLSjnsn`, `FLSjnsa`, `Fjnsn`, `Fjnsa`, `FSjnsn`, `FSjnsa`, `FRjnsn`, `FRjnsa`, `FRSjnsn`, `FRSjnsa` | No | No |
| `FLjasn`, `FLjasg`, `FLSjasn`, `FLSjasg`, `Fjasn`, `Fjasg`, `FSjasn`, `FSjasg`, `FRjasn`, `FRjasg`, `FRSjasn`, `FRSjasg` | No | Yes |
| `FLjasa`, `FLSjasa`, `Fjasa`, `FSjasa`, `FRjasa`, `FRSjasa` | Yes | Yes |

For example, `FLjasn` at aim 32 landed from that airborne state with velocity `[-0.12949, -0.07840, -0.21700]`. The full per-aim landing velocities are in the diagnostic record. This shows that some real nearby states can reach the target, but **does not isolate momentum’s effect**: their positions and airborne status differ from the rest state, and the 40-tick test is not the reach table’s free-arc construction.

The offline flight command was run from `D:\path-tracing-wt\probes\engine`, once with `PLAN_RECORD` set to `D:/path-tracing-evidence/probes/tube` before the edit and once to `D:/path-tracing-evidence/probes/tube-after` after it:

```powershell
$env:CARGO_TARGET_DIR='D:/path-tracing-target/probes'
$env:PLAN_RECORD='D:/path-tracing-evidence/probes/tube-after'
$env:PLAN_TRACKS='fences,fencemaze,redfakemaze,butoonpressureplates,carpetwater,fencestraightbridge,flowers,grassjumpeasy,mangrovejumpeasy,stonesimple,stonestaircase'
$env:PLAN_MODES='5'
$env:PLAN_CAP='1'
$env:PLAN_AUDIT='1'
cargo test --offline --release -p plan --test flight the_flight_on_the_tracks -- --ignored --nocapture *> "$env:PLAN_RECORD/log.txt"
```

The momentum diagnostic used the same command with `PLAN_TRACKS='grassjumpeasy'` and `PLAN_RECORD='D:/path-tracing-evidence/probes/momentum4'`; its temporary instrumentation is saved in that folder’s `probe.patch`. Its diagnostic work was included in its reported thinking time, so use the flight table above for timing.

Records: [baseline flight](D:/path-tracing-evidence/probes/tube/flight.txt), [tube-check flight](D:/path-tracing-evidence/probes/tube-after/flight.txt), [momentum diagnostic](D:/path-tracing-evidence/probes/momentum4/flight.txt), and [inputs and limitations](D:/path-tracing-evidence/probes/inputs.txt). The normal record wrapper could not start because Windows denied its CPU-load query; the direct runs wrote the flight and log files.

The working tree retains only the two requested skip removals. No live run, commit, push, or merge was performed.