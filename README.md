# Path tracing lab

A Minecraft 26.2 pathfinding project around one measure: how many server ticks pass from the moment you click a destination until **PathBot** gets there, with any temporary change to the world restored and its kit recovered. Thinking time counts. The goal is the best in the world, from first principles: a million-block straight line thought in microseconds, and the hardest tracks played as if they were easy.

**Start with [map/](map/):** [the goal](map/goal.md), [the principles](map/principles.md), [the architecture](map/architecture.md), [the vocabulary](map/vocabulary.md), [how we work](map/work.md) and [what is in flight](map/now.md).

## The idea in five lines

- **The law** is a Rust reproduction of Mojang's game, bit-exact where it matters; the game and the server are the judge.
- **The space under a kit is a crystal with defects:** where the ground gives the same answers, travel is one word and a count; thinking happens only at defects, so compute follows the terrain's complexity, never its length.
- **One loop for every terrain:** bracket T by the light cone, take the biggest word, cut where its certificate breaks, compose sets backward in min-plus, move forward at once.
- **A tool is a law input you carry:** a bucket is a water block, a pearl a teleporter; no tool gets its own search.
- **Many bots, one mind in the server,** sharing fields; building is the same problem in max-plus.

## Run

Double-click in this folder:
- **`Play.cmd`**: the server and your client.
- **`Play with bot.cmd`**: the server, the bot on your second monitor, and your client.
- **`Stop server.cmd`**: saves and stops the server.

From a terminal, `.\play.ps1` does the same as `Play with bot.cmd`; the first run calls `scripts\setup.ps1`.

## In game

Hold a **stick**: right-click a block and the bot goes to the face you pointed at; sneak + right-click stops it. The same as whispers: `/msg PathBot goto <x> <y> <z>`, `/msg PathBot come`, `/msg PathBot stop`.

**Tracks:** `/track` creates and races his courses from A to B; `/map` switches between the imported maps. The track clock times a run from leaving A to touching B, the same for him and for the bot.

**Flights:** `scripts\play-track.ps1 -Track <name> -Mode solve` (being built in `jobs/flat-ground`) has the bot play the planner's plan for a track in his game; `scripts\flight-scoreboard.ps1` compares the clock's T with the prediction and with his best time.

## How it is put together

- `engine/` is the Rust side: `law` (the tick law), `plan` (`solve` and the representation; `plan-track` sends a plan to the game), `world` (region files and the block table), `words` (the programs), `native` (the mod's channel).
- `bot/` is the Fabric mod in both clients (`-Dpathtracing.role` picks the half): the player half turns stick clicks into whispers; the bot half plays plans, records and runs the track clock. `bot\build.ps1` compiles it with plain `javac` against the unobfuscated game jar.
- Nothing in your Immersive Minecraft launcher changes: the scripts reuse its Java 25, Minecraft 26.2, Fabric and libraries from `%LOCALAPPDATA%\ImmersiveMinecraft\data`. Clients start offline as `Brain_Forge` and `PathBot`.
- Performance mods are copied from your pack; Lithium and C2ME stay out of the bot's client and the server, so the judge runs Mojang's own code.
- The server is vanilla 26.2 on Fabric, on 127.0.0.1 only. `scripts\rcon.ps1` sends it commands.
- Companions (GPT through Codex, Google through Antigravity, GLM through omp) are launched by `scripts\launch.ps1`; their jobs live in `jobs/<id>/`.
