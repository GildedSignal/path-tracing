# One mind in the server (the bots' server side, boundary spec)

Bots are real players on Julien's server (S46), driven by one engine. This spec freezes the boundary between the server and the engine (`oracles/server-probe/`, evidence in `D:\path-tracing-evidence\server-probe-20261118`, running FerriteCore, Krypton, and ScalableLux).

## What the measurements show

All 24 probe runs (`packet`, `lazy`, `virtual`, 20 TPS and 100 TPS) ran with **0 teleports** and **0 `"moved wrongly"` warnings**.

| Mode | Layout | N | 20 TPS median / p99 (ms) | 100 TPS median / p99 (ms) | Sustained TPS | Resident MiB | Chunks | Dominant JFR server-thread cost |
|---|---|---:|---|---|---:|---:|---:|---|
| Naive `doTick()` (`swarm-0`) | dense | 100 | 6.777 / 15.103 | 7.084 / 11.373 | 100.04 | 1,994 | — | waypoints 36.9%, tracking 20.2% |
| Naive `doTick()` (`swarm-0`) | dense | 1,000 | refused (timeout at #701) | — | — | — | — | **waypoints 58.2%, tracking 30.9%** |
| `packet` (observer fixes ON) | dense | 1 | 0.532 / 1.061 | 0.327 / 0.551 | 100.01 | 1,697 | 2,601 | movement 25–45% |
| `packet` (observer fixes ON) | dense | 10 | 0.576 / 1.190 | 0.379 / 0.708 | 100.00 | 1,721 | 2,601 | movement 23–50% |
| `packet` (observer fixes ON) | dense | 50 | 1.094 / 1.918 | 1.252 / 1.923 | 99.98 | 1,900 | 2,652 | movement 24–52%, chunks 28–30% |
| `packet` (observer fixes ON) | dense | 200 | **5.541 / 7.226** | **4.768 / 6.340** | 100.04 | 1,991 | 2,652 | **movement 56–67%**, chunks 9–11%, adv/inv 5–6%, push 3% |
| `packet` (observer fixes ON) | dispersed | 1 | 0.593 / 1.186 | 0.411 / 0.602 | 100.01 | 1,697 | 2,601 | movement 29–73% |
| `packet` (observer fixes ON) | dispersed | 10 | 5.545 / 8.275 | 3.883 / 5.040 | 99.98 | 2,385 | 16,681 | **chunks/spawning 85–93%** |
| `packet` (observer fixes ON) | dispersed | 50 | **31.154 / 54.526** | 26.330 / 38.029 | **37.15** | 4,094 | 60,681 | chunk-ticking ceiling breached at 50 |
| `lazy` (`absSnapTo` + section `move`) | dense | 50 | 0.342 / 0.640 | 0.257 / 0.409 | 99.99 | 1,720 | 2,652 | idle |
| `lazy` (`absSnapTo` + section `move`) | dense | 200 | 0.491 / 0.979 | 0.336 / 0.545 | 99.99 | 1,951 | 2,652 | idle |
| `lazy` (`absSnapTo` + section `move`) | dense | 1,000 | **1.142 / 2.758** | **0.927 / 2.197** | 100.02 | 2,390 | 2,756 | chunks/spawning 47–48%, tracking 4% |
| `virtual` (primitive arrays only) | dense | 1,000 | 0.035 / 0.117 | 0.029 / 0.089 | 100.02 | 1,301 | 0 | idle |
| `virtual` (primitive arrays only) | dense | 10,000 | **0.050 / 0.147** | **0.040 / 0.102** | 100.00 | 1,301 | 0 | **~1.2–1.7 ns/bot/tick**, 0 MB growth |

What those numbers prove:
1. **Observer fixes kill the 89% quadratic wall:** keeping bots out of `ChunkMap` tracking and `ServerWaypointManager.players` drops waypoint + tracking shares from 89.1% to < 1%.
2. **Active `ServerPlayer`s hit a single-thread physics ceiling well before 10⁴ bots:** slope 50→200 is **~23.4 µs/bot** at 100 TPS (`~29.6 µs/bot` at 20 TPS), i.e. ~235–296 ms/tick at 10,000 bots (5×–6× over the 50 ms budget). Running Mojang's `doTick()` / `travel()` directly on the server is rejected both for client/server divergence and duplicate compute.
3. **Dispersed vanilla chunk discs hit a 50-bot wall:** with `view-distance=12` and `simulation-distance=10`, each lone bot adds ~1,100 loaded chunk holders, **~42.7 MiB resident** (~39.8 KB/chunk, ~34.5 KB/chunk heap), and **~0.62–0.64 ms/tick**, breaching 50 ms p99 at 50 lone bots.
4. **Lazy `ServerPlayer`s cut CPU 33× (`~0.70 µs/bot` at 100 TPS, `~0.84 µs/bot` at 20 TPS),** yet empty `ServerPlayer` objects still cost **~0.71–0.75 MiB/bot** (~7.2 GB for 10,000 shells) and trigger `anyPlayerCloseEnoughForSpawningInternal` / `ServerChunkCache.move`.
5. **Virtual bots run at ~1.2–1.7 ns/bot/tick and ~0 JVM bytes.** 10,000 bots take 0.040–0.050 ms/tick inside the server.

The Fermi floor (representation.md) is about 3 ms per tick at 10⁴ bots in the Rust engine with three-tier presence. The server boundary, not the planner, decides scale.

## Three-tier bot lifecycle (Virtual → Visual Projection → Materialized `ServerPlayer`)

A `ServerPlayer` is a heavy materialization of a bot, never the bot itself. Every bot lives in the Rust engine (~32 bytes hot state / ~512 bytes full `PlayerState` + inventory) and takes on Java server weight only where causality or human eyes demand it:

1. **Virtual (quiet read-only travel away from humans and active server-only entities):**
   - Zero `ServerPlayer` objects, zero JVM RAM, zero server chunk tickets.
   - Rust holds the shared class-space world (loaded once in parallel from `.mca` region files and patched by the server's live delta stream). Away from loaded chunks Mojang's world is frozen, so read-only travel needs no JVM chunk loaded.
2. **Visual Projection (inside a human player's view distance or locator bar, outside direct vanilla interaction):**
   - Projected to real clients using vanilla protocol packets (`PlayerInfoUpdate` for skin/UUID/name, `AddEntity`, `MoveEntity`/`EntityPositionSync`, `SetEntityData` for sprint/sneak/pose, `SetEquipment`, `Animate` for arm swing, `TrackedWaypoint` for the locator bar) without instantiating or ticking a `ServerPlayer`.
   - If thousands cluster inside one human's view frustum, cap full animated models to the nearest K while simulating all 10⁴ truthfully in Rust.
3. **Materialized `ServerPlayer` (stripped, packet-path):**
   - Instantiated only within causal/interaction range of humans, mobs, containers, hooked blocks, active world edits, or rotating audit canaries (so sampled bots stay audited against Mojang's server and never drift into a simulacrum).
   - Speaks the vanilla client's serverbound packet sequence (`PlayerInput`, `PlayerCommand`, `MovePlayer` / `MoveVehicle`, `AcceptTeleportation`, `PlayerLoaded`, `ClientTickEnd`, keep-alive) into `ServerGamePacketListenerImpl` at the vanilla packet phase before the level tick, then calls `listener.tick()`.
   - Every server `teleport()` enters the deviation stream as an anomaly.

## The server savings and bytecode catches

1. **Bots don't observe or drive chunk loading in `ChunkMap`:** return `true` from `ChunkMap.skipPlayer(bot)` (stopping `distanceManager.addPlayer` from loading a `25×25` chunk disc per materialized bot), cancel `ChunkMap.updateChunkTracking(bot)`, filter bots out of `ServerLevel.players()` inside `ChunkMap.tick()`, `ChunkMap.addEntity()`, and `ChunkMap.move()`, and cancel `ChunkMap$TrackedEntity.updatePlayer(bot)` at `HEAD`. Real players still see materialized bots normally.
2. **`ServerWaypointManager` bytecode catch:** setting `Attributes.WAYPOINT_RECEIVE_RANGE = 0` alone does **not** eliminate the $O(N^2)$ loop, because `ServerWaypointManager.updatePlayer(bot)` still runs `Sets.difference(waypoints, row.keySet())` across all transmitters every tick. Keep bots out of `ServerWaypointManager.players` (observers) in `addPlayer`/`updatePlayer`/`removePlayer` while calling `trackWaypoint`/`untrackWaypoint` so real players still see bots on the locator bar.
3. **World changes reach the engine once:** a block-change and entity hook streams tick-stamped deltas into the engine's single world snapshot in shared memory, not once per bot.
4. **Player queries go through a spatial index:** `Mob.checkDespawn`, `PlayerSensor`, and `NaturalSpawner` otherwise scan `level.players()` every tick ($O(\text{mobs} \times N)$). Only materialized bots with presence sit in those lists.
5. **No synchronous per-bot player saves:** `PlayerListMixin` cancels `PlayerList.save(ServerPlayer)` for `SwarmBot`s so spawns, demotions, and `saveAll` avoid writing per-bot `.dat`, stats, and advancement JSON files synchronously on the server thread; bots persist through the engine in bulk.

## The self channel (server → engine, per materialized bot)

A bot on an in-process sink connection receives no network stream; after a `teleport()`, `handleMovePlayer` ignores every move until the matching `AcceptTeleportation` arrives. The server mod therefore forwards per materialized bot:
- teleports, with their ids and target coordinates;
- impulses: explosions and `SetEntityMotion` (`hurtMarked`);
- respawn and dimension change (each needs a `PlayerLoaded` reply);
- health, food, inventory, attributes, effects;
- keep-alive.

## Worldtube updating rules, chunk tickets, and presence

- **Exact 26.2 chunk ticket levels (`unloaded-chunk-move`, bytecode-pinned):**
  - `≥ 34` (`INACCESSIBLE`, not loaded): server collision treats blocks as air (ghosting), and `checkInsideBlocks` triggers a **synchronous main-thread chunk load** on `Level.getBlockState`. Never let a materialized bot enter an unloaded chunk.
  - `33` (`FULL`): collisions see real blocks; scheduled block/fluid ticks and entities are frozen.
  - `32` (`BLOCK_TICKING`): scheduled block ticks, fluids, redstone, and gravity blocks advance.
  - `31` (`ENTITY_TICKING`): mobs, dropped items (`pickupDelay`, aging), ender pearls, and minecarts tick.
- **Causal promotion rules:**
  - Read-only corridors carry a **hooked-block bitmask** (`stepOn`, `fallOn`, `entityInside`: pressure plates, tripwires, sculk sensors, farmland, dripleaf, redstone ore, turtle eggs, powder snow, lava, water, cactus, magma, berry bushes, cobwebs, portals). Any intersection promotes the bot and requests a level `32` (or `31`) ticket ahead of arrival.
  - Any world write (place, break, interact, container, dropped item, thrown pearl) requests a worldtube ticket ahead of arrival (`32` for block/fluid/redstone writes, `31` if entities/items/pearls are involved) and holds it until scheduled ticks, fluid spread, and falling blocks in that footprint settle.
- **Presence is a tool (S65):** GLM's `server-player-readers` audit (`lab/results/local-20261111-1108-1-5-server-player-readers/dossier.md`) maps every presence site in `server-26.2.jar` — **230 sites across 77 classes**, of which **38 sites** decide world liveness (17 spawning, 12 sleeping, 3 random-ticks, 2 raid, 2 entity-ticks, 1 siege, 1 despawning) plus `MinecraftServer.tickServer`'s `pauseWhenEmptySeconds` count check. By default a traveling bot holds no live-bubble presence; the swarm turns presence on only where a farm or site pays for it, and around a human everything stays vanilla.
- **Bots are players (S52):** `playersSleepingPercentage = 0` so one real sleeper skips the night, phantoms stay natural, and real players see bots with skins, names, and locator-bar waypoints.

## Hosting and vanilla-preserving perf mods

- **Separate Rust process over shared memory, pipelined one tick ahead:** an uncatchable fault (`abort`, stack overflow, OOM, unsafe/SIMD/GPU crash) inside the server JVM would kill the server mid-region-save and risk corrupting Julien's world (ruin, not a price), and Windows locks loaded JNI DLLs so every rebuild would force a server restart. Out-of-process IPC gives zero world-ruin risk and ~1 s Rust hot-reloads while the server and his client stay live. On disconnect, materialized bots drop to scripted idle. Only flips back to in-process if a measured handoff at 2,000 TPS cannot be pipelined and exceeds ~5% of the 0.5 ms tick.
- **Perf-mod policy:** `FerriteCore`, `Krypton`, and `ScalableLux` run in both the server and lab rig (strict vanilla parity). `Lithium` and `C2ME` run on the live swarm server (strict vanilla logic parity, parallel chunk loading/I/O) and stay out of the bare law-verification judge rig so the judge stays pure Mojang bytecode ([engine.md](engine.md)). `Alternate Current` is excluded because it alters vanilla redstone dust update order.

## Live server runtime and verified flights

`server-mod/` (`ServerMindMod`, `WorldBridge`, `SwarmManager`, `VisualProjector`, `SwarmFramePayload`, `SwarmBot`, `ShmBridge`, plus 10 mixins) connects dedicated Mojang 26.2 directly to out-of-process Rust `engine/mind/` (`mind.exe`, `WorkerPool`, `WorldMirror`, `DiskWorld`, `protocol`, `shm`) over Windows memory-mapped lock-free SPSC rings (`s2e` 16 MiB, `e2s` 4 MiB), paired with companion client mod `server-mod/client/` (`pathtracing-swarm-client`, built via `server-mod/build-client.ps1`):

- **Sharded parallel engine & lazy virtual execution (`WorkerPool`, up to 12 workers):** each worker owns an independent `GeneratorStore` and `Interner`. `goal_batch` and `goal_one` preload start-to-goal corridors via shared `insert_corridor_chunks` from parallel `DiskWorld` (cached `.mca` files opened with `FILE_SHARE_READ | WRITE | DELETE` and decoded in `std::thread::scope`), snap airborne goals down to solid supporting terrain (`snap_airborne_goal` / `snapGoalToGround`), and solve into `CertifiedStep` chains. Unhooked `Virtual` bots advance lazily by counter/arc with displacement-derived heading yaw, snapping at step boundaries via `step.end.materialize()` and replaying lazily only on tier promotion or block-edit interruption (`0.247 ms` median `mind_ms` and `1.142 ms` median `tick_ms` for `10,000` live bots; `13–43 MiB` `mind.exe` RAM; `10 passed; 0 failed` in `cargo test --offline -p mind`).
- **World delta stream & frozen `WorldMirror`:** initial and reconnected server chunks stream via `MSG_SECTION_AIR` / `MSG_SECTION_RLE`, stored compactly as `AllAir`, uniform-Y `Layered`, or bit-packed `Palette` sections with `FxHasher`. `LevelChunkMixin` queues live `MSG_BLOCK_DELTA` edits; `unload_chunk` removes server-loaded status while keeping sections frozen in `WorldMirror` for unloaded Virtual corridors.
- **Observer tiering, full/delta `swarm_frame` stream & Early-Z Blaze3D 3-LOD client rendering:** when modded clients are present (`MSG_RENDER_STREAM`), `mind.exe` skips non-materialized visual candidates in `CollectStepIntents` (`effective maxVisual = 0` when all humans run the client mod), emits compact `20 B/bot` frames (`MSG_RENDER_FRAME`, `32 B` origin-relative header; bit `31` of `count` flags sparse delta updates when only tier flips or subset changes occur), and forwards via Fabric's large payload `pathtracing:swarm_frame` while keeping `VisualProjector` vanilla packets active for unmodded clients. Client `SwarmBuffer` ingests full and delta frames with per-slot `frameSeq` interpolation (so idle bots do not jitter on partial delta ticks), derives the exact Mojang `DefaultPlayerSkin.get(UUID)` index from each offline bot UUID, frustum-culls against 5 Reverse-Z clip planes (`BOT_BOUNDING_RADIUS = 2.2`), and bins visible non-materialized bots into 3 front-to-back LODs (`<96`, `96..220`, `≥220` blocks) with precomputed yaw/limb trig (`32 B/bot`). `SwarmRenderer` binds Mojang's active `Projection` UBO (`ProjMat`, keeping terrain and bots 1:1 through view bobbing) alongside `camState.viewRotationMatrix` and draws all `10,000` opaque bots (`PlayerMesh`: `244` verts, `366` `short` indices across 6-box articulated LOD 0, color-matched 3-box rigid LOD 1, and 2-box rigid LOD 2; stitched `576×64` 9-skin wide atlas in `DefaultPlayerSkin` order; **zero `discard` in the fragment shader** so Hardware Early-Z works on both OpenGL 3.3 Core and Vulkan). `PlayerListMixin` suppresses join/leave chat broadcasts for `SwarmBot`s, `materialize_radius` defaults to `5.0` blocks (`+2.0` demote hysteresis, max `8`), and live OpenGL 3.3 testing on `AMD Radeon RX 7900 XTX` (`10,000` bots) draws `9,054` bots (`lod=4428/4626/0`) at `tick_ms` `1.812 / 10.145 ms`, `mind_ms` `0.438 / 8.998 ms`, `exact_T=10000/10000`, `teleports=0`, `0` join chat lines, and `0` `"moved wrongly"`.
- **Causal tickets & audit canaries:** materialized promotions check `world.is_server_loaded` on current and next corridor chunks, request/refresh `TicketType.ENDER_PEARL` tickets every 15 ticks via `WorldBridge.requestTicket` (`ticketLevel = Math.max(31, level - 1)` for `FULL` travel at `32` and `BLOCK_TICKING` hooked blocks at `31`), pace non-hooked promotions (`<= 1`/tick), and snapshot `prev_state` before stepping `law::try_tick` so mid-flight tier switches incur zero teleports. `corridor_has_hooked` scans sections and triggers immediate causal promotion when entering hooked blocks (`oak_pressure_plate`, fluids, inside blocks), while staggered audit canaries periodically sample Virtual bots against the server.
- **Commands:** `/swarm spawn <1..10000> [spacing] [auto|virtual|materialized] [x y z [cols]]`, `/swarm goto <x y z>` (snaps airborne or solid-surface coordinates to standing ground), `/swarm parade [10..500|off]`, `/swarm come`, `/swarm follow [on|off]`, `/swarm stop|remove|formation|stick|config|observer|status|profile reset`, and `/msg PathBot goto|stop`. On `mind.exe` reconnect, `engine_epoch` increments, `WorldBridge.syncAll` snapshots server-loaded chunks, and active `SwarmBot`s resync cleanly.

Stage 3 three-tier verification (`server-mod/verify-stage3.ps1`, evidence in `D:\path-tracing-evidence\stage3d-s3check-20261119`) passes all 5 flights with **0 teleports** and **0 `"moved wrongly"`**:

| Flight | Bots & mode | Goal | Solve (µs, min/med/max) | Exact T (`pred / act`) | Tick ms (`min/med/p90/p99/max`) | `mind_ms` (`med/max`) | Tiering, chunks & RAM |
|---|---|---|---:|---:|---|---|---|
| 1. 1k dense | `1,000 auto` (8 canaries) | `80` blks | `4.2 / 5.6 / 669.6` | `1000/1000` (`228`) | `0.559 / 1.434 / 7.201 / 24.262 / 194.795 (n=237)` | `0.110 / 15.707` | `mat:8`, `prom=26 dem=18`, `52` chunks, `1,352.3 MiB` |
| 2. 10k dense | `10,000 auto` (8 canaries) | `80` blks | `4.2 / 5.7 / 109.4` | `10000/10000` (`228`) | `0.417 / 1.142 / 5.408 / 12.539 / 42.824 (n=237)` | `0.247 / 9.269` | `mat:8`, `prom=26 dem=18`, `95` chunks, `1,582.1 MiB` (`23.2` mind) |
| 3. 1k dispersed `.mca` | `1,000 auto` (4 canaries) | `5310 -60 3960` | `4.0 / 5.5 / 18.6` | `1000/1000` (`171`) | `0.230 / 0.829 / 1.758 / 5.803 / 7.493 (n=179)` | `0.096 / 1.189` | `4,560` disk sec, `102` chunks, `tickets=107`, `1,581.6 MiB` |
| 4. Observer tiering | `200 auto` + `TestHuman` | `-140 -60 -200` | `4.5 / 5.4 / 54.6` | `200/200` (`169..171`) | `0.104 / 0.648 / 3.518 / 5.572 / 5.763 (n=188)` | `0.094 / 1.482` | `obs`: `virt:120,vis:75,mat:5`; `prom=34 dem=34`, `vis_pkts=11,847` |
| 5. Hooked plates | `1 auto` across `14..19` | `30 -60 0` | `70.8 / 70.8 / 70.8` | `1/1` (`87`) | `0.204 / 0.742 / 1.010 / 10.158 / 10.158 (n=95)` | `0.074 / 0.119` | `hook_prom=1`, `tickets=14`, lit `copper_bulb`s `15,16,19` |

Stage 2 regression (`server-mod/verify-stage2.ps1`, evidence in `D:\path-tracing-evidence\stage3d-s2check-20261119`) passes all 4 flights with **0 teleports** and **0 `"moved wrongly"`**:

| Flight | Bots | Goal | Solve (µs, min/med/max) | Exact T (`pred / act`) | Server tick ms (`min/med/p90/p99/max`) | Outcome |
|---|---:|---|---|---|---|---|
| 1. Single bot flat | 1 | `50 -60 0` | `137.8 / 137.8 / 137.8` | `1/1` (`144`) | `0.811 / 1.137 / 1.676 / 8.674 / 10.013 (n=153)` | `ready:1`, settled |
| 2. 50-bot flat | 50 | `80 -60 0` (grid) | `4.5 / 5.2 / 95.6` | `50/50` (`228`) | `1.068 / 1.795 / 3.449 / 7.803 / 14.537 (n=237)` | `ready:50`, settled |
| 3. Hot-reload + return | 50 | `mind` restart → `10 -60 0` | `4.4 / 5.0 / 150.8` | `50/50` (`204`) | `0.871 / 1.394 / 1.934 / 2.467 / 45.806 (n=213)` | `engine_epoch=2`, `9 → 27` chunks, settled |
| 4. Live `oak_fence` delta | 1 | `setblock 20 -60 0` → `30 -60 0` | `37.0 / 37.0 / 37.0` | `—` | `0.400 / 1.593 / 3.299 / 50.601 / 54.913 (n=510)` | `joints:1`, `NeedsJoints_at_[20,_-60,_0]` |

## Interfaces and gates

- **Server → engine, each tick:** tick number; world deltas (block edits, chunk loads/unloads, entity deltas); per-materialized-bot self channel and server-owned outcomes (for the deviation stream); per real player, position and view distance; stick/overseer commands.
- **Engine → server, each tick:** per materialized bot, serverbound packet sequence; per visual bot, light pose/animation/waypoint deltas; chunk worldtube tickets `(chunk, level, from_tick, to_tick)` and presence toggles.
- **Determinism:** server randomness is recorded and enters the engine as input ([footprints.md](footprints.md)).
- **Flight & kill:**
  - **Flight:** 10–50 bots (and scaled 1,000–10,000 dispersed/dense tests) join Julien's server, move directly from server world deltas on `solve`, promote seamlessly into real `ServerPlayer`s around him, and hold 20 TPS with p99 under the 50 ms tick budget inside the 30.88 GB machine RAM budget.
  - **Kill:** if virtual/visual/materialized promotion causes an unexplained teleport, chunk-ghosting, broken block update, mismatch inside a human's bubble, or breaches 50 ms p99 / machine RAM at target scale, re-derive the boundary.
