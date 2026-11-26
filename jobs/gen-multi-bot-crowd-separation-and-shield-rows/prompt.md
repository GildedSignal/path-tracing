# GLM 5.4 live verification: 26.2 multi-entity crowd push, player attack knockback, and 4D sweep timing

**Goal:** Execute live 26.2 `LivingEntity.pushEntities`, `Entity.push(Entity)`, `Player.attack`, `LivingEntity.knockback`, and shield block arc/disable rules tick-by-tick on `server-26.2.jar`.

Supplies exact multi-body push and melee timing ground truth for Stage 4 (`engine/mind` + `server-mod`) multi-bot coordination and PvP melee without hardcoded approximations. See `lab/results/local-20261124-130147-11356-1-0-vibe-check-block-place-break/` for clean boot.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`; java/javac: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/`.

**Live scenes to execute:**
1. **Dense crowd push (`pushEntities`):** `4`, `16`, and `25` overlapping `ServerPlayer` / `Mob` entities in a `1x1` and `3x3` cell: record raw `u64` IEEE velocity deltas `(dvx, dvy, dvz)`, push ordering, and `maxEntityCramming` threshold.
2. **Sprint knockback & shield timing:** `Player.attack` at `attackStrengthScale = 0.2, 0.9, 1.0`, sprint knockback impulse (`0.4`/`0.5`), hurtCooldown `10`-tick invulnerability frames, and axe vs shield 100-tick disable.

**Write in your run folder:**
- `rows.tsv`, `witnesses.tsv`, `dossier.md` (`Claim: ...`), `reproduce.sh`, `wiki.md`.
- Write only in your run folder; no game launch, network, or `work.patch`.
