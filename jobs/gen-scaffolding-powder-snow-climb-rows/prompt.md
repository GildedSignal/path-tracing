# GLM 5.4 live verification: 26.2 Scaffolding, Powder Snow, and Leather Boots climbing physics

**Goal:** Execute live 26.2 `ScaffoldingBlock.getCollisionShape`/`getInteractionShape`, `PowderSnowBlock.getCollisionShape`/`entityInside`/`canEntityWalkOnPowderSnow`, and `LivingEntity.handleRelativeFrictionAndCalculateMovement` (`onClimbable`, sneak descend `-0.15`, jump `+0.42`, `-0.08` settle) tick-by-tick IEEE bits on `server-26.2.jar`.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`; java/javac: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/`.

**Live scenes to execute:**
1. `scaffolding` climb up (`jump=true`), hover (`shiftKeyDown=false`), descend (`shiftKeyDown=true`, `bottom=true` floor vs floating top support), recording raw `u64` IEEE bits for `(x, y, z, vx, vy, vz)` per tick.
2. `powder_snow` bare feet (`freeze` counter, fall reset, slowness) vs `leather_boots` top walk (`canEntityWalkOnPowderSnow`), climbing up/down inside `powder_snow`, and bucket pickup/place state IDs.

**Write in your run folder:**
- `rows.tsv`, `witnesses.tsv`, `dossier.md` (`Claim: ...`), `reproduce.sh`, `wiki.md`.
- Write only in your run folder; no game launch, network, or `work.patch`.
