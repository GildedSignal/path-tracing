# Path tracing lab

A Minecraft 26.2 pathfinding project built around one measure. How many server ticks pass from the moment
you pick a destination until **PathBot** gets there, restores any temporary changes to the world, and
recovers its kit? Planning time counts. Health, hunger and inventory constrain what the bot can do along
the way.

## Philosophy

- **One law at three grains.** Mojang's code and the server are nature and the judge. The law is the
  generative structure that reproduces them: the tick recurrence, **words** (feedback programs that run from
  one event to the next) and the **medium** (travel time per region).
- **A tool is a law input you carry.** A bucket carries a water block, a pearl a teleporter. Its value is
  how much it lowers travel time where it's used, so creative moves fall out of the representation.
- **Compute scales with what's new,** not with bots × ticks: words are compiled once per distinct situation,
  reused wherever the pattern recurs, and recomputed only when their footprint changes.
- **One mind in the server.** Bots are real players on the server, driven by one engine, cooperating through
  shared fields and missions that end only when the world is restored.

Read [the framework](docs/representation.md), [the principles](docs/PRINCIPLES.md), [the engine](docs/engine.md)
and [the ladder](docs/ladder.md).

## Current work

The roadmap and its state are in [HANDOFF](docs/HANDOFF.md): Phase 0 consolidates the Rust law, the recorder,
the tracks and the 100× clock into one line, then Phase 1 makes the law fast and gives it the world.

## Run

Double-click in this folder:
- **`Play.cmd`**: the server and your client.
- **`Play with bot.cmd`**: the server, the bot on your second monitor, and your client.
- **`Stop GPT.cmd`**: stops every running GPT build job.
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
