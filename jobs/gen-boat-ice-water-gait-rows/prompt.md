# GLM 5.4 live verification: 26.2 boat mode on water, land, packed ice, and blue ice

**Goal:** Execute live 26.2 `Boat.tick`, `Boat.floatBoat`, `Boat.controlBoat`, and passenger mount/dismount (`ServerPlayer.startRiding` / `stopRiding`) tick-by-tick IEEE bits on `server-26.2.jar` across water, stone land, packed ice, and blue ice.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`; java/javac: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/`.

**Live scenes to execute:**
1. **Steady straight boat gaits:** oak boat driven by `ServerPlayer` holding `forward` across still water, flat stone, packed ice, and blue ice for 80 ticks each; record raw `u64` IEEE bits `(x, y, z, vx, vy, vz)`, `yRot`, `deltaRotation`, `invFriction`, and `status` (`IN_WATER`, `ON_LAND`, `IN_AIR`).
2. **Turning & drag response:** steer left (`left=true`, `right=false`) and combined (`forward+left`) on water and blue ice; pin angular velocity decay (`0.9f`), forward impulse (`0.04f`), and turning impulse (`0.005f`).
3. **Mount / dismount boundary:** mount an idle oak boat from shore and dismount onto adjacent solid stone vs water edge (`DismountHelper` / `findStandUpPosition` safe-cell scan order and post-dismount player state).

**Write in your run folder:**
- `rows.tsv`, `witnesses.tsv`, `dossier.md` (`Claim: ...`), `reproduce.sh`, `wiki.md`.
- Write only in your run folder; no game launch, network, or `work.patch`.
