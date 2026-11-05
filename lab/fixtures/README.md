# Movement replay fixtures

Place an unchanged job 1.2 recorder file at `<name>.jsonl` and its world description at `<name>.world.jsonl`, including in subdirectories. Keep other JSONL files elsewhere. No live recordings are included yet. The Cargo build checks for recordings and explicitly ignores the replay test while none exist.

## Recorder file

The harness accepts the implementation's optional first-line metadata object with `type` equal to `recording`. When present, `autoJump`, `toggleSprint` and `toggleCrouch` must be false and `sprintWindow` must be zero. Headerless files follow the same assumed options.

Every subsequent line is a post-tick state. `t` starts at zero and is contiguous. Optional `tickCount` must advance by one. Any `anomaly` field fails the replay.

- `inputs` contains `forward`, `back`, `left`, `right`, `jump`, `sneak`, `sprint`, `yaw` and `pitch`.
- `position` and `velocity` each contain x/y/z decimal fields and `xBits`, `yBits`, `zBits` strings of exactly 16 hexadecimal digits.
- `yRotBits` and `xRotBits` are eight-digit float hex strings.
- `onGround`, `horizontalCollision`, `verticalCollision`, `isSprinting`, `isShiftKeyDown`, `isInWater`, `onClimbable`, `pose` and 16-digit `fallDistanceBits` are compared.
- Optional minorHorizontalCollision, verticalCollisionBelow, speed/speedBits, swimming and crouching are also compared.
- Poses are `STANDING`, `CROUCHING` or `SWIMMING`.
- Recorder `health`, `foodLevel` and `saturationLevel` remain client observations. Use a server snapshot for resource comparisons.

The parser uses raw bits whenever supplied. It has no external dependencies. Its nested `inputs` and vector objects match `Recorder.java` and `TraceJson.java` from job 1.2.

## World description

Each line is an object with `kind`. Coordinates and all collision boxes are doubles. Blocks use local boxes, whereas extra collisions use absolute boxes. Regions have inclusive integer bounds. Explicit blocks override regions, and later regions override earlier regions.

A minimal settled fixture follows. Its numeric values describe the vanilla standing body and gravity drag, and are checked by `replay_stationary_tick` in `engine/law/tests/replay.rs`.

```jsonl
{"kind":"player","position":{"x":0.5,"y":0,"z":0.5},"velocity":{"x":0,"y":-0.0784000015258789,"z":0},"onGround":true,"pose":"STANDING"}
{"kind":"region","min":[-10,-1,-10],"max":[10,-1,1000],"boxes":[[0,0,0,1,1,1]],"friction":0.6}
```

Exactly one `player` line supplies the state **before recording tick zero**. It requires `position` and `velocity`. All other fields default to PlayerState::default(). The replay initializes the sprint latch from the recording tick-zero isSprinting before simulating that tick, following job 1.5b. Vector fields can include raw bits. Float/double scalar fields can include `<name>Bits`.

Player metadata supports the recorder flag names and these hidden fields.

- Boolean fields include `verticalCollisionBelow`, `minorHorizontalCollision`, `crouching`, `swimming`, `underwater`, `eyeInWater`, `wasEyeInWater`, `isInLava`, `leatherBoots`, `inPowderSnow`, `wasInPowderSnow`, `onGroundNoBlocks` and noPhysics.
- Float fields include `speed`, `stepHeight`, `frictionModifier`, `airDragModifier`, `waterMovementEfficiency`, `movementEfficiency`, `sneakingSpeed`, `health`, `saturationLevel` and `exhaustion`.
- Double fields include `movementSpeed`, `jumpStrength`, `gravity`, `safeFallDistance`, `fallDamageMultiplier`, `bounciness`, `fallDistance`, `waterHeight` and `lavaHeight`.
- Integers include `noJumpDelay`, `foodLevel`, `foodTickTimer` and remainingFireTicks.
- `mainSupportingBlock` is an optional integer triplet. `stuckSpeedMultiplier` is an optional vector. `inputs`, `yRot`, `xRot` and `pose` can also be supplied.

`block` requires `at` and `boxes`. `region` requires `min`, `max` and `boxes`. Boxes have the form `[x0,y0,z0,x1,y1,z1]`. Empty boxes represent a non-solid block. Optional block properties follow.

- Float properties include `friction`, `speedFactor`, `jumpFactor`, `fallOnMultiplier` and `bounce`.
- Boolean properties include `climbable`, `scaffolding`, `slime`, `fence`, `wallOrGate`, `suppressesBounce`, `resetsFallDistance`, `waterBlock`, `air`, bed, suffocating and endPortalOrGateway.
- `aboveBoxes`, `belowBoxes` and `sneakBoxes` supply context-dependent collision shapes. `insideBoxes` supplies the entity-inside shape, whose default is the full block independently of physical collision.
- `fluid` is `water`, `lava` or `empty`, accompanied by `fluidHeight` and optional vector `flow`. Heights and currents are evaluated world data, supplied by the fixture.
- `inside` is `none`, `cobweb`, `powderSnow` or `honey`. Every other inside name, including bubbleColumn and sweetBerryBush, fails explicitly through unimplemented!. Supply unknown hooks explicitly. The none value asserts that the block has no inside hook. An optional stuckMultiplier vector provides a generic hook.
- Optional `ladderFacing` and `openTrapdoorFacing` are matching direction identifiers. Equality is what matters for the trapdoor/ladder rule.

Set `waterBlock` for actual water or bubble-column blocks. Waterlogged solids retain their own speed-factor rule. Supply vanilla material and landing properties explicitly, including slime friction/bounce, hay/honey landing multipliers and honey speed/jump factors. `slime` supplies the slime landing-damage and step hooks. Powder-snow landing/collision/stuck hooks come from `inside`. Beds use bed=true and fallOnMultiplier=1. Set suffocating from vanilla data for push-out queries. endPortalOrGateway enables the player fall-reset portal rule. Lava contact fails explicitly at the deferred fluid hook.

An optional `world` line supplies `minY`, `difficulty` (the vanilla integer id), `naturalRegeneration`, `fastLava` and `unloadedChunks` as chunk-coordinate pairs. Their defaults are the `BoxWorld` defaults. Optional `collision` lines supply absolute extra collision boxes for entities or a world border.

An optional `serverEnd` line supplies `health`, `foodLevel`, `saturationLevel` and `exhaustion`, copied from a tick-aligned final server snapshot. All four are compared, with optional raw bits for the float fields. Supply initial server resources in the `player` line. The replay prints how many server snapshots it checked. A movement-only recording does not establish the server-resource part of L0.

Run `cargo test --release --test replay -- --nocapture` from `engine/` to compare fixtures and print tick-throughput distributions. L0 also needs an independent check of server corrections and coverage sufficient to catch every mutant.