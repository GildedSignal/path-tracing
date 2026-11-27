# A map's datapacks as law inputs

## The saved source

`datapack-graph` reads the six enabled unpacked packs in Parkour Spiral 3.0.5. At saved server tick `11,979,186` it finds **166 functions**, **2 function tags** and **61 other data resources**. The tags have 11 load and 5 tick entries. Seven saved callbacks are due at `11,979,187` (2), `11,979,188` (1), `11,979,192` (1) and `11,979,193` (3). The saved pack list on this copy contains vanilla and the six file packs.

Load, tick and the saved callback roots reach **132 function files**, 915 command sites and 149 distinct source-to-target pairs across 219 references. The two saved command blocks in the stage box `[-27, 48, -57]..[9, 64, -30]` raise those totals to **137 files**, 934 command sites and 152 pairs across 222 references. Both root sets have two unresolved literal references, `main:rank` and `main:win/display_time`. Reachability follows literal calls, schedule targets and function conditions. Advancement reward roots, dynamic macro targets, pack overlays and command sources outside the inspected box need their own resolution.

| Source | Command cell | Plate cell | Support cell |
|---|---|---|---|
| Race start | `[0, 50, -49]` | `[0, 52, -49]` | `[0, 51, -49]` |
| Checkpoint 1 | `[-17, 57, -39]` | `[-17, 59, -39]` | `[-17, 58, -39]` |

Both sources are unpowered, non-automatic repeating command blocks in the save. The first checkpoint's write uses the command source for position and the selected player for yaw. The plate is a weighted light plate with no collision support, so the planner uses the block beneath it.

## Consequences and causes

Counts are static command sites across the 137 files, including inactive branches.

| Sites | Consequence and law form | Refusal boundary |
|---|---|---|
| 17 `setblock`, 7 `fill`, 3 `place` | Canonical states in `WorldDelta`; neighbour and scheduled consequences belong to `SimWorld`. | Write predicates, relative entity origins, neighbours and structure bounds are unresolved. The snake has 7 write sites relative to the `snake_1` displays and a trigger at `[29, 222, -8]`. A pair of absolute redstone edits target `[27, 142, -15]`. |
| 8 `tp` | A wormhole with server-resolved pose and velocity, including relative flags. | Context selects the mover and sometimes the destination entity. Unknown destination or motion still refuses. The deep-dark exit source starts at `[-16, -46, -20]` with delta `[-2, 0, -2]`. |
| 4 `effect`, 2 `damage`, 2 `attribute` | Status grants supported by the existing effect book use it directly. Damage goes through the server hurt ledger and burning time belongs to the fire clock. | Resistance, invisibility, explosion/drown command sources and the burning-time attribute still need effect or context rules. The drown cause follows the tagged player between Y `50` and `52` and requires a water cell. The explosion source is centred at `[44, 140, 14]` with radius `8`. |
| 6 `spawnpoint`, 4 `gamemode` | A spawn write changes the mission's restart point. Supported game-mode transitions use `PlayerState::set_game_mode`. | Local-coordinate, cross-dimension, mounted and spectator side effects need their own facts. |
| 4 `summon`, 5 `kill` | An entity birth, removal or contact belongs to the entity light cone. | The reachable summons include relative firework and display births. The source selectors and entity motion must furnish their cells and ticks before a flight can use the affected area. |
| 5 `clear`, 4 `give`, 14 `item`, 1 `loot` | Inventory changes alter the kit's verbs and resources. The race-start clear originates at the start plate. | Item components, replacement rules, entity drops and loot choice need their own resolved facts. |
| 100 `data`, 25 `advancement`, 13 `team`, 6 `random` | Data and advancement rewards can feed causes. Team rules can change contact, and random results can choose a branch. | Entity/storage resolution, advancement roots and choice outcomes still need a sound footprint. |
| 247 `scoreboard`, 77 `tag`, 7 `schedule` | These commands can pass facts between causes. The reader preserves them, including the saved schedule's same-tick order. The player command evaluator can set observed player scores and apply tag changes. | Fake holders, scheduled re-arming, entity selectors and result stores are unresolved. Menu scores can lead to restart and spawn, so menu logic still participates in reachability. |
| 71 `title`, 22 `playsound`, 48 `tellraw`, 5 `particle` | The player evaluator drops these presentation leaves after resolving the command's conditions. They do not change the player's motion, resources or terrain in the supported execution context. | A consumer of the command's result, such as `execute store`, still refuses. |

The player evaluator takes an explicit single-player context. It can evaluate score and tag filters, game mode, distance, and block tests using the loaded registry's predicate. A missing rule returns its function, source line, player cell and tick. Whole-world propagation of a refusal through cells and ticks still needs the effect footprint. `WorldSnapshot::load` keeps the static-world gate. `DatapackSnapshot` makes saved terrain and its command sources available for inspection and a terrain proposal.

## Stage 1

The terrain proposal uses bare legs, Adventure mode, a settled start at `(0.5, 52, -48.5)` and the top face of B's support with a settled goal. Its 12 words replay through `law::try_tick` in **87 movement ticks**. The terrain light cone gives **19 ticks**. One cold solve measured `82,598 µs` after preparation of the terrain source. A swept-touch-only goal gives **85 ticks** and ends short of the checkpoint plate.

The detached 26.3 server received the law's movement samples through the server's normal movement-packet handler. All **87 of 87** positions matched, with **0** differing coordinates. It began with the map's forced respawn at A, armed plate A, and an active saturation effect. Plate B's power first rose to 1 after move **86**, at server tick `11,979,303`. The map changed the bot's forced respawn to B on that sample, with yaw `49.479675`. Move **87** ended at `(-16.06399816806303, 59.0, -38.947798310045556)`, health `20`, food `20`, saturation `20`. The sampled health was `19` at move `69` and back to `20` by move `78`.

The Rust player-command witness reads the same sources. The saturation command yields an effect-book entry of duration `-1` and amplifier `10`. The evaluator ticks that entry and runs the pressure-plate score function during an **87/87** position-exact replay through `law::try_tick`. With the observed command-block firing at move `86` as input it produces the game's respawn cell, yaw and pitch exactly. The water tag becomes true during the replay and the drown command's conditions stay false. This check covers the command predicates and writes from a resolved firing. The engine does not yet predict the repeating block's firing from redstone, or the rest of the pack's effects from their causes.

The saved-before versus saved-after comparison of the `17,612` cells in the stage box finds one changed state, B's plate at `[-17, 59, -39]` with power `1`. A final-state comparison has no visibility into transient writes.

This is a detached checkpoint witness, with the command effects run by Minecraft. The whole Rust replay still needs the function clock, remaining world effects and local refusals. The live `track run` flight belongs to the integrator. No map or world files are required in git.

## Reproduction and suite gate

```powershell
$env:RUSTUP_HOME = 'D:\rust\rustup'
$env:CARGO_HOME = 'D:\rust\cargo'
$env:PATHTRACING_DEPS = 'D:\path-tracing-deps\data'
$env:PATHTRACING_MC_VERSION = '26.3'
$env:CARGO_TARGET_DIR = 'D:\path-tracing-target\map-datapacks\26.3'
$world = 'D:\path-tracing-rigs\26.3\server\maps\spiral'
& D:\rust\cargo\bin\cargo.exe run --offline --manifest-path engine/Cargo.toml --release -p world --bin datapack-graph -- $world -27 48 -57 9 64 -30
& D:\rust\cargo\bin\cargo.exe run --offline --manifest-path engine/Cargo.toml --release -p plan --bin datapack-candidate -- $world D:\path-tracing-evidence\map-datapacks\stage1.tsv 0 51 -49 -17 58 -39 -27 48 -57 9 64 -30
```

Both workspace release suites exit 0 with **283 passed, 0 failed and 11 ignored** per version. The 26.2 suite uses an isolated fixture with the two fuller region files from the integrator's fixture. The abbreviated regions checked into this worktree omit chunks for several existing tracks. The 26.3 suite uses `D:\path-tracing-evidence\law-26.3\fixture-26.3`. No witness rows are removed.

Raw logs and both witness harnesses are under `D:\path-tracing-evidence\map-datapacks`. The detached server binds only to loopback and runs the map packs and four saved overworld regions copied around the stage. The compact sentence is in `stage1-sentence.txt`, with the command witness in `command-replay.txt` and the release suites in `cargo-26.2-final.txt` and `cargo-26.3-final.txt`.
