# Yarn-cold fold

The behavior-first pass builds on the verified Slice 2+3 commit `288eceb7` on `wt/yarn-cold-fold`. Its source changes are in the working tree for the integrator. The earlier canonical state and cold-cell cuts are at `212434c3` and `288eceb7`.

## What runs now

- Pistons and shulker lids use a floating progress clock and directional box primitives. The broad and narrow piston queries use the same swept boxes. The piston retains entity-specific overlap, the travel cap and the retracting-base correction. The lid retains the whole-slab push distance. Geometry shares the 0.01 push padding, the axis-only expansion that keeps untouched signed zeros, face slices, one-sided contraction and a clearance query over collision shapes and entities. The piston installer uses one construction of the moving state and retains the write and notification order.
- Hoppers, bamboo, tripwire and pressure plates read columns from one box primitive. Button and hook shapes fit the same mounted rectangle to the support face. Piston extraction, dismount floor height and shape clipping read the same occupied lattice. Ray clipping consumes the ordered cells without the intermediate vector.
- Free-body motion has one gravity and drag basis with a movement callback. The falling-block oracle uses gravity, move, drag. Pearls trace with post-drag velocity. Arrows move with the entry velocity and sample water on both sides for drag before final gravity. Items keep their fluid buoyancy, rest cadence, ground response and shared gravity and friction operations.
- The boat passes two floors and its exit column to one allocation-free pose scan. The scan keeps floor evaluation order and standing, crouching, swimming preference. The current mount and minecart code has no dismount scan. Mount and boat aim reuse the same angle conversion with the jar's per-call operation order. The minecart's three rail entry points share its current-position and below-rail selection.
- Directional signals reduce by one ordered neighbor scan with an explicit ceiling and optional excluded side. Levers, buttons, plates and hooks share binary strength and support-face projection. Scheduled sensor updates retain the canonical writes and delays banked in Slice 2+3. Stair placement shares inside and outside guards, and mining shares the packet and witness progress expression over the wrapped tick count.
- Combat and jar-facing callers share critical and sweep decisions. Entity shapes, pearl flight and wind charge bounds share dimension construction. Wind charge offsets and piston displacement use the same direction vector. Portal radius ranking uses the existing squared-distance primitive while preserving integer subtraction before widening.

No new world state is required. The 19 existing witness files and their expected bits have zero diff. The hot movement source and `basis::sweep` have zero diff. No live flight was run in this pass.

## Fermi meter

| Stage | Raw law code | Rustfmt law code |
|---|---:|---:|
| `537a70ac` | 18,738 | 23,417 |
| `212434c3` | 18,508 | 23,194 |
| `288eceb7` | 17,594 | 22,372 |
| Behavior-first working tree | 17,266 | 22,211 |
| **Deleted since checkpoint** | **328** | **161** |
| **Deleted since baseline** | **1,472** | **1,206** |

Against the 7,920-line floor, the whole-law raw gap is now 0.3385 decades, down from 0.3740. The baseline-to-current reductions are 0.0355 decades raw and 0.0230 after rustfmt. The earlier 2,500-line target still needs 1,028 raw and 1,294 formatted lines. The five measured primitive footprints shrink by 0.0496 raw and 0.0345 formatted decades. They remain short of the earlier 0.25-decade target.

The work removed repeated scans and arithmetic and left the different inputs and event order where the witnesses require them. Full-file line footprints include unique behavior. Separate pulse timing by cell, restitution, medium-dependent move order, and the portal's frame search remain supported behavior. The 7,920-line floor remains the comparison.

Each family below uses a fixed set of source files across all three columns. Mixed files count once, under their largest relevant family. The free-motion footprint grew as the motion callback moved projectile phase handling into the shared basis. Raw counts use `scripts/law-lines.pl`. The same meter ran after `rustfmt --edition 2024` on a scratch copy.

| Family | Raw baseline | Raw checkpoint | Raw final | Raw decades | Rustfmt baseline | Rustfmt checkpoint | Rustfmt final | Rustfmt decades |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Moving boxes and ramps | 2041 | 1928 | 1885 | 0.0345 | 2252 | 2135 | 2175 | 0.0151 |
| Free motion and drag | 1262 | 1278 | 1295 | -0.0112 | 1513 | 1540 | 1555 | -0.0119 |
| Dismount and rider motion | 1393 | 1393 | 1342 | 0.0162 | 1452 | 1452 | 1412 | 0.0121 |
| Signal and delayed updates | 6761 | 5704 | 5477 | 0.0915 | 7259 | 6365 | 6193 | 0.0690 |
| Contact and impulses | 2289 | 2289 | 2264 | 0.0048 | 2534 | 2534 | 2529 | 0.0009 |

## Differential against the checkpoint

| Probe | Comparisons |
|---|---|
| Directional geometry | 14,406 box sweep inputs including signed zeros, 168 animation steps, 36 attached button shapes and six directional hook and wind charge cases. |
| Piston displacement | 672 candidate push decisions and 480 simulated ticks in 96 scenes covering all six directions, normal, slime and honey bodies, bedrock, and server players. |
| Ballistic flight | 32,768 steps across pearls, arrows, falling blocks and items. The probe hashes output coordinates and velocities by IEEE bits. |
| Dismount clearance | 8,192 exits against a floor, a wall and a partial-height floor. |
| Attack gates | 2,048 decisions across combinations of flags and finite, zero and NaN inputs. |
| Mixed cell trace | 192 worlds, 4,608 tick records, 576 lever pulls and 645,120 state reads. |
| Plan command outcomes | The recorded final state bits, refusals and T for all 182 commands match `537a70ac`. |

The checkpoint and candidate produce identical probe digests and identical mixed cell traces. The old geometry probe exposed 3,276 signed-zero differences in 12,348 cases when using full-axis expansion in place of the axis-only sweep. The retained axis-only operation matches the checkpoint.

## Required suites

| Suite | Passed | Failed | Ignored |
|---|---:|---:|---:|
| `world` | 21 | 0 | 0 |
| `law --lib` | 95 | 0 | 0 |
| `law --test jar` | 21 | 0 | 0 |
| `law --test replay` | 31 | 0 | 5 |
| `plan --test kit_unfold` | 5 | 0 | 0 |
| `plan --test generalize` | 5 | 0 | 0 |
| `plan --test hard_track` | 5 | 0 | 0 |
| `plan --test real_time` | 1 | 0 | 0 |
| `pathtracing_native` | 4 | 0 | 0 |

The suites passed 188 cases with five ignored. The first replay run exercised 220,000 synthetic ticks while the recordings path was unset. The final run loaded the recording corpus, replayed all 18 required tracks byte-identically, and checked 227,887 event-face ticks including 7,887 recorded ticks. The saved recordings and region fixture were read in place.

The final serial kit run measured 3.45 µs median, 4.1 µs p90 and 7.8 µs max over 30 paired mixed-ground queries. Both kits arrived at T = 60. Two earlier runs used parallel tests and exceeded the 25 µs ceiling at 51.35 and 31.05 µs median. The serial timing makes the test independent of the other kit tests running at the same time. The hard track arrives at T = 583 with digest `d0f6b1ddbc5cacc8`.

The final command suite checks 60 queries per series. The first command was 138.3 µs.

| Command | Fermi floor | p50 | p90 | p99 |
|---|---:|---:|---:|---:|
| New goal, standing | 1–10 µs | 14.4 µs | 23.0 µs | 36.5 µs |
| New goal, moving | 1–10 µs | 13.1 µs | 18.3 µs | 28.5 µs |
| Repeat | 1 µs | 5.2 µs | 5.2 µs | 8.7 µs |

## Paired hot-path verification

The read-only `537a70ac` archive and the final source snapshot were built at same-length paths with the judge's 64-byte alignment flags. The benchmark `.text` hashes to `4AAFF7126BCC4E2C6866029D92CD968150DDF582ED71E5150303AEB1F759A97B` on both sides. The `.rdata` section differs. The other eight loadable section hashes match. The pinned benchmark alternates baseline-first and candidate-first after the workspace tests.

Each case passed the 1.005 median limit. The Windows process used high priority and affinity mask `10` for 12 interleaved rounds. Each side contributed 84 samples per case.

| Case | Baseline median [IQR], ns/tick | Candidate median [IQR], ns/tick | Paired median ratio [min–max] |
|---|---:|---:|---:|
| walk | 966.7 [964.5–968.3] | 968.0 [966.2–969.3] | 1.000890 [0.999204–1.005760] |
| sprint-jump | 632.4 [631.5–633.5] | 632.4 [630.9–634.8] | 1.000380 [0.995022–1.010076] |
| ceiling | 885.4 [884.3–886.5] | 884.9 [883.8–886.7] | 1.000102 [0.997686–1.004807] |
| bumpy | 930.4 [929.4–932.6] | 930.5 [929.1–934.3] | 1.000737 [0.991846–1.014708] |

## Per-file law code meter

| File under `engine/law/src` | Raw baseline | Raw checkpoint | Raw final | Rustfmt baseline | Rustfmt checkpoint | Rustfmt final |
|---|---:|---:|---:|---:|---:|---:|
| `attributes.rs` | 230 | 230 | 230 | 313 | 313 | 313 |
| `basis/body.rs` | 117 | 133 | 171 | 280 | 307 | 339 |
| `basis/cell.rs` | 479 | 498 | 498 | 913 | 869 | 869 |
| `basis/clock.rs` | 17 | 17 | 22 | 47 | 47 | 52 |
| `basis/mod.rs` | 5 | 5 | 5 | 5 | 5 | 5 |
| `basis/ray.rs` | 166 | 166 | 166 | 357 | 357 | 357 |
| `basis/sweep.rs` | 137 | 137 | 137 | 336 | 336 | 336 |
| `bin/bench.rs` | 30 | 30 | 30 | 51 | 51 | 51 |
| `block_hooks.rs` | 883 | 770 | 693 | 892 | 775 | 705 |
| `boat.rs` | 690 | 690 | 616 | 725 | 725 | 662 |
| `breath.rs` | 55 | 55 | 55 | 68 | 68 | 68 |
| `codec.rs` | 272 | 272 | 272 | 564 | 564 | 564 |
| `codec_planes.rs` | 68 | 68 | 68 | 123 | 123 | 123 |
| `codec_proof.rs` | 145 | 145 | 145 | 339 | 339 | 339 |
| `codec_state.rs` | 68 | 68 | 68 | 88 | 88 | 88 |
| `combat.rs` | 393 | 393 | 382 | 428 | 428 | 434 |
| `core.rs` | 755 | 746 | 746 | 1733 | 1716 | 1716 |
| `dismount.rs` | 0 | 0 | 39 | 0 | 0 | 39 |
| `effects.rs` | 203 | 203 | 203 | 282 | 282 | 282 |
| `elytra.rs` | 427 | 427 | 427 | 432 | 432 | 432 |
| `entity_contact.rs` | 625 | 625 | 618 | 624 | 624 | 617 |
| `explosion.rs` | 200 | 200 | 193 | 212 | 212 | 198 |
| `face.rs` | 155 | 155 | 155 | 361 | 361 | 361 |
| `falling.rs` | 136 | 136 | 137 | 136 | 136 | 137 |
| `fluid.rs` | 354 | 354 | 354 | 397 | 397 | 397 |
| `footprint.rs` | 309 | 309 | 309 | 425 | 425 | 425 |
| `interact.rs` | 362 | 282 | 283 | 490 | 343 | 344 |
| `items.rs` | 479 | 479 | 478 | 548 | 548 | 547 |
| `lib.rs` | 88 | 88 | 89 | 63 | 63 | 64 |
| `math.rs` | 152 | 152 | 156 | 160 | 160 | 164 |
| `minecart.rs` | 380 | 380 | 365 | 373 | 373 | 358 |
| `mining.rs` | 542 | 531 | 527 | 576 | 561 | 558 |
| `mounts.rs` | 171 | 171 | 166 | 194 | 194 | 189 |
| `neighbor.rs` | 454 | 328 | 332 | 471 | 390 | 394 |
| `pattern.rs` | 362 | 362 | 362 | 671 | 671 | 671 |
| `pearl.rs` | 205 | 205 | 190 | 215 | 215 | 201 |
| `piston.rs` | 714 | 714 | 679 | 733 | 733 | 739 |
| `place.rs` | 434 | 408 | 399 | 455 | 434 | 432 |
| `player.rs` | 271 | 271 | 271 | 300 | 300 | 300 |
| `portal.rs` | 481 | 481 | 478 | 512 | 512 | 514 |
| `projectiles.rs` | 325 | 325 | 319 | 334 | 334 | 331 |
| `redstone.rs` | 345 | 345 | 351 | 341 | 341 | 351 |
| `shapes.rs` | 427 | 427 | 491 | 580 | 580 | 679 |
| `sim.rs` | 2912 | 2593 | 2449 | 3075 | 2827 | 2745 |
| `solve_meter.rs` | 21 | 21 | 21 | 28 | 28 | 28 |
| `state.rs` | 372 | 372 | 372 | 519 | 519 | 519 |
| `state_reads.rs` | 20 | 20 | 20 | 39 | 39 | 39 |
| `ticks.rs` | 326 | 262 | 262 | 329 | 270 | 270 |
| `world.rs` | 590 | 590 | 593 | 758 | 758 | 766 |
| `world_hooks.rs` | 1386 | 955 | 874 | 1522 | 1199 | 1099 |
| **Total** | 18738 | 17594 | 17266 | 23417 | 22372 | 22211 |

## Raw paired samples

Seven samples per run, in ns/tick. The fourth sorted sample is the run median. The evidence directory is `D:/path-tracing-target/job-yarn-cold-fold/behavior-first/`.

~~~text
0 base walk: n=7, ns/tick min=964.4 median=965.2 max=969.3; samples=[964.36, 964.365, 964.545, 965.195, 966.2, 968.99, 969.29]
0 base sprint-jump: n=7, ns/tick min=630.8 median=632.3 max=635.0; samples=[630.785, 631.72, 632.17, 632.295, 632.355, 633.03, 635.01]
0 base ceiling: n=7, ns/tick min=884.2 median=885.9 max=888.0; samples=[884.23, 885.355, 885.85, 885.93, 886.4, 886.84, 887.955]
0 base bumpy: n=7, ns/tick min=927.0 median=929.8 max=948.6; samples=[926.99, 928.86, 929.61, 929.785, 932.61, 934.42, 948.61]
0 cand walk: n=7, ns/tick min=962.8 median=964.8 max=968.8; samples=[962.825, 963.77, 963.97, 964.765, 965.295, 968.495, 968.835]
0 cand sprint-jump: n=7, ns/tick min=630.9 median=634.8 max=636.3; samples=[630.9, 633.43, 633.745, 634.825, 635.305, 636.1, 636.265]
0 cand ceiling: n=7, ns/tick min=882.5 median=883.9 max=897.3; samples=[882.525, 883.255, 883.7, 883.88, 886.275, 888.735, 897.27]
0 cand bumpy: n=7, ns/tick min=927.8 median=930.5 max=983.4; samples=[927.815, 927.835, 929.3, 930.47, 936.275, 948.485, 983.41]
1 cand walk: n=7, ns/tick min=963.9 median=967.2 max=969.7; samples=[963.9, 964.3, 964.475, 967.2, 967.6, 968.64, 969.72]
1 cand sprint-jump: n=7, ns/tick min=629.5 median=630.8 max=632.8; samples=[629.45, 630.355, 630.57, 630.755, 630.91, 631.93, 632.8]
1 cand ceiling: n=7, ns/tick min=883.0 median=885.4 max=886.2; samples=[882.995, 883.27, 884.62, 885.375, 885.935, 885.935, 886.225]
1 cand bumpy: n=7, ns/tick min=926.7 median=929.2 max=934.2; samples=[926.73, 928.355, 928.705, 929.235, 929.505, 929.65, 934.18]
1 base walk: n=7, ns/tick min=962.7 median=966.3 max=970.3; samples=[962.665, 965.27, 965.895, 966.33, 967.675, 968.5, 970.32]
1 base sprint-jump: n=7, ns/tick min=630.3 median=632.0 max=634.8; samples=[630.285, 631.16, 631.575, 632.005, 632.32, 632.775, 634.765]
1 base ceiling: n=7, ns/tick min=883.8 median=884.6 max=893.2; samples=[883.82, 884.04, 884.34, 884.6, 886.375, 892.67, 893.175]
1 base bumpy: n=7, ns/tick min=928.2 median=929.5 max=932.3; samples=[928.25, 928.68, 929.445, 929.525, 929.765, 930.19, 932.28]
2 base walk: n=7, ns/tick min=962.0 median=965.5 max=970.7; samples=[961.955, 963.915, 964.48, 965.54, 967.245, 967.77, 970.71]
2 base sprint-jump: n=7, ns/tick min=630.4 median=631.9 max=633.4; samples=[630.43, 631.335, 631.75, 631.905, 632.025, 632.075, 633.365]
2 base ceiling: n=7, ns/tick min=883.9 median=886.0 max=888.7; samples=[883.86, 885.35, 885.475, 886.005, 886.525, 887.89, 888.655]
2 base bumpy: n=7, ns/tick min=927.5 median=930.0 max=935.7; samples=[927.545, 929.165, 929.385, 930.0, 930.02, 931.78, 935.71]
2 cand walk: n=7, ns/tick min=966.0 median=967.8 max=970.2; samples=[965.98, 966.035, 967.77, 967.775, 967.875, 968.35, 970.21]
2 cand sprint-jump: n=7, ns/tick min=629.2 median=632.5 max=641.6; samples=[629.195, 629.515, 630.12, 632.46, 640.11, 640.79, 641.56]
2 cand ceiling: n=7, ns/tick min=883.2 median=884.4 max=887.6; samples=[883.25, 883.44, 883.495, 884.39, 885.11, 885.665, 887.64]
2 cand bumpy: n=7, ns/tick min=927.3 median=930.5 max=936.1; samples=[927.28, 928.58, 929.715, 930.515, 931.6, 933.745, 936.15]
3 cand walk: n=7, ns/tick min=961.8 median=967.9 max=971.5; samples=[961.815, 966.48, 967.795, 967.87, 967.99, 968.2, 971.485]
3 cand sprint-jump: n=7, ns/tick min=631.1 median=632.3 max=637.1; samples=[631.06, 631.155, 631.925, 632.295, 633.06, 633.74, 637.11]
3 cand ceiling: n=7, ns/tick min=881.2 median=884.1 max=884.8; samples=[881.24, 882.315, 883.87, 884.07, 884.255, 884.5, 884.82]
3 cand bumpy: n=7, ns/tick min=928.1 median=932.0 max=934.3; samples=[928.095, 928.83, 930.03, 932.045, 932.13, 932.54, 934.33]
3 base walk: n=7, ns/tick min=961.6 median=967.2 max=974.7; samples=[961.56, 963.38, 963.98, 967.19, 969.505, 969.91, 974.72]
3 base sprint-jump: n=7, ns/tick min=631.2 median=632.1 max=643.9; samples=[631.25, 631.54, 631.9, 632.055, 632.93, 637.525, 643.87]
3 base ceiling: n=7, ns/tick min=884.0 median=886.0 max=909.7; samples=[884.0, 884.37, 885.7, 886.015, 888.015, 891.305, 909.745]
3 base bumpy: n=7, ns/tick min=926.6 median=930.1 max=964.4; samples=[926.56, 929.335, 929.35, 930.135, 930.425, 934.74, 964.435]
4 base walk: n=7, ns/tick min=962.3 median=965.2 max=968.6; samples=[962.3, 964.665, 964.81, 965.25, 966.255, 967.895, 968.6]
4 base sprint-jump: n=7, ns/tick min=631.4 median=632.3 max=634.1; samples=[631.425, 631.615, 631.715, 632.325, 632.96, 632.975, 634.14]
4 base ceiling: n=7, ns/tick min=882.9 median=884.3 max=886.0; samples=[882.86, 883.595, 884.27, 884.29, 884.365, 885.11, 885.985]
4 base bumpy: n=7, ns/tick min=928.7 median=929.9 max=933.2; samples=[928.66, 929.26, 929.665, 929.85, 930.04, 930.84, 933.17]
4 cand walk: n=7, ns/tick min=968.0 median=969.3 max=972.8; samples=[968.015, 968.185, 968.9, 969.335, 970.02, 971.0, 972.76]
4 cand sprint-jump: n=7, ns/tick min=631.7 median=633.7 max=667.4; samples=[631.735, 632.185, 632.385, 633.745, 636.22, 639.625, 667.43]
4 cand ceiling: n=7, ns/tick min=883.2 median=885.6 max=893.9; samples=[883.195, 883.275, 884.045, 885.56, 886.015, 888.1, 893.855]
4 cand bumpy: n=7, ns/tick min=929.6 median=930.6 max=939.8; samples=[929.6, 929.63, 930.15, 930.56, 930.765, 931.95, 939.8]
5 cand walk: n=7, ns/tick min=968.7 median=971.0 max=973.7; samples=[968.705, 968.98, 969.19, 970.99, 971.04, 971.675, 973.725]
5 cand sprint-jump: n=7, ns/tick min=635.8 median=638.1 max=639.4; samples=[635.76, 636.39, 637.845, 638.06, 638.19, 638.73, 639.415]
5 cand ceiling: n=7, ns/tick min=888.2 median=889.4 max=899.7; samples=[888.21, 889.155, 889.275, 889.4, 890.025, 892.46, 899.685]
5 cand bumpy: n=7, ns/tick min=941.4 median=943.8 max=945.8; samples=[941.355, 942.855, 943.385, 943.81, 944.335, 944.475, 945.83]
5 base walk: n=7, ns/tick min=962.7 median=967.2 max=978.0; samples=[962.705, 963.1, 966.7, 967.25, 969.375, 970.065, 978.015]
5 base sprint-jump: n=7, ns/tick min=629.2 median=631.7 max=634.1; samples=[629.17, 629.725, 631.015, 631.695, 632.125, 632.345, 634.11]
5 base ceiling: n=7, ns/tick min=884.5 median=885.1 max=889.9; samples=[884.495, 884.525, 884.925, 885.145, 885.255, 886.245, 889.895]
5 base bumpy: n=7, ns/tick min=927.5 median=930.1 max=933.7; samples=[927.51, 927.83, 928.98, 930.13, 931.085, 931.845, 933.685]
6 base walk: n=7, ns/tick min=962.0 median=966.2 max=970.9; samples=[962.03, 963.465, 965.385, 966.155, 967.185, 967.73, 970.85]
6 base sprint-jump: n=7, ns/tick min=632.5 median=633.8 max=635.9; samples=[632.52, 632.91, 633.35, 633.845, 634.165, 634.615, 635.92]
6 base ceiling: n=7, ns/tick min=882.0 median=884.7 max=886.9; samples=[882.0, 884.475, 884.535, 884.72, 886.12, 886.63, 886.85]
6 base bumpy: n=7, ns/tick min=930.1 median=931.1 max=939.5; samples=[930.125, 930.535, 930.735, 931.1, 932.695, 937.23, 939.45]
6 cand walk: n=7, ns/tick min=965.1 median=967.0 max=973.2; samples=[965.12, 965.895, 966.09, 967.015, 968.025, 968.175, 973.21]
6 cand sprint-jump: n=7, ns/tick min=630.9 median=631.5 max=632.6; samples=[630.875, 631.245, 631.3, 631.54, 631.8, 632.18, 632.64]
6 cand ceiling: n=7, ns/tick min=882.4 median=884.0 max=885.4; samples=[882.365, 883.225, 883.475, 883.95, 884.07, 884.705, 885.35]
6 cand bumpy: n=7, ns/tick min=929.9 median=930.5 max=934.3; samples=[929.88, 929.93, 930.325, 930.45, 930.66, 931.035, 934.28]
7 cand walk: n=7, ns/tick min=964.0 median=968.0 max=972.2; samples=[963.95, 966.655, 967.075, 967.96, 968.515, 970.77, 972.24]
7 cand sprint-jump: n=7, ns/tick min=632.4 median=633.3 max=638.3; samples=[632.385, 632.51, 633.13, 633.32, 633.325, 634.14, 638.31]
7 cand ceiling: n=7, ns/tick min=883.8 median=885.5 max=887.7; samples=[883.815, 883.86, 884.74, 885.47, 885.725, 886.66, 887.705]
7 cand bumpy: n=7, ns/tick min=929.5 median=932.3 max=937.8; samples=[929.475, 929.535, 930.21, 932.29, 935.76, 936.89, 937.83]
7 base walk: n=7, ns/tick min=959.8 median=967.7 max=969.5; samples=[959.775, 962.13, 965.7, 967.695, 968.015, 968.285, 969.48]
7 base sprint-jump: n=7, ns/tick min=631.0 median=633.2 max=634.0; samples=[631.0, 631.515, 632.565, 633.23, 633.265, 633.565, 633.98]
7 base ceiling: n=7, ns/tick min=881.9 median=884.6 max=888.7; samples=[881.85, 882.995, 883.44, 884.605, 886.22, 886.98, 888.745]
7 base bumpy: n=7, ns/tick min=929.8 median=930.4 max=932.6; samples=[929.84, 930.15, 930.31, 930.435, 930.6, 930.77, 932.615]
8 base walk: n=7, ns/tick min=963.7 median=966.9 max=969.2; samples=[963.655, 964.425, 966.015, 966.925, 967.34, 967.455, 969.225]
8 base sprint-jump: n=7, ns/tick min=630.8 median=633.4 max=636.1; samples=[630.77, 631.465, 632.795, 633.35, 633.67, 635.16, 636.065]
8 base ceiling: n=7, ns/tick min=882.8 median=884.0 max=885.5; samples=[882.76, 884.01, 884.04, 884.04, 884.64, 884.785, 885.525]
8 base bumpy: n=7, ns/tick min=929.4 median=935.7 max=1039.4; samples=[929.39, 929.595, 930.565, 935.705, 943.49, 956.765, 1039.4]
8 cand walk: n=7, ns/tick min=963.4 median=967.0 max=969.3; samples=[963.375, 964.52, 966.25, 967.02, 967.65, 969.245, 969.285]
8 cand sprint-jump: n=7, ns/tick min=630.6 median=633.0 max=636.8; samples=[630.61, 630.74, 631.45, 633.02, 633.255, 633.94, 636.775]
8 cand ceiling: n=7, ns/tick min=881.9 median=884.1 max=884.9; samples=[881.92, 882.64, 883.845, 884.13, 884.165, 884.42, 884.915]
8 cand bumpy: n=7, ns/tick min=927.0 median=928.1 max=938.1; samples=[926.975, 927.265, 927.315, 928.075, 928.26, 930.925, 938.06]
9 cand walk: n=7, ns/tick min=964.1 median=967.2 max=968.9; samples=[964.12, 964.89, 966.765, 967.235, 967.355, 967.61, 968.94]
9 cand sprint-jump: n=7, ns/tick min=630.0 median=631.3 max=635.2; samples=[629.955, 630.43, 631.11, 631.33, 631.455, 632.46, 635.22]
9 cand ceiling: n=7, ns/tick min=882.7 median=888.0 max=893.5; samples=[882.715, 883.7, 886.91, 887.985, 888.7, 888.735, 893.55]
9 cand bumpy: n=7, ns/tick min=929.1 median=933.6 max=1080.0; samples=[929.145, 931.55, 932.35, 933.555, 935.54, 953.415, 1080.045]
9 base walk: n=7, ns/tick min=962.6 median=967.2 max=970.0; samples=[962.62, 964.47, 965.02, 967.195, 967.71, 967.715, 970.035]
9 base sprint-jump: n=7, ns/tick min=630.5 median=630.9 max=633.1; samples=[630.515, 630.59, 630.88, 630.945, 630.99, 631.145, 633.12]
9 base ceiling: n=7, ns/tick min=881.9 median=884.8 max=888.8; samples=[881.875, 883.595, 884.005, 884.75, 884.83, 888.12, 888.81]
9 base bumpy: n=7, ns/tick min=927.2 median=929.3 max=941.0; samples=[927.23, 927.46, 928.115, 929.27, 930.84, 931.34, 940.995]
10 base walk: n=7, ns/tick min=963.9 median=967.2 max=969.8; samples=[963.86, 965.465, 966.195, 967.22, 967.79, 969.735, 969.755]
10 base sprint-jump: n=7, ns/tick min=631.8 median=633.8 max=640.0; samples=[631.77, 633.32, 633.42, 633.775, 635.51, 637.29, 640.03]
10 base ceiling: n=7, ns/tick min=884.8 median=886.1 max=889.1; samples=[884.835, 885.055, 885.465, 886.145, 887.17, 887.845, 889.075]
10 base bumpy: n=7, ns/tick min=931.4 median=933.3 max=1096.0; samples=[931.355, 931.745, 932.275, 933.32, 935.285, 943.045, 1096.0]
10 cand walk: n=7, ns/tick min=965.7 median=966.5 max=971.1; samples=[965.68, 965.9, 966.225, 966.45, 968.955, 970.625, 971.1]
10 cand sprint-jump: n=7, ns/tick min=628.9 median=630.6 max=632.3; samples=[628.935, 629.91, 630.08, 630.62, 630.945, 631.03, 632.3]
10 cand ceiling: n=7, ns/tick min=882.1 median=885.2 max=886.9; samples=[882.14, 882.845, 885.055, 885.165, 885.28, 885.915, 886.935]
10 cand bumpy: n=7, ns/tick min=928.3 median=929.6 max=932.6; samples=[928.34, 928.77, 929.12, 929.65, 929.855, 930.815, 932.56]
11 cand walk: n=7, ns/tick min=966.5 median=973.5 max=1024.0; samples=[966.525, 969.355, 970.055, 973.51, 985.0, 987.15, 1023.995]
11 cand sprint-jump: n=7, ns/tick min=629.1 median=630.6 max=635.6; samples=[629.085, 630.205, 630.6, 630.6, 632.905, 633.775, 635.63]
11 cand ceiling: n=7, ns/tick min=883.8 median=886.1 max=1002.8; samples=[883.825, 883.93, 885.025, 886.15, 886.74, 919.53, 1002.815]
11 cand bumpy: n=7, ns/tick min=926.1 median=926.8 max=949.1; samples=[926.09, 926.56, 926.595, 926.845, 927.925, 931.95, 949.085]
11 base walk: n=7, ns/tick min=965.0 median=967.9 max=969.9; samples=[965.03, 965.395, 965.815, 967.935, 968.41, 969.51, 969.945]
11 base sprint-jump: n=7, ns/tick min=630.5 median=633.4 max=638.7; samples=[630.54, 631.07, 632.375, 633.41, 633.47, 635.26, 638.705]
11 base ceiling: n=7, ns/tick min=884.3 median=886.4 max=887.1; samples=[884.28, 886.065, 886.395, 886.395, 886.485, 886.89, 887.08]
11 base bumpy: n=7, ns/tick min=927.5 median=929.4 max=933.0; samples=[927.485, 928.83, 929.275, 929.425, 930.535, 932.925, 932.985]
~~~
