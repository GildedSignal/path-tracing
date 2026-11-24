# Generator rows: tag-bound re-witness of terrain, hazard, and special movement blocks

**Goal:** Re-witness all tag-sensitive ground, stall, bounce, damage, and hazard blocks with real 26.2 jar tags bound (`MappedRegistry.prepareTagReload`), producing verified parameters for the `body`, `contact`, and `cell` generators.

`gen-fluid-bucket-rows` caught that running without `MappedRegistry.prepareTagReload(...).apply()` leaves `#fall_damage_resetting`, `#climbable`, `#ice`, `#dampens_vibrations`, etc. empty (`cobweb` changed `8.0 -> 0.0`). This lane supplies tag-exact witnesses across all special terrain blocks.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`.

**Harness requirements:**
- Mandatory first boot step: load all `data/minecraft/tags/{block,fluid,entity_type,item}/*.json` via `MappedRegistry.prepareTagReload(...).apply()`. Print loaded tag counts in `dossier.md`.
- Use real `LocalPlayer` (or override `isLocalPlayer() => true` and `isLocalClientAuthoritative() => true`) over a real `LevelChunk` for every movement/landing/inside-block witness.
- Write `dossier.md` (`Claim: ...`), `rows.tsv`, and `witnesses.tsv` early after core bytecode + primary witnesses pass, then extend cleanly.

**Blocks & rules to extract and witness:**
- Movement modifiers: `stone`, `slime_block`, `honey_block`, `soul_sand`, `ice`, `packed_ice`, `blue_ice`, `cobweb`, `sweet_berry_bush`, `powder_snow` (with and without `leather_boots`), `hay_block`, `white_bed`, `carpet`, `snow` layers `1..8`, `big_dripleaf`, `scaffolding`, `lily_pad`.
- Inside/step hazards: `magma_block`, `campfire` / `soul_campfire`, `fire` / `soul_fire`, `cactus`, `wither_rose`, `pointed_dripstone`.
- Record friction (`floatToBits`), `speedFactor`, `jumpFactor`, stuck multiplier `(x,y,z)`, bounce rule, `stepOn` / `fallOn` / `entityInside` firing conditions, and `leather_boots` / sneaking guards.

**Write in your run folder:**
- `rows.tsv`: `block_id	friction_hex	speed_factor_hex	jump_factor_hex	stuck_hex	fall_mult_hex	tags_used	scars	source`
- `witnesses.tsv`: `case_id	tick	x_hex	y_hex	z_hex	vx_hex	vy_hex	vz_hex	health_hex	flags	events`
- `dossier.md` (first line `Claim: `), `reproduce.sh`, and `wiki.md`.

**Falsifier:** Any witness tick with empty bound tags, or any difference in position, velocity, fall damage, or status flags between the table row and the tag-bound jar run.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
