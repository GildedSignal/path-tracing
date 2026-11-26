# Vibe live verification: 26.2 Elytra fall-flying movement and FireworkRocket boost

**Goal:** Execute live 26.2 `LivingEntity.updateFallFlyingMovement` and `FireworkRocketEntity.tick` boost impulse/duration tick-by-tick on `server-26.2.jar`.

Supplies bit-exact fall-flying drag, lift, pitch-to-horizontal conversion, firework thrust, and duration bounds without approximations. See `lab/results/local-20261124-130147-11356-1-0-vibe-check-block-place-break/` for clean `DataComponentInitializers` / tag boot.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`; java/javac: `D:/path-tracing-deps/data/java/java-runtime-epsilon/`.

**Harness requirements (mandatory executed witnesses):**
- Boot tags (`MappedRegistry.prepareTagReload(...).apply()`) + `DataComponentInitializers` + Unsafe `ServerLevel` + real `LevelChunk` + real `ServerPlayer` (equipped with `Items.ELYTRA` in `EquipmentSlot.CHEST`, `startFallFlying()`) + `FireworkRocketEntity`.
- Inspect bytecode with `javap -c -p -constants` for `LivingEntity.updateFallFlyingMovement`, `FireworkRocketEntity.tick`, and `Player.tryToStartFallFlying`, then execute live on `server-26.2.jar`.
- Write `dossier.md` (`Claim: ...`), `rows.tsv`, and `witnesses.tsv` early after core glide/boost witnesses pass, then extend cleanly.

**Live scenes to execute:**
1. **Glide pitch sweep (`-45 deg`, `0 deg`, `+30 deg`):** execute 12 ticks of unboosted elytra glide recording raw `u64` IEEE bits for `(x, y, z, vx, vy, vz)` and `fallDistance` per tick.
2. **Firework rocket boost:** execute 25 ticks of attached `FireworkRocketEntity` boost (`Flight=1`, `Flight=2`, `Flight=3` lifetime RNG range and velocity formula `vec + look*0.1 + (look*1.5 - vec)*0.5`).
3. **Transition guards & landing:** check `tryToStartFallFlying` (not on ground, not in fluid, not fall-flying, elytra non-broken `willBreakNextUse`) and landing reset / crash damage (`horizontalCollision` kinetic damage).

**Write in your run folder:**
- `rows.tsv`: `rule_id	method	drag_or_coeff	pitch_or_flight	formula_constants	scars	source`
- `witnesses.tsv`: `case_id	tick	x_hex	y_hex	z_hex	vx_hex	vy_hex	vz_hex	events`
- `dossier.md` (first line `Claim: `), `reproduce.sh`, and `wiki.md`.

**Falsifier:** Any executed tick where glide position/velocity IEEE bits, firework boost vector, or start-glide guard disagrees with `server-26.2.jar`.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
