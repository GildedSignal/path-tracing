# Generator rows: 26.2 player-to-player Entity.push crowd repulsion, clamp, cramming, and boat/climbable guards

**Goal:** Extract and execute live 26.2 `Entity.push(Entity)` player-to-player repulsion (`max(|dx|,|dz|)` denominator, `0.05` clamp, `1/d` vs `1/d^2`), `LivingEntity.pushEntities` cramming rule, and climbable/passenger guards on `server-26.2.jar`.

Supplies dense-regime physics ground truth (`d_loc > 3` / Knudsen near `1`) for `engine/mind` swarm `Battle` / dense crowd simulation (`map/detail/representation.md` §7, `es-1b-probe`). See `lab/results/local-20261124-130147-11356-1-0-vibe-check-block-place-break/` for clean boot.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`; java/javac: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/`.

**Harness requirements:**
- Mandatory first boot step: `MappedRegistry.prepareTagReload(...).apply()` + `DataComponentInitializers` + live `ServerLevel` + `LevelChunk` + multiple `ServerPlayer` instances.
- Inspect `javap -c -p -constants` for `Entity.push`, `LivingEntity.push`, `LivingEntity.pushEntities`, `LivingEntity.doPush`, and `Entity.isPushable`.
- Write `dossier.md` (`Claim: ...`), `rows.tsv`, and `witnesses.tsv` early, then extend cleanly.

**Cases & rules to extract and witness:**
1. **Symmetric & asymmetric `Entity.push`:** two `ServerPlayer`s at offsets `dx = 0.0, 0.005, 0.05, 0.3, 0.59, 0.61`, `dz = 0.0, 0.1`, recording exact `(dvx, dvz)` IEEE bits added to each player (`< 0.01` zero guard, `> 1.0 -> sqrt`, `0.05` scale).
2. **Multi-player crowd & guards:** 8/25 overlapping players (`MAX_ENTITY_CRAMMING = 24`), player on ladder/vine (`isPushable` climbable disable), and passenger vs vehicle push suppression.

**Write in your run folder:**
- `rows.tsv`: `rule_id	method	threshold_hex	scale_hex	cramming_limit	scars	source`
- `witnesses.tsv`: `case_id	dx_hex	dz_hex	dvx_a_hex	dvz_a_hex	dvx_b_hex	dvz_b_hex	cram_dmg	events`
- `dossier.md` (first line `Claim: `), `reproduce.sh`, and `wiki.md`.

**Falsifier:** Any executed `Entity.push` velocity delta IEEE bit, `0.01`/`1.0`/`0.05` boundary, or climbable/cramming verdict that disagrees with `server-26.2.jar`.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
