# Vibe generator rows: game mode gates, spectator/creative flight, death, and respawn

**Goal:** Bit- and server-exact parameter/scar table for 26.2 `GameType` / `Abilities` (`SURVIVAL`, `CREATIVE`, `ADVENTURE`, `SPECTATOR` flags, flight speeds `0.05f`, noPhysics spectator rules) and `LivingEntity.die` / `ServerPlayer.respawn` mechanics (`jobs/law-grouping-audit/table.md` group 23) executed against `server-26.2.jar`.

Covers Group 23 (`Game mode, abilities, death and respawn`) with executed `ServerPlayer` witnesses.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`.

**Harness requirements (mandatory executed witnesses):**
- Boot jar tags + `DataComponentInitializers` + Unsafe `ServerLevel` + `LevelChunk` + real `ServerPlayer` (`ServerPlayerGameMode`, `Abilities`).
- Write `dossier.md` (`Claim: ...`), `rows.tsv`, and `witnesses.tsv` early after first runs pass, then extend cleanly.

**From bytecode + live execution:**
1. **`GameType.updatePlayerAbilities` & movement (`Abilities` `0.05f` fly / `0.1f` walk):** exact flags (`invulnerable`, `mayfly`, `instabuild`, `mayBuild`, `flying`) across all 4 `GameType`s; `SPECTATOR` `isSpectator()` -> `noPhysics=true`, skips `checkInsideBlocks` and block collision; creative flying `travelFlying` drag/accel/sprint `2x` boost (`0.1f`).
2. **Death & Respawn (`ServerPlayer.die`, `PlayerList.respawn`, `RespawnConfig` / `BedBlock` / `RespawnAnchorBlock`):** health `<= 0` death transition (`pose = DYING`), `keepInventory` gamerule vs clear, respawn position search (`BedBlock.findStandUpPosition` / `RespawnAnchorBlock.findStandUpPosition`), state resets (`health=20`, `food=20`, `sat=5.0f`, `exhaustion=0`, `fallDistance=0`, `fire=0`, `invulnerableTime=60`).

**Write in your run folder:**
- `rows.tsv`: `rule_id	mode_or_phase	flags_mask	constants_hex	scars	source`
- `witnesses.tsv`: `case_id	tick	mode	x_hex	y_hex	z_hex	health_hex	food	abilities	events`
- `dossier.md` (first line `Claim: `), `reproduce.sh`, and `wiki.md`.

**Falsifier:** Any executed ability flag, flying velocity bits, spectator collision rule, or respawn state value differing from `server-26.2.jar`.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
