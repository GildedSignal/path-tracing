# Generator rows: 26.2 block-breaking cooldown, hardness, tool speed, and reach

**Goal:** Extract and execute live 26.2 `MultiPlayerGameMode` and `ServerPlayerGameMode` breaking mechanics (`destroyDelay = 5`, `startDestroyBlock`, `continueDestroyBlock`, `stopDestroyBlock`, tool speed, airborne/underwater `0.2x`, Haste/Mining Fatigue, instant creative/insta-break, and server tolerance) on `server-26.2.jar`.

Supplies bit-exact jar truth to close the missing client 5-tick break cooldown in `engine/law/src/mining.rs` (`map/now.md`) and verify live `KIT_PLAN` break-then-restore sequences (`ametwoodbox`, `doorcorridor`). See `lab/results/local-20261124-130147-11356-1-0-vibe-check-block-place-break/` for clean `DataComponentInitializers` / tag boot.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`; java/javac: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/`.

**Harness requirements:**
- Boot jar tags (`MappedRegistry.prepareTagReload(...).apply()`) + `DataComponentInitializers` + live `ServerLevel` / `LevelChunk` / `ServerPlayerGameMode` / `BlockBehaviour.getDestroyProgress` calls (and inspect client `MultiPlayerGameMode` bytecode or class/JAR references if present, citing exact classes found in `server-26.2.jar`).
- Inspect bytecode with `javap -c -p -constants` for `BlockBehaviour.getDestroyProgress`, `Player.getDestroySpeed`, `ServerPlayerGameMode.handleBlockBreakAction`, `ServerPlayerGameMode.tick`, and `ServerPlayerGameMode.destroyBlock`, then execute live cases on `server-26.2.jar`.
- Write `dossier.md` (`Claim: ...`), `rows.tsv`, and `witnesses.tsv` early after core bytecode + primary witnesses pass, then extend cleanly.

**Cases & rules to extract and witness:**
1. **Break progress & speed modifiers (`f32` exact IEEE bits):** empty hand vs iron/diamond/netherite axe & pickaxe on `oak_log`, `stone`, `obsidian`, `amethyst_block`, `mud`, `dirt`; Efficiency I–V (`lvl*lvl + 1`); Haste I–II, Mining Fatigue I–III, underwater without Aqua Affinity (`0.2f`), not on ground (`0.2f`).
2. **Insta-break vs timed break & `destroyDelay = 5` boundary:** sequential two-block break across `stopDestroyBlock` / completion (`5`-tick cooldown vs insta-break `destroyProgress >= 1.0f` no cooldown), plus `ABORT_DESTROY_BLOCK` progress reset.
3. **Server reach & packet tolerance:** exact eye-pos distanceSq ceiling (`canInteractWithBlock`), mismatched-progress `Time` tolerance / anti-desync accept vs reject threshold, and held-tool `hurtEnemy`/`mineBlock` durability/drop handoff.

**Write in your run folder:**
- `rows.tsv`: `rule_id	method	speed_or_coeff_hex	cooldown_ticks	reach_or_cap_hex	tags_used	scars	source`
- `witnesses.tsv`: `case_id	tick	block_id	tool_id	progress_hex	cooldown	destroyed	flags	events`
- `dossier.md` (first line `Claim: `), `reproduce.sh`, and `wiki.md`.

**Falsifier:** Any executed tick where break progress `floatToRawIntBits`, completion tick, cooldown tick, or reach boundary disagrees with `server-26.2.jar`.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
