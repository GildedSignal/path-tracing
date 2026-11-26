# Vibe live verification: 26.2 block-breaking cooldown, tool speed, and oak-log break-drop-pickup

**Goal:** Independently execute live 26.2 block breaking (`destroyDelay = 5`, `startDestroyBlock`/`continueDestroyBlock`/`abort`, axe/hand speeds, underwater/airborne `0.2x`, and `ametwoodbox` oak-log break -> `ItemEntity` spawn -> 10t pickup) on `server-26.2.jar`.

Cross-checks GLM's `gen-break-cooldown-and-reach-rows` from a second model family and exercises the live `ametwoodbox` failure (`two logs broken, one carried: restored stays NeedsJoints` in `map/now.md`).

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`; java/javac: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/`.

**Harness requirements (mandatory executed witnesses):**
- Boot jar tags (`MappedRegistry.prepareTagReload(...).apply()`) + `DataComponentInitializers` + live `ServerLevel` + `LevelChunk` + `ServerPlayer` + `ServerPlayerGameMode`.
- Inspect `javap -c -p -constants` for `BlockBehaviour.getDestroyProgress`, `ServerPlayerGameMode.handleBlockBreakAction`, `ServerPlayerGameMode.tick`, and `ItemEntity.playerTouch`, then run live cases.
- Write `dossier.md` (`Claim: ...`), `rows.tsv`, and `witnesses.tsv` early, then extend cleanly.

**Live scenes to execute:**
1. **Sequential break cooldown:** break `oak_log` with bare hand vs stone/diamond axe (`CORRECT_FOR_MINEABLE` / axe tag speed) -> break second adjacent `oak_log` immediately vs after 5 ticks; verify insta-break (`slime_block` / `amethyst_cluster` / Haste insta-break) skips cooldown.
2. **Mid-air & underwater `0.2x` penalty:** verify `oak_log` & `stone` break tick count while standing vs airborne vs submerged without Aqua Affinity.
3. **Full `ametwoodbox` break-to-pickup timeline:** seed deterministic RNG, break two `oak_log` blocks with `ServerPlayerGameMode.destroyBlock`, tick `ItemEntity` + player position across ticks `0..15`, verify exact `pickupDelay = 10 -> 0` inventory count change (`0 -> 1 -> 2` logs).

**Write in your run folder:**
- `rows.tsv`: `case_id	method	speed_hex	expected_ticks	cooldown_ticks	drop_item	scars	source`
- `witnesses.tsv`: `case_id	tick	progress_hex	broken	item_pos_y_hex	pickup_delay	inv_count	events`
- `dossier.md` (first line `Claim: `), `reproduce.sh`, and `wiki.md`.

**Falsifier:** Any executed break tick count, `destroyProgress` float bit pattern, cooldown gate, or item pickup tick that disagrees with `server-26.2.jar`.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
