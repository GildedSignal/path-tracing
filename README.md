# Path tracing lab

A Minecraft 26.2 pathfinding project built around one measure. How many server ticks pass from the moment
you pick a destination until **PathBot** gets there, restores any temporary changes to the world, and
recovers its kit? Planning time counts. Health, hunger and inventory constrain what the bot can do along
the way.

## Philosophy

- **Run the law ahead.** Port the Minecraft rules that affect decisions to Rust. Given a world state and
  candidate controls, the engine should predict motion, resource costs and world changes. The goal is to
  evaluate many possible futures in the time a client takes to execute one.
- **Let capabilities shape the route.** Momentum, terrain and the bot's toolkit determine which moves are
  possible and what they cost. A bucket can open a route by changing the world. The planner should be able
  to discover that sequence by composing legal actions.
- **Reuse work across futures.** Share terrain, block rules and unchanged predictions. When something
  changes, recompute only the predictions that depend on it.
- **Use vanilla as the judge.** Compare player motion tick by tick with the vanilla client and check
  server-owned outcomes against the server. Once a case passes, push a boundary harder or combine mechanics
  that can interfere with each other. Reduce failures to the first divergent event and keep the replay.

Read the [project principles](docs/PRINCIPLES.md), [engine contract](docs/engine.md) and
[test ladder](docs/ladder.md) for the full design and verification rules.

## Current work

This branch contains the local two-client lab, course recorder and a straight-line placeholder planner.
The [Rust port branch](https://github.com/GildedSignal/path-tracing/tree/wt/l0) contains movement and collision
work, resource replay and tests against vanilla. The [clock branch](https://github.com/GildedSignal/path-tracing/tree/wt/clock)
contains the opt-in lab client that measured 100× active tick throughput on the focused replay workload.
That is a faster way to check predictions. The Rust engine's target is the throughput needed to compare
many futures before the next decision.

## Future directions

1. **Finish the port.** Extend coverage across movement modes, block and entity interactions, fluids,
   scheduled updates, tools and resources. Group cases by the rule they exercise and test how those rules
   compose in mixed terrain. See the [port plan](https://github.com/GildedSignal/path-tracing/blob/wt/l0/docs/port-plan.md).
2. **Run one headless bot.** Drive a server-visible player through the normal movement path and compare
   scripted runs with the vanilla reference. Use that result to measure the cost of scaling to 100 bots
   that share a world view.
3. **Develop the representation and planner.** Find ways to reuse reachability and motion predictions
   when goals, terrain or tools change. Judge routes by total server ticks, including the work to plan,
   execute and restore. The [representation notes](docs/representation.md) are a draft.

## Run

Double-click in this folder:
- **`Play.cmd`**: the server and your client.
- **`Play with bot.cmd`**: the server, the bot on your second monitor, and your client.
- **`Run queue.cmd`**: runs the GPT build jobs listed in `jobs\queue.txt`, one after another. Safe to run again after a shutdown. **`Stop GPT.cmd`** stops them.
- **`Set GLM key.cmd`**: paste a new Z.ai key into a hidden dialog. It is stored encrypted for your account, copied to the GitHub Actions secret, then checked once with Z.ai.

At spawn, the **Courses** button (4 blocks north) takes you onto the roof of the first test course. Each roof has **Spawn** and **Next** buttons. The courses are closed boxes: press F3+N to switch to spectator and fly in, and F3+N again to press buttons. `scripts\place-teleports.ps1` rebuilds the courses and the buttons.

From a terminal, the same as `Play with bot.cmd`:

```powershell
.\play.ps1
```

This opens the server console (type commands there, `stop` to save and quit), the bot's client (maximised on your
second monitor) and your client (fullscreen, with your Immersive keybinds). The first run calls `scripts\setup.ps1`
automatically.

## In game

Hold a **stick**:

- **right-click** a block: the bot walks to the cell in front of the face you pointed at (up to 256 blocks away)
- **sneak + right-click**: the bot stops

These are just whispers, so you can type them yourself:
`/msg PathBot goto <x> <y> <z>`, `/msg PathBot come`, `/msg PathBot stop`. The bot whispers back with the
outcome and the planning time.

The world is a superflat at permanent noon with clear weather, peaceful difficulty and no mob spawning. You are
in creative. The bot puts itself in survival on join, so it walks, jumps and falls like a normal player (and can
take fall damage).

## Where the algorithm goes

`bot/src/pathtracing/planner/Planner.java` is the one call that matters:

```java
List<BlockPos> plan(BlockGetter world, BlockPos start, BlockPos goal);
```

It returns the cells the bot's feet pass through, goal last. The current `StraightLine` planner is a placeholder
that returns only the goal and ignores the world, so the bot walks in a straight line and reports "Stuck" at the
first wall. To try a planner, change the `PLANNER` field in `Bot.java`, then:

```powershell
.\bot\build.ps1; .\scripts\setup.ps1
```

and restart the bot's client (`scripts\launch-client.ps1 -Role bot`). Planning runs on the bot client's main
thread, so a slow planner freezes that window while it runs.

`Bot.java` follows the path by holding the movement keys a person would: it turns at up to 30° per tick, walks
forward, jumps when blocked or when the next cell is higher, moves to the next cell once within 0.35 blocks, and
gives up after 3 seconds without getting closer.

## Scripting the server

`scripts\rcon.ps1` sends commands to the server over RCON (bound to 127.0.0.1 only), for example to build test
courses:

```powershell
.\scripts\rcon.ps1 "fill 0 -60 0 0 -58 20 stone" "tp PathBot 5 -60 10"
.\scripts\rcon.ps1 "execute as Brain_Forge run msg PathBot goto 20 -60 10"
```

## How it is put together

- **Nothing in your Immersive Minecraft launcher is changed.** The scripts reuse what it already downloaded
  (Java 25, Minecraft 26.2, Fabric Loader 0.19.5, libraries, assets) from `%LOCALAPPDATA%\ImmersiveMinecraft\data`.
- **Clients** (`clients\player`, `clients\bot`) start directly from `scripts\launch-client.ps1` in offline mode,
  as `Brain_Forge` and `PathBot`. Offline mode means default skins.
- **Performance mods** are copied from your pack (exact jars and configs): Sodium (+ Extra, Reese's options),
  FerriteCore, ImmediatelyFast, Entity Culling, More Culling (+ Cloth Config), BadOptimizations, Krypton,
  ScalableLux, Fast IP Ping, RSLS, Mod Menu and Fabric API. Dynamic FPS, Lithium and C2ME are installed in
  your client only.
- **The bot client** runs lighter: render distance 10, 60 FPS cap, no sound, never pauses or drops its frame rate
  when unfocused.
- **The server** (`server\`) is vanilla 26.2 on Fabric, with FerriteCore, Krypton and
  ScalableLux. It listens on 127.0.0.1 only, in offline mode, so nothing outside this computer can join.
- **The mod** (`bot\`) is a single jar in both clients; `-Dpathtracing.role` picks which half runs. The player half
  turns stick clicks into whispers. The bot half reads them, plans and walks. Because 26.x ships unobfuscated,
  `bot\build.ps1` compiles with plain `javac` against the game jar (no Gradle), as your modpack tools do.
- Re-running `scripts\setup.ps1` refreshes mods, configs and the bot jar. Your client's `options.txt` is only
  created once, so changes you make in game are kept; the bot's is regenerated each time.
- Server console quirk: commands piped in through stdin before the world loads fail with "An unexpected error
  occurred" (the console attaches each command to a level when it reads the line). Commands typed in the window
  or sent over RCON are fine.
