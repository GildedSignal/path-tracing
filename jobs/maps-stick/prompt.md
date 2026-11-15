# Maps in one click, "_" in names, the stick in races, and the bot started from A

**Julien (S156), verbatim:** "this will be the last AI track i think, it's too expensive and can't get better, it doesn't scale we'll search for parkour maps in the future and the other kinds of maps and i'll just create start and end points with the stick for all of the maps, though i'd like to change maps easily in a click within the game otherwise it would be too slow and allow me to use "_" in the naming of the tracks. also a command to have the bot start the track when i click on its starting point with the stick. also while racing only i keep the stick in hand to allow me for a fast reset when i fail."

**Consumer:** Julien racing downloaded parkour maps, and the bot racing the same tracks.

**Read first:**
- `bot\src\pathtracing\Pointer.java` (the stick);
- `Tracks.java` (commands, A detection at :275, `startBot` at :445, the RCON teleport at :477);
- `TrackMode.java` (the human kit transaction; `withoutSticks` at :61 and :93);
- `TrackStore.java` (`validateName` at :51–52, the list filter at :96);
- `TrackInventory.java`, `TrackRcon.java`;
- `scripts\common.ps1`, `scripts\start-server.ps1`, `play.ps1`, `Stop server.cmd`;
- `engine\plan\src\bin\plan-track.rs`, and wherever it resolves the world's region folder.

## Decisions already made (do not reopen them)

Do the four parts in this order. Finish each one completely before the next. Do not stop between them.

### 1. "_" in names
- Track and map names match `[a-z0-9_-]+`. Change `TrackStore.java:52` and `:96`.
- Fix every other place that parses a track name: grep `bot\`, `engine\`, `scripts\` and `oracles\` for `a-z0-9-` and for name parsing in `tracks.txt` readers.
- Existing names stay valid.

### 2. The stick in races, and fast reset
- **The stick in races:** when human track mode applies a kit, it also puts one stick in the first empty hotbar slot and selects that slot. If the hotbar has no empty slot, refuse with a chat message naming the track.
  - Kit comparisons ignore sticks on both sides (`withoutSticks` on the kit and on the observed inventory).
  - Saved kits stay stick-free, and the run record's `kitSha256` is unchanged.
  - Outside track mode, nothing about the stick changes.
- **Reset:** in human track mode, a right-click with the stick, not sneaking, resets:
  - If a run is being timed, it is abandoned, nothing is recorded, and the chat says "Reset.".
  - The player is teleported to A's centre, (x + 0.5, y + 1.0, z + 0.5), keeping their current yaw and pitch, through the same command channel human track mode already uses (`TrackHumanClient.command`).
  - The existing on-A path then re-arms the track and refills the kit, as when he walks onto A. Write no second arming path.
- **Exit:** in human track mode, sneak + right-click with the stick runs `/track exit`.

### 3. The bot started from A
- Outside track mode, when not creating a track: if the stick's right-click hits a block that is a stored track's A, in the player's current dimension, the mod whispers `/msg PathBot track run <name>`. This is the same command a human types.
- Any other block keeps today's `goto`.

### 4. Maps as dimensions of the one world, switched in one click
Switching must not restart the server.

- **The importer:** a new script, `scripts\import-map.ps1 -Source <folder or .zip> -Name <name>`. `<name>` matches `[a-z0-9_-]+`. It:
  1. stops the server if it is running, the way `Stop server.cmd` does;
  2. takes the source's overworld: `region`, `entities` and `poi` from `<src>` (old layout) or `<src>\dimensions\minecraft\overworld` (new layout), unzipping a `.zip` into a fresh temporary folder first;
  3. copies them into `<server world>\dimensions\pathtracing\<name>\`;
  4. adds `data\pathtracing\dimension\<name>.json` to a datapack `<server world>\datapacks\pathtracing-maps\`:
     - dimension type `minecraft:overworld`;
     - generator `minecraft:flat` with no layers and biome `minecraft:the_void`;
     - a `pack.mcmeta` whose pack format is read from the staged 26.2 server jar's `version.json`, not typed in;
  5. records the map in `<rig>\maps.json`: the name, the spawn x, y, z read from the source's `level.dat`, the source path, and the import time.
- **The importer's limits:**
  - It refuses a name that already exists, and never deletes or overwrites a map.
  - It does not start the server. It ends by printing "Imported <name>. Start the game with Play.".
  - Map files, `maps.json` and the world are never committed (check `.gitignore`).
- **`/map` in game:** the client mod's new `/map` lists `overworld` and every map in `maps.json`, as chat lines he can click. Each line runs `/map <name>`.
- **`/map <name>`:** sends, through the player's own command channel:
  - `execute in pathtracing:<name> run tp @s <spawn>`;
  - then the same command for `PathBot`, so the bot follows him.

  `overworld` uses `minecraft:overworld` and the overworld spawn from the server world's `level.dat`. Read every `level.dat` with Minecraft's own `NbtIo`, never a hand parser.
- **The bot's track teleport** (`Tracks.java:477`) becomes `execute in <track dimension> run tp ...`, so a bot outside the track's dimension is brought into it. Remove the "Bot must be in the track's dimension" refusal at `Tracks.java:450` only if this teleport makes it unnecessary; otherwise keep it.
- **The planner:** `plan-track` reads the region folder of the track's dimension:
  - `minecraft:overworld` → the world's existing path;
  - `pathtracing:<name>` → `<world>\dimensions\pathtracing\<name>\region`;
  - any other dimension → refuse with its name.

## Must not touch

- `engine\law\`, `engine\words\`, `engine\world\src\` (read only), and `engine\native\`.
- Any game jar.
- Julien's saved tracks (`lab\tracks\*.json`, `*.kit.json`) and his runs (`lab\tracks\runs\`).
- **Launch nothing.** No game, server, client or RCON. You are the Terra builder: never run `scripts\launch.ps1`, `gpt.ps1` or `follow-up.ps1`.

## Done means

**Gates:**
- `bot\build.ps1` builds;
- `cargo test --offline --workspace` passes in `engine\`, with `CARGO_TARGET_DIR=D:\path-tracing-target\maps-stick`;
- a Rust or Java test shows that the name rule accepts `hard_track_2` and `a-b` and refuses `A`, `a b` and `a/b`;
- `import-map.ps1` runs end to end against a synthetic source world you build under `D:\path-tracing-target\maps-stick\` (one region file and a `level.dat` copied from a fixture), with `-WhatIf`-style output pointed at a temporary world folder. It never touches the real server world.

**`jobs\maps-stick\last.md`** lists:
- the files changed;
- each part's behaviour in one line;
- what Julien must try in game to check each part: the in-game test is his, after the judge.

End with `STATUS: DONE` or `STATUS: BLOCKED: <why>`, blocking only on a question this prompt leaves open.
