# Port delta: creative flight double-jump toggle and vertical speed

Use the staged Mojang 26.2 client jar in `D:/path-tracing-deps/data`, the server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`, and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`. `port-movement-modes` already settled `handlePlayerAbilities` and horizontal `Player.getFlyingSpeed` (`1.4g`); do not re-ask those.

**Consumer:** Julien's track `parkour_race` (`16821` ticks with Creative/Survival mode & flying switches, `jobs/flight-corpus/report.md` §4.1.4) and batch `P4e` (`jobs/port-backlog/report.md`).

**From the 26.2 client and server bytecode**, state with class, method and offsets:
- `LocalPlayer.aiStep` (to locate): the double-tap jump timer (`jumpTriggerTime` or 26.2 name, to locate) that toggles `abilities.flying` when `abilities.mayfly` is true — exact window in ticks, reset conditions, and whether `abilities.flying` clears on landing (`onGround`) in Creative;
- `Player.travel` / `LivingEntity.aiStep` (to locate): the exact vertical velocity updates (`jump` ascend, `shift` descend, vertical drag `0.6f` or exact 26.2 constant, `getFlyingSpeed` multiplier) while `abilities.flying` is true, and exact order relative to `move()`.

**Build a jar-backed reproduction:** run `reproduce.sh` over the 26.2 bytecode and a direct jar-backed trace printing: (1) double-tap jump gap values `0..8` ticks showing which gaps toggle `flying`, and (2) 5 ticks of vertical ascend (`jump`), descend (`sneak`), and neutral hover (`flying = true`) with raw velocity bits. Write `dossier.md` starting with `Claim: ` on its first physical line.

**Falsifier:** any toggle gap, landing clear, or vertical flying velocity bit that differs from the 26.2 bytecode.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
