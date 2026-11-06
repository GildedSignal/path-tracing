# Job 1.5f — Live movement divergence

The Rust law used the previous tick's speed for grounded travel. The replay supplied the correct sprint latch and attribute inputs in both witnesses. The law now evaluates the movement-speed attribute once after updating the latch.

## 26.2 bytecode

- `LocalPlayer.aiStep` calls `setSprinting(true)` at bytecode offset 443 for the sprint key and `setSprinting(false)` at 484 for the running stop predicate. It enters `AbstractClientPlayer.aiStep` at 1095, which calls `Player.aiStep` at 5. `LivingEntity.setSprinting` updates the flag at 2, removes the movement-speed modifier at 20 and adds it back at 32 only when sprinting.
- `Player.aiStep` calls its superclass at 50. This reaches `LivingEntity.aiStep`, which calls `travel` at 617. For the grounded path, `LivingEntity.getFrictionInfluencedSpeed` calls virtual `getSpeed()` at 17 or 32. `Player.getSpeed` reads `MOVEMENT_SPEED` at 1–4 and converts the result to float at 7. `handleRelativeFrictionAndCalculateMovement` applies that acceleration at 7 and calls `move` at 30.
- After superclass movement returns, `Player.aiStep` calls `setSpeed` at 74. That writes the separate `LivingEntity.speed` field. The earlier virtual getter reads the live attribute, so that write has no bearing on these witnesses.

The first differing quantity was the speed operand entering grounded acceleration. At full-blocks tick 297 it was sprint speed `3e051eb9` in the old law and walk speed `3dcccccd` in vanilla. Slime tick 160 reverses those operands. The first directly recorded difference was in the collision request. For each witness the client's resolved vector retained the requested horizontal components and zeroed vertical displacement.

| Witness | Sprint transition | Old requested x | Client and corrected requested x | Old position.x | Client and corrected position.x |
|---|---|---|---|---|---|
| Full-blocks 297 | On → off | `3f871290ca8de372` | `3f8fd45227ad44ff` | `40b01295fc265d61` | `40b01297145e8905` |
| Slime 160 | Off → on | `3fb483ca0308ed7f` | `3fb9d636a3aaad53` | `40b391a6ff11024a` | `40b391ac517da2ec` |

## Checks

| Original pair | Movement | Server resources after the recorded rest offset |
|---|---|---|
| Corner | 420 of 420 ticks matched | Pass |
| Full-blocks | 420 of 420 ticks matched | Fail on saturation, law 19 (`41980000`), server 20 (`41a00000`) |
| Slime | 420 of 420 ticks matched | Pass |

Two server snapshots passed, one failed and zero were skipped. Full-blocks had a six-tick rest offset. Health, food, exhaustion and food tick timer matched in the diagnostic output. The saturation mismatch is the next anomaly and remains open. The original pairs kept their SHA-256 hashes. L0 remains open.

Commands from `engine/`

```powershell
cargo test --offline --release --test replay replay -- --exact --nocapture
cargo test --offline --release --test law grounded_travel_uses_current_sprint_modifier -- --exact --nocapture
cargo test --offline --release -- --exact --skip replay
```

The unfiltered replay stops at the first failure. A temporary file-stem filter ran each original pair independently, and a temporary trace printed `movement_this_tick` beside each recorded collision request. Both diagnostics were removed afterward. The reduced test failed under the old order on the requested x bits for sprint-off, `3fc04ea4c367a100` versus `3fb916873978d500`. It passed under the corrected order for both transitions.

The offline suite passed 56 tests. The ignored registry test passed separately through `./lab/gates/tests/test-exporter.ps1` from the root. `./engine/probes/Regenerate.ps1` regenerated the braking corpus with the real `Player.getSpeed` dispatch. Bytecode was read with staged `javap -cp <26.2 client jar> -c -p -constants <class>` for `LocalPlayer`, `AbstractClientPlayer`, `Player`, `Avatar` and `LivingEntity`.
