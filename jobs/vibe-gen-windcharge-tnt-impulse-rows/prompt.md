# Vibe live verification: 26.2 WindCharge and PrimedTnt explosion impulse envelopes

**Goal:** Execute live 26.2 `WindCharge` (`Player` & breeze burst radius, knockback `Δv`, `10t` item cooldown, `resetFallDistance`) and `PrimedTnt` (`80t` fuse, blast exposure raycast, `Δv` impulse, shield/armor reduction) on `server-26.2.jar`.

Supplies exact impulse-producer envelopes for localized `LB_kit` bounds in the causal hypergraph (`map/detail/representation.md` §5–6). See `lab/results/local-20261124-130147-11356-1-0-vibe-check-block-place-break/` for clean boot.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`; java/javac: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/`.

**Harness requirements (mandatory executed witnesses):**
- Boot tags (`MappedRegistry.prepareTagReload(...).apply()`) + `DataComponentInitializers` + live `ServerLevel` + `LevelChunk` + real `WindCharge`, `PrimedTnt`, `ServerExplosion`, and `ServerPlayer`.
- Inspect `javap -c -p -constants` for `WindChargeItem.use`, `WindCharge.explode`, `AbstractWindCharge.tick`, `PrimedTnt.tick`, `ServerExplosion.explode`, `ServerExplosion.hurtEntities`, and `ServerExplosion.getSeenPercent`.
- Write `dossier.md` (`Claim: ...`), `rows.tsv`, and `witnesses.tsv` early, then extend cleanly.

**Live scenes to execute:**
1. **`WindCharge` burst & fall reset:** fire downward beneath standing vs falling (`fallDistance = 12.0`) `ServerPlayer`, measure burst origin, `(vx, vy, vz)` impulse bits, `10`-tick `ItemCooldowns`, and `fallDistance` reset vs height check (`y <= burstY + ...`).
2. **`PrimedTnt` 80-tick fuse & blast `Δv`:** ignite TNT at distance `1.0`, `2.0`, `4.0`, `7.9` (exposed vs behind single stone slab / pillar cover), record fuse ticks, `getSeenPercent` exposure, `knockback` vector, and survival damage before/after shield/armor.
3. **Chained wind-charge + piston/jump:** throw velocity (`1.5f` speed, `0` drag? verify exact drag/gravity!) and impact trigger tick.

**Write in your run folder:**
- `rows.tsv`: `rule_id	method	radius_hex	impulse_or_scale_hex	cooldown_or_fuse	scars	source`
- `witnesses.tsv`: `case_id	tick	dist_hex	exposure_hex	vx_hex	vy_hex	vz_hex	damage_hex	events`
- `dossier.md` (first line `Claim: `), `reproduce.sh`, and `wiki.md`.

**Falsifier:** Any executed explosion exposure fraction, impulse vector bit pattern, fuse/cooldown tick, or fall-distance reset that disagrees with `server-26.2.jar`.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
