# Port lenses for GLM's overnight lanes

21 narrow, jar-backed port lenses in `jobs/port-lenses/lenses/`, ordered by priority (`docs/plan.md` steps 5 & 8, `jobs/flight-corpus/report.md` §4, `jobs/port-backlog/report.md`). None collide with `lab/lenses/` or re-ask facts already settled in `lab/wiki/index.md`.

## Lenses in priority order

### Priority 1: Mechanics his recorded tracks need that the engine lacks
1. `port-pressure-plate-hook`: `BasePressurePlateBlock.entityInside` / `checkPressed` overlap box, signal update, release delay, and whether player motion/pose is untouched — `butoonpressureplates` (`docs/HANDOFF.md` 198t vs 70t), `docs/plan.md` step 5.
2. `port-crawl-pose-transition`: `Player.updatePlayerPose` (to locate) dry-land `Pose.SWIMMING` (`0.6`h) entry/exit under `1.0`b / `1.5`b ceilings and `0.3` slow-input trigger — `amertystcoral`, `docs/plan.md` step 5.
3. `port-creative-flight-toggle`: `LocalPlayer.aiStep` double-jump `abilities.flying` toggle window and `Player.travel` vertical ascend/descend velocity bits — `parkour_race` (`16821` ticks), batch `P4e`.
4. `port-status-effect-kinematics`: `MobEffects.JUMP_BOOST` jump/fall arithmetic and `SPEED` / `SLOWNESS` attribute modifier constants — `frosted_ice`, `engine/law/src/movement.rs:121-123` (`state.effects` refusal).
5. `port-movement-exhaustion-branches`: Every branch, rounding cast, and multiplier of `ServerPlayer.checkMovementStatistics` vs `engine/law/src/movement.rs:1104-1121` — `parkour_race` (`20.0 -> 13.0` saturation), `trial-food-crossing`, batch `P1a`.

### Priority 2: Editing the world (`docs/plan.md` step 5)
6. `port-use-item-on-dispatch`: `ServerPlayerGameMode.useItemOn` and client `MultiPlayerGameMode` (to locate) priority between `BlockState.useItemOn`, `useWithoutItem`, sneak-bypass, and `BlockItem.place` — `docs/plan.md` step 5 (`P2c`), `ametwoodbox`, `doorcorridor`, `doorparkourblue`, `trapdoorparkour`, `amertystcoral`.
7. `port-door-trapdoor-toggle`: `DoorBlock`, `TrapDoorBlock`, and `FenceGateBlock` `useWithoutItem` toggle gates (`canOpenByHand`), both-half door writes, `setBlock` flags, and trapdoor close inside player AABB — `doorcorridor`, `doorparkourblue`, `trapdoorparkour`, `amertystcoral`, step 5 (`P2a`).
8. `port-button-press-schedule`: `ButtonBlock.useWithoutItem` / `tick` (to locate) stone vs wooden press duration, `setBlock` flags, neighbor updates, and arrow/entity hold — `docs/plan.md` step 5 (`P2a`), `butoonpressureplates`.
9. `port-block-place-commit`: `BlockItem.place` / `canPlace` `isUnobstructed` against placing player's AABB (feet vs below feet vs grazing), `setBlock` flags, and survival stack decrement — `mud`, `ametwoodbox`, step 5 (`P2c`).
10. `port-bucket-place-pickup`: `BucketItem.use` / `emptyContents` / `pickupBlock` (to locate) raycast fluid mode, waterlogging vs adjacent cell, `setBlock` flags, fluid `scheduleTick`, and bucket item swap — `docs/plan.md` step 5 (water bucket climb & undo ledger), `item-use-writes`.
11. `port-mining-destroy-speed`: `BlockState.getDestroyProgress` -> `Player.getDestroySpeed` (to locate) tool, Efficiency, Haste/Mining Fatigue, Aqua Affinity, and `!onGround` float order — `docs/plan.md` step 5 (`P2c`), `engine/law/src/mining.rs:19-27` (`MiningSample`).
12. `port-block-support-pop-family`: Census of `updateShape` synchronous support-loss pops (`TorchBlock`, `CarpetBlock`, `DoorBlock`, `BasePressurePlateBlock`, `ButtonBlock`, `ScaffoldingBlock`, etc., to locate) — `docs/plan.md` step 5 (ledger), batch `P2a`, `port-update-order`.

### Priority 3: Carried movement and wormholes (`docs/plan.md` step 8)
13. `port-boat-land-and-collision`: `AbstractBoat.floatBoat`, `getGroundFriction`, and `move` for `ON_LAND`, `IN_AIR`, `UNDER_WATER`, `maxUpStep`, and `outOfControlTicks` ejection — `docs/plan.md` step 8 (`P4a`), `engine/law/src/boat.rs`.
14. `port-boat-mount-dismount`: Boat mount slot rules (`interact` -> `startRiding`), `positionRider` `±105°` yaw clamp, and `getDismountLocationForPassenger` exit search (to locate) — `docs/plan.md` step 8 (`P4a`), `engine/law/src/movement.rs:124-129`.
15. `port-elytra-firework-and-impact`: `FireworkRocketEntity.tick` attached boost velocity formula & `lifeTime`, plus `handleFallFlyingCollisions` wall-impact threshold and damage (to locate) — `docs/plan.md` step 8 (`P4b`), `port-elytra`.
16. `port-pearl-raytrace-and-landing`: `ThrownEnderpearl.tick` / `onHit` (to locate) raytrace clip mode, `0.8f` vs `0.99f` water drag order, exact landed teleport coords, and `resetFallDistance` — `docs/plan.md` step 8 (`P4c`), `port-wormhole-facts`.
17. `port-portal-entry-and-create`: `NetherPortalBlock.entityInside` / `PortalProcessor` / `Entity.handlePortal` timer reset & tick phase, plus `PortalForcer.createPortal` frame/platform writes (to locate) — `docs/plan.md` step 8 (`P4c`), `port-wormhole-facts`.

### Priority 4: Law bugs found by the port backlog
18. `port-minor-collision-acos`: Whether `LocalPlayer.isHorizontalCollisionMinor` (to locate) clamps `dot / sqrt(a * b)` to `[-1, 1]` or returns `NaN < 0.13962633907794952 -> false` — `engine/law/src/movement.rs:818`, `monotone-tick`, batch `P1a`.
19. `port-boat-trig-index`: Whether `Vec3.yRot`, `AbstractBoat.getPassengerAttachmentPoint`, and `AbstractBoat.controlBoat` invoke `Mth.sin/cos(D)F` or `(F)F` across the 49 divergent yaws — `engine/law/src/boat.rs:5-11, 44-45`, `port-boat-rider-attachment`, batch `P4a`.
20. `port-regen-dead-guard-and-phase`: `ServerPlayer.tickRegeneration` / `Player.isHurt` / `FoodData.tick` behavior on `health <= 0.0f` (whether `saturation` or `foodLevel` still mutates when dead) — `engine/law/src/movement.rs:1257` (`server_regeneration`), `port-peaceful-regen`, batch `P3b`.
21. `port-lava-entity-inside`: `LavaFluid.entityInside` `CLEAR_FREEZE`, `LAVA_IGNITE`, and `Entity.lavaHurt` (to locate) order, fire ticks, and damage cooldown — `engine/law/src/movement.rs:1054` (`Inside::Lava` refusal), batch `P3a`.

## What was left out and why

- **Already settled in `lab/wiki/index.md` (not re-asked):**
  - `port-sprint-window` (2026-11-16): `Options.sprintWindow`, `LocalPlayer.sprintTriggerTime`, and `LocalPlayer.aiStep` double-tap sprint are already reproduced.
  - `port-amethyst-state` (2026-11-16): cluster shapes and waterlogged collision are already reproduced.
  - Mining state machine (`port-breaking-threshold`, `port-break-stop-boundary`, `port-mining-abort-target`, `port-mining-target-switch`): START `>= 1.0f`, STOP `>= 0.7f`, ABORT, and target-switch packets are already reproduced.
  - Placement legality & replaceability (`place-reach-legality`, `port-place-replaceability`): `handleUseItemOn` reach/border gates and `BlockPlaceContext.replaceClicked` are already reproduced.
  - Scheduled ticks & non-fluid updates (`port-update-order`, `port-collapse-schedule`, `port-collapse-due-tick`, `port-falling-block-landing-place`, `port-scheduled-tick-dedup`, `port-schedule-order-tie`): falling blocks, pistons, redstone wire, ladder pop, and tick dedup/tie-order are already reproduced.
  - Fluid mechanics (`port-fluid-tick-delay`, `port-water-delay-minimum`, `port-water-source-neighbor-count`, `port-flowing-water`, `port-fluid-height-witness`, `port-water-flow-axis`, `port-fluid-rules` checker): fluid queries, delays, source conversion, and `SimpleWaterloggedBlock` `== Fluids.WATER` check are already reproduced.
  - L0 & sprint-split bugs (`port-freeze-damage`, `port-freeze-step-fluid`, `l0-freeze-tick-111`, `port-contact-cooldown`, `port-step-contact-witness`, `port-sprint-packet-split`, `trial-sprint-jump-basis`, `trial-sprint-replay-input`): jar rules are settled and awaiting Terra integration (batches `P3a`–`P3b`).
  - Entity push & knockback (`port-ordered-push-query`, `port-push-order-noncommuting`, `port-push-threshold`, `port-entity-contact`, `port-entity-rules`): section order, twice-per-tick pair commits, and knockback resolvers are already reproduced (awaiting batch `P4d`).
  - Unloaded-chunk `vy = -0.1` client-only gate, basic portal wait/cooldown/search, and pearl throw/gravity/drag/RNG (`port-wormhole-facts`), plus free-glide elytra `_dcos` / `onClimbable` (`port-glide-cosine-exact`, `port-elytra-cosine-domain`, `port-elytra-delta`, `port-elytra-climbable-branch`) and still-water boat recurrence (`port-boat-delta`, `port-boat-surface-transition`, `trial-boat-water-surface-bits`, `port-boat-turn-trace`, `port-boat-rider-attachment`): already reproduced.
- **Not port-lens work for GLM:**
  - Rust production patches (`P1a`–`P4e` wiring into `movement.rs`, `world.rs`, `mining.rs`, `boat.rs`, `attributes.rs`) belong to Terra (`AGENTS.md`).
  - Planner/words/certificate architecture (`idiot-index`, `arc-certificate`, `gait-corridor`, `backward-tools`, `ProposalTable`) belongs to Astra/Opus/Terra.
- **Deferred to keep this batch to 21 crisp lanes (<1h each):**
  - Secondary `MobEffects` (`SLOW_FALLING`, `LEVITATION`, `DOLPHINS_GRACE`), directional `getStateForPlacement` (`ScaffoldingBlock`, stairs, slabs, trapdoors), non-block item uses (`BowItem`, `TridentItem`, `FishingRodItem`, `LeadItem`), non-boat riding (`Minecart`, horses), and `makeStuckInBlock` multi-block overlap (`WebBlock` + `PowderSnowBlock`).

queue: port-pressure-plate-hook port-crawl-pose-transition port-creative-flight-toggle port-status-effect-kinematics port-movement-exhaustion-branches port-use-item-on-dispatch port-door-trapdoor-toggle port-button-press-schedule port-block-place-commit port-bucket-place-pickup port-mining-destroy-speed port-block-support-pop-family port-boat-land-and-collision port-boat-mount-dismount port-elytra-firework-and-impact port-pearl-raytrace-and-landing port-portal-entry-and-create port-minor-collision-acos port-boat-trig-index port-regen-dead-guard-and-phase port-lava-entity-inside
