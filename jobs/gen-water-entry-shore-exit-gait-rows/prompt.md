# GLM 5.4 live verification: 26.2 water entry, steady sprint-swim gait, and shore exit

**Goal:** Execute live 26.2 `Entity.updateFluidHeightAndDoFluidPushing`, `Player.updateSwimming`, and `LivingEntity.travelInFluid` tick-by-tick IEEE bits on `server-26.2.jar` across water entry, seabed startup, steady sprint-swim stroke, and shore step-up exit.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`; java/javac: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/`.

**Live scenes to execute:**
1. **Surface entry to steady sprint-swim:** sprint-jump off stone edge `[0, 0, 0]` into a 3-deep × 16-long water basin (`source` water at `y = 0, -1, -2`), holding forward + sprint (and jump when bobbing) until the periodic sprint-swim gait repeats; record raw `u64` IEEE bits for `(x, y, z, vx, vy, vz)` plus `pose`, `isSwimming`, `isInWater`, `isUnderWater`, `eyeInWater`, `wasEyeInWater`, and `waterHeight` per tick.
2. **Seabed-to-swim startup:** start standing on the water floor (`y = -2`), hold forward + sprint (+ jump pulses) until `pose = SWIMMING` and steady stroke resumes; pin the exact tick water depth crosses `0.4` (`fluidJumpThreshold`) and sprint-swim activates.
3. **Shore exit (`dy = 0` flush edge & `dy = +1` bank):** swim out onto stone bank at water level and one block above water level; record the exact exit ticks (`horizontalCollision`, `autoStep` / jump impulse, `isSwimming` clearing, landing on stone).

**Write in your run folder:**
- `rows.tsv`, `witnesses.tsv`, `dossier.md` (`Claim: ...`), `reproduce.sh`, `wiki.md`.
- Write only in your run folder; no game launch, network, or `work.patch`.
