# Minecraft 26.3 port report

The build selects Mojang 26.2 or 26.3. Both Fabric mods and both preview fixtures pass. Spiral reaches the existing datapack refusal with the 26.3 law. The [jar diff](diff.md) gives the exact boundaries of lava ignition, ridden travel, entity collision, sleep and shield contact. The live flight remains the next gate.

The staged client and server SHA-1 values match Mojang's metadata. Both SHA-256 values and the Fabric API SHA-256 match Julien's staged values. The 26.3 server bundle provides Mojang's inner server jar and server-side libraries. The flight needs the launcher to supply the client runtime libraries, including SDL3 and the Windows natives. [Mojang client jar](https://piston-data.mojang.com/v1/objects/e877b6a07acd633fb3bb475002175cec036e7b87/client.jar), [Mojang server jar](https://piston-data.mojang.com/v1/objects/33680f5f2ac32864d6d7cf5e56a705fdb3e05f4c/server.jar), [Fabric 26.3 notes](https://www.fabricmc.net/2026/09/15/263.html)

## Changes and lines

`scripts/versions.json` holds the jar names, client digest, DataVersions, Fabric Loader, and Fabric API. `-McVersion` and `PATHTRACING_MC_VERSION` select the same jars for extraction and compilation. The mod builds package the selected release restriction in `fabric.mod.json`. One Rust engine selects that release's tables at compile time, which keeps `StateId` in the matching Mojang registry and costs no version lookup on the tick path. The region reader admits both supported versions. A snapshot requires the selected table's DataVersion and checks each chunk against the level.

- The default is 26.2 until cutover. `engine/world/data/26.3/version.txt` is `5023`. The default 26.2 tables stay at the root, so their paths in the checked-in witnesses remain stable.
- Bought: the generated 26.3 tables hold 1,286 blocks, 35,723 states and 1,658 item defaults. The generator models 35,609 states in 601 physical rows. The two table sets preserve Mojang's registry numbers and allow both builds to run without jar access from Cargo.
- Bought: the Rust and Java additions select version-matched tables and law hashes, carry the observed pearl, fluid, bed and spectator changes, and adapt the mod boundaries to Mojang's signatures. The kit reads the generated rules through `StateId` and drops its second TSV parser. `models.ps1` extracts block and item data in one call and replaces `use-effects.ps1`.
- The region changes from `bf7d5c6f` added 26 lines. The two region files modified at checkout and the untracked companion reports belong to other work. The migration uses a data-fixed fixture copy outside the repository.

Text lines against `bf7d5c6f`, including new files and excluding the existing binary fixture edits. Generated lines count as additions. The relocated 26.2 rules file nets to zero.

| Path | Added | Removed | Net | Bought |
|---|---:|---:|---:|---|
| `engine/` | 15,418 | 1,507 | +13,911 | 13,777 generated table and witness lines, plus 134 net lines for selection, changed rules, the extractor and jar-judged checks. |
| `bot/` | 131 | 66 | +65 | Selected Java APIs and mixin targets preserve the bot and recorder across both releases. |
| `server-mod/` | 136 | 99 | +37 | Selected tracker, acknowledgement and sync packets, with shared Fabric build steps. |
| `scripts/` | 118 | 18 | +100 | One manifest and shared version, classpath, extraction and metadata helpers. |

Staging returned `fatal: Unable to create 'C:/Users/PASCAL/Desktop/path tracing/.git/worktrees/law-26.3/index.lock': Permission denied`. The work is saved in the worktree for the integrator's commit.

## Read-only Spiral gate

`WorldSnapshot::load` at `[0, 53, -97]` with the 26.3 build reaches the datapack barrier. The call exits 1 with this reason.

```text
unsupported: law models require only the vanilla datapack; enabled ["vanilla", "axiom", "fabric-convention-tags-v2", "file/more_maps", "file/random", "file/time", "file/training_mode", "file/main", "file/menu"]
```

The pack needs command execution beyond the vanilla snapshot. `main:tick` applies lethal damage in tagged water zones, `main:tick_10` applies saturation, `main:tick_2` applies resistance, and `main:snake/1/tick` changes terrain. The main and training packs teleport players. These rules change what can be reached and the T for reaching it.

`world-probe` reads the three palette forms from the copy. An independent NBT read and unpack of the palette indices gives the same cells.

```text
-512 -64 -512: minecraft:stone
-416 48 -512: minecraft:air
-401 51 -512: minecraft:water[level=1]
```

The read-only Overworld palette audit reads 9,772 chunks across 127 region files (111 empty), all with zlib compression. 5,119 chunks are DataVersion 5023 (5,040 full) and 4,653 are 4903 (860 full). A corridor touching an older chunk still needs Mojang's upgrade on a separate copy. Five of the 1,321 distinct palette entries are refused by both tables as `non-local shape`. Each occurs once in the palette of chunk X/Z `[1, -1]`, section Y `8`.

| Unmodeled against the 26.3 table |
|---|
| `minecraft:lime_shulker_box` |
| `minecraft:magenta_shulker_box` |
| `minecraft:orange_shulker_box` |
| `minecraft:red_shulker_box` |
| `minecraft:yellow_shulker_box` |

The completed Vibe dossier `local-20261127-095742-21892-1-0-vibe-gen-shulker-box-and-ladder-vine-climb-live` measures sideways and vertical lid extension beyond the cell. The 26.3 `ShulkerBoxBlock.getShape` bytecode gives the same shape query. A palette omits the block entity, so the model cannot infer a closed lid from the state alone. The palette census is independent of the snapshot loader.

## Cargo, witnesses and mod builds

- The refreshed 26.2 baseline reports 268 passed, 1 failed and 11 ignored. `mind::tests::fence_delta_is_crossed_around` reports `world mirror cell [-16, -65, -16]: unloaded cell [-16, -65, -16]`. Its fixture starts at Y = −64. Julien reports the fix on another branch after `bf7d5c6f`. Raw baseline `D:\path-tracing-evidence\law-26.3\cargo-26.2-refresh.txt`.
- The 26.3 suite uses the local 5023 vanilla fixture generated by `UpgradeFixture.java`. It feeds the fixture through Mojang's DataFixer and writes a new destination. Raw logs are `cargo-26.2-gate.txt`, `cargo-26.3-gate.txt`, `cargo-26.2-mind.txt`, `cargo-26.3-mind.txt`, and `spiral-palettes-26.3.txt` under `D:\path-tracing-evidence\law-26.3\`.
- The bot and dedicated-server builds pass with Fabric Loader `0.19.5`, Fabric API `0.160.0+26.2` or `0.167.0+26.3`, and their selected game jars. Outputs are `D:\path-tracing-target\job-law-26.3\bot-<version>\pathtracing.jar` and `D:\path-tracing-target\job-law-26.3\server-mod-<version>\pathtracing-server.jar`. The compiled bot's 52 track and packet captures pass the bytecode audit against each jar. Build and capture logs use `bot-<version>-build.txt`, `server-<version>-build.txt`, and `bot-<version>-capture-audit.txt` in the evidence directory.
- Both `scripts/preview.ps1 -Fixture` runs report `PASS`, lantern `15`, six neighbors `14`, opaque wall `0`, 12 image sizes, 19 emitter/state controls, 72 pixels changed by the wrong-facing mutant, and zero refusals. Each `preview-<version>\fixture-result.txt` records the jar digest and the command.
- The optional 26.3 swarm renderer exits `1` (`server-client-26.3-build.txt`). The Maven metadata resolves `org/lwjgl/lwjgl/3.4.3/lwjgl-3.4.3.jar` (SHA-1 `7ab5265a73d8a959ed62332d546536ae6be2f0d5`) and its Windows native alongside SDL3 and the other LWJGL modules. The exact jars are absent from the staged client libraries, and `libraries.minecraft.net:443` returned `curl (7)`. Full client dependencies would restore `org.lwjgl.system.MemoryUtil`. The old imports `com.mojang.blaze3d.GpuFormat`, `buffers.GpuBuffer`, and `pipeline.RenderPipeline` also fail. The vanilla jar contains their RenderPearl counterparts under `com.mojang.renderpearl.api`. The optional renderer needs its own graphics port even with the complete libraries.

| Serial gate | 26.2 | 26.3 |
|---|---|---|
| `world`, `law`, `plan`, `pathtracing_native`, `build` | 256 passed, 0 failed, 11 ignored | 256 passed, 0 failed, 11 ignored |
| `mind --bin mind` | 13 passed, 1 failed at Y = −65 | 13 passed, 1 failed at Y = −65 |

The regenerated pearl momentum witness uses `PearlMomentum.java`. It reads the relative-delta flag from `ThrownEnderpearl.onHit` and executes the jar's `PositionMoveRotation.calculateAbsolute`. IEEE-754 bits, in X/Y/Z order:

```text
4903    3fd0000000000000    bfc0000000000000    3fe0000000000000
5023    0000000000000000    0000000000000000    0000000000000000
```

The world-hook witness uses block names to obtain these IDs from each regenerated registry. Its shape and trace assertions are unchanged.

| Block | 26.2 ID | 26.3 ID |
|---|---:|---:|
| `stone_pressure_plate` | 259 | 307 |
| `oak_pressure_plate` | 261 | 309 |
| `stone_button` | 275 | 324 |
| `tripwire_hook` | 401 | 452 |
| `tripwire` | 402 | 453 |
| `oak_button` | 443 | 495 |
| `light_weighted_pressure_plate` | 471 | 524 |
| `heavy_weighted_pressure_plate` | 472 | 525 |

`JarBoundaries.java` produced these values with the vanilla pack tags bound. The raw outputs are `jar-boundaries-26.2.txt` and `jar-boundaries-26.3.txt` in the evidence directory.

| Witness | 26.2 | 26.3 |
|---|---|---|
| Water beside oak sign / air over water | `(0, 0, 0)` / `(1, 0, 0)` | `(0, 0, 0)` / `(1, 0, 0)` |
| Cobweb, oak sign, oak slab, air admit fluid. Stone and structure void reject it. | all six match | all six match |
| White bed height, restitution, fall-distance reduction | `0.5625`, `0.75`, `0.5` | `0.5625`, `0.75`, `0.5` |
| Straw bed default height, restitution, fall-distance reduction | absent | `0.25`, `0`, `0` |
| `setBlockAndUpdate` write flags and recursion limit | direct three-argument write with flags `3` | `3`, `512` |
| Portal creation attempts on missing POI, ordinary / spectator | `1 / 1` | `1 / 0` |
| Temporary invulnerability with damage cooldown `20` and immunity `0`, then cooldown `0` and immunity `4` | one old field | `false`, `true` |

Select the release and run the offline gate from the worktree. The 26.3 selection needs the generated vanilla fixture in `PATHTRACING_TRACK_WORLD`.

```powershell
$env:PATH = 'D:\rust\cargo\bin;' + $env:PATH
$env:RUSTUP_HOME = 'D:\rust\rustup'
$env:CARGO_HOME = 'D:\rust\cargo'
$env:CARGO_TARGET_DIR = 'D:/path-tracing-target/job-law-26.3'
$env:PATHTRACING_DEPS = 'D:\path-tracing-deps\data'
$env:PATHTRACING_MC_VERSION = '26.3'
$env:PATHTRACING_TRACK_WORLD = 'D:\path-tracing-evidence\law-26.3\fixture-26.3'
cargo test --offline --manifest-path engine/Cargo.toml --release --no-fail-fast -p world -p law -p plan -p pathtracing_native -p build -- --test-threads=1
cargo test --offline --manifest-path engine/Cargo.toml --release -p mind --bin mind -- --test-threads=1
```

For 26.2 use `PATHTRACING_MC_VERSION=26.2` or leave it unset, and remove `PATHTRACING_TRACK_WORLD` to use the checked-in 4903 fixture.

To reproduce the 5023 vanilla fixture at a fresh destination, use the jar and its exact libraries extracted by `models.ps1`. This runs Mojang's data fixers without starting a server.

```powershell
$fixture = (Resolve-Path 'engine/world/tests/fixtures/world').Path
$upgrade = (Resolve-Path 'jobs/law-26.3/UpgradeFixture.java').Path
$models = Join-Path $env:CARGO_TARGET_DIR 'world-models'
. .\scripts\common.ps1 -McVersion 26.3
$jars = Get-ChildItem "$models/26.3/vanilla/libraries" -Recurse -File -Filter '*.jar'
$cp = (@((Get-VerifiedClientJar)) + $jars.FullName) -join ';'
Push-Location $models
try { & "$Jdk/java.exe" -cp $cp $upgrade $fixture $env:PATHTRACING_TRACK_WORLD }
finally { Pop-Location }
```

## Course bench

Seven repeats per course. Measurements from different runs share the workload. The [benchmark table](../../map/detail/benchmarks.md) gives the medians side by side.

```text
exact: 50 players x 2000 ticks, n=7, ns per player-tick min=1418.3 median=1525.5 max=1580.8; refused ticks=0
floored: 50 players x 2000 ticks, n=7, ns per player-tick min=3163.8 median=3326.7 max=3452.4; refused ticks=0
straight: 1 players x 2000 ticks, n=7, ns per player-tick min=2376.3 median=2634.8 max=3201.4; refused ticks=0
```

26.3 raw output:

```text
exact: 50 players x 2000 ticks, n=7, ns per player-tick min=1010.8 median=1013.9 max=1169.7; refused ticks=0
floored: 50 players x 2000 ticks, n=7, ns per player-tick min=2306.6 median=2331.9 max=2583.8; refused ticks=0
straight: 1 players x 2000 ticks, n=7, ns per player-tick min=1722.9 median=1733.5 max=2079.8; refused ticks=0
```

## Julien's 26.3 instance and flight

After the 26.3 law gate closes, make two separate 26.3 launcher instances on Java 25 with Fabric Loader `0.19.5` and Fabric API `0.167.0+26.3`. Use the bot jar above in both instances. Set `-Dpathtracing.role=player` or `-Dpathtracing.role=bot` and set `-Dpathtracing.rig` to the copy of the rig. Use matching role names (`Brain_Forge` and `PathBot`), connect both clients to the 26.3 server on the copied world, and install the dedicated-server jar if running the server mind. The native DLL and `mind.exe` must use the same table selection as the game. [Fabric 26.3 notes](https://www.fabricmc.net/2026/09/15/263.html)

```powershell
$env:PATHTRACING_MC_VERSION = '26.3'
cargo build --offline --manifest-path engine/Cargo.toml --release -p pathtracing_native -p mind
$env:PATHTRACING_NATIVE = Join-Path $env:CARGO_TARGET_DIR 'release\pathtracing_native.dll'
```

From the player client, let each run finish before starting the next.

```text
/msg PathBot track run doorcorridor
/msg PathBot track run doorparkourblue
/msg PathBot track run grassjumpeasy
/msg PathBot track run fencemaze
/msg PathBot track run trapdoorparkour
/msg PathBot track run stonestaircase
/msg PathBot track run stonesimple
/msg PathBot track run flowers
/msg PathBot track run amertystcoral
/msg PathBot track run mud
/msg PathBot track run ametwoodbox
```

From the rig root, extract the last record for each track. Check that its timestamp is from the 26.3 flight and that it reached B with `restored: true`.

```powershell
$tracks = 'doorcorridor','doorparkourblue','grassjumpeasy','fencemaze','trapdoorparkour',
          'stonestaircase','stonesimple','flowers','amertystcoral','mud','ametwoodbox'
foreach ($track in $tracks) {
    Get-Content "lab/tracks/runs/$track.jsonl" -Tail 1 |
        ConvertFrom-Json |
        Select-Object @{ Name = 'Track'; Expression = { $track } }, date, T, predictedT,
            @{ Name = 'TMinusPredicted'; Expression = {
                if ($null -ne $_.T -and $null -ne $_.predictedT) { $_.T - $_.predictedT }
            } }, restored, commandToFirstInputTicks
}
```

The known `ametwoodbox` gap is the 10-tick item-drop pickup. Check it in the same flight.
