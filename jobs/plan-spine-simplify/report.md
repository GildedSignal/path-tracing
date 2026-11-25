# Planner section spine

Base `537a70ac`, branch `wt/plan-spine-simplify`. The two expanded region fixtures and their backups were already in the clone when measurement began. Both revisions read those same fixtures.

## Design

- The repository has no reader of the inverse event map, its phase tables, or the lazy binade trace. `WordEvents` retains explicit marks and the gait span used by corridor cuts. `advance_cycles` samples two repetitions and skips through one binary64 chart at a time using integer mantissa steps. It does native additions at chart boundaries. The work follows binades, so a million-block command does not step through millions of ticks. The law's event face is appropriate when observing contacts. Its general state sampling and tick-wise lane advance would add distance-dependent work to the straight gait.
- The planner projects terrain into its ground layers and 16³ sections. A 4³ mask skips subcells with no differences, and collision masks skip cells whose shape cannot enter the queried box. Unknown cells remain candidates even in the halo. A row can carry an exact `StateId` even when the legs model is the same as the generator. Canonical collision models come from the law's shared table. Generic sources with no canonical model lend a block during ingestion, which the planner keeps once per classified row.
- Saved tracks call the snapshot's `uniform_state` for an exact state and model in one probe. Generic worlds use `World::uniform` to avoid repeated model reads and still query fine states when only the model is proved uniform. A source with no uniform proof falls back to cells in that box. The existing JNI source currently exposes the generic interface. The planner retains the owned constructor and uses the same section projection for native, saved and synthetic callers. The offline adapter needs the existing ground layer and partial-coverage rules for corridor certificates. The fixture snapshot remains the replay judge.
- `TopoField` stores T-to-go and reads the same sections as raw law ticks. `ObservedWorld` guards missing cells directly. The fast support proof queries the support plane through `World::uniform`, including when the law is wrapped in `WorldDelta`. The separate `LocalGrid` allocation, the unused source read counter and the temporary player state used to ask for a base label are deleted.
- The audit of `cut`, `store`, `solve`, `walk`, `flight`, `state_view` and `deviation` leaves the replay judge and the corridor bisection in place. The replay reads its pre-tick bounding box and tick directly. `StateAt` moves the phase handle it receives and carries only the geometry the goal consumes in the borrowed view.

## Lines

Counted with `scripts/engine-lines.pl` on raw sources and on a scratch copy formatted with `rustfmt --edition 2024`. Blank lines, comments and test lines are excluded from the code columns. Each decade column is `log10(before / after)`.

| File in `engine/plan/src/` | Raw before | Raw after | Raw decades | Formatted before | Formatted after | Formatted decades |
|---|---:|---:|---:|---:|---:|---:|
| `bin/deviation.rs` | 10 | 10 | 0.000 | 17 | 17 | 0.000 |
| `bin/plan-track.rs` | 61 | 61 | 0.000 | 127 | 127 | 0.000 |
| `certify.rs` | 293 | 293 | 0.000 | 329 | 329 | 0.000 |
| `cut.rs` | 178 | 178 | 0.000 | 265 | 265 | 0.000 |
| `defect.rs` | 2210 | 1933 | 0.058 | 2685 | 2397 | 0.049 |
| `deviation.rs` | 75 | 75 | 0.000 | 352 | 352 | 0.000 |
| `events.rs` | 473 | 33 | 1.156 | 1142 | 57 | 1.302 |
| `flight.rs` | 125 | 125 | 0.000 | 278 | 278 | 0.000 |
| `goal.rs` | 34 | 34 | 0.000 | 73 | 73 | 0.000 |
| `inline.rs` | 45 | 45 | 0.000 | 175 | 175 | 0.000 |
| `kit.rs` | 991 | 991 | 0.000 | 1392 | 1392 | 0.000 |
| `lattice.rs` | 255 | 255 | 0.000 | 558 | 558 | 0.000 |
| `lib.rs` | 24 | 24 | 0.000 | 24 | 24 | 0.000 |
| `solve.rs` | 510 | 479 | 0.027 | 840 | 789 | 0.027 |
| `solve_meter.rs` | 34 | 34 | 0.000 | 58 | 58 | 0.000 |
| `source.rs` | 259 | 504 | -0.289 | 742 | 1106 | -0.173 |
| `state_view.rs` | 19 | 19 | 0.000 | 120 | 97 | 0.092 |
| `store.rs` | 95 | 95 | 0.000 | 204 | 204 | 0.000 |
| `time.rs` | 10 | 10 | 0.000 | 12 | 12 | 0.000 |
| `walk.rs` | 67 | 67 | 0.000 | 127 | 127 | 0.000 |
| `word_store.rs` | 745 | 745 | 0.000 | 875 | 875 | 0.000 |
| **Total** | **6513** | **6010** | **0.0349** | **10395** | **9312** | **0.0478** |

The planner removes **503 raw code lines** and **1,083 after rustfmt**. Test code lines go from 124 to 112 raw and 326 to 270 formatted. The removed tests exercised the deleted inverse. The cycle test compares raw bits against native additions for 231 cases, including signed zero, subnormals and round-to-even ties.

## Evidence

Measurements are collected with the configured Rust toolchain, `--offline` and the job's dedicated target directory. The transport and server clock are outside this fixture run.

### Planning latency

Times are in microseconds and ordered **min / p50 / p90 / max**. The `real_time` series have 60 commands each. Mixed-ground measurements have 30 commands.

| Case | Base `537a70ac` | Section spine | Fermi floor | After median gap |
|---|---:|---:|---:|---:|
| New command, standing | 6.90 / 14.05 / 19.40 / 26.90 | 6.70 / 15.10 / 26.90 / 34.20 | 1–10 µs | 0.179 decades from 10 µs |
| New command, moving | 8.60 / 15.65 / 19.90 / 21.40 | 6.50 / 13.95 / 18.10 / 24.10 | 1–10 µs | 0.145 decades from 10 µs |
| Same command again | 5.10 / 5.20 / 5.30 / 9.20 | 4.90 / 5.00 / 6.70 / 7.80 | 1 µs | 0.699 decades |
| Mixed ground, full kit | 3.30 / 6.95 / 54.50 / 69.00 | 2.50 / 2.95 / 3.50 / 6.70 | One warm flat solve | |

All 182 flat commands match the baseline's T and final state bits. The 181 warm calls go from 168 allocated bytes each to **0**. Both builds prepare 0 law ticks on those calls and refuse 0 commands. The first call measured 229.9 µs and 11,771 bytes at the base, and 159.4 µs and 11,603 bytes on the section spine, with 15 preparation ticks in both samples.

The mixed-ground result uses one test thread. A default parallel run during the other checks measured 3.00 / 27.35 / 175.90 / 227.10 µs and hit the fixture's 25 µs timing assertion. The serial suite passes all five checks with the original assertions and both mixed-ground plans reach B at T=60. The first-use learning effort shifts on four shared-store tracks while the repeated, translated and rotated cases still report zero learning ticks.

### Fixture flights

Five paired runs per track use `plan-track --start settled --mode solve` and the saved overworld fixture. T here is the fixture's last-on-A clock. First touch is counted from the settled start. All 16 baseline and changed scripts have matching SHA-256 hashes, including the input and predicted state bits at each tick. The scratch outputs and raw samples are in `D:/path-tracing-target/job-plan-spine-simplify/verified-flights/`.

| Track | T | First touch | First solve µs, base → section spine | Repeat µs, base → section spine |
|---|---:|---:|---|---|
| `grassjumpeasy` | 38 | 38 | 42989 / 43942 / 52881 / 52881 → 27558 / 27890 / 30437 / 30437 | 555.3 / 561.6 / 759.7 / 759.7 → 355.8 / 357.4 / 594.7 / 594.7 |
| `flowers` | 48 | 48 | 128 / 132 / 134 / 134 → 123 / 132 / 135 / 135 | 3.4 / 3.4 / 3.5 / 3.5 → 3.8 / 3.8 / 3.9 / 3.9 |
| `stonestaircase` | 99 | 99 | 43026 / 43975 / 45087 / 45087 → 30941 / 31280 / 33158 / 33158 | 731.5 / 742.7 / 767.5 / 767.5 → 454.1 / 455.8 / 461.7 / 461.7 |
| `fences` | 111 | 111 | 144435 / 145388 / 151991 / 151991 → 100708 / 100846 / 113006 / 113006 | 1817.8 / 1832.7 / 1883.5 / 1883.5 → 1398.1 / 1422.3 / 1517.5 / 1517.5 |
| `fencestraightbridge` | 46 | 46 | 39507 / 39932 / 41062 / 41062 → 34190 / 34523 / 36215 / 36215 | 920.5 / 929.7 / 989.9 / 989.9 → 759.3 / 773.6 / 795.0 / 795.0 |
| `fencemaze` | 68 | 73 | 416075 / 418730 / 462407 / 462407 → 296455 / 307108 / 357340 / 357340 | 1037.0 / 1050.8 / 1382.4 / 1382.4 → 814.1 / 823.7 / 970.8 / 970.8 |
| `mangrovejumpeasy` | 53 | 55 | 44151 / 44665 / 55959 / 55959 → 31926 / 32297 / 44877 / 44877 | 699.6 / 704.3 / 1079.3 / 1079.3 → 445.9 / 449.4 / 756.8 / 756.8 |
| `redfakemaze` | 61 | 61 | 52011 / 54323 / 78098 / 78098 → 34469 / 34598 / 43009 / 43009 | 862.0 / 882.3 / 1309.1 / 1309.1 → 632.0 / 648.7 / 1019.5 / 1019.5 |
| `carpetwater` | 95 | 95 | 125726 / 127271 / 148516 / 148516 → 84614 / 86486 / 87896 / 87896 | 1379.5 / 1393.6 / 2219.3 / 2219.3 → 1064.6 / 1075.3 / 1083.2 / 1083.2 |
| `butoonpressureplates` | 79 | 83 | 34361 / 34759 / 49949 / 49949 → 20648 / 20724 / 21359 / 21359 | 874.2 / 890.9 / 952.2 / 952.2 → 650.2 / 657.8 / 848.2 / 848.2 |
| `trapdoorparkour` | 122 | 126 | 98206 / 100624 / 108706 / 108706 → 66192 / 66448 / 67648 / 67648 | 1302.3 / 1323.4 / 1335.7 / 1335.7 → 725.9 / 728.4 / 745.9 / 745.9 |
| `doorparkourblue` | 86 | 90 | 53711 / 53924 / 55058 / 55058 → 31737 / 31831 / 32308 / 32308 | 1111.4 / 1118.2 / 1147.5 / 1147.5 → 634.0 / 640.7 / 647.7 / 647.7 |
| `stonesimple` | 85 | 85 | 126 / 127 / 129 / 129 → 121 / 125 / 131 / 131 | 5.4 / 5.5 / 5.7 / 5.7 → 5.5 / 5.6 / 5.6 / 5.6 |
| `straight100` | 288 | 288 | 124 / 126 / 138 / 138 → 125 / 128 / 129 / 129 | 5.8 / 5.9 / 5.9 / 5.9 → 4.7 / 4.8 / 5.5 / 5.5 |
| `straight300` | 845 | 845 | 132 / 135 / 148 / 148 → 131 / 137 / 139 / 139 | 6.5 / 6.6 / 6.7 / 6.7 → 5.4 / 5.5 / 5.5 / 5.5 |
| `straight800` | 2248 | 2248 | 136 / 140 / 145 / 145 → 134 / 140 / 150 / 150 | 7.1 / 7.2 / 7.2 / 7.2 → 5.8 / 6.0 / 6.0 / 6.0 |

Across the full 16-track ladder, the sum of first-solve time is **1104.2 / 1127.1 / 1202.1 / 1202.1 ms → 761.3 / 782.2 / 842.9 / 842.9 ms**. The sum of process time including snapshot load and export is **2087.8 / 2133.9 / 2215.0 / 2215.0 ms → 1712.6 / 1790.5 / 1837.2 / 1837.2 ms**.

### Judge results

| Gate | Result |
|---|---|
| `plan --lib` | 5 passed, 0 failed. Cycle fold compares 231 cases at 0 ULP. |
| `real_time` | 1 passed, 0 failed. All 182 T and end-bit records match the base. |
| `generalize` | 5 passed, 0 failed. Full suite 1.36 s at the base and 0.97 s in the default run after the change. Serial full suite 1.17 s. Five exact 11-track runs measured 0.94 / 0.95 / 0.97 / 0.97 s against the 1.4 s ceiling. |
| `hard_track` | 5 passed, 0 failed. Witness first touch T=583 with the same digest `d0f6b1ddbc5cacc8`. The base suite took 0.21 s on one sample. Five default runs measured 0.22 / 0.22 / 0.23 / 0.23 s. |
| `kit_unfold` | 5 passed after rerun with default test scheduling. Mixed-ground distribution 2.60 / 4.55 / 76.20 / 187.10 µs. Serial run 5 passed at a 2.95 µs median. |
| `deviation` | 33 passed, 0 failed, 5 existing ignored. |
| `pathtracing_native` | 4 passed, 0 failed. |
| `world` | 21 passed, 0 failed. |
| `law --lib` | 95 passed, 0 failed. |
| Plan binaries and tests | Offline compile passes with and without `solve-counters`. |

The configured local Git staging operation returns `Unable to create '.git/index.lock': Permission denied`. The verified changes are in the requested clone at the original base commit. The code and map diff is also at `D:/path-tracing-target/job-plan-spine-simplify/plan-spine-simplify.patch` and passes `git apply --reverse --check` against this worktree.

STATUS: BLOCKED: The sandbox denied writes to `.git/index.lock`, so the verified slices could not be committed locally.
