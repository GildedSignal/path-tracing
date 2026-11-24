# Vibe second-family check: ThrownEnderpearl recurrence, raytrace, oldPosition teleport, and spread cone

**Goal:** Independently verify GLM's `gen-projectile-pearl-rows` (`C:/Users/PASCAL/Desktop/path tracing/lab/results/local-20261124-183342-27616-1-0-gen-projectile-pearl-rows/`) live against `server-26.2.jar` (`ThrowableProjectile.tick` gravity `-0.03` before `0.99f`/`0.8f` drag, `COLLIDER+NONE` clip, entity sweep margin `clamp((t-2)/20, 0, 0.3)`, `oldPosition()` teleport, `resetFallDistance` before `5.0f` hurt, `20`-tick cooldown, RNG spread cone bounds, and closed-form error bound).

Second-family independent verification (`clan.md` §1) of GLM's `gen-projectile-pearl-rows` so the pearl wormhole model + `pearl.rs` laws carry zero single-family blind spots. Read GLM's output files first: `C:/Users/PASCAL/Desktop/path tracing/lab/results/local-20261124-183342-27616-1-0-gen-projectile-pearl-rows/{dossier.md,rows.tsv,witnesses.tsv,wiki.md}` (if present).

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`.

**Harness requirements (mandatory executed witnesses):**
- Boot jar tags (`MappedRegistry.prepareTagReload`) + `DataComponentInitializers` + Unsafe `ServerLevel` + real `LevelChunk` + real `EnderpearlItem`, `ThrownEnderpearl`, and `ServerPlayer`.
- Execute live against `server-26.2.jar` — never copy GLM's numbers without running your own jar harness; explicitly verify or falsify every constant, closed-form claim, and edge case in GLM's dossier.
- Write `dossier.md` (`Claim: ...`), `rows.tsv`, and `witnesses.tsv` early after core tick/hit/teleport witnesses pass, then extend cleanly.

**Live scenes & cross-checks to execute:**
1. **`EnderpearlItem.use` & launch:** exact spawn `y = eyeY - 0.1`, thrower momentum subtraction/addition order (moving vs stationary, vertical clamp on pitch), speed `1.5`, inaccuracy `1.0`, `RandomSource.triangle`/Gaussian spread draw order & cone envelope, stack decrement, and `ItemCooldowns` `20`-tick gate.
2. **Air & water tick recurrence (`ThrowableProjectile.tick`):** verify `applyGravity` (`-0.03`) runs BEFORE `0.99f` (air) or `0.8f` (water) drag across 60 ticks, checking exact IEEE `u64` bits and closed-form recurrence error vs GLM's `2.1e-5` bound.
3. **Raytrace & sweep margin:** `ClipContext.Block.COLLIDER + Fluid.NONE` (passes through tall grass, water, cobweb? check!), entity sweep margin `clamp((tickCount - 2) / 20.0f, 0.0f, 0.3f)` on ticks `0..15`, and bubble-column interaction if any.
4. **`ThrownEnderpearl.onHit` impact order:** confirm teleport lands at **`this.oldPosition()`** (start of impact tick, NOT ray hit point!), **strict** ordering `resetFallDistance()` BEFORE `resetCurrentImpulseContext()` BEFORE `hurtServer(enderPearl, 5.0f)` (test low health `health <= 5.0f`, armor/feather-falling mitigation vs resistance/absorption!).

**Write in your run folder:**
- `rows.tsv`: `check_id	glm_claim	vibe_jar_result	max_ulp_or_err	status	notes	source`
- `witnesses.tsv`: `case_id	tick	x_hex	y_hex	z_hex	vx_hex	vy_hex	vz_hex	events`
- `dossier.md` (first line `Claim: `), `reproduce.sh`, and `wiki.md`.

**Falsifier:** Any executed tick where raw IEEE bits, hit clip behavior, `oldPosition()` landing coords, fall-damage reset ordering, cooldown, or spread cone disagrees with `server-26.2.jar` (or confirms/refutes a discrepancy in GLM's dossier).

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
