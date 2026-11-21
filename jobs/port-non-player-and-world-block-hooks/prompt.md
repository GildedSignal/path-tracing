# Port delta: non-player and world-only block hooks in ModelTable (crops, sensors, rails, hoppers, eggs)

**Goal:** the jar's exact rule for this, with a reproduction that runs against Mojang's 26.2 jar: `dossier.md` (first line `Claim: `), `reproduce.sh`, and the Rust port delta as `work.patch`.

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`.

**Consumer:** `engine/world/tools/ModelTable.java:36-41` (`HOOKS` table), `engine/world/data/census.tsv` (13 refused hook classes), `engine/law/src/world.rs`.

**From the 26.2 server bytecode**, verify and state with class, method and offsets for all 13 classes:
- `entityInside` overriders: `CropBlock`, `PitcherCropBlock`, `EyeblossomBlock`, `LilyPadBlock`, `FrogspawnBlock`, `HopperBlock`, `DetectorRailBlock`:
  - Prove from the bytecode which entity types each checks (`instanceof Ravager`, `Bee`, `AbstractBoat`, `FallingBlockEntity`, `ItemEntity`, `AbstractMinecart`), and confirm whether ANY of these 7 classes ever mutates a `Player`'s position, velocity, pose, health, food, effects, or flags (or whether for a `Player` on foot they are pure `"none"` or `"world"` hooks in `ModelTable.java`);
- `stepOn` overriders: `SculkSensorBlock`, `CalibratedSculkSensorBlock` (inherits `SculkSensorBlock.stepOn`?), `SculkShriekerBlock`, `RedStoneOreBlock`, `TurtleEggBlock`:
  - Prove from the bytecode whether any of these 5 `stepOn` implementations ever mutates a `Player`'s position, velocity, pose, health, food, or flags, or whether they only mutate/schedule world state (`"world"` in `ModelTable.java`). Also state `TurtleEggBlock.stepOn` / `fallOn` trample conditions (`!isSteppingCarefully()`, chance `1/100` vs `1/3`).

**Build a jar-backed reproduction:** run `reproduce.sh` disassembling `entityInside` / `stepOn` / `fallOn` on all 12 classes above, and invoke each directly on a `Player` in a test harness verifying zero mutation of the player's motion, health, and flags. Update `ModelTable.java` in the worktree (`work.patch`) to classify verified `"none"` / `"world"` hooks and run it to show the new `census.tsv` modeled state count. Write `dossier.md` starting with `Claim: ` on its first physical line.

**Falsifier:** any player state mutation by those 12 classes on a walking/running `Player` that `ModelTable.java` would miss under `"none"` / `"world"`.
