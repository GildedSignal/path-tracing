# 1.3e offline record

Consumer: L0 Full, powder-snow run 7 and the 94 previously kept runs. No client or server was started.

Changed files:
- `engine/law/src/movement.rs`: block and fluid collector advances; server freeze recurrence, frost value and damage request, leaving resolved resources server-owned.
- `engine/law/src/lib.rs`: exports the server freeze recurrence.
- `engine/law/tests/law.rs`: period, threshold, decay, immunity and frost boundaries.
- `oracles/1.6f-landing-motion/verify.rs`: checks freeze damage and its mark, writes, wire and resource event, including motion received after the movement fence.
- `jobs/1.2-footprints/verify.ps1`: optional output directory, used because the existing workspace log denied writes.
- `jobs/1.3e-freeze-damage/last.md`: this record.

Jar offsets (26.2): `Entity.lambda$checkInsideBlocks$0` 34–65, 66–121, 140–166, 194–214, 287–313; `BlockBehaviour.getEntityInsideCollisionShape` 0–3; `InsideBlockEffectApplier.StepBasedCollector.advanceStep` 0–17 and `flushStep` 65–83; `InsideBlockEffectType.lambda$static$0` 0–29; `PowderSnowBlock.entityInside` 27–39 and 180–195; `LivingEntity.aiStep` 697–722 and 733–768; `LivingEntity.removeFrost/tryAddFrost` 1045–1096; `Entity.getPercentFrozen` 0–17 and `isFullyFrozen` 0–16. The existing general status-effect refusal covers cobweb WEAVING.

Replay: `D:/path-tracing-target/1.3e/full-replay.log`: 95 exact 420-tick replays, including all 94 retained runs and powder-snow 7 (seed 95). Script exits at missing powder-snow 8, never captured after the original refusal; no retained run was skipped. Run 7: four freeze rows at server player ticks 94920, 95120, 95160, 95200, each `3f800000`, no source; 1,231 resource checkpoints and four motion packets matched, two packets after the 420-tick movement fence. Counter and frost are server-owned inputs to client replay; the damage recorder does not independently record the server freeze counter, so the event check validates the recorded phase and amount conditional on a fully frozen server, not an independently reconstructed server counter. Capture's six run-7 inputs are SHA-256 identical to `D:/path-tracing-evidence/l0-veto-20261114-1.3d-c/full`; the assembled capture also supplies the pre-run world.

Checks: `cargo test --offline` passed (`D:/path-tracing-target/1.3e/cargo-test.log`); `jobs/1.2-footprints/verify.ps1` passed, 12 mutants caught (`D:/path-tracing-target/1.3e/footprints.log`); period 40→41 and decay 2→1 each failed `server_freeze_boundary_and_decay` (`mutant-period.log`, `mutant-decay.log` in the same target). Original fixture-region modifications and untracked backups predated this job and were preserved.

Replay command: set `PATHTRACING_ROOT=D:/path-tracing-lab`, `PATHTRACING_DEPS=D:/path-tracing-deps/data`; build with `oracles/1.6g-terrain-corpus/build-verifier.ps1 -Target D:/path-tracing-target/1.3e`; run `oracles/1.6g-terrain-corpus/verify.ps1 -Mode Full -Capture D:/path-tracing-wt/l0-veto/lab/gates/results/20261114T1459531151826Z-fixtures -Target D:/path-tracing-target/1.3e -ReferenceServer D:/path-tracing-lab/server`. The first 95 completed; the next input was absent. Bit-exact movement/resource comparisons have zero allowed margin; no mismatch among those 95. The missing run 8 is the recorded refusal of the batch script, not a replay result.
