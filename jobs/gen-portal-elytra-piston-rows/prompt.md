# Generator rows & kernel scars: portals, elytra glide/fireworks, and piston structure push

**Goal:** Bit- and behaviour-exact witness tables for the three non-affine 26.2 kernels (`Portals` / `PortalForcer`, `Elytra` + fireworks, and `Piston` structure resolve + entity push) mapped to `jobs/law-grouping-audit/table.md` groups 5, 14, and 15.

Locks down parameters, thresholds, and exact edge scars for portal wormholes, elytra flight, and piston contraptions before merging `wt/port-fold` kernels.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`.

**Harness requirements:**
- Bind real jar tags via `MappedRegistry.prepareTagReload(...).apply()` (`data/minecraft/tags/**/*.json`).
- Use real `LocalPlayer` (`isLocalPlayer() => true`, `isLocalClientAuthoritative() => true`), real `ServerLevel`/`LevelChunk`, and `LevelTicks` / block events.
- Write `dossier.md` (`Claim: ...`), `rows.tsv`, and `witnesses.tsv` early as soon as core bytecode + initial runs pass, then extend cleanly.

**From the bytecode** (`class.method@offset`):
1. **Elytra glide & boost (`LivingEntity.travelFallFlying`, `FireworkRocketEntity.tick`):** pitch/look vector trig, exact constants (`0.08`, `-0.08`, horizontal gain, speed floor/ceiling, drag `(0.99, 0.98, 0.99)`, kinetic crash damage `wallHit`), plus attached firework velocity boost `(1.5 / 0.5)` and duration ticks.
2. **Portals (`Entity.handlePortal`, `NetherPortalBlock`, `PortalForcer`, `EndPortalBlock`):** creative instant vs survival `80`-tick portal delay cooldown, `8x` Overworld/Nether coordinate scaling + world-border clamp, `PortalForcer` closest-POI search radius (`128` Overworld / `16` Nether) and fallback frame creation offsets, End spawn platform `(100, 48, 0)` obsidian + clearing box.
3. **Pistons (`PistonBaseBlock`, `PistonStructureResolver`, `PistonMovingBlockEntity`):** push cap `12`, non-pushable / destroy push reactions, slime/honey mutual non-stick scar, `0.5` progress increments across `2` ticks, `moveCollidedEntities` bounding-box sweep + slime bounce `vel`, and `finalTick` block placement order.

**Write in your run folder:**
- `rows.tsv`: `rule_id	kernel	constants_hex	limits_or_delays	scars	source`
- `witnesses.tsv`: `case_id	tick	entity_or_pos	coords_or_vel_hex	state_summary	events`
- `dossier.md` (first line `Claim: `), `reproduce.sh`, and `wiki.md`.

**Falsifier:** Any witness tick where glide velocity bits, firework boost, portal wait/destination coords, piston push resolution, or pushed-entity delta differs from the 26.2 jar.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
