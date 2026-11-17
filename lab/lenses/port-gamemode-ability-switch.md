# Port delta: game-mode switch and abilities reset

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`, the client jar at `D:/path-tracing-deps/minecraft-26.2-client.jar`, and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`. `port-creative-flight-toggle` already settled `LocalPlayer.aiStep` double-tap toggle; do not re-ask that.

**Consumer:** `parkour_race` (`recordings/track-parkour_race-Brain_Forge-1.jsonl` Creative/Survival game-mode switches, `jobs/flight-corpus/report.md` §4.1.4), `docs/plan.md` step 3, and track A/B mode transitions (`docs/PRINCIPLES.md` S34).

**From the 26.2 server and client bytecode**, state with class, method and offsets:
- `GameType.updatePlayerAbilities` (to locate) for all four modes (`SURVIVAL`, `CREATIVE`, `ADVENTURE`, `SPECTATOR`): the exact writes to `mayfly`, `flying`, `invulnerable`, `instabuild`, `mayBuild`, `flyingSpeed`, and `walkingSpeed`;
- `ServerPlayer.setGameMode` (to locate) and `MultiPlayerGameMode.setLocalMode` (to locate): exact order of `updatePlayerAbilities`, `onUpdateAbilities()`, whether `setGameMode` forces `flying = false` or clears `sprinting` / `usingItem` / `fallDistance` / `teleport` on Spectator->Survival, and what packet syncs it to the client.

**Build a jar-backed reproduction:** run `reproduce.sh` disassembling `GameType.updatePlayerAbilities`, `ServerPlayer.setGameMode`, and `MultiPlayerGameMode.setLocalMode`, and invoke `GameType.updatePlayerAbilities` directly on a `Player`/`Abilities` instance across all transitions (e.g. `CREATIVE (flying=true) -> SURVIVAL`, `CREATIVE -> SPECTATOR`, `SPECTATOR -> SURVIVAL`), printing every `Abilities` field bit before and after. Write `dossier.md` starting with `Claim: ` on its first physical line.

**Falsifier:** any `Abilities` field value, `flying` preservation/clear rule, or `setGameMode` side effect that disagrees with the 26.2 bytecode.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
