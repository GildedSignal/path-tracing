# Swarm 0: what the server costs per bot

**Why.** Phase 2 (HANDOFF): bots are real `ServerPlayer`s on Julien's server, driven by one engine inside it (representation.md, section 7). Before designing the engine host, measure the server's own cost per bot. Its answer decides how early lazy bots must come. The Fermi floor: naive about 30 µs per bot per tick; with one mind and lazy bots, about 3 ms per tick at 10⁴ bots.

**Build (small, disposable).**
- A Fabric server mod, `oracles/swarm-0/`, that spawns N server-side players with an in-process fake connection: no network, no client.
- Each bot follows a scripted input loop, walking a fixed square and sprint-jumping, through the server's normal player movement path.
- A command to set N, and a layout: **dispersed** (bots far apart, so each loads its own chunks) or **dense** (all within a 32-block square).
- Use the reference rig (`PATHTRACING_ROOT=D:\path-tracing-lab`, port 25566, a disposable world copy), never Julien's world. Take the rig lock.

**Measure.**
- N = 0, 1, 10, 100, 1000, dispersed and dense, at 20 TPS.
- Then `/tick rate 100`, to see the highest sustained rate per N.
- Report the server's ms per tick as a distribution (min, median, p90, p99, max) over 600 ticks after a 200-tick warm-up, plus memory.
- Where profiling is cheap (JFR is available), give the share spent in chunk loading and ticking, entity tracking, player movement, and the rest.
- Check one bot's movement against the same inputs on the real client once. Count corrections, so legal movement can be told from teleports.

**Must not.** Touch `engine/` or `bot/`. Choose a planner or engine hosting. Run on Julien's server, or while he drives the bot.

**Done means.** The final message gives a table of ms per tick by N and layout, the sustained rate, memory, the profile shares, and the one-bot check, plus anything that refused to run and why. Answer it as fast as possible (AGENTS.md, "Tests answer fast"): no sweep beyond this matrix.

## Continuation

The reference rig (`D:\path-tracing-lab`) and `D:\path-tracing-evidence` are now writable for this job: take the rig lock and run the matrix as specified.
