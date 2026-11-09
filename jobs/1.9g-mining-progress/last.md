# 1.9g mining progress handoff

The offline differential passed on `wt/law` at base `2712dc1`. The integrator
can expose `engine/law/src/mining.rs` after review. The verifier includes it by
path.

## Contract

`MiningClock::apply` takes server-ordered accepted START and STOP inputs.
START expects a stable eligible non-air block and the evaluated speed at the
post-hit-hook boundary. Vanilla fetches the block state before the hooks
(`handleBlockBreakAction` 263–349). Supply a sample only when that state still
identifies the intended block and uses the default destroy-progress law.
`MiningClock::tick` advances the clock and reads the current sample only for
the selected work. `None` means observed air. The world adapter must resolve
unavailable data before calling the clock. Slots carry identity and start tick.
The latest speed, hardness and correct-tool flag are sampled afresh. Progress
is elapsed time multiplied by the current rate.

The [gate](gate.md) maps the behavior to the Mojang method offsets. It uses
the staged jar from the two checked GLM reports
(`local-20261107-1528-1-5-port-breaking-threshold` and
`local-20261107-1548-1-5-port-break-stop-boundary`). The runner pins the
jar and the compiled Java feeder, elapsed arithmetic and threshold compares
with `javap`. The JVM executes a transcription of the pinned clock paths.

Identity invalidation is an adapter rule. A replacement clears the selected
slot and returns `TargetChanged`. Accepted STOP against air has no effect.
A normal tick against air clears the selected slot. START while active, START
at the delayed position, STOP without a live active slot and STOP at another
position are refused. The caller must handle ABORT separately. The clock
reports destruction as an intent with the expected target identity. At the
same event, the adapter checks the identity, attempts the edit and records
whether it succeeded. Packet acceptance and destroy-attempt success are
separate. Vanilla's `destroyAndAck` checks the boolean return at 2–5.
The delayed tick calls `destroyBlock` and discards that return at 74–77.

## Reproduce

Run from the worktree root in PowerShell. The first command uses the staged
server jar and JDK by default. It takes a lock in the offline scratch directory
and writes disassemblies, oracle rows and both logs there.

```powershell
& ./jobs/1.9g-mining-progress/run.ps1
cargo test --offline --locked --manifest-path engine/Cargo.toml -p law --target-dir D:/path-tracing-target/1.9g-mining-cargo
```

The runner calls `javap -cp <jar> -c -p <class>`,
`javac -d <result> jobs/1.9g-mining-progress/MiningOracle.java`,
`javap -cp <result> -c -p MiningOracle` and
`java -Xint -cp <result> MiningOracle`. Each Rust binary is built with
`rustc --edition 2024 --test -O <test-path> -o <exe>` and invoked with
`--ignored --nocapture --test-threads=1` and the same `MINING_ORACLE` TSV.
The fixture and mutant results regenerate from the runner.

| Entry | Rows | Original differences | Strict delayed mutant differences |
|---|---:|---:|---:|
| Reset | 28 | 0 | 0 |
| Synthetic clock seek | 2 | 0 | 0 |
| START | 37 | 0 | 0 |
| STOP | 22 | 0 | 0 |
| Tick | 34 | 0 | 4 |
| Total | 123 | 0 | 4 |

The original exited 0 and the mutant exited 101. Raw first difference from
`strict-delayed-threshold.log`:

```text
row 24
prior Rust state: 10/-/4:71@8
JVM row: delayed-boundary-0|tick|0|4:71:40f00000:3f800000:1|1|11/-/-|4:71|destroy:4:71:3f800000
Rust state|lookup|step: 11/-/4:71@8|4:71|progress:4:71:3f800000
```

Original evidence is in
`D:/path-tracing-target/1.9g-mining-progress/20261107T2155476010786Z`.
The runner reported 3.734 s to verdict on that run. Cargo reported 62 passed
and 5 ignored, including the JVM-backed test exercised by the first command.

## Live qualification owed

Capture server-side sequence and `gameTicks` at packet handling and game-mode
tick entry and exit. Record the accepted action, target position and identity,
the active and delayed positions and start ticks, raw per-sample progress
bits, the destroy attempt and its return, and the post-event block state.
These observations establish packet phase relative to `tick()` and separate
an intent from a successful world edit.

Exercise an accepted START at `p = 1.0f`, a STOP with `p = 0.35f` and
`gameTicks - startTick = 1`, and a STOP that arms delayed work with a tick at
`p = 0.25f` and `gameTicks - originalStartTick = 3`. Record the adjacent
below-threshold controls, a tool-speed change while work is pending and
overlapping active and delayed work. Include a refused destroy attempt and
a target replacement to qualify the world adapter's identity boundary.
This job used the staged jar and launched no live game.
