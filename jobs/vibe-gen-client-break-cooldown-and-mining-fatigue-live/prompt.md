# Vibe live verification: 26.2 client 5-tick break cooldown, insta-mine, and mining fatigue

**Goal:** Execute live 26.2 `BlockBehaviour.getDestroyProgress`, `Player.getDestroySpeed`, and `MultiPlayerGameMode` (`destroyDelay = 5`, `destroyProgress >= 1.0` insta-mine, `0.2x` airborne, `0.2x` underwater without Aqua Affinity, Haste & Mining Fatigue multipliers) on `server-26.2.jar` / `client-26.2.jar` if present (fallback `server-26.2.jar` + bytecode inspection).

Supplies bit-exact breaking tick counts and cooldown boundaries for the 26.2 law & solver kit model. See `lab/results/local-20261124-130147-11356-1-0-vibe-check-block-place-break/` for clean `DataComponentInitializers` / tag boot.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` (check if client jar exists under `D:/path-tracing-lab/` too); javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`; java/javac: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/`.

**Harness requirements (mandatory executed witnesses):**
- Boot tags (`MappedRegistry.prepareTagReload(...).apply()`) + `DataComponentInitializers` + Unsafe `ServerLevel` + real `LevelChunk` + real `ServerPlayer` + `ServerPlayerGameMode` (and `MultiPlayerGameMode` bytecode if client classes available).
- Inspect bytecode with `javap -c -p -constants` for `BlockBehaviour.getDestroyProgress`, `Player.getDestroySpeed`, `ServerPlayerGameMode.handleBlockBreakAction` / `tick`, and `MobEffectUtil.getDigSpeedAmplification`.
- Write `dossier.md` (`Claim: ...`), `rows.tsv`, and `witnesses.tsv` early, then extend cleanly.

**Live scenes to execute:**
1. **Tool/block matrix:** `getDestroyProgress` per tick (`f32` hex bits) for `stone`, `oak_planks`, `cherry_fence`, `cobweb`, `obsidian` with `air` (hand), `iron_pickaxe`, `diamond_axe` (`Efficiency V`).
2. **Airborne & underwater penalties:** live `getDestroyProgress` with `onGround=false`, `isEyeInFluid(WATER)=true`, and Aqua Affinity helmet.
3. **Cooldown & insta-mine boundary:** verify exact break tick (`sum >= 1.0f`) and `destroyDelay = 5` non-insta-mine vs `0`-tick cooldown on insta-mine (`>= 1.0f` first tick), plus Haste II vs Mining Fatigue III multipliers (`0.3^level`).

**Write in your run folder:**
- `rows.tsv`: `case_id	block_id	tool_id	progress_hex	break_ticks	cooldown_ticks	scars	source`
- `witnesses.tsv`: `case_id	tick	progress_hex	accum_hex	broken	flags`
- `dossier.md` (first line `Claim: `), `reproduce.sh`, and `wiki.md`.

**Falsifier:** Any executed `getDestroyProgress` float bit pattern, break tick count, or cooldown rule that disagrees with `server-26.2.jar`.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
