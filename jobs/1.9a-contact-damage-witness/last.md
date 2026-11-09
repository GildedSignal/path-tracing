# 1.9a law and handoff

## First unsupported rule

A grounded entity's support block can generate an ordered contact damage request through `stepOn`. Its acceptance and health debit belong to the server damage path. The current Rust representation produces the same result for the two full floors and has no input for that contact request. The first missing event for settled standing on magma is `hotFloor(1.0F)` at the tick's block-effect phase. With full health and a clear cooldown, the first predicted health mismatch is the end of relative tick 0, Java 19 (`41980000`) and Rust 20 (`41a00000`). The server-side value is derived from bytecode pending the live witness.

I reran the existing GLM scratch executable at `D:/path-tracing-wt/fixtures/lab/results/local-20261107-1500-1-0-port-block-laws/target/debug/magma-stepon-check.exe`. Its source is `check/src/main.rs` in that result. The standing run starts at the center of a full floor, feeds neutral input for 21 ticks and asserts health 20 for each material. It reports 20 for magma and netherrack. Its separate landing row reports 10 for both floors. The executable observes Rust health. Its accompanying claim about hotFloor every contact tick concerns requests and cannot establish debits. [Frozen gate](gate.md)

## Staged 26.2 bytecode

Run the following from any directory. Offsets below are decimal bytecode positions inside the named methods.

```powershell
$deps = 'D:/path-tracing-deps/data'
$javap = "$deps/java/java-runtime-epsilon/bin/javap.exe"
$server = 'D:/path-tracing-lab/server/versions/26.2/server-26.2.jar'
$classes = @(
    'server.MinecraftServer', 'server.level.ServerLevel', 'server.level.ServerPlayer',
    'server.network.ServerGamePacketListenerImpl', 'world.entity.Entity',
    'world.entity.LivingEntity', 'world.entity.Avatar', 'world.entity.player.Player',
    'world.level.block.Block', 'world.level.block.Blocks', 'world.level.block.MagmaBlock',
    'world.damagesource.DamageSources', 'world.damagesource.DamageTypes'
)
foreach ($class in $classes) { & $javap -cp $server -c -p -s "net.minecraft.$class" }
$client = "$deps/libraries/com/mojang/minecraft/26.2/minecraft-26.2-client.jar"
& $javap -cp $client -c -p -s net.minecraft.client.player.LocalPlayer
& "$deps/java/java-runtime-epsilon/bin/jar.exe" tf $server |
    Select-String 'hot_floor|bypasses_cooldown|damage_type/is_fire'
```

- **Server phase.** `MinecraftServer.tickChildren` calls the level at 172 and the connection phase at 233. `ServerLevel.tickNonPassenger` increments the entity tick count at 15 and calls entity `tick` at 48. `ServerPlayer.tick` decrements positive `invulnerableTime` at 22–35. `ServerGamePacketListenerImpl.tickPlayer` calls `ServerPlayer.doTick` at 50. `doTick` calls `Player.tick` at 15 and runs `FoodData.tick` at 46. `LivingEntity.baseTick` skips its own `invulnerableTime` decrement for a `ServerPlayer` at 474–488. The world tick and the entity tick are independent counters.
- **Grounded dispatch.** `LivingEntity.tick` calls `aiStep` at 178. In `LivingEntity.aiStep`, 620–638 selects the server or the locally authoritative client and calls zero-argument `applyEffectsFromBlocks()` at 638. That overload falls back to an old-position/current-position movement at 34–66 if needed and calls `applyEffectsFromBlocks(List)` at 140. In the list overload, 1 checks `isAffectedByBlocks`, 9 checks `onGround`, 16 picks `getOnPosLegacy`, 25 reads the block state and 40 calls `Block.stepOn`. `getOnPosLegacy` passes `0.2f` to the support-aware `getOnPos(float)` at 1–4. The witness hooks offset 40 in the list overload.
- **Request.** `MagmaBlock.stepOn` checks `isSteppingCarefully` at 2 and the living-entity type at 10. It obtains `hotFloor` at 22 and passes `fconst_1` to `Entity.hurt` at 25–26. The parent `Block.stepOn` is a return at 0. `Blocks.<clinit>` uses the plain-block registration for netherrack at 10595–10636. `DamageTypes.<clinit>` registers the `hot_floor` key at 74–86. The data entry `data/minecraft/damage_type/hot_floor.json` uses the message ID `hotFloor` and scales only when caused by a living non-player. The block supplies no causing entity.
- **Sneak and pose.** `Entity.isSteppingCarefully` returns `isShiftKeyDown` at 1–4. On the server that reads shared flag 1 at 1–2 of `Entity.isShiftKeyDown`. `ServerGamePacketListenerImpl.handlePlayerInput` stores the input at 20 and sets the flag from `Input.shift` at 45–48 for a loaded client. `Player.getDesiredPose` chooses crouching at 44–64 for a shifting, nonflying player in the ordinary walking regime, and standing at 65 otherwise. `Player.updatePlayerPose` applies the resulting fit-checked pose at 66. The gate checks both pose and key.
- **Acceptance.** `Entity.hurt` dispatches to `hurtServer` at 24 only for a `ServerLevel` and discards the return at 27. `ServerPlayer.hurtServer` checks invulnerability at 3 and delegates at 95. Its `isInvulnerableTo` additionally checks changing dimension and client readiness at 10–40. `Player.hurtServer` checks the player ability at 11–32 and dispatches through `Avatar` at 120 to the inherited `LivingEntity.hurtServer`. Its entry checks invulnerability, death and fire resistance at 3–41. At 185–216 it returns false when the cooldown exceeds 10, the source lacks `BYPASSES_COOLDOWN` and the amount is at most `lastHurt`. A stronger hit in the window passes the difference to `actuallyHurt` at 217–226. Outside that branch, 240–255 saves the amount, sets the cooldown to 20 and passes the full amount. The separate 10-tick hurt animation is set at 258–269. With the frozen setup, the accepted return and one-point debit follow from `Player.actuallyHurt`, which applies armor, magic and absorption at 10–48 and writes health at 113–120. The normal-difficulty, bare, effect-free case leaves the raw amount intact.
- **Fire damage.** The server jar's `data/minecraft/tags/damage_type/is_fire.json` includes `minecraft:hot_floor`. `Player.isInvulnerableTo` consults `FIRE_DAMAGE` at 81–115. The recipe enables and records that gamerule. The staged jar has no built-in `bypasses_cooldown` data entry for hot floor.
- **Client boundary.** Client code reaches the contact logic for the locally authoritative player. `Entity.hurt` exits at 28 for a client level. The server spends health. `LocalPlayer.hurtTo` applies the server's health update at 22, 53 or 75.

## Checks executed

- Read the staged 26.2 server and client disassembly and the server damage-type data. Reran the existing GLM Rust scratch witness.
- Compiled and packaged the probe with the staged JDK. Fabric applied the required Mixins and linked both target classes in the offline transform check.
- The verifier self-test passed six synthetic case/rate traces and rejected six faults, including the every-request-accepted and dropped-authoritative-transition mutants. The CLI also refused a missing capture. The PowerShell recipe blocks parsed offline.
- The live 20 TPS reference and 2,000 TPS recheck are pending integrator capture.
