# Vibe second-family verify & close open entity-contact limits

**Goal:** Independently verify `gen-entity-contact-rows` against `server-26.2.jar` and close its open limits (same-vehicle push exclusion, `CollisionGetter.findFreePosition` dismount on a chunk with full empty air space, and `fallOn` hook order).

Second-family (`vibe`) independent verification of `C:/Users/PASCAL/Desktop/path tracing/lab/results/local-20261124-111947-18412-1-0-gen-entity-contact-rows/` plus resolution of its documented open cases. Do NOT trust any prior dossier claim without re-running it from `server-26.2.jar`.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`.

**Harness requirements:**
- Bind real jar tags via `MappedRegistry.prepareTagReload(...).apply()`.
- Use real `ServerLevel` + real `LevelChunk` (so `CollisionGetter.findFreePosition` has real chunk/world bounds rather than mocked shape gaps), real `PersistentEntitySectionManager`, `Raft`/`Minecart`, and `ServerPlayer`/`LocalPlayer`.
- Write `dossier.md` (`Claim: ...`), `rows.tsv`, and `witnesses.tsv` early after core witnesses pass, then extend cleanly.

**Verify & close these specific items:**
1. **Cross-check GLM's findings** on 26.2 bytecode: `LivingEntity.pushEntities` exact query box (`!=(0.2,0,0.2)`), `Entity.push` double arithmetic & `needsSync` (`hasImpulse` removed), cramming `6.0f` on `random.nextInt(4) == 0` when `list.size() > rule - 1`, `SoulSandBlock` absence of `stepOn`/`entityInside`, and `visited-before-intersect` ordering.
2. **Same-vehicle push exemption:** place two passengers in the same vehicle / vehicle-rider pair and witness `Entity.push` / `pushEntities` skipping.
3. **Open flat-ground safe dismount (`LivingEntity.dismountVehicle` -> `CollisionGetter.findFreePosition`):** test `Raft` and `Minecart` on a real chunk floor so `findFreePosition` actually succeeds vs falls back to vehicle dismount rules, recording exact dismount hex coordinates across open and walled layouts.
4. **`Entity.fallOn` dispatch order:** witness how landing `fallOn` interacts with `stepOn` and `entityInside` on hay, bed, slime, honey, powder snow, and dripstone.

**Write in your run folder:**
- `rows.tsv`: `rule_id	kind	verified_status	coords_or_vel_hex	scars	source`
- `witnesses.tsv`: `case_id	tick	entity_id	x_hex	y_hex	z_hex	vx_hex	vy_hex	vz_hex	events`
- `dossier.md` (first line `Claim: `), `reproduce.sh`, and `wiki.md`.

**Falsifier:** Any witness tick or dismount coordinate that differs from the 26.2 jar on real `LevelChunk` + `ServerLevel`.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
