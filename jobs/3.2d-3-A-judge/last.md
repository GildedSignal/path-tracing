# Judge verdict, 3.2d-3 slice A (R0), branch wt/3.2d-3-A, commit eb3632a

**PASS WITH FINDINGS.** No blocking constraint violation. Judged by Sonnet 5.5, fresh context, offline only (no game, client or server launched; Idle priority, hidden).

## 1. Gates
- `cargo test --offline` from `engine/` (my own run, target judge-A): 191 passed, 18 ignored, 0 failed. Matches the builder's 191/18.
- Gate tests exist and test what they claim. They are not tautological:
  - `engine/law/tests/from_bits.rs::observation_requires_every_word_and_the_exact_mask_length` clears each known bit in turn and asserts the refusal position and reason, plus mask-length refusals.
  - `engine/native/tests/state_schema.rs::frozen_java_rust_schema_witness` does the same per word against a Java-produced fixture. `recorded_observation_preserves_unknown_fields_without_defaults` checks the pinned recording is refused.
  - `bot/tests/NativeChannelTests.java` loops every word as unknown through the real DLL, asserting echo and synchronous refusal (the Java to Rust to Java round trip). A mask-dropping mutant would fail these.
  - `engine/native/src/resident.rs` tests: palette, edit and unload are the only loaded evidence; the observed start is exact and returns a memory sentence.
- Not re-run by me, corroborated from the builder's raw logs in `D:\path-tracing-evidence\3.2d-3-A\`: the Java native test (PASS), click-path grep test (PASS), bot build, `test-tracks.ps1` (419 + 50 + 101 checks). I did not run `jobs\1.2-footprints\verify.ps1` because it plants mutants into the worktree sources, which the constraints forbid. A's own footprints.log is UTF-16 (a grep for CAUGHT counts 0 on the raw file, but the tail shows `restored PASS`). B's clean log shows the same 13 mutants caught.
- The click-path test is grep-based over four files only (ResidentRunner, NativeChannel, ResidentSource, resident.rs), plus a first-statement regex for the issuance stamp. It catches the named old paths, but proves nothing about what plan::flight calls. Acceptable for an offline gate; weak.

## 2. Constraint violations: none blocking
- No tolerance, epsilon, retry or made-up number found. No new external crate (the Cargo.lock delta is only `pathtracing_native`, mandated by the prompt). JNI table indices checked against the spec: 171, 180, 204, 212 are right.
- `movement.rs`, `pattern.rs`, `footprint.rs`, `class.rs` and `table.rs` are untouched (git diff names). The law change is only `from_observed_bits` in `state.rs` plus a trailing newline.
- Files outside the allowed area: none. The extra tracked files (jobs/3.2d-3 scripts, observer mixins) are the spec's section 7 and 8 row-A skeleton.
- No settle, process start, script file or log poll in resident goal dispatch. `StraightLine` and `Planner.java` are deleted; `Bot.goTo` goes through `ResidentRunner.request`.

## 3. Findings, ranked
1. **(Important, deferred by design) Live `goto` and `come` always refuse.** `PlayerSnapshot.observed` deliberately removes `exhaustionBits` and `foodTickTimer` ("Local FoodData is not the dedicated server's exhaustion/timer"). `from_observed_bits` refuses on any unknown word, and the source delta sends natural regeneration as Unknown (`ResidentSource.join`, flag 2). So every live request is refused until K supplies server resource rows. This is sound per section 1 (no fabricated state) and the report says so. But the bot's only planner, the Java StraightLine, is gone, so goto and come are dead on the click until E/F/K. Julien should know before he tests live.
2. **(Moderate) Hot path on the client thread: `ResidentSource.chunk`.** For every chunk packet it calls `canonical(state)` for each of 4096 blocks per section, over about 24 sections (property strings built and sorted, about 100k per chunk), and ships 4096-word index arrays through JNI. Not measured. Same family: `ResidentRunner.event` builds a full JSON snapshot into bits on every position, motion, attributes, entity-data, health or rotation packet.
3. **(Moderate, unverified live) Playback cancels on any event.** `ResidentRunner.event` cancels executing legacy playback on any observed packet, including `entity-data` and `attributes`. The server echoes sprint and pose through entity-data (engine.md), so legacy playback may abort on its first sprint echo. This cannot be tested offline; K must check it.
4. **(Minor) In-process result poll per client tick.** `ResidentRunner.tick` calls `ObserveEvent({0,version})` every tick while waiting, and dispatch happens on the next client tick after the click, not in the command callback. It is not log polling and does no thinking, but it is a poll and it can add up to one tick before dispatch (stamped, not hidden). A latch or push from the worker would remove it.
5. **(Minor) Panic as Unknown.** `Source::block` panics on an unreceived cell and `natural_regeneration` calls `expect`; `catch_unwind` in the worker turns these into a refusal. Correct, but control flow by panic; a typed Unknown would be cleaner. Also `unwrap` on mutex poison.
6. **(Minor) Mixin risk.** `HopRconMixin` targets the synthetic `lambda$runCommand$0`, and the mixin config has `defaultRequire 1`. A wrong lambda name would crash at launch; only the compiled injection audit protects it. `fabric.mod.json` is now `environment: "*"` with server mixins in the shipped jar. HopCapture is gated by a system property, but the per-packet mixin calls remain.
7. **(Noted) `legacy_sentence_adapter` still runs `law::try_tick` per planned tick on the worker.** It is labelled temporary, off the client thread, and the prompt allowed it. F must remove it.

## 4. Spec departures
- Pinned immutable far source: none configured (unreceived cells are Unknown). Sound, deferred.
- `goto` carries the issuer QPC stamp and frequency as chat arguments and rejects a frequency mismatch; the legacy chat hop is recorded. Sound.
- Live baseline capture: not run (prompt item 8). Deferred; verify.ps1 `CaptureBaseline` throws honestly.
- Known mask is per bit-word, and any unknown refuses the whole state. Sound for R0.

## 5. Merge risk
Disjoint from B by file name (the `git diff --name-only` intersection is empty). `git merge-tree --write-tree wt/3.2d-3-A wt/3.2d-3-B` is clean (tree 24bc287). Each merges cleanly onto current main (031cbaa), which only moved docs and hard-track files. A's native crate uses neither `cell_digest` nor `Start`, so B's API changes do not break it. The tracked `r.0.*.mca` fixtures are pre-existing dirt in both worktrees; do not commit them.

## 6. Net lines
+1662/-116 over 53 files: about 640 are jobs scripts, reports and fixtures, about 560 Rust (lib.rs 132, resident.rs 237, tests), about 700 Java. Bought: the resident channel, the observed from_bits gate, QPC and the hop capture that row A lists; removes StraightLine, Planner and about 100 lines of Bot. The commit needs a `Bought:` line.
