# Vibe second-family verify: tag-bound ground classes & clocks cross-check

**Goal:** Independently verify GLM's 154 ground classes and 21 clock thresholds against `server-26.2.jar` with real jar tags bound (`MappedRegistry.prepareTagReload`) and real `LocalPlayer` + `ServerPlayer` + `LevelChunk`.

Second-family (`vibe`) independent cross-check of `C:/Users/PASCAL/Desktop/path tracing/lab/results/local-20261123-211113-3024-1-0-gen-ground-rows/` and `C:/Users/PASCAL/Desktop/path tracing/lab/results/local-20261123-222540-17108-1-0-gen-clock-rows/`. Because `gen-fluid-bucket-rows` discovered `gen-ground-rows` ran with empty tags (`cobweb` differed `8.0 -> 0.0`), this check explicitly verifies every tag-sensitive ground row with jar tags bound and re-verifies all clock cadences.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`.

**Harness requirements:**
- Mandatory first boot step: bind all jar tags via `MappedRegistry.prepareTagReload(...).apply()` (`#climbable`, `#fall_damage_resetting`, `#ice`, `#dampens_vibrations`, `#flowers`, etc.).
- Use real `LocalPlayer` (or `isLocalPlayer() => true` + `isLocalClientAuthoritative() => true`) and real `ServerLevel`/`LevelChunk`.
- Write `dossier.md` (`Claim: ...`), `rows.tsv`, and `witnesses.tsv` early after core witnesses pass, then extend cleanly.

**Verify specifically:**
1. **Tag-bound ground classes:** verify which blocks of the 154 equivalence classes change behaviour or fall reset when jar tags are bound (`cobweb`, `sweet_berry_bush`, `ladder`, `scaffolding`, `vine`, `ice`, `flowers`, `snow`, `carpet`, `bed`, `slime`, `honey`, `soul_sand`, `magma`), recording exact `Block` parameters and multi-tick trajectory hex bits.
2. **Clock generator thresholds:** independently witness all 21 clock counters across `server-26.2.jar`: `foodTickTimer` (`10` fast / `80` slow), air supply (`300` max, drown cadence `20` ticks `2.0f`, bubble refill), `invulnerableTime` (`20`), nether portal (`80` ticks survival / `1` creative), ender pearl cooldown (`20` ticks by item group), shield cooldown (`100` ticks), jump delay (`10` ticks), remaining fire ticks (`+20` grace, fire ticks), freeze (`140` max, `+1/-2`, freeze hurt every `40` ticks `1.0f`).

**Write in your run folder:**
- `rows.tsv`: `rule_id	kind	verified_status	threshold_or_tag_diff	constants_hex	scars	source`
- `witnesses.tsv`: `case_id	tick	x_hex_or_counter	vx_hex_or_value	health_hex	flags	events`
- `dossier.md` (first line `Claim: `), `reproduce.sh`, and `wiki.md`.

**Falsifier:** Any clock threshold or tag-bound ground trajectory that disagrees between our reproduced jar run and `rows.tsv`.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
