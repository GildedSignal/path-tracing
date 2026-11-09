# One mind in the server (Phase 2 spec, draft)

Bots are real players on Julien's server (S46), driven by one engine. This spec fixes the boundary between the server and the engine. It is a one-way door: the Designer red-teams it, Julien approves it, and then it freezes.

## What the numbers say

Swarm 0 (9 November, `D:\path-tracing-evidence\swarm-0-20261109-d`): server-side `ServerPlayer` bots with an in-process connection, packed together, scripted walking.

| Bots | Server ms per tick at 20 TPS (median / p99) |
|---:|---|
| 0 | 0.04 / 0.16 |
| 1 | 0.64 / 1.47 |
| 10 | 0.99 / 2.20 |
| 100 | 6.8 / 15.1 |

That's about 65 µs per bot. At 1,000 bots the server alone needs about 65 ms per tick, below 20 TPS; creation already slowed near 700. The 1,000-bot and dispersed cells are still to run.

The Fermi floor (representation.md) is about 3 ms per tick at 10⁴ bots, with one mind and lazy bots. So the server mod, not the planner, decides scale, and its three savings come first.

## The three savings (all in the server mod)

1. **Bots don't observe.** A bot's connection sends nothing, and bots are excluded from entity tracking as viewers, because the engine already knows every bot's state. This removes the n² term in dense crowds. Real players still see every bot normally.
2. **World changes reach the engine once.** A block-change and entity hook in the server streams tick-stamped deltas into the engine's own world (the snapshot from 2.1a, updated in place), not once per bot.
3. **Lazy bots.** A bot executing a certified word is not ticked by the server. Its state is the word evaluated at the current tick. It's materialized and ticked normally when:
   - a real player could see it (tracking range);
   - a delta touches its word's footprint;
   - its word ends, or a decision is due.

   Materializing must never show a real client a correction or a teleport: a bot inside a real player's tracking range is always ticked.

## Movement: who computes it

- **The default: the engine's law.** The engine computes each active bot's movement (bit-exact to the client, the L0 class) and submits it through the server's normal player-movement handler, as a real client would. The server validates it as it does for any client.
- **The judge stays the client.** The lab client audits sampled words at 100×.
- **Server-owned outcomes are nature:** damage, food, block updates, fluids, entities, pickups. The engine predicts them every tick for active bots, and each mismatch is an anomaly with a witness, the **deviation stream** (engine.md, "How the law grows").
- **The alternative,** the server running Mojang's common movement code for bots, was rejected. The client-only parts (input conversion, the sprint latch, push-out) would differ from the real client, and then nothing judges movement.

## Hosting the engine

**Recommended: in-process.** The engine is a Rust `cdylib` loaded by the server mod through Java 25's foreign-function API. The same crate also builds standalone for the offline fast loop.
- **It buys:** no second process, protocol or lifecycle; deltas and inputs cross as function calls with shared buffers; one clock.
- **It costs:**
  - A Rust panic must never cross into the JVM: every entry point catches unwinding and returns an error, and the mod then drops the engine's bots to scripted idle instead of crashing the server.
  - Debugging spans two runtimes.
- **The alternative is a separate process over shared memory.** It isolates crashes, and the cost per tick is microseconds. Either way the engine keeps its own world, so in-process saves less than it might seem. It wins on simplicity, not on speed.

This revises the "engine as a separate Rust process" default of 4 November, and needs Julien's yes.

## Interfaces (frozen with the spec)

- **Server → engine, each tick:**
  - the tick number;
  - world deltas (position, new state);
  - per active bot, the server-owned outcomes it predicted (for the deviation stream);
  - per real player, position and tracking range (for laziness);
  - commands.
- **Engine → server, each tick:**
  - per active bot, its movement and inputs (move, look, use, attack, hotbar);
  - per lazy bot, nothing until it's materialized.
- **Determinism:** the server's randomness is recorded, and enters the engine as input (job 1.2).

## Flight and kill

- **Flight:** Julien joins and sees 100 bots, then 1,000, walking scripted routes as players, at 20 TPS with p99 under 50 ms per tick.
- **Kill:** if the three savings leave the server above about 30 µs per active bot, or lazy bots can't materialize without a visible correction, the boundary is wrong. Re-derive it.

## Open

- **Chunk loading by dispersed bots.** Should a lone lazy bot keep chunks ticking? Vanilla would, because a player loads chunks. Not loading them changes game logic near the bot (crops, vines), so the default is vanilla behaviour, and its cost is what the dispersed Swarm 0 cells measure.
