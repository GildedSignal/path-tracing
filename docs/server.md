# One mind in the server (Phase 2 spec, draft)

Bots are real players on Julien's server (S46), driven by one engine. This spec fixes the boundary between the server and the engine. It is a one-way door. The Designer red-teamed it on 9 November ("freeze with changes"); those changes are folded in below, with Julien's rulings (S52). It freezes after the packet-path Swarm 0 re-measure.

## What the numbers say, and what they don't

Swarm 0 (9 November, `D:\path-tracing-evidence\swarm-0-20261109-d`): server-side `ServerPlayer` bots on an in-process sink connection, packed together.

| Bots | Server ms per tick at 20 TPS (median / p99) |
|---:|---|
| 0 | 0.04 / 0.16 |
| 1 | 0.64 / 1.47 |
| 10 | 0.99 / 2.20 |
| 100 | 6.8 / 15.1 |

- **This measures the rejected design.** The prototype calls `doTick()`, which runs Mojang's movement on the server (`travel`). It never goes through `handleMovePlayer`. So about 65 µs per bot is the cost of server-side movement, not of the boundary below.
- **Memory:** about 2.5 MB resident per bot (1,751 → 1,994 MiB from 1 to 100 bots). At 10⁴ bots that's about 25 GB on a 31 GB machine.
- **Next measurement:** Swarm 0 again, through the packet path, with the cost split by component (`Profile.java`). Until then the 30 µs kill threshold has no data behind it.

The Fermi floor (representation.md) is about 3 ms per tick at 10⁴ bots, with one mind and lazy bots. The server mod, not the planner, decides scale.

## The savings (all in the server mod)

1. **Bots don't observe.** Game logic is safe here: tracking only sends packets (only `WitherBoss` overrides `startSeenByPlayer`, for its boss bar). Mob targeting, despawn, spawning, sleep and phantoms read `level.players()` and positions, never tracking. So:
   - **The tracking candidate list holds real players only,** filtered at `ChunkMap.tick`. Rejecting bots later, inside `updatePlayer`, still pays O((entities + bots) · moved bots).
   - **Each bot's `WAYPOINT_RECEIVE_RANGE` is 0** (a vanilla attribute). Otherwise the locator bar is a second n² term (`ServerWaypointManager.updatePlayer` loops over every transmitter).
   - Real players still see every bot normally.
2. **World changes reach the engine once.** A block-change and entity hook streams tick-stamped deltas into the engine's own world (the 2.1a snapshot, updated in place), not once per bot.
3. **Player queries go through a spatial index.** `Mob.checkDespawn` calls `getNearestPlayer` for every mob every tick, `PlayerSensor` sorts every player, and `NaturalSpawner` scans them all: O(mobs × players). The index keeps vanilla's list-order tie-breaks, behind a differential gate.
4. **Bot persistence in bulk.** `PlayerList.saveAll` writes three files per player on the server thread every 6,000 ticks (every 3 s of wall time at 100×). The engine saves bots in bulk instead.
5. **Lazy bots** (below).

## Movement: the engine speaks the vanilla client's packets

- **Engine → server, per bot per tick:** the vanilla client's serverbound sequence: `PlayerInput`, `PlayerCommand` (sprint, fall-flying), `MovePlayer` or `MoveVehicle`, `AcceptTeleportation`, `PlayerLoaded`, `ClientTickEnd`, keep-alive replies. The packets are injected at the vanilla packet phase, before the level tick. The mod calls the listener's `tick()` at the vanilla point; the sink connection isn't in `ServerConnectionListener`, so nothing else would.
- **The server validates them as for any client.** `handleMovePlayer` re-runs `ServerPlayer.move` with its own shift state and collisions, rejects horizontal error² > 0.0625, ignores vertical error within ±0.5, and calls `jumpFromGround` itself. Its speed check is skipped when the tick rate isn't normal, and 100× is harmless as long as one packet arrives per tick.
- **The server is a weak judge** (it accepts errors up to 0.25 blocks). **The judge stays the client:** the lab client audits sampled words at 100×.
- **Every server `teleport()` of a bot enters the deviation stream.** A correction is an anomaly with a witness.
- **Server-owned outcomes are nature:** damage, food, block updates, fluids, entities, pickups. The engine predicts them every tick for active bots, and each mismatch is an anomaly (engine.md, "How the law grows").
- **The alternative,** the server running Mojang's movement for bots, was rejected. The client-only parts (input conversion, the sprint latch, push-out) would differ from the real client, and then nothing judges movement.

## The self channel (server → engine, per bot)

A bot never sees its own packets, so "the connection sends nothing" would freeze it: after a `teleport()` the server ignores every move until the matching `AcceptTeleportation`, and resends every 20 ticks. The engine therefore receives, per bot:
- teleports, with their ids;
- impulses: explosions and `SetEntityMotion` (through `hurtMarked`);
- respawn and dimension change (each needs a `PlayerLoaded` reply);
- health, food, inventory, attributes, effects;
- keep-alive.

## Lazy bots

A bot executing a certified word skips the server's tick, and its state is the word evaluated at the current tick. Skipping the tick drops a lot (`ServerPlayer.tick`, `Player.tick`, `FoodData.tick`, stats, cooldowns, pickups, pushing, `Entity.move`'s vibrations and `stepOn`/`fallOn`). So:
- **(a) Positions are written every tick** (`absSnapTo`, plus `ChunkMap.move` on a section change). Chunk tickets, mobs, spawning and despawn never act on a ghost.
- **(b) Lazy only on a quiet footprint:** the word's footprint, widened by listener and interaction radii, holds no hooked block, no game-event listener (sculk) and no entity.
- **(c) Materializing writes a state record:** food with its tick timer and exhaustion, effects, cooldowns, air, fire, freeze and portal timers, `tickCount`, stats.
- **(d) A frozen gate:** lazy-then-materialized equals ticked-throughout, on the saved player NBT and the fields in (c).
- **(e) Visibility covers every channel** a real client receives: tracking, waypoints, sounds, block changes. A bot visible through tracking, sounds or block changes is ticked. Bots do appear on the locator bar (S52). It carries only positions, and (a) writes those every tick, so the locator bar alone doesn't force a tick.
- At the 1,000-bot flight Julien sees every bot, so laziness saves nothing there. At 30 µs per active bot that's 30 ms per tick: fine at 20 TPS, and about 330 bots fit at 100 TPS.

## Hosting the engine: a separate process

**The engine runs as its own process,** over shared memory, pipelined one tick ahead (the engine is anytime). If it disconnects, the bots drop to scripted idle.
- **Why not in-process** (the 9 November draft): a fault `catch_unwind` can't catch (abort, stack overflow, out of memory, a double panic, an access violation in unsafe, SIMD or GPU-driver code) would kill the JVM mid-save. Region writes aren't atomic, so Julien's world could be lost: that is ruin, not a price. And Windows locks a loaded DLL, so every engine rebuild would mean a server restart, which slows the feedback loop.
- **It costs little:** the engine keeps its own world either way, so in-process bought only simplicity.
- **What would flip it back:** a measured handoff at 2,000 TPS that can't be pipelined and costs more than about 5% of the 0.5 ms tick.

This keeps the 4 November default ("the engine as a separate Rust process").

## Interfaces (frozen with the spec)

- **Server → engine, each tick:** the tick number; world deltas; the self channel per bot; per bot, the server-owned outcomes (for the deviation stream); per real player, position and every visibility range; commands.
- **Engine → server, each tick:** per active bot, its serverbound packet sequence; per lazy bot, its position only.
- **Determinism:** the server's randomness is recorded and enters the engine as input (job 1.2).

## Bots are players (Julien, S52)

"they are a player unless it really causes an issue". So:
- **Sleep:** one sleeper skips the night, through the vanilla sleeping-percentage game rule (0, so one sleeper suffices). Bots then never block the night.
- **Phantoms** stay: they're natural mobs.
- **The locator bar** shows bots.

## Chunks: memory follows the area covered, not the bot count

- **The rule in 26.2 (javap):**
  - Every player holds a `PLAYER_LOADING` ticket at the *server's* view distance, one value for everyone (`ChunkMap.setServerViewDistance → updatePlayerTickets`).
  - A player's own render distance only decides what is sent to it (`getPlayerViewDistance` in `updateChunkTracking`).
  - The `PLAYER_SIMULATION` ticket, at simulation distance, decides what ticks.
- **The Fermi numbers.** Julien's server: view distance 12, simulation distance 10.
  - A lone player loads (2·12+1)² = 625 chunks, of which 441 tick.
  - A loaded overworld chunk is about 30–100 KB in the JVM: block palettes for its non-empty sections, light at 4 KB per section, heightmaps and object overhead.
  - So a lone player costs about 20–60 MB, and nearby players share theirs.
- **The bound is the area.** 10⁴ bots can't load more than the area they spread over. A 10k × 10k region is about 3.9×10⁵ chunks, 12–39 GB, and that's reached from about 900 evenly spread bots on. Bots on missions cluster (bases, farms, mines), so the real number is the area of the world where bots are at once.
- **Ways to save that keep the game vanilla:**
  1. **Measure the per-chunk KB** in the dispersed Swarm 0 cells (Swarm 0, continuation 2). The 30–100 KB range is a guess.
  2. **The 184-chunk ring between simulation distance (10) and view distance (12) doesn't tick.** It's held only to be sent to clients, and bots aren't sent chunks (they don't observe). A bot could load to simulation distance plus a one-chunk border, which is 29% less per lone bot. Nothing in that ring is simulated, but vanilla's behaviour at the edge (fluids or mobs reaching an unloaded chunk instead of a loaded, frozen one) could differ. So it ships only after a differential gate shows no change at the edge.
  3. **Lowering `simulation-distance` in server.properties** is vanilla, but it also changes what ticks around Julien. It's his call, and not proposed.
- **What doesn't help:** a smaller render distance requested by the bot's client (it changes only what's sent).
- **CPU, not memory, binds lone bots** (Fermi, 9 November):
  - A ticking chunk costs about 2–5 µs per tick: random ticks in its few reactive sections, a spawn attempt, and the weather checks.
  - 441 chunks is then about 1–2 ms per tick for each lone bot, so roughly 25–50 lone bots fit in the 50 ms tick at 20 TPS. Dense bots share their chunks.
  - The dispersed Swarm 0 cells test this. The floor is event-driven world ticking: each reactive block's next change drawn once, the same distribution at a cost per event, not per chunk. It's an idea for Julien, not yet a plan.

## Flight and kill

- **Flight:** Julien joins and sees 10–50 bots (S53), packed and spread out, walking scripted routes as players, at 20 TPS with p99 under 50 ms per tick. The chunk-ticking wall for lone bots (Chunks, above) is kept in mind for Phase 5, not solved now.
- **Kill:** if the savings leave the server above about 30 µs per active bot (through the packet path), or above about 1 MB per bot, or lazy bots can't materialize without a visible correction, the boundary is wrong. Re-derive it.
