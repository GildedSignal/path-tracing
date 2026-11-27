# A map's datapacks as law inputs

**Goal:** Read a map's datapacks into the law as causes and effects, so the planner loads Parkour Spiral 3.0.5 on 26.3 and plans its first checkpoint-to-checkpoint stage with a sentence that replays, instead of refusing the whole map.

## Why

Hielke's maps carry elite human records on the speedrun.com `26.2+` board (`map/detail/benchmarks.md`), the goal's first scoreboard row. They now open with their datapacks and scoreboards intact (`scripts/map.ps1`, each map its own world on its version's rig). The law still refuses them whole: `WorldSnapshot::load` stops at the datapack barrier. A datapack is a causal program like Mojang's code: a cause (the player in a box, a score, a tag, a tick count) fires an effect. The law already has the effects: a block edit is a `WorldDelta`, a `tp` is a wormhole, `effect`/`damage` change the player's state, `spawnpoint` sets where a mission restarts. So a datapack needs no new search code, only the law reading it.

## Read first

- `map/` (loaded whole), `map/detail/engine.md`, `map/detail/representation.md` (wormholes, the kit as law input).
- `map/principles.md` 4, 6, 7, 11, 21, 22 (no bandaids: name a gap, never patch around it), 27.
- `jobs/law-26.3/report.md`: where the Spiral refusal is and why.

## The map

- The world: `D:\path-tracing-rigs\26.3\server\maps\spiral\` (read only; it is the rig's served world). Its datapacks: `main`, `menu`, `more_maps`, `random`, `time`, `training_mode` (166 `.mcfunction` files; `minecraft:tick` → `main:tick`). Hielke's terms forbid redistributing the map or its files: nothing from it enters git, quote no function bodies in the report beyond a line per finding.
- What the functions do to a player, counted: 48 `fill`, 20 `setblock`, 10 `tp`, 7 `effect` (resistance, instant health, infinite saturation, invisibility), 2 `damage @s`, 2 `attribute … burning_time`, 5 `spawnpoint`, 82 `summon`, plus scoreboard and tag bookkeeping. The `spawnpoint` targets are the checkpoints: a stage is checkpoint to checkpoint.
- Build with `$env:PATHTRACING_MC_VERSION = '26.3'`; `-McVersion 26.3` for scripts. The 26.3 jar and tables are staged (`scripts/versions.json`).

## The work, as you judge it

1. From `minecraft:tick` and `load`, the call graph of functions, each with its causes (selector boxes, scores, tags, the clocks it reads) and its effects on what the law sees: blocks, the player's position, velocity, health, food, effects, attributes, spawn, game mode. Generated from the files, not written by hand: it is the generator, stored once (principle 7).
2. Those effects in the law, in the forms it already has. Bookkeeping that changes no behaviour the law observes (titles, sounds, tellraw, the menu) is read and dropped, with the reason in the report.
3. What the law cannot yet hold (moving entities from `summon`, a cause the law cannot evaluate) is refused for the cells and ticks it can touch, not for the whole map: the rest of the map stays plannable. Name each such gap; do not approximate it.
4. `WorldSnapshot::load` (or its successor) reads Spiral with these, and `plan-track` (or a small bin of your own) plans the first stage, checkpoint to checkpoint, with bare legs and the kit the map gives, its sentence replayed through `law::try_tick` with the datapack's effects firing on their ticks.

## Gate

- Every existing suite green on 26.2 and 26.3 (`map/work.md` commands; `PATHTRACING_TRACK_WORLD` for the 26.3 fixture), with no witness row removed.
- Stage 1 of Spiral: a sentence that replays, its T, and the checkpoints' coordinates for the flight. The integrator flies it on the 26.3 rig (`.\scripts\map.ps1 spiral`, then `track run`); the game judges it.
- `report.md` in this folder: the call graph's size, each effect class and its law form, each refusal with its cells, what is not done. Raw numbers.

Codex's sandbox mounts `.git` read-only: leave verified edits in the worktree; the integrator commits.
