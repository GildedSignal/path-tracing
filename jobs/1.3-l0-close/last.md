# Job 1.3 state (Claude Opus 5.5, 9 November)

L0 is still open. The last refusal is outside the law's contract: a server correction caused by network timing.

1. **Step-ups seed 17, tick 204, speed** (GPT's diagnosis). The sprint flag and the sprint modifier are independent: `ClientPacketListener.handleSetEntityData` (36) -> `SynchedEntityData.assignValues` sets shared flag 3 without `LivingEntity.setSprinting` (remove at 20, add at 32). Fix: the law's `set_sprinting` updates both, and the recorder writes `startSprinting`/`startSprintSpeedModifier` as input. Live: all 8 step-ups runs passed, in both v6 and v7.
2. **Ceilings seed 28, tick 251, collision result.y** (0.42 law, 0.2 client). `ServerEntity.sendDirtyEntityData` (44, `sendToTrackingPlayersAndSelf`) echoes the server's lagging STANDING pose, and `Entity.onSyncedDataUpdated` (0-11) calls `refreshDimensions` before the move. `Player.tick` re-derives the pose only after the move (`updatePlayerPose` at 323). Fix: `startPose` is input (plus `startSwimming`, carried by the same flags byte). `pose-diagnostic.rs` checks the immutable capture: with STANDING before tick 251, all 420 rows are exact.
3. **Ceilings seed 28, tick 233, `ClientboundPlayerPositionPacket`** (v7). The client's crouched jump to y=-59.58 is legal. The server checks it with its own pose, which is still STANDING (`ServerGamePacketListenerImpl.handleMovePlayer` 907 `isEntityCollidingWithAnythingNew` -> 923 `teleport`). In v6, the identical client trajectory through tick 233 was accepted. Whether the server's pose update runs between the two move packets depends on packet arrival. The pass line refuses any correction, and the law does not simulate the server.

Rig prep fixes in `run-live.ps1`: the Fabric API goes in the copied server mods, and the options file is written as ASCII. With a UTF-8 BOM, the client misreads `version`, resets its options and never joins.

Evidence: every live capture (v6 preflight/audit/full, v7 preflight/audit/full) is copied to `D:/path-tracing-evidence/l0-close-20261109-b/captures`. Disposable rigs are in `D:/path-tracing-evidence/l0-close-20261109{,-b}`.
