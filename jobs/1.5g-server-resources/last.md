# Job 1.5g — Server resource replay

The first differing field is peaceful full-blocks `serverEnd.saturationLevel`. The coupled replay predicts 19 (`41980000`) and the server recorded 20 (`41a00000`). The start state uses the authoritative snapshot from `bd44e37`. The original fixture lacks the server schedule needed for exact replay. L0 remains open.

## 26.2 bytecode

| Method | Bytecode offsets and order |
|---|---|
| `MinecraftServer.processPacketsAndTick`, `tickChildren` | Drain queued packets at 33 before `tickServer` at 61. Tick the level at 172 before the connection at 233. |
| `ServerLevel.tickNonPassenger` | Increment the entity's `tickCount` at 10–15 and call `Entity.tick` at 48. |
| `ServerGamePacketListenerImpl.tick`, `tickPlayer` | Call `tickPlayer` at 39, which calls `ServerPlayer.doTick` at 50. The connection also has its own counter. |
| `ServerPlayer.doTick`, `Player.tick`, `LivingEntity.tick`, `Player.aiStep` | `doTick` calls `Player.tick` at 15 and `FoodData.tick` at 46. `Player.tick` enters the inherited tick at 162, which reaches virtual `aiStep` at 178. `Player.aiStep` calls virtual `tickRegeneration` at 18, before the superclass step at 50. |
| `ServerPlayer.tickRegeneration` | Check peaceful and natural regeneration at 0–32. At entity tick count divisible by 20 (35–42), heal one point below max health and add 1.0 to saturation below 20 (45–85). `FoodData.setSaturation` stores the result directly, so fractional saturation can exceed 20. At tick count divisible by 10 (88–95), add one food point below 20 (98–121). |
| `FoodData.tick` | At 10–78, strictly more than 4 exhaustion spends 4 once, consuming saturation first or food if saturation is empty and difficulty allows it. Next come saturated healing (96–176), slower healing (179–235), starvation and the timer reset (238–324). The healing predicates read the resources after peaceful regeneration and the exhaustion debit. |
| `LocalPlayer.sendPosition`, `ServerGamePacketListenerImpl.handlePlayerCommand` | The client sends sprint changes before the move (offset 1) and sends position based on displacement or its reminder (66–107). The server updates the sprint flag at 85 or 96. |
| `ServerGamePacketListenerImpl.handleMovePlayer` | Marshal onto the server thread at 9. An upward ground-to-air packet calls `jumpFromGround` at 657 before moving at 689. The accepted path installs packet position at 988 and ground state at 1160, checks fall damage at 1186, and calls `checkMovementStatistics` at 1295 with the displacement from the pre-packet server position. |
| `ServerPlayer.jumpFromGround`, `checkMovementStatistics`, `Player.causeFoodExhaustion` | Jump adds 0.2 while sprinting or 0.05 otherwise (11–32). On dry ground, sprint movement adds `0.1f * roundedHorizontalCentimetres * 0.01f` (244–306). Exhaustion writes occur only for a non-invulnerable server player (0–26). |

The passive probe has 45 samples per case. Peaceful with regeneration refilled at world/player ticks 16828/180 and 16848/200. Normal with regeneration and peaceful with regeneration disabled each stayed flat. The raw CSVs under `lab/gates/results/20261106T1238228940063Z-server-phase` contain world tick, entity tick, food and saturation bits.

## What the fixture determines

The fixture supplies authoritative starting resources, client inputs and post-tick movement, and a world-time bracket for the end RCON read. `atRest` comes from the last client row. The fixture has no server entity phase and no accepted movement packet order relative to `doTick`. Positive saturation changes alone do not trigger a health packet (`ServerPlayer.doTick` 145–221), so the recorded client saturation of 20 cannot distinguish server saturation 19 from 20.

The packet-path audit applies the dry ground and jump charges to the recorded positions, using the position packet threshold and reminder. Under prompt acceptance, a sprint jump at client row 25 (client entity tick 324, client game time 10165) causes the only exhaustion debit. The last client row is at game time 10559. With the recorded offset of six, all 20 possible player phases yield saturation 20 by the end sample under this delivery order.

A second FIFO delivery schedule permits two outcomes for the same recorded inputs. Buffer outgoing movement and sprint packets, accept through row 25 before `doTick` at world tick 10564, and accept the remaining packets before `doTick` at 10565. This preserves the accepted position sequence and applies the same single debit before later charges. Vanilla drains queued packets before the level and connection tick. The more-than-five-moves branch logs and sets the speed-check multiplier to one (`handleMovePlayer` 419–449). The two batches' maximum squared displacements from their tick origins are 3.645 and 22.916, below the base movement check of 100 (460–489). The movement is on the same static course and the second batch leaves the server player at rest. This offline witness keeps inbound time sync fixed. The live packet delivery schedule is unobserved.

| Server player count at 10564 / 10565 | Debit and refill order | End saturation |
|---|---|---:|
| 600 / 601 | Cadence sees saturation 20, then the debit makes it 19. The sample precedes the next cadence. | 19 |
| 619 / 620 | Debit makes saturation 19, then the next tick refills before the sample. | 20 |

The two schedules end with the same health, food, exhaustion bits and food timer as the capture. For this single-debit, initially full case, the missing observation is one ordering fact: whether an authoritative 20-tick regeneration call followed the saturation debit before the sample. To replay general cases, record the server player phase and the ordered resource-changing events between resource ticks, with start and end samples on the same server timeline. A server phase anchor alone does not locate packet charges.

`tick_server_resources` now accepts the server player's actual entity tick count and applies the traced order for a living unmounted player with default max health and no intervening resource changes. The legacy fixture comparison exercises the client-coupled estimate.

## Offline results

Every pair matched 420 movement ticks. Health and food were 20 in every prediction and sample. All food timers were zero. Each exhaustion value below matched bit for bit.

| Pair | End offset | Saturation, predicted / sampled | Exhaustion bits | End check |
|---|---:|---:|---|---|
| Corner peaceful | 6 | 20 / 20 | `3fde9738` | Five fields match |
| Full-blocks peaceful | 6 | 19 / 20 | `3fde9738` | First and only mismatch is saturation |
| Slime peaceful | 6 | 20 / 20 | `403d3f4d` | Five fields match |
| Full-blocks normal | 7 | 19 / 19 | `3f6c076d` | Five fields match |

Zero resource skips in the diagnostic audit. The strict replay still fails at peaceful full-blocks saturation. The unit run passed 57 tests, with the separate registry export test still ignored. The reduced check compares ticks 9–11 and 19–21 in peaceful, normal and peaceful with regeneration disabled. The planted food cadence shift from remainder 0 to 1 failed at peaceful tick 10. Expected food 20 enabled the saturated heal on that tick, while the mutant left food and health at 19. The originals were read-only and no live rig commands were run.

## Reproduce

From `engine/` in the worktree, with the staged Rust toolchain on PATH.

```powershell
cargo test --offline --release -- --exact --skip replay
cargo test --offline --release --test replay replay -- --exact --nocapture

$target = 'D:/path-tracing-target/1.5g-audit'
$env:CARGO_MANIFEST_DIR = (Resolve-Path law).Path
$env:L0_NORMAL_RECORDING = 'D:/path-tracing-evidence/l0-normal-resource-probe/lab/fixtures/full-blocks/full-blocks-1.jsonl'
cargo build --offline --release --lib --target-dir $target
rustc --edition 2024 --test --extern "law=$target/release/liblaw.rlib" ../jobs/1.5g-server-resources/audit.rs -o "$target/audit.exe"
& "$target/audit.exe" --nocapture --exact resource_audit
& "$target/audit.exe" --nocapture --exact peaceful_full_blocks_witness
```

For the mutant, copy `engine/` to a scratch directory under `D:/path-tracing-target`, replace `server_player_tick % 10 == 0` with `server_player_tick % 10 == 1` in the copy and run `cargo test --offline --lib server_resource_cadences_precede_food_data` there with a scratch `--target-dir`.

The server disassembly command was `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe -cp D:/path-tracing-lab/server/versions/26.2/server-26.2.jar -c -p <class>` for the server classes above and `net.minecraft.world.food.FoodData`. For `net.minecraft.client.player.LocalPlayer`, use the staged `data/libraries/com/mojang/minecraft/26.2/minecraft-26.2-client.jar` under `D:/path-tracing-deps`.
