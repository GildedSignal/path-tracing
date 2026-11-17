# Port delta: LavaFluid.entityInside collector order and hurtServer scaling

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`. Read `lab/results/local-20261117-1637-1-0-port-lava-entity-inside/dossier.md` (rate-limited before checker ran) and verify/narrow its unchecked paths rather than guessing.

**Consumer:** `engine/law/src/movement.rs:1054` (`Inside::Lava` refusal), batch `P3a`, closing `port-lava-entity-inside` (no `check.md`).

**From the 26.2 server bytecode**, state with class, method and offsets:
- `LavaFluid.entityInside` and `StepBasedCollector` (`apply`, `runAfter`, `advanceStep`, `flushStep`, `applyAndClear`): exact execution order (`clearFreeze -> lavaIgnite -> lavaHurt`), `igniteForSeconds(15.0f)` (`300` ticks, max-only), and behavior when **two lava cells** are visited in one step (`runAfter` appends twice vs dedup);
- `Entity.lavaHurt` and `Player.hurtServer` / `LivingEntity.hurtServer` (to locate) for `damageSources().lava()` (`4.0f` base): `fireImmune()` gate, `ServerLevel`-only gate, `MobEffects.FIRE_RESISTANCE` + `DamageTypeTags.IS_FIRE`, `scalesWithDifficulty` (`PEACEFUL=0`, `EASY=min(a/2+1, a)`, `NORMAL=4.0`, `HARD=a*3/2`), strict `(float)invulnerableTime > 10.0f` (`20`-tick window, `11` blocked, `10` hits), and `lava.json` exhaustion (`0.1f`).

**Build a jar-backed reproduction:** run `reproduce.sh` disassembling `LavaFluid.entityInside`, `StepBasedCollector`, `Entity.lavaHurt`, and `Player.hurtServer`/`LivingEntity.hurtServer`, and run direct jar calls covering: (1) non-`ServerLevel` vs `ServerLevel`, (2) `invulnerableTime ∈ {0, 11, 10}` and 2-cell same-step lava contact, (3) `PEACEFUL`/`EASY`/`NORMAL`/`HARD` difficulty scaling, and (4) `MobEffects.FIRE_RESISTANCE` blocking `4.0f` damage while still running `clearFreeze`/`lavaIgnite`. Write `dossier.md` starting with `Claim: ` on its first physical line.

**Falsifier:** any health, `ticksFrozen`, `remainingFireTicks`, `invulnerableTime`, or difficulty-scaled bit value that disagrees with the 26.2 bytecode.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
