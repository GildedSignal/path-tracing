# Minecraft 26.3 port report

The region reader accepts a 26.3 level and decodes its compact palettes. The law, generated tables and mods still target 26.2. The source comparison and 26.3 build gates need the official Mojang jars and matching Fabric dependencies. The build shell returned `curl: (7) Failed to connect` for Mojang's manifest, Mojang's jar host and Fabric Maven. The browser connection reported its host unavailable. [Mojang version manifest](https://piston-meta.mojang.com/mc/game/version_manifest_v2.json) · [Fabric 26.3 notes](https://www.fabricmc.net/2026/09/15/263.html)

Stage the vanilla archives as `D:\path-tracing-deps\minecraft-26.3-client.jar` and `D:\path-tracing-deps\minecraft-26.3-server.jar` and verify their digests against the version manifest before using them. Use a 26.3 build of Fabric API. [Mojang client jar](https://piston-data.mojang.com/v1/objects/e877b6a07acd633fb3bb475002175cec036e7b87/client.jar) · [Mojang server jar](https://piston-data.mojang.com/v1/objects/33680f5f2ac32864d6d7cf5e56a705fdb3e05f4c/server.jar) · [Fabric API 0.167.0+26.3](https://maven.fabricmc.net/net/fabricmc/fabric-api/fabric-api/0.167.0%2B26.3/)

## Changes and lines

The region reader admits each supported release with its DataVersion. It decodes the 26.3 palette into the existing block state type and keeps the strict per-chunk version check. The old 26.2 decoder remains necessary for the version the law and existing fixtures use.

- Bought: **+22** net in `region.rs` for the observed 26.3 NBT encodings and level version check.
- Bought: **+4** net in `nbt.rs` so the existing list validator accepts either of the two element types Minecraft writes to a 26.3 palette.
- `engine/` text diff is **+26 net**. The two region fixture edits already present at checkout are outside that count.
- Staging the scoped files failed with `fatal: Unable to create 'C:/Users/PASCAL/Desktop/path tracing/.git/worktrees/law-26.3/index.lock': Permission denied`. The changes are in the worktree on `wt/law-26.3`.

## Read-only Spiral gate

The baseline `WorldSnapshot::load` call at `[0, 53, -97]` prints `unsupported: level.dat version 26.3; expected 26.2`. The same call with the reader change passes that check and prints the existing datapack refusal.

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

The Overworld palette audit reads 9,772 chunks across 127 region files (111 empty), all with zlib compression. 5,119 chunks are DataVersion 5023 (5,040 full) and 4,653 are 4903 (860 full). A corridor touching an older chunk still needs Mojang's upgrade on a separate copy. The 1,321 distinct palette entries include five that the 26.2 table refuses as `non-local shape`. Each occurs once in the palette of chunk X/Z `[1, -1]`, section Y `8`.

| Unmodeled against the 26.2 table |
|---|
| `minecraft:lime_shulker_box` |
| `minecraft:magenta_shulker_box` |
| `minecraft:orange_shulker_box` |
| `minecraft:red_shulker_box` |
| `minecraft:yellow_shulker_box` |

The 26.3 census depends on the generated 26.3 tables. The source of the first refusal is now the datapacks, so there is no successful vanilla snapshot or snapshot-derived census for this map.

## Cargo, witnesses and mod builds

- Before changes, the requested serial Cargo suite reports 268 passed, 1 failed and 11 ignored. The failing case is `mind::tests::fence_delta_is_crossed_around`, which prints `world mirror cell [-16, -65, -16]: unloaded cell [-16, -65, -16]`. Its fixture starts at Y = −64. See `D:\path-tracing-evidence\law-26.3-baseline-cargo.txt` for the raw run.
- After the reader change, `cargo test --offline --manifest-path engine/Cargo.toml --release -p world -- --test-threads=1` passes all 21 tests. The raw run is `D:\path-tracing-evidence\law-26.3\world-tests.txt`.
- The full serial suite with the new reader reports 268 passed, 1 failed and 11 ignored, with the same fence failure. The raw run is `D:\path-tracing-evidence\law-26.3\reader-cargo.txt`.
- The 26.3 witnesses, table regeneration and Fabric builds await the official jars. There are no regenerated values to compare.
- The Spiral NBT audit, direct probes, and snapshot output are in `D:\path-tracing-evidence\law-26.3\`.

## Course bench

These are the 26.2 law results with the current section spine. The 26.3 comparison is pending in the [benchmark table](../../map/detail/benchmarks.md).

```text
exact: 50 players x 2000 ticks, n=7, ns per player-tick min=1418.3 median=1525.5 max=1580.8; refused ticks=0
floored: 50 players x 2000 ticks, n=7, ns per player-tick min=3163.8 median=3326.7 max=3452.4; refused ticks=0
straight: 1 players x 2000 ticks, n=7, ns per player-tick min=2376.3 median=2634.8 max=3201.4; refused ticks=0
```

## Julien's 26.3 instance and flight

After the 26.3 builds pass, make a separate 26.3 instance on Java 25 for each client role and use Fabric Loader 0.19.5 and a Fabric API marked for 26.3. Give both clients the rebuilt `bot/build/pathtracing.jar`, set their existing `pathtracing.role` and `pathtracing.rig` JVM properties, and point `PATHTRACING_NATIVE` to the matching release DLL. Use a 26.3 server on a copy of the rig world. Install `pathtracing-server.jar` in that server's mods if using the server mind. Start the server and both clients with the existing role names. Fabric lists Loader 0.19.5 for 26.3. [Fabric 26.3 notes](https://www.fabricmc.net/2026/09/15/263.html)

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
