# Generator rows: 26.2 WindCharge and PrimedTnt explosion exposure and impulse bounds

**Goal:** Extract and execute live 26.2 `WindCharge` (`Player` burst `Δv`, `10t` cooldown, fall-reset rule) and `PrimedTnt` (`80t` fuse, blast `getSeenPercent` rays, `Δv` impulse, damage) on `server-26.2.jar`.

Cross-checks Vibe's `vibe-gen-windcharge-tnt-impulse-rows` to anchor `LB_kit` impulse envelopes for causal hypergraph planning (`map/detail/representation.md` §5–6). See `lab/results/local-20261124-130147-11356-1-0-vibe-check-block-place-break/` for clean boot.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`; java/javac: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/`.

**Harness requirements:**
- Mandatory first boot step: `MappedRegistry.prepareTagReload(...).apply()` + `DataComponentInitializers` + live `ServerLevel` + `LevelChunk` + real `WindCharge`, `PrimedTnt`, `ServerExplosion`, and `ServerPlayer`.
- Inspect `javap -c -p -constants` for `WindChargeItem.use`, `WindCharge.explode`, `PrimedTnt.tick`, `ServerExplosion.hurtEntities`, and `ServerExplosion.getSeenPercent`.
- Write `dossier.md` (`Claim: ...`), `rows.tsv`, and `witnesses.tsv` early, then extend cleanly.

**Cases & rules to extract and witness:**
1. **`WindCharge` trajectory, `10t` cooldown, burst `Δv`, and fall reset:** launch speed/gravity, knockback magnitude & `y` boost, exact `lowestFallDistanceTracker` / fall reset condition.
2. **`PrimedTnt` 80t fuse & explosion ray exposure:** open ground vs single slab / wall obstruction at distances `1.0..8.0`, exact `(vx, vy, vz)` impulse bits & armor/shield damage.

**Write in your run folder:**
- `rows.tsv`: `rule_id	method	radius_hex	impulse_or_scale_hex	cooldown_or_fuse	scars	source`
- `witnesses.tsv`: `case_id	tick	dist_hex	exposure_hex	vx_hex	vy_hex	vz_hex	damage_hex	events`
- `dossier.md` (first line `Claim: `), `reproduce.sh`, and `wiki.md`.

**Falsifier:** Any executed explosion exposure fraction, impulse vector bit pattern, fuse/cooldown tick, or fall-distance reset that disagrees with `server-26.2.jar`.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
