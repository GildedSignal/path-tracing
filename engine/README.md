# Movement law

The dependency-free workspace contains `law`, a one-player movement tick port from the unobfuscated 26.2 client jar. L0 awaits recorder fixtures and a fresh judge. No live game or server result is claimed here.

## Build and checks

Run Cargo **from `engine/`** so it reads `.cargo/config.toml` and writes to `D:/path-tracing-target`.

```powershell
Set-Location engine
cargo test --offline
cargo test --offline --test law -- --nocapture
cargo build --offline --features mutant-axis-tie
cargo build --offline --features mutant-step-up
cargo build --offline --features mutant-edge-backoff
cargo build --offline --features mutant-ceiling
```

Invoking Cargo from the repository root with only `--manifest-path engine/Cargo.toml` bypasses the nested configuration and writes to `engine/target/`. That invocation occurred during this continuation. Cleanup was denied at `D:/path-tracing-wt/law/engine/target/.rustc_info.json`, so local artifacts remain. All final commands run from `engine/` and use the configured D: target directory. From the root, add `--config engine/.cargo/config.toml` explicitly.

The installed toolchain lacks the rustfmt component. No Rust or development dependencies are used.

## Interface and coverage

`tick(&mut PlayerState, &impl World, &Input)` advances one tick. `World` supplies block collision shapes and contexts, material factors, climbability, fluid height/current, block hooks, loaded chunks, environmental settings and extra collision shapes. `BoxWorld` provides sparse blocks, inclusive regions and those settings for tests. It does not read the block dump.

- `LivingEntity.aiStep` thresholds, jumping, jump delay, air/ground and fluid travel, friction/drag modifiers, climbable handling and fall calculations.
- `Entity.move`, axis-ordered collision sweeps, candidate-height step-up, supporting-block selection, stuck multipliers, speed/jump factors, fall-distance bookkeeping and fall-reset rays.
- `Player` edge back-off, standing/crouching/swimming poses, airborne acceleration, food threshold, movement/jump exhaustion and food-data updates.
- LocalPlayer's sprint latch and four-corner push-out, with suffocating-block queries.
- Cobweb and powder-snow stuck hooks, powder-snow collision contexts, honey sliding, bed distance halving and supplied landing/bounce properties.
- Inside-block visited-set handling for block or fluid intersections, water extinguishing and explicit unsupported-hook failures.
- BlockGetter.clip voxel traversal for fall-reset hits, water shapes and the player end-portal/end-gateway rule.
- `Mth` floor/casts and the full sine table. `sin.bin` is extracted directly from the jar's initialized `Mth.SIN`, preserving every float bit.
- Voxel occupancy, coordinate merging for intersections, collision sweeps, AABB clipping and the block-intersection traversal used by inside hooks.

Each port function carries its Mojang method provenance. No Azalea code was copied.

Ground acceleration and the movement-speed term in water call `Player.getSpeed()`, which reads the current attribute. `LocalPlayer.aiStep` updates sprint and its modifier before travel, so sprint and supplied modifier changes affect that tick's acceleration. `Player.aiStep` writes the separate `LivingEntity.speed` field after travel. The attribute has a base and non-sprint modifiers. The law supplies sprint once and computes the getter value once per tick. Calculation follows ADD_VALUE, ADD_MULTIPLIED_BASE (using the value after additions), ADD_MULTIPLIED_TOTAL, then the attribute clamp, with vanilla Identifier hashing and fastutil iteration within each operation.

## Regenerating jar evidence

From the repository root, run `engine/probes/Regenerate.ps1`. It uses the staged Java runtime and libraries under `D:/path-tracing-deps/data`, extracts `sin.bin`, and regenerates every TSV corpus in law/tests/data/ through JarOracle.java. The Rust jar tests compare collision results bit for bit, shape intersections and ordered block visits. The corpora also call the real sprint predicates, landing hooks and fall-damage routine, epsilon routines, ray traversal, shape clipping and movement-speed attribute calculation (including operation-map rehash boundaries). Small Java test doubles supply player/environment observations. Constructor-free probe allocation uses the staged runtime's Unsafe.

Braking expectations come from the jar's input modification, input-vector, friction-speed, modified-friction and movement-attribute routines. The probe supplies the recurrence and threshold and exercises `Player.getSpeed()` with the attribute updated before travel and the stored field refreshed afterward. Both braking tests remain regression-only. Whole-player tick validation belongs to recorder replay.

`probes/ReadMethods.ps1` selects methods from the ignored disassemblies in `.bytecode/`. Regenerate a disassembly with the staged `javap.exe -cp <client jar> -c -p -constants <class>`.

The steady-speed, ladder, terminal-fall and fall-damage checks are generated by `tests/law.rs`. They match every requested printed speed. The jump-speed checks tap jump on landing and release it in flight. They keep food full to isolate the movement recurrence.

## Braking

Both Rust braking tests warm up with forward input, then measure the first `aiStep` horizontal threshold event. Tick 1 is the first tick after forward ends. Reverse k means hold back for k ticks and then release, with k from 1 through 6. Food and terrain remain fixed.

JarOracle regenerates these expectations for both `braking_recurrence_regression` and `braking_player_tick_regression`. Reverse walking acceleration applies immediately. The standalone Braking.java is a historical regression probe.

| Ground / initial gait | Release | Reverse 1 | Reverse 2 | Reverse 3 | Reverse 4 | Reverse 5 | Reverse 6 |
|---|---:|---:|---:|---:|---:|---:|---:|
| Stone / walk | 8 | 5 | 8 | 10 | 11 | 12 | 13 |
| Stone / sprint | 8 | 6 | 8 | 10 | 11 | 12 | 13 |
| Ice / walk | 38 | 36 | 35 | 33 | 30 | 25 | 7 |
| Ice / sprint | 40 | 39 | 38 | 36 | 35 | 32 | 27 |

## Replay and negative controls

See `../../lab/fixtures/README.md` for paired recordings and world descriptions. With no recordings, the build emits a warning and the replay test is ignored with an explicit reason. Adding recordings enables it automatically. A recording must contain at least one tick. Mismatched bits, anomalies, tick gaps and incompatible option headers fail. Unit checks exercise the parser and replay entry point, including one-bit corruption and an anomaly.

Replay compares raw positions/velocities, rotations, collision/ground/sprint/sneak/water/climb flags, pose and fall distance. It also compares minorHorizontalCollision, verticalCollisionBelow, speed/speedBits, swimming and crouching whenever supplied. The fixture's required player.isSprinting initializes the latch before simulating tick zero, and every tick's latch is compared. Optional fixture `serverEnd` snapshots check predicted health, food, saturation, exhaustion and foodTickTimer separately from packet-delayed client health. With atRest=true and a nonnegative signed serverTickOffset, replay advances that many zero-input ticks before comparison. Negative/unknown offsets, an absent/false atRest, or an absent snapshot are skipped with a reason. The final report counts fixtures compared and skipped. It reports per-fixture ticks/s and the min, median, p90, p99 and max across fixtures. No replay performance figures exist until fixtures run.

The four features alter the specified axis tie, step-height ulp, back-off decrement and ceiling restitution. The corresponding targeted Rust tests fail under their feature. L0 still requires the independent recorder fixtures to catch each mutant.

## Assumptions and remaining uncertainty

- Survival movement, a living player, no riding, elytra, creative flight, item use or status effects. The supplied options disable automatic/toggled input. Sprint starts through the jar predicates and remains latched until the run/swim stop predicate holds.
- Defaults describe an ordinary settled player. Fixture metadata must supply any differing hidden state, especially the jump delay, previous eye-water state, support block, food timer and pending stuck multiplier.
- The pre-tick player line records movementSpeedBase, speedModifiers, the sprint flag and independent sprintSpeedModifier presence. The local `set_sprinting` method updates both sprint fields; synchronized entity-data inputs update only the flag. New rows supply startSprinting, startSprintSpeedModifier, startPose and startSwimming (the server's echoed pose and swimming flag: the pose sizes the box for that tick's move) before controls, plus the existing speedModifiers and ticksFrozen inputs. END sprintSpeedModifier and speed remain compared outputs. Legacy settled fixtures keep their original initialization; the terrain corpus requires all independent observations. The optional initial speed records `Player.getSpeed()`. Travel derives it from the attribute after the local sprint update. Modifier lists seed fresh vanilla operation maps in listed insertion order. Idle end replay retains the last supplied attributes and facing.
- Block::full() assumes a suffocating solid. Custom boxes and replay blocks require explicit suffocating data. Set bed with a landing multiplier of 1, and supply hay/honey's 0.2 multiplier separately.
- Inside::None asserts no entity-inside hook. Unmodeled inside hooks use Inside::Unsupported(name), while unmodeled landing hooks use Block::unsupported_fall_on. Bubble columns and sweet berry bushes refuse inside contact, and pointed dripstone refuses an exercised fallOn. Ambiguous legacy fixture rows preserve both refusals.
- The supplied world is the complete collision/material/current description for that tick. Block updates, moving entities and currents must be refreshed by the caller. Entities and world borders can contribute supplied extra collision shapes.
- Whole-player bit exactness remains unverified until L0. Pose transitions, fluid entry/exit, support choices at mixed-material edges, simultaneous hooks and fall-reset rays need the live recordings. Jar corpora establish only their exercised routine inputs.
- The coupled movement tick estimates food and fall damage from client motion. The server resource replay uses `apply_server_resource_event` for ordered exhaustion, accepted health damage, player regeneration and `FoodData.tick`. `tick_server_resources` is the adjacent regeneration and FoodData case. The server entity tick count determines peaceful regeneration. The opt-in [1.5h gate](../oracles/1.5h-resource-order/gate.md) supplies the schedule independently of the client recording and compares the server checkpoints.
- Environmental fire, lava contact, drowning, suffocation and freezing damage, armor and damage invulnerability are outside the listed movement/food/fall methods and are not modeled. Server-only frozen-counter decay and removeFrost/tryAddFrost are not simulated: attribute packets are inputs. The client also runs InsideBlockEffectType.FREEZE, so its once-per-tick frozen-counter increment capped at the freeze threshold is retained, independently of speed.
- Shape boxes retain their supplied coordinate partition. Unusual unions whose source VoxelShape had optimized-away interior planes may expose different step candidates. The block dump adapter will need to preserve native shape coordinates or verify equivalence on those cases.

Explicit unimplemented! branches cover status effects, boats, riding, elytra/Math.cos, creative flight and item-use slowdown. Block contact also refuses unmodeled entityInside and fallOn hooks and lava's deferred CLEAR_FREEZE/LAVA_IGNITE/damage hooks.

Fall-reset clipping returns the hit/miss used by Entity.move, with the jar's endpoint extension, voxel tie order, inside-shape probe and AABB entry rules. endPortalOrGateway supplies the player-specific portal rule. resetsFallDistance supplies tag membership. A server-level forecast with a zero Nether-portal delay must also mark that eligible portal as resetsFallDistance. The caller supplies evaluated fluid heights, including same-fluid-above behavior.
