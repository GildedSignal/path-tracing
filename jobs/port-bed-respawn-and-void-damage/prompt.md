# Port delta: Bed bounce/explosion, RespawnAnchor, void damage and death/respawn state reset

**Goal:** the jar's exact rule for this, with a reproduction that runs against Mojang's 26.2 jar: `dossier.md` (first line `Claim: `), `reproduce.sh`, and the Rust port delta as `work.patch`.

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`.

**Consumer:** `engine/law/src/movement.rs`, `engine/law/src/player.rs`, `map/principles.md` (death as respawn teleport with a toll, S189).

**From the 26.2 server bytecode**, state with class, method and offsets:
- `BedBlock`: `fallOn` (`super.fallOn(level, state, pos, entity, fallDistance * 0.5)` exact `f64` multiplier!), `updateEntityMovementAfterFallOn` (`bounceUp`: `-vy * 0.6600000262260437D` when `!isSuppressingBounce`), `useWithoutItem` (`BedRule` / dimension rule explosion power `5.0f` with fire at `pos.getCenter()`);
- `RespawnAnchorBlock`: charge levels `0..=4`, glowstone charge vs `canSetSpawn` / `explode` (`5.0f` explosion with fire);
- `Entity.checkBelowWorld` & `onBelowWorld` & `LivingEntity.onBelowWorld`: exact threshold `getY() < (double)(level.getMinY() - 64)` (`-128.0` in overworld), `4.0f` `fellOutOfWorld` damage (`BYPASSES_INVULNERABILITY` — hurts creative too! verify invuln/cooldown interaction), every-tick cadence;
- `ServerPlayer.die` & `PlayerList.respawn` / `ServerPlayer.restoreFrom`: exact reset of `health`, `foodData`, `exhaustion`, `saturation`, `ticksFrozen`, `remainingFireTicks`, `fallDistance`, `deltaMovement`, `effects`, `absorption`, and `keepInventory` / portal cooldown.

**Build a jar-backed reproduction:** run `reproduce.sh` disassembling `BedBlock`, `RespawnAnchorBlock`, `Entity`, `LivingEntity.onBelowWorld`, and `ServerPlayer.restoreFrom`, plus executing 26.2 jar witnesses for bed bounce `-0.66f` & `0.5` fall distance, anchor charge/explode, void `y = minY - 64` vs `nextDown` across Survival and Creative, and respawn reset fields. Write `dossier.md` starting with `Claim: ` on its first physical line and `work.patch` implementing these rules in `engine/law`.

**Falsifier:** any 1-ulp bounce/fall difference, wrong void `y` threshold (`minY - 64`), wrong creative void bypass, or respawn field reset mismatch against the 26.2 jar.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
