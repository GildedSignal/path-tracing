# GLM digest: what the newest lab results change

> Working report, appended incrementally as results and context files are read.


## 1. Per-result digest (appended incrementally)

### `local-20261115-1224-1-0-trial-food-crossing`
- **Lens:** `trial-food-crossing`
- **Grade:** `reproduced` (author + checker agree; `DONE` line 1, `check.md` line 3)
- **Settled fact:** In Mojang 26.2 (`server-26.2.jar` sha1 `86765a5899bd9c96461036a628796b4245715058`, `minecraft-26.2-client.jar` sha1 `2dc72797acbc1b63fc16a11c4ac393605f453754`), a held forward+sprint+jump on flat stone settles to a 12-tick cycle (4.2761 b/cycle, 7.1268 b/s) costing `0.2f` jump (`ServerPlayer.jumpFromGround` offsets 12–32) + `0.1f * 31 * 0.01f` sprint cm (`ServerPlayer.checkMovementStatistics` offsets 294–306) = `0.231` exhaustion/cycle. On Normal, food hits 6 (strict `foodLevel > 6` in `FoodData.hasEnoughFood` offsets 4–16, checked client-side at `LocalPlayer.isSprintingPossible` offset 29) after **330 cycles / 3,949 ticks / ~1,407 blocks** from spawn bars `(20, 5, 0)` and **589 cycles / 7,057 ticks** from `(20, 20, 0)`. On Peaceful, `FoodData.tick` offsets 57–61 skip `foodLevel -= 1` and `ServerPlayer.tickRegeneration` offsets 62–85 refill saturation to 20 every 20 ticks (`FoodData.tick` 10–28 drains `exh > 4.0f` unconditionally, so `exh` stays `< 4.0f`, correcting the dossier's Peaceful aside). Crucially, `(cycle phase, exhaustion bits, saturation bits)` does **not** recur before the crossing (`dossier.md` lines 9, 64–65, 84; `check.md` lines 7, 37).
- **Wiki status:** Page exists: [trial-food-crossing.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/lab/wiki/trial-food-crossing.md).
- **What it changes:**
  - `jobs\zones-design\redteam-opus.md` lines 16 (confirms crossing ~320 cycles / ~1.4k blocks at 330 cycles / ~1,407 blocks) and **refutes** line 130 (U2's claim that `(phase, exhaustion bits)` recurs after 4.0 wraps with a super-period).
  - `jobs\3.2d-3\spec.md` lines 63–65 & 139: confirms Peaceful needs no food-crossing/U2 super-period, and updates line 139 ("food-crossing event/U2 debit summary") because U2's exact recurrence premise is falsified.
  - `docs\engine.md` lines 149–152 (`FoodConstants`): clarifies `FoodData.tick` drains `exhaustionLevel > 4.0f` unconditionally (strict `>`), capped at `40.0f` (`FoodData.addExhaustion` offsets 0–15), and `ServerPlayer.checkMovementStatistics` rounds horizontal cm (`Math.round((float)Math.sqrt(dx*dx+dz*dz)*100.0f)`) with airborne non-elytra ticks charging 0 exhaustion.

### `local-20261115-1236-1-0-trial-offset-monotone`
- **Lens:** `trial-offset-monotone`
- **Grade:** `reproduced` (author + checker agree on claim; checker corrects harness bisection/window numbers; `DONE` line 1, `check.md` line 1)
- **Settled fact:** In Mojang 26.2 client physics (`minecraft-26.2-client.jar` sha1 `2dc72797acbc1b63fc16a11c4ac393605f453754`, `LivingEntity.aiStep` / `travelInAir` / `Entity.move` / `Entity.collide`), a held sprint-jump word's first exit event is **NOT globally monotone** in start offset `s`: on stone→ice, exit tick is monotone non-increasing across 5 constant-tick plateaus (4 single-ulp thresholds: bits `4057ee5506e671bf`, `4058025c608315b0`, `405816941351bc48`, `4058d8d1fd512ea5` per `check.md` lines 46–47), while on a 1-block gap exit tick reverses **12 → 14 → 12** across a fall-in window `s ∈ (96.336888156561680, 96.736888332719810)` of true width `0.40000017615813` (`check.md` lines 48–51; thresholds `4058158f9357b5e6` and `40582f292dae757a`). Every continuous exit field (`x, y, vx, vy`) sawtooths across exit-tick breakpoints; within each constant-tick plateau all fields are monotone, and every boundary is a single clean ulp flip without ulp-level ripples (`check.md` lines 40–44). (Checker note: author's `grid_point_bits + 1` was an indexing bug in `Sweep.refineFlip`, `check.md` lines 31–35.)
- **Wiki status:** Page exists: [trial-offset-monotone.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/lab/wiki/trial-offset-monotone.md).
- **What it changes:**
  - `jobs\zones-design\redteam-opus.md` lines 69–73 (finding 7): **refutes** global monotonicity of exit outcome in `s` (interval order does not give outcome order at a gap, and continuous fields sawtooth across breakpoints), while **confirming** finite intervals with clean single-ulp boundaries and within-plateau monotonicity, and confirming T7's named suspects (axis order, step-up) caused no within-plateau violation here.
  - `jobs\3.2d-3\spec.md` lines 136–137 (R5 portal band design door): records that R5 portal intervals cannot assume outcome monotonicity across interval boundaries (must be per-plateau/per-branch intervals, not a single global monotone map in `s`).

### `local-20261115-1239-1-0-port-wormhole-facts`
- **Lens:** `port-wormhole-facts`
- **Grade:** `reproduced` (`DONE` line 1, `check.md` line 1)
- **Settled fact:** In Mojang 26.2 (`server-26.2.jar` sha1 `86765a5899bd9c96461036a628796b4245715058`, `minecraft-26.2-client.jar` sha1 `2dc72797acbc1b63fc16a11c4ac393605f453754`):
  1. Portal wait (`NetherPortalBlock.getPortalTransitionTime` real offsets 0–50; `GameRules` offsets 502, 516): 80 ticks survival, 0 creative (`abilities.invulnerable`, not `isCreative()`), 0 non-players; post-transit cooldown (`Player.getDimensionChangingDelay` real offsets 0–2, `Entity.handlePortal` 63–66): **10 ticks** for players (300 base `Entity`, 2 `Projectile`).
  2. Coordinate scale (`DimensionType.getTeleportationScale` offset 14): `from.coordinateScale()/to.coordinateScale()` (8:1) on X/Z only (`NetherPortalBlock.getPortalDestination` offsets 64–98), used as search origin; linking (`PortalForcer.findClosestPortalPosition` offsets 9–21, 35–107): radius **16** (Nether) / **128** (Overworld), nearest by `distSqr` with ties broken by lowest `Vec3i::getY`, else `createPortal` builds a 3×2 obsidian pad + 4×5 frame + 2×3 interior (`offsets 83–490`).
  3. Pearl (`EnderpearlItem.use` 50–80; `Projectile.getMovementToShoot` 4–74; `shootFromRotation` 70–110; `ThrowableProjectile.tick` 4–55; `ThrownEnderpearl.onHit` 137–413): power `1.5f`, divergence `1.0f`, per-axis triangular spread `±0.0172275` on the pearl entity's own `RandomSource` before scaling by `1.5`, thrower momentum added as `(known.x, onGround ? 0.0 : known.y, known.z)`, gravity `0.03`, drag `0.99f` air / `0.8f` water, lands at `oldPosition()` for `5.0f` `ender_pearl` damage, `20`-tick item cooldown (`stacksTo(16).useCooldown(1.0f)`), and sets 10-tick portal cooldown.
  4. Unloaded-chunk branch (`LivingEntity.travelInAir` offsets 101–159): client-only `level.isClientSide()` + `!level.hasChunkAt(getBlockPosBelowThatAffectsMyMovement())` (where `ClientChunkCache.getChunk(..., FULL, false)` returns null; `EmptyLevelChunk` does NOT count) **replaces** gravity with `vy := (Y > minY) ? -0.1 : 0.0` then `× 0.98f` drag (`-0.098`/tick steady fall); `LocalPlayer` has zero chunk references, and "fluids skip" is `EmptyLevelChunk` returning void air.
- **Wiki status:** Page exists: [port-wormhole-facts.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/lab/wiki/port-wormhole-facts.md).
- **What it changes:**
  - `jobs\zones-design\redteam-opus.md` lines 92 (finding 11) and 111–119 (finding 14): **confirms** every portal/pearl/unloaded-chunk claim, refining that `-0.1` is a *set* (`vy := -0.1`, then `×0.98 = -0.098/tick`, or `0` at/below `minY`), living in `LivingEntity.travelInAir` (not `LocalPlayer`), and thrower momentum adds `Y` only when `!onGround()`.
  - `jobs\3.2d-3\spec.md` lines 135 & 140: confirms R7+ portal/pearl facts and unloaded-chunk `loaded(p,t)` semantics (`EmptyLevelChunk` does not count as loaded).
  - `docs\engine.md` lines 163–166 (Ender pearl): missing the exact triangular spread `±0.0172275` per axis before `1.5` scaling on the entity's own `RandomSource`, thrower momentum `(known.x, onGround ? 0 : known.y, known.z)`, `0.99f` air drag (`0.8f` water), `oldPosition()` landing, and 10-tick portal cooldown; also no `Nether portal` facts section yet.

### `local-20261115-1302-1-0-port-push-threshold`
- **Lens:** `port-push-threshold`
- **Grade:** `reproduced` (`DONE` line 1, `check.md` line 1)
- **Settled fact:** In Mojang 26.2 (`server-26.2.jar` sha1 `86765a5899bd9c96461036a628796b4245715058`), `Entity.push(Entity)` (offsets 0–169) gates on `Mth.absMax(dx, dz) >= (double)0.01f` (`0.009999999776482582d`, bits `3f847ae140000000`, Chebyshev distance, non-strict; NaN skips via `dcmpl`/`iflt 169` — `check.md` line 71 corrects dossier prose). Direct invocation on two `OAK_BOAT` entities reproduces bit-exact pair impulses (`A.x = bf747ae148f5c28f`, `B.x = 3f747ae148f5c28f` at exact cutoff and 1 ulp up; `0` at 1 ulp down `3f847ae13fffffff`), using `d0/=sqrt(d2)`, `d3=min(1.0, 1.0/d2)`, `*=d3`, `*= (double)0.05f` (`3fa99999a0000000`), applied additively via `Entity.push(DDD)` (5203–5242). Rejects strict `>` and `0.02` mutants.
- **Wiki status:** Page exists: [port-push-threshold.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/lab/wiki/port-push-threshold.md).
- **What it changes:**
  - Extends `port-direct-pair-push` (`local-20261114-1224-1-0`) with bit-exact runtime impulse outputs, divide-before-cap arithmetic order, and Chebyshev `euclidTrap` proof; supports closing `docs\HANDOFF.md` line 60 (Anomaly 5: entity push producer wiring).

### `local-20261115-1337-1-0-port-ordered-push-query`
- **Lens:** `port-ordered-push-query`
- **Grade:** `reproduced` (checker corrected one parenthetical on `x`-axis sign ordering; `DONE` line 1, `check.md` line 1)
- **Settled fact:** In Mojang 26.2 (`server-26.2.jar` sha1 `86765a5899bd9c96461036a628796b4245715058`), `Level.getPushableEntities` (`0–10`) → `Level.getEntities` (`0–121`) → `EntitySectionStorage.forEachAccessibleNonEmptySection` (`37–250`) **never sorts** by distance. It widens the query AABB asymmetrically (`minX-2, minY-4, minZ-2, maxX+2, maxY+0, maxZ+2`), iterates **signed `x` ascending** (`@75–82, 244–247` — negative-`x` columns first, refuting the dossier's "negative `x` last" parenthetical; `check.md` lines 53–63), then within each `x` column iterates `sectionIds.subSet(...)` in ascending masked `(z, y)` (`SectionPos.asLong` `(x&0x3FFFFF)<<42 | (z&0x3FFFFF)<<20 | (y&0xFFFFF)` — negative `z`/`y` sort after non-negative `z`/`y`), skips empty/inaccessible sections, and reads `ClassInstanceMultiMap.allInstances` in pure append order (`dragonParts` appended last). Equivalent to sorting section keys as signed `i64`.
- **Wiki status:** Page exists: [port-ordered-push-query.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/lab/wiki/port-ordered-push-query.md).
- **What it changes:**
  - Port batch (`engine/law/src/entity_contact.rs:13-15`, see `jobs/port-backlog/report.md` line 93): `entity_contact` requires vanilla iteration order; a Rust port must NOT sort sections as unsigned `u64` (must use signed `x` ascending, then masked `z, y`), and must preserve per-section append order + `(-2,-4,-2,+2,+0,+2)` AABB widen.
  - Supports closing `docs\HANDOFF.md` line 60 (Anomaly 5).

### `local-20261115-1340-1-0-port-push-order-noncommuting`
- **Lens:** `port-push-order-noncommuting`
- **Grade:** `lead` (checker downgraded from author's claim; `DONE` line 1, `check.md` line 1)
- **Settled fact:** In Mojang 26.2 (`server-26.2.jar` sha1 `86765a5899bd9c96461036a628796b4245715058`), `Entity.push(Entity)` (`0–169`) commits immediately via `push(DDD)` (`5203–5242`, `setDeltaMovement(getDeltaMovement().add(x,y,z))`). Author showed 2-term/same-order batch commutativity holds when starting from `deltaMovement = 0`, and 1-ulp mantissa order divergence for `>=3` terms (`9,786/59,520`) and 3-entity mutual loops (`288/992`). **Checker proved** (`check.md` lines 11–29) that at the real vanilla call site (`aiStep -> travel:617 -> pushEntities:817`), the receiver normally has `v0 != 0`, breaking **both** 2-term order commutativity (`264/1,260`) and same-order batching (`297/1,260`, 1 ulp). Thus a bit-exact port must commit each per-pair impulse immediately in vanilla iteration order (never batch unless `v0 == 0`).
- **Wiki status:** Page exists: [port-push-order-noncommuting.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/lab/wiki/port-push-order-noncommuting.md).
- **What it changes:**
  - Contradicts the dossier's "may batch only order-preserving" guidance for nonzero `v0` (`check.md` lines 24–28); confirms `engine/law/src/entity_contact.rs::push_pair` (`15–37`, bit-exact on 128 points) already commits immediately per pair, so caller only needs vanilla loop sequencing (`jobs/port-backlog/report.md` lines 94, 171).

### `local-20261115-1143-1-0-port-freeze-step-fluid`
- **Lens:** `port-freeze-step-fluid`
- **Grade:** `reproduced` (checker corrected two minor prose claims; `DONE` line 1, `check.md` line 1)
- **Settled fact:** In Mojang 26.2 (`minecraft-26.2-client.jar` sha1 `2dc72797acbc1b63fc16a11c4ac393605f453754`, byte-identical in `server-26.2.jar` `86765a5899bd9c96461036a628796b4245715058`), `Entity.lambda$checkInsideBlocks$0` calls `StepBasedCollector.advanceStep(step)` on both the block branch (`@194–198`, before `BlockState.entityInside` `@201–214`) and the fluid branch (`@292–296`, before `FluidState.entityInside` `@299–313`). `BlockBehaviour.getEntityInsideCollisionShape` defaults to `Shapes.block()` (`@0–3`, only 5 overriders jar-wide: `EndPortalBlock`, `LavaCauldronBlock`, `LayeredCauldronBlock`, `PowderSnowBlock`, `TripWireBlock`), so every non-air cell triggers `bl14` and advances step; DDA step resets per `Movement` (`bipush 16` budget per `Movement`, correcting dossier's "shared across movements", `check.md` lines 64–67), while `visitedBlocks` and `lastStep` span the tick. A water cell (or glass cell) at step 2 between two step-1 powder-snow cells across an X+Z two-movement tick splits the flush-run and doubles freeze (`+2` vs `+1` with air).
- **Wiki status:** Page exists: [port-freeze-step-fluid.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/lab/wiki/port-freeze-step-fluid.md) (also holds `local-20261114-1628-1-0`).
- **What it changes:**
  - Extends `l0-freeze-tick-111` (`2026-11-14`) and flags `engine/law/src/movement.rs:201-203, 994-1061` (`check.md` lines 68–72): Rust port lacks `StepBasedCollector` step-run set-dedup flushing, default `Shapes.block()` step advance on arbitrary non-air cells, and per-Movement reset; blocks `L0` (`l0-veto-20261114`, `jobs/port-backlog/report.md` lines 85, 165–166).

### `local-20261115-1144-1-0-port-breaking-threshold`
- **Lens:** `port-breaking-threshold`
- **Grade:** `reproduced` (`DONE` line 1, `check.md` line 1)
- **Settled fact:** In Mojang 26.2 (`server-26.2.jar` sha1 `86765a5899bd9c96461036a628796b4245715058`), `ServerPlayerGameMode.handleBlockBreakAction` START insta-mine gate (`@359–363`: `fload 6; fconst_1; fcmpl; iflt 379`, class-file offset `17132`, opcode `0x9B` at `17136`) is **non-strict** `f >= 1.0f` (`1.0f` destroys, `0x3F7FFFFF` continues; flipping byte `17136` `0x9B -> 0x9E` `ifle` diverges at `1.0f` sending stage 10 without breaking). `gameTicks` is the class's own counter (`tick()@0–7`, `gameTicks += 1`), `destroyProgressStart = gameTicks` (`@252–257`), progress is recomputed via `getDestroyProgress(...) * (float)(gameTicks - startTick + 1)` (`incrementDestroyProgress@24–29`), and delayed-destroy (`tick()@58–61`) also uses `>= 1.0f`.
- **Wiki status:** Page exists: [port-breaking-threshold.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/lab/wiki/port-breaking-threshold.md).
- **What it changes:**
  - Backs `engine/law/src/mining.rs:36-39, 147-151, 191, 210-215` with jar byte-level proof (`check.md` lines 29–30); `BlockState.getDestroyProgress` modifier chain remains caller-supplied via `MiningSample` (`jobs/port-backlog/report.md` line 86).

### `local-20261115-1145-1-0-port-mining-abort-target`
- **Lens:** `port-mining-abort-target`
- **Grade:** `reproduced` (`DONE` line 1, `check.md` line 1)
- **Settled fact:** In Mojang 26.2 (`server-26.2.jar` sha1 `86765a5899bd9c96461036a628796b4245715058`), `ServerPlayerGameMode.handleBlockBreakAction` ABORT_DESTROY_BLOCK (`@644–740`) runs only after reach (`@0–22`) and world-height (`@23–68`) gates, clears ONLY `isDestroyingBlock = false` (`@651–653`, unconditional; leaves `hasDelayedDestroy`, `destroyPos`, `destroyProgressStart`, `lastSentState` untouched), then emits `ClientboundBlockDestructionPacket(breakerId, pos, -1)` broadcasts via `ServerLevel.destroyBlockProgress` (same level, non-breaker, strict `dist² < 1024.0`): 1 reset at `pos` on `Objects.equals(destroyPos, pos)` match, or 2 resets in order — OLD `destroyPos` (`@683–699`) with `LOGGER.warn` (`@667–678`) then REQUESTED `pos` (`@713–726`) — on mismatch (including fresh ctor `destroyPos = BlockPos.ZERO`, `check.md` line 19). `Direction` is never read.
- **Wiki status:** Page exists: [port-mining-abort-target.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/lab/wiki/port-mining-abort-target.md).
- **What it changes:**
  - Port batch (`engine/law/src/mining.rs:43-52, 96-100`, `jobs/port-backlog/report.md` line 87): `MiningAction` currently has only `Start`/`Stop` and defers `Abort`; this pins the exact abort/reset packet law and delayed-lane independence.

### `local-20261115-1146-1-0-port-break-stop-boundary`
- **Lens:** `port-break-stop-boundary`
- **Grade:** `reproduced` (`DONE` line 1, `check.md` line 1)
- **Settled fact:** In Mojang 26.2 (`server-26.2.jar` sha1 `86765a5899bd9c96461036a628796b4245715058`), `ServerPlayerGameMode.handleBlockBreakAction` STOP_DESTROY_BLOCK (`@488–638`) is gated on `pos.equals(this.destroyPos)` (`@494–502`) and `!isAir` (`@526–531`), and destroys immediately (`destroyAndAck(..., "destroyed")` `@593`) iff `getDestroyProgress * (float)(gameTicks - destroyProgressStart + 1) >= 0.7f` (`ldc_w #327` `0x3F333333` `@561`, `fcmpl` `@564`, `iflt 600` `@565` — inclusive; NaN or `< 0.7f` hands off to `hasDelayedDestroy` `@600–627`, which completes at `>= 1.0f` in `tick()`).
- **Wiki status:** Page exists: [port-break-stop-boundary.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/lab/wiki/port-break-stop-boundary.md).
- **What it changes:**
  - Backs `engine/law/src/mining.rs:36-39, 159-178` (`MiningAction::Stop` inclusive `>= 0.7` and `delayed` handoff, `jobs/port-backlog/report.md` line 88) with jar-level byte proof (`cp#327 = 0x3F333333`, `check.md` line 10).

### `local-20261114-1013-1-0-port-fluid-rules`
- **Lens:** `port-fluid-rules`
- **Grade:** `lead` (checker falsified C4 / flowing-front waterlogging; `check.md` lines 1–10, 59–114)
- **Settled fact:** In Mojang 26.2 (`server-26.2.jar` sha1 `86765a5899bd9c96461036a628796b4245715058`), water crossing non-plain cells is gated by `FlowingFluid.canHoldAnyFluid` (`@0–115`: `LiquidBlockContainer` → true, `blocksMotion()` → false **before** shapes so fences deny pre-geometry, explicit deny list → false), `FlowingFluid.canPassThroughWall` (`@22–183`: full-cube early outs, else `!Shapes.mergedFaceOccludes(fromShape, toShape, dir)` on block collision shapes only — bottom slab/stair seal bottom face, top slab/stair leak downward), and `canHoldSpecificFluid` (`@0–35`). **Checker falsified** the author's C4 waterlogging claim (`check.md` lines 63–87): `SimpleWaterloggedBlock.canPlaceLiquid` (`@0–13`) checks `fluid == Fluids.WATER` (reference equality, not `WaterFluid.isSame`), and `FlowingFluid.getSpread`/`spread` check it with `newLiquid.getType()` *before* `spreadTo`, so **a decaying flowing front or single adjacent source NEVER waterlogs a dry carrier** (detours like a fence, wetting behind-cell at `t=25`). Dry carriers waterlog only under `>=2`-source conversion (`getNewLiquid` `@126–184`).
- **Wiki status:** Page exists: [port-fluid-rules.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/lab/wiki/port-fluid-rules.md).
- **What it changes:**
  - Port batch P2b (`engine/law/src/world.rs`, `engine/world/tools/ModelTable.java:108, 114`): `ModelTable` lacks container/`blocksMotion` markers; fluid-spread port must use `== Fluids.WATER` (source conversion only) for `canPlaceLiquid`, not `isSame` (`jobs/port-backlog/report.md` lines 54, 168–169).

### `local-20261114-1013-1-1-port-movement-modes`
- **Lens:** `port-movement-modes`
- **Grade:** `reproduced` (checker noted two extra triggers contradicting "exactly", `check.md` lines 4–10, 69–84)
- **Settled fact:** In Mojang 26.2 (`server-26.2.jar` sha1 `86765a5899bd9c96461036a628796b4245715058`), equipment-driven movement effects are a server-only, data-driven state machine: `LivingEntity.baseTick` (`@419–447`), `checkFallDamage` (`@33–51`), `collectEquipmentChanges` (`@0–234`), plus two checker-found triggers — `ServerPlayer.setGameMode` (`@60, @80`, spectator toggle) and `LivingEntity.onEquippedItemBroken` (`@23`) — call `Enchantment.runLocationChangedEffects` (`@12–295`) / `EnchantmentAttributeEffect` (`onChangedBlock@34`, `onDeactivated@28`) to add/remove transient modifiers (e.g. `minecraft:enchantment.soul_speed/feet`, `0.0405 + 0.0105*(lvl-1)` on `movement_speed`). Shared movement re-reads attributes only (`Player.getSpeed@0–8`, `LivingEntity.travelInWater@24–83` blending `water_movement_efficiency` toward `0.54600006f`). Whole-jar scan (7,445 classes, `check.md` lines 56–62) proves `SNEAKING_SPEED` has **no** server reader (Swift Sneak input scaling is client-side).
- **Wiki status:** Page exists: [port-movement-modes.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/lab/wiki/port-movement-modes.md).
- **What it changes:**
  - Port batch P4e (`engine/law/src/movement.rs:469-470`, `attributes.rs:57-80`, `state.rs:7-17`): missing `onChangedBlock` / `collectEquipmentChanges` / `setGameMode` / `onEquippedItemBroken` modifier hooks, Soul Speed, minecart `lastClientInput` (`handlePlayerInput@12–48`), and client-jar `sneaking_speed` reader (`jobs/port-backlog/report.md` lines 55, 153).

### `local-20261114-1103-1-0-port-peaceful-regen`
- **Lens:** `port-peaceful-regen`
- **Grade:** `reproduced` (checker validated `health > 0` delta, flagged oracle `isHurt` model bug and scenario-A prose overstatement, `check.md` lines 30–39)
- **Settled fact:** In Mojang 26.2 (`server-26.2.jar` sha1 `86765a5899bd9c96461036a628796b4245715058`), `ServerPlayer.tickRegeneration` (`@0–124`, called at `Player.aiStep@18`, after `ServerLevel.tickNonPassenger@8–15` increments `Entity.tickCount` and before `ServerPlayer.doTick@46 → FoodData.tick`) runs iff `difficulty == PEACEFUL && NATURAL_HEALTH_REGENERATION`: on `tickCount % 20 == 0` heals `1.0f` if `getHealth() < getMaxHealth()` (via `LivingEntity.heal@0–18` which guards `health > 0.0`) and increments `sat + 1.0f` if `sat < 20.0f` via clamp-free `FoodData.setSaturation` (`@0–5`, proven at `19.75 → 20.75`); on `tickCount % 10 == 0` increments `food + 1` if `foodLevel < 20` (`FoodData.needsFood@0–14`). `Entity.tickCount` is never saved in NBT or synced on the wire, and `ClientboundSetHealthPacket` skips sub-20 saturation +1 steps (`ServerPlayer.doTick@145–264`).
- **Wiki status:** Page exists: [port-peaceful-regen.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/lab/wiki/port-peaceful-regen.md).
- **What it changes:**
  - Anomaly 2 (`docs\HANDOFF.md` line 58) & `engine/law/src/movement.rs:1253-1268` (`server_regeneration`): missing `health > 0.0` guard at `:1257`, `tick_server_resources` uncalled by `tick_movement`, and `FoodData.tick`'s `Player.isHurt()` is `health > 0 && health < max` (no `food < 20` term; `check.md` line 32).
  - `jobs\3.2d-3\spec.md` lines 12, 20, 64: explains why peaceful `tickCount` phase is invisible on wire/NBT unless observed at login or via a debit/regen transition.

### `local-20261114-1103-1-1-port-boat-turn-trace`
- **Lens:** `port-boat-turn-trace`
- **Grade:** `reproduced` (checker corrected trend prose on CONT converging after `t=800`, `check.md` lines 60–73)
- **Settled fact:** In Mojang 26.2 (`server-26.2.jar` sha1 `86765a5899bd9c96461036a628796b4245715058`), `Mth.sin(double)` (`@0–15`) and `Mth.cos(double)` (`@0–19`, `+16384.0d`) use `f64` `SIN_SCALE = 10430.378350470453d`, and `Mth.SIN` matches `engine/law/src/sin.bin` `65536/65536` entries bit-for-bit. Running the jar's own `Mth.sin/cos` in the 800-tick W+A occupied-boat recurrence (`AbstractBoat.tick@150, 164` → `floatBoat`, `controlBoat@0–152`) reproduces `engine/law/tests/boat.rs:49` (`turn_table_record`) speed bits `4596625426452742353` (`0.20698460063159144`) and all 8 fields × 800 ticks bit-for-bit (`200,000` ticks and `2,000,000` random angles in checker). The `8.17480e-08` offset vs the analytic fixed point is Minecraft's own table + `f32` `deltaRotation = -9.9999933` quantization, not a port bug.
- **Wiki status:** Page exists: [port-boat-turn-trace.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/lab/wiki/port-boat-turn-trace.md).
- **What it changes:**
  - Closes `docs\HANDOFF.md` line 63 (Anomaly 10's boat-turn record open question) and `1.4d` judge finding C (`engine/law/src/math.rs:5-17`, `boat.rs:29-47`, `tests/boat.rs:49`).

### `local-20261114-1107-1-0-l0-freeze-tick-111`
- **Lens:** `l0-freeze-tick-111`
- **Grade:** `reproduced` (checker refuted dossier mechanic 7's "empty / no freeze while standing" claim, `check.md` lines 12–15, 90–105)
- **Settled fact:** In Mojang 26.2 (`minecraft-26.2-client.jar` sha1 `2dc72797acbc1b63fc16a11c4ac393605f453754`), the client's only local `ticksFrozen` writer is `InsideBlockEffectType.FREEZE` (`lambda$static$0@0–29`: `min(140, frozen + 1)`), fired once per **distinct step index** flushed by `InsideBlockEffectApplier$StepBasedCollector.advanceStep` (`@0–17`, `flushStep@65–83`) inside `Entity.applyEffectsFromBlocks` (`@0–143`) → `checkInsideBlocks` (`@65–211`, `forEachBlockIntersectedBetween`). At tick 111 of `l0-veto-20261114` powder snow run 3, crossing an integer `y` boundary visits the start box at step 0 and the entered slab at step 1 (`+2`: `3 → 5`). **Checker correction:** `PowderSnowBlock.getEntityInsideCollisionShape` (`@0–30`) returns `Shapes.block()` when `fallDistance <= 2.5`, NOT empty, so a standing overlapping entity DOES freeze (`check.md` lines 90–105).
- **Wiki status:** Page exists: [l0-freeze-tick-111.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/lab/wiki/l0-freeze-tick-111.md).
- **What it changes:**
  - Explains `docs\HANDOFF.md` line 47 (`l0-veto-20261114` tick-111 `+2` freeze gap); requires port batch P3a (`engine/law/src/movement.rs:199-203, 994-1061`, `world.rs:444-476`: `Shapes.block()` when `fallDistance <= 2.5` and `StepBasedCollector` step-index flush).

### `local-20261114-1158-1-0-port-update-order`
- **Lens:** `port-update-order`
- **Grade:** `reproduced` (`check.md` line 1)
- **Settled fact:** In Mojang 26.2 (`server-26.2.jar` sha1 `86765a5899bd9c96461036a628796b4245715058`), the fourth non-fluid update family is **support-loss pop**: synchronous edit-time destruction inside `Level.setBlock` (`@177–229`, `(flags&16)==0 && limit>0`, propagated `flags&-34`) → `BlockStateBase.updateNeighbourShapes` (`BlockBehaviour.UPDATE_SHAPE_ORDER = {W,E,N,S,D,U}`) → `CollectingNeighborUpdater` LIFO drain → `NeighborUpdater.executeShapeUpdate` (`@2–60`) → `LadderBlock.updateShape` (`@0–31`, pop iff `direction.getOpposite()==FACING && !canSurvive`) → `Block.updateOrDestroy` (`@2–58`) → `Level.destroyBlock` (`drop=(flags&32)==0`, `@81–94`). No `scheduleTick` or block-event in the support-loss pop path.
- **Wiki status:** Page exists: [port-update-order.md](file:///C:/Users/PASCAL/Desktop/path%20tracing/lab/wiki/port-update-order.md).
- **What it changes:**
  - Port batch P2a (`engine/law/src/world.rs:193-195` has `climbable`/`ladder_facing` but no `setBlock`/`updateShape`/`destroyBlock` producer, `jobs/port-backlog/report.md` line 58).

### `local-20261114-1158-1-1-item-use-writes`
- **Lens:** `item-use-writes`
- **Grade:** `reproduced` (checker noted 4 inherited-method edges on `BowItem`/`CrossbowItem` and `EntityType.spawn@23` `addFreshEntityWithPassengers`)
- **Settled fact:** Direct bytecode scan of 70 `Item` `use`/`useOn`/`releaseUsing` declarations (68 seeds across 51 classes; `use`=29) finds 25 own-class writes (16 classes) and 41 direct-helper writes -> 66 rows across 28 classes (block 23, entity 18, velocity 23, fluid-dispatch 2, 0 tick/effect/state), plus 12 blind-spot classes (`EggItem`, `EnderpearlItem`, `ExperienceBottleItem`, `FireworkRocketItem`, `FishingRodItem`, `LeadItem`, `PlaceOnWaterBlockItem`, `SnowballItem`, `SolidBucketItem`, `ThrowablePotionItem`, `TridentItem`, `WindChargeItem`) whose throwables share `Projectile.spawnProjectileFromRotation -> spawnProjectile` (`@9/@17/@67/@110`). (`lab\results\local-20261114-1158-1-1-item-use-writes\dossier.md`, `check.md`)
- **Wiki page:** `lab\wiki\item-use-writes.md` (already generated in `lab\wiki\index.md`).
- **What it changes:** Establishes the complete item-use side-effect inventory (`R7+` toolkit/edits & pearl/throwable item-use semantics in `jobs\3.2d-3\spec.md:140`; tool obligations in `docs\representation.md:100-103`) and shows `BowItem`/`CrossbowItem` + `EntityType.spawn@23` require inherited-method / passenger traversal beyond own+1-hop scanning.

### `local-20261114-1224-1-0-port-direct-pair-push`
- **Lens:** `port-direct-pair-push`
- **Grade:** `reproduced` (checker noted `reproduce.sh` cwd bug when run from repo root)
- **Settled fact:** `Entity.push(Entity)@53-59` uses non-strict `>= (double)0.01f` (`0x3f847ae140000000`), firing `∓0x3f747ae148f5c28f` at equality and killing the 1-byte strict mutant `iflt->ifle` (`@59`, class byte `120841`). Matches `engine/law/src/entity_contact.rs:23`. (`lab\results\local-20261114-1224-1-0-port-direct-pair-push\dossier.md`, `check.md`)
- **Wiki page:** `lab\wiki\port-direct-pair-push.md` (already listed in `lab\wiki\index.md` / `jobs\port-backlog\report.md`).
- **What it changes:** Seals the exact equality boundary for `Entity.push(Entity)` matching `engine/law/src/entity_contact.rs:23` (supporting `Anomaly 5` in `docs\HANDOFF.md:60` and direct pair-push port verification).

### `local-20261114-1225-1-0-port-flowing-water`
- **Lens:** `port-flowing-water`
- **Grade:** `reproduced`
- **Settled fact:** Flowing-water query (`getFlow@35-193`, `getHeight@1214-1225`, `hasSameAbove@1202-1212`, `Entity.updateFluidInteraction`, `Tracker.applyCurrentTo@0-134`) is bit-exact with `engine/law/src/world.rs:60-137` and `engine/law/src/movement.rs:845-926`; missing an advancing fluid-tick producer causes a silent tick-5 divergence (`in_water` `0` vs `1`) in a walled-source shaft. (`lab\results\local-20261114-1225-1-0-port-flowing-water\dossier.md`, `check.md`)
- **Wiki page:** `lab\wiki\port-flowing-water.md` (checker noted `dossier.md` referenced it before `lab-wiki.ps1` regenerated `lab\wiki\index.md:49`).
- **What it changes:** Confirms fluid flow/current queries in `world.rs:60-137` and `movement.rs:845-926` are bit-exact once fluid scheduled ticks advance, directly wiring fluid-tick scheduling into water-interaction fidelity (`docs\representation.md:135`).

### `local-20261114-1226-1-0-port-falling-block-landing-place`
- **Lens:** `port-falling-block-landing-place`
- **Grade:** `reproduced` (checker noted concrete-powder `@147-237` raytrace overwrites `pos`)
- **Settled fact:** `FallingBlockEntity.tick` grounded placement checks `@241-250` (`onGround || inWater`), reads fresh landing `@258`, checks `@324` (`canBeReplaced(DirectionalPlaceContext)`) AND `@340/@372` (`carried.canSurvive && !FallingBlock.isFree(below)`), branches `@390/395->741`, and commits `@461` (`setBlock(pos, carried', 3)`); 3-byte mutant `@392 pop;nop;nop` flips blocked stone to `PLACE`. (`lab\results\local-20261114-1226-1-0-port-falling-block-landing-place\dossier.md`, `check.md`)
- **Wiki page:** `lab\wiki\port-falling-block-landing-place.md` (generated in `lab\wiki\index.md` by post-`2026-11-15 14:20` `lab-wiki.ps1` run).
- **What it changes:** Provides exact grounded-placement bytecode contract for `FallingBlockEntity.tick` (including concrete-powder `@147-237` raytrace overwrite of `pos`) for the falling-block/schedule port backlog (`jobs\port-backlog\report.md`).

### `local-20261114-1255-1-0-port-scheduled-tick-dedup`
- **Lens:** `port-scheduled-tick-dedup`
- **Grade:** `reproduced` (`dossier.md:1-53`, `check.md:1-43`)
- **Settled fact:** In Mojang 26.2 (`server-26.2.jar` sha1 `86765a5899bd9c96461036a628796b4245715058`), `LevelChunkTicks.schedule(ScheduledTick)@0-18` runs `if (ticksPerPosition.add(tick)) scheduleUnchecked(tick)`, where `ticksPerPosition` is an `ObjectOpenCustomHashSet` under `ScheduledTick$1` (`UNIQUE_TICK_HASH`: `hashCode = 31*pos.hashCode() + type.hashCode()`, `equals` at `@25` `if_acmpne` reference-equal `type` and `@36` value `pos.equals`); due time, priority, and `subTickOrder` are absent from the key, so any duplicate `(type, BlockPos)` is ignored (first schedule wins) regardless of earlier due time or `EXTREMELY_HIGH` priority, and only `poll()@17-27` or matching `removeIf()@45-55` (not `peek()` or no-op `removeIf()`) releases the key (`dossier.md:1-29`, `check.md:7-16`). (`unpack(long)` bypasses via `scheduleUnchecked` after ctor probes pre-populate `@63-83`, `check.md:40`.)
- **Wiki page:** Generated via `lab-wiki.ps1` (`lab\wiki\index.md`) (`lab\wiki\port-scheduled-tick-dedup.md`).
- **What it changes:** Deepens `port-update-order` (`check.md:18-19`): the scheduled-tick recurrence for falling blocks and fluids must preserve first-scheduled due time and priority rather than replacing on revisit (`jobs\port-backlog\report.md`).

### `local-20261114-1300-1-0-port-glide-cosine-exact`
- **Lens:** `port-glide-cosine-exact`
- **Grade:** `reproduced` (`dossier.md:1-61`, `check.md:1-95`; checker corrected two prose numbers without affecting frozen bits: actual worst-row `cos` is `1.267590795056392e-06` not `~6e-6`, and total legal pitch floats are `2,238,185,474` with bit-uniform disagreement rate `0.062%`, `check.md:56-64`)
- **Settled fact:** `LivingEntity.updateFallFlyingMovement` computes lift cosine via `ldc_w 0.017453292f@9`, `fmul@12`, `f2d@51`, `Math.cos:(D)D@52`, `Mth.square@55` (`dossier.md:13`, `check.md:20-22`); on the staged Microsoft OpenJDK 25.0.1 Zen 4 rig, default `Math.cos` calls HotSpot's `UseLibmIntrinsic` stub (`-Xint` byte-identical, `-XX:-UseLibmIntrinsic` collapses 100% onto `StrictMath.cos`), returning `0x3fe6e7ab883308cb` (+1 ulp vs CR/`StrictMath.cos` `0x3fe6e7ab883308ca`) on pitch `0x42312bde` (`angle=0x3fe8bcdfa0000000`), disagreeing with `StrictMath.cos` on `1487/65536` seeded angles (exhaustive legal-pitch census confirms max 1 ulp, 0 sign flips, exact ± symmetry), while Rust `f64::cos` (Windows UCRT) is a third distribution disagreeing on `84/65536` seeded rows up to 10 result-ulps near zeros (`dossier.md:1,20-24,38-42`, `check.md:23-54`).
- **Wiki page:** Generated via `lab-wiki.ps1` (`lab\wiki\index.md`) (`lab\wiki\port-glide-cosine-exact.md`; `check.md:68` noted no prior wiki page at check time).
- **What it changes:** Pinpoints why `oracles/1.9b-elytra-oracle` (`compare.rs:19,53`) diverged from server glide (`check.md:68-73`): bit-exact elytra lift cannot use UCRT `f64::cos` or fdlibm/`StrictMath.cos` and requires reproducing HotSpot's Zen-4 libm cosine intrinsic table/approximation (`jobs\port-backlog\report.md`).

### `local-20261114-1553-1-0-port-elytra-bits`
- **Lens:** `port-elytra-bits`
- **Grade:** `reproduced` (checker refuted the generalization of `direct` Rust UCRT `f64::cos` being bit-exact, `dossier.md:1-48` vs `check.md:1-119`)
- **Settled fact:** In `server-26.2.jar` (`86765a5899bd9c96461036a628796b4245715058`), `LivingEntity.updateFallFlyingMovement` (`@0-262`) uses `getXRot()*0.017453292f` (`@9-12`) -> `f2d@51` -> `Math.cos:(D)D@52` -> `Mth.square:(D)D@55` for lift (while `Mth.sin:(D)F` `@151-157` and `Entity.calculateViewVector` `@0-67` use the float `SIN` table), and `-Xint` matches warmed C2 bit-for-bit (`dossier.md:11-12,29`, `check.md:29-34`). Checker's 4424-row adversarial test (`check/adv.csv`) reproduces 4424/4424 from JVM `dcos`, confirming the vanilla law, **but breaks author's UCRT claim**: rustc/MSVC `f64::cos` (UCRT) mismatches JVM `Math.cos` on `86/4424` rows (306 ulps apart at ±90°, `be6777a5cf72cffe` vs `be6777a5cf72cecc`, `check.md:62-78,86-100`). (Also checker noted prose slips: 31 ulps not `~7` at row 1029, and 265/262 unique inputs vs 530/524 rows, `check.md:90-93`.)
- **Wiki page:** Generated via `lab-wiki.ps1` (`lab\wiki\index.md`) (`lab\wiki\port-elytra-bits.md`).
- **What it changes:** Together with `port-glide-cosine-exact`, proves `engine/law/src/movement.rs:118` (`unimplemented!("LivingEntity.travelFallFlying; Math.cos")`) cannot use `Mth.cos` table (misses 4396/4424 adversarial rows, `check.md:79`), `f64` π/180 (4388/4424, `check.md:79`), or UCRT `(f3 as f64).cos()` (`86/4424` adversarial misses, `check.md:66,99`); it must reproduce JVM `Math.cos` (`jobs\port-backlog\report.md`).

### `local-20261114-1554-1-0-port-step-contact-witness`
- **Lens:** `port-step-contact-witness`
- **Grade:** `reproduced` (`dossier.md:1-119`, `check.md:1-90`)
- **Settled fact:** In `server-26.2.jar` (`86765a5899bd9c96461036a628796b4245715058`), `Entity.move@0-681` has no `stepOn` call (`dossier.md:18`, `check.md:29`); grounded contact fires once per tick in `LivingEntity.aiStep` (`travel@617` -> `applyEffectsFromBlocks()@638` -> `pushEntities@817`) -> `Entity.applyEffectsFromBlocks()@0-143` -> `applyEffectsFromBlocks(List)@0-43`, gated by `isAffectedByBlocks()@0-7` and `onGround()@8-12` (no sneak gate upstream), choosing support via `getOnPosLegacy() = getOnPos(0.2f)@15-16` (`dossier.md:24-47`, `check.md:23-39`), and sneaking only suppresses inside `MagmaBlock.stepOn@2-5` (`isSteppingCarefully() = isShiftKeyDown()` -> `ifne 29`, skipping `hurt(hotFloor(), 1.0f)` -> `Entity.hurt@8-27` `hurtServer`, `dossier.md:49-76`, `check.md:20-22,40-44`). Whole-jar census of 7445 classes confirms `Entity` is the sole caller and 6 overriders (`MagmaBlock`, `RedStoneOreBlock`, `SculkSensorBlock`, `SculkShriekerBlock`, `SlimeBlock`, `TurtleEggBlock`), with `Cat`/`Ocelot` overriding `isSteppingCarefully` (`dossier.md:79-85`, `check.md:56,75`).
- **Wiki page:** Generated via `lab-wiki.ps1` (`lab\wiki\index.md`) (`lab\wiki\port-step-contact-witness.md`).
- **What it changes:** Supplies the exact jar-side `grounded_contact` contract missing from earlier `port-block-laws` (`dossier.md:114-117`, `check.md:67-71`) for `MagmaBlock` damage, sneak suppression, and `0.2f` support selection (`jobs\port-backlog\report.md`).

### `local-20261114-1555-1-0-trial-slab-upstep-candidate`
- **Lens:** `trial-slab-upstep-candidate`
- **Grade:** `reproduced` (`dossier.md:1-55`, `check.md:1-46`; checker noted minor prose count "12/12" vs 11 `check()` calls, `check.md:38`)
- **Settled fact:** In `server-26.2.jar` (`86765a5899bd9c96461036a628796b4245715058`), `Entity.collectCandidateStepUpHeights` (`@84-91 d2f` on `(double)coordY - bb.minY`, `@94-101` skip `<0`, `@104-111` skip `== collidedDy`, `@114-121 fcmpl; ifle 124`, `@124-128 FloatSet.add`, `@151 FloatArrays.unstableSort`) admits `h <= maxUpStep` in `f32` after narrowing the difference and rejects `nextUp(0.6f)` (`dossier.md:7-24`, `check.md:13,21-22`); `Entity.collide(Vec3)@264-348` walks sorted ascending candidates against fixed unstepped `local 4` via `dcmpl; ifle 341` (`@305-316`) with immediate `areturn` (`@319-340`), so ties (e.g. `0.5f` vs `0.6f` both `0x3fb70a3d70a3d70a > 0x3f847ae147ae1478`) resolve to the **lower** height (first ascending candidate strictly beating unstepped, `dossier.md:15-18,22,28`, `check.md:14,23-24`).
- **Wiki page:** Generated via `lab-wiki.ps1` (`lab\wiki\index.md`) (`lab\wiki\trial-slab-upstep-candidate.md`).
- **What it changes:** Jar-side execution pin for `engine/law/src/movement.rs:623-676` (`mutant-step-up` at line `633`) and establishes the step-up tie-break rule ("lowest improving candidate", not greatest distance) (`dossier.md:32`, `check.md:30,34`).

### `local-20261114-1628-1-0-port-freeze-step-fluid`
- **Lens:** `port-freeze-step-fluid`
- **Grade:** `reproduced` (`dossier.md:1-67`, `check.md:1-86`; checker added adversarial non-intersecting Tripwire confirmation `check.md:57-66` and minor prose notes `check.md:78-83`)
- **Settled fact:** In `client-26.2.jar` (`2dc72797acbc1b63fc16a11c4ac393605f453754`), `Entity.lambda$checkInsideBlocks$0` calls `StepBasedCollector.advanceStep(step)` on **both** block (`@194-198`, before `BlockState.entityInside@214`) and fluid (`@292-296`, before `FluidState.entityInside@313`) paths, while `BlockBehaviour.getEntityInsideCollisionShape` returns `Shapes.block()` singleton by default (`if_acmpeq` `@80-85`; only 5 vanilla overriders, none overriding `getFluidState`), so any freshly-visited non-air cell (even `torch`) advances the step once on the block path and the fluid call is a same-step no-op (`dossier.md:1,14-22,29-42`, `check.md:20-41`). Counterfactuals confirm `ticksFrozen` is invariant (`+2`) to dropping water `advanceStep`, whereas per-epoch `clearFire` count changes (`3` vs `2`, `dossier.md:40-42`, `check.md:45`).
- **Wiki page:** `lab\wiki\port-freeze-step-fluid.md` (later updated by `local-20261115-1143-1-0-port-freeze-step-fluid`; both covered in `lab\wiki\index.md`).
- **What it changes:** Flags `engine/law/src/movement.rs:1027` (`!(inside || fluid_inside) || !visited.insert(p)`): matches jar on water-only cells, but diverges on exported shape-empty fluid-less non-air blocks (`inside_shape: Some(empty)`) via epoch grouping (`clearFire` `3`-vs-`2`), though never on `ticksFrozen` (`dossier.md:46-48`, `check.md:70-74`).

### `local-20261114-1633-1-0-port-freeze-damage`
- **Lens:** `port-freeze-damage`
- **Grade:** `reproduced` (`dossier.md:1-59`, `check.md:1-103`; checker falsified one explanatory sub-sentence about surviving `%40` outside snow while confirming all 6 consumer rules, `check.md:8-9,69-76`)
- **Settled fact:** In `server-26.2.jar` (`86765a5899bd9c96461036a628796b4245715058`), `LivingEntity.aiStep` runs `travel@617` -> server-only freeze block `@666-779` (`ServerLevel` gate `@667-685`, decay `-2` `@697-722` if `!(isInPowderSnow && canFreeze())`, `removeFrost`/`tryAddFrost` `@725-730` transient `-0.05f * getPercentFrozen()`, damage gate `tickCount % 40 == 0 && isFullyFrozen() && canFreeze()` `@733-768` firing `hurtServer(level, damageSources().freeze(), 1.0f)`) -> `pushEntities@817`, verified live in headless boot (hits every `40` ticks, health `-1.0`, `HurtTime=10`, identical `[0.0,0.0,0.0]` motion across 660 samples, `ticksFrozen` cap `140`) (`dossier.md:1,5-30,36-40`, `check.md:22-65`). **Checker correction:** because `Entity.baseTick@75-85` clears `isInPowderSnow` and decay `@697-722` runs before `@733-768`, `tf >= 140` survives **zero** ticks outside snow (`140 -> 138`), refuting `dossier.md:9`'s parenthetical "survives up to 20 ticks" (`check.md:69-76`); also minor offset citation nits `250-256`, `27-39`, `51-107` (`check.md:98-99`).
- **Wiki page:** Generated via `lab-wiki.ps1` (`lab\wiki\index.md`) (`lab\wiki\port-freeze-damage.md`).
- **What it changes:** Delivers the complete server-side `freeze_damage` event, `hurtServer` writes (`lastHurt=1.0`, `invulnerableTime=20`, `hurtTime/hurtDuration=10`, `hurtMarked=true`, armor bypassed, absorption first, no knockback, no difficulty scaling, no exhaustion), frost `-5%` modifier, and powder-snow `(0.9, 1.5, 0.9)` stuck rule (`dossier.md:44-49`, `check.md:92-93`) for `engine/law` replay and `docs\engine.md` (`ticksFrozen` / `hurtMarked` server-driven semantics).

### `local-20261114-1645-1-0-port-sprint-packet-split`
- **Lens:** `port-sprint-packet-split`
- **Grade:** `reproduced` (`dossier.md:1-62`, `check.md:1-47`)
- **Settled fact:** In `client-26.2.jar` (`2dc72797acbc1b63fc16a11c4ac393605f453754`), sprint flag and sprint speed modifier are separate stores: `ClientPacketListener.handleSetEntityData@12-39` calls only `SynchedEntityData.assignValues` (bit `3` / `0x8`), `handleUpdateAttributes@61-204` (`removeModifiers()@152`, `setBaseValue@142`, `addTransientModifier@157-198`) replaces only attributes without touching flags, and whole-jar scan confirms `LivingEntity.setSprinting@0-35` is the sole writer of both (`dossier.md:8-12`, `check.md:9-13,23-24`). Direct jar invocation reproduces the recorded split window `(sprint=false, modifier=true, speedBits=3e051eb9)` bit-for-bit (`float32(0.10000000149011612 * (1 + 0.30000001192092896))`), rejecting any single-bit sprint representation (`dossier.md:22-34`, `check.md:17-20`).
- **Wiki page:** Generated via `lab-wiki.ps1` (`lab\wiki\index.md`) (`lab\wiki\port-sprint-packet-split.md`).
- **What it changes:** Pins why any client/port sprint model (and `engine/law` attribute/flag handling) must separate the synced `FLAG_SPRINTING` bit from the `MOVEMENT_SPEED` attribute modifier set (`base = 0.10000000149011612`, `0x3e051eb9` vs literal `0.1` `0x3e051eb8`) so packet-split states `(false, true, 3e051eb9)` replay cleanly (`jobs\port-backlog\report.md`).

### `local-20261114-1730-1-0-trial-sprint-jump-basis`
- **Lens:** `trial-sprint-jump-basis`
- **Grade:** `reproduced` (`dossier.md:1-57`, `check.md:1-38`; checker noted a second `jumpFromGround` site in `LocalPlayer.aiStep@628` after sprint sites `@399-484` for creative double-jump, `check.md:28`)
- **Settled fact:** In `server-26.2.jar` (`86765a5899bd9c96461036a628796b4245715058`) and `client-26.2.jar` (`2dc72797acbc1b63fc16a11c4ac393605f453754`), `LivingEntity.jumpFromGround` reads **only** `isSprinting() = getSharedFlag(3)` (`@40-44`) for the horizontal `+0.2` impulse (`0` `getAttribute`/`MOVEMENT_SPEED` reads; direct `2x2` flag×modifier `ArmorStand` probe confirms only `flag=true` boosts `dx` `0.25 -> 0.04999999999999999`, `0x3fd0000000000000 -> 0x3fa9999999999998`, even at `1024+100` speed in checker's adversarial run), while `LocalPlayer.aiStep` sprint updates (`@399-484`) precede `super.aiStep()@1095` -> `LivingEntity.aiStep` (`applyInput@218` -> `LocalPlayer.applyInput@35-46`, `jumpFromGround@462`, `travel@617`) (`dossier.md:1,9-42`, `check.md:12-24`).
- **Wiki page:** Generated via `lab-wiki.ps1` (`lab\wiki\index.md`) (`lab\wiki\trial-sprint-jump-basis.md`; `check.md:35` noted no prior wiki page at check time).
- **What it changes:** Separates the two consumers of sprint state in `engine/law`: jump horizontal boost (`movement.rs`) depends strictly on the shared sprint flag, not `MOVEMENT_SPEED` / `sprint_speed_modifier` (`check.md:34`).

### `local-20261114-1809-1-0-trial-sprint-replay-input`
- **Lens:** `trial-sprint-replay-input`
- **Grade:** `reproduced` (`dossier.md:1-78`, `check.md:1-34`)
- **Settled fact:** In the 200 TPS step-ups recording (`bot-latest.log`, sha1 `428f97fc9be32adc14d0c694083e2f8af6488ea8`), all four `(flag, modifier)` cells occur across `11119` probes (`(F,F)=10882`, `(F,T)=14`, `(T,F)=1`, `(T,T)=222`), and `Player.travel` executes `playerTick=2584` and `2621` in the split state `(sprint=false, modifier=true, speedBits=3e051eb9)`; deriving `modifier` from `flag` (`3dcccccd`) causes `0.440797` blocks drift in 10 ticks (`dossier.md:1,14-30,40-47`, `check.md:13-19`). Across all `11119` probes, `speedBits` tracks `modifier` exclusively (`T->3e051eb9`, `F->3dcccccd`; pure `f32` `0.1f*1.3f` gives `3e051eb8`, proving `f64` attribute pipeline), and `handleSetEntityData@24-36` / `handleUpdateAttributes@61-198` vs `LivingEntity.setSprinting@0-35` establish that replays must record `(flag, modifier, ordered packet/setSprinting event stream)` (`dossier.md:36-59`, `check.md:13-18`).
- **Wiki page:** Generated via `lab-wiki.ps1` (`lab\wiki\index.md`) (`lab\wiki\trial-sprint-replay-input.md`).
- **What it changes:** Directly challenges `engine/law/src/attributes.rs:51-54` + `movement.rs:41-45,221,230` coupling `sprinting` and `sprint_speed_modifier`, and pins the required replay/recorder input fields (`jobs\port-backlog\report.md`, `docs\representation.md`).

### `local-20261114-1832-1-0-port-contact-cooldown`
- **Lens:** `port-contact-cooldown`
- **Grade:** `reproduced` (`dossier.md:1-48`, `check.md:1-52`)
- **Settled fact:** In `server-26.2.jar` (`86765a5899bd9c96461036a628796b4245715058`), `LivingEntity.hurtServer` decides equal `minecraft:hot_floor` `1.0F` hits by strict `(float)invulnerableTime > 10.0F` (`@184-193`: `i2f; ldc 10.0f; fcmpl; ifle 240`): cooldown 9 (`invulnerableTime=11`) enters `@206-216` (`amount <= lastHurt`) and returns `false` with zero writes, whereas cooldown 10 (`invulnerableTime=10`) and 11 (`9`) enter `@240-269` (`lastHurt=amount`, `invulnerableTime=20`, full `actuallyHurt(amount)`, `hurtDuration=hurtTime=10`), giving accept spacing `10` ticks (`0, 10, 20, 30` vs mutant `>=` `0, 11, 22`) (`dossier.md:3,9-24`, `check.md:17-23,34-36`). Checker also proved `data/minecraft/tags/damage_type/bypasses_cooldown.json` is absent in vanilla, and `ServerPlayer.tick()@21-35` handles the once-per-tick decrement (`LivingEntity.baseTick@475` skips `ServerPlayer`) (`check.md:27-28`).
- **Wiki page:** Generated via `lab-wiki.ps1` (`lab\wiki\index.md`) (`lab\wiki\port-contact-cooldown.md`; `check.md:42` noted no prior wiki page at check time).
- **What it changes:** Pins `ContactDamage::HotFloor` accept cadence (`10` ticks / health point on magma) and zero-write refusal at `invulnerableTime > 10.0f` for `engine/law` damage/resource modeling (`check.md:42`, `jobs\port-backlog\report.md`).

### `local-20261114-1846-1-0-port-legacy-support-overhang`
- **Lens:** `port-legacy-support-overhang`
- **Grade:** `reproduced` (`dossier.md:1-74`, `check.md:1-56`)
- **Settled fact:** In `server-26.2.jar` (`86765a5899bd9c96461036a628796b4245715058`), whenever `Entity.mainSupportingBlockPos` is present (`@0-7`), `Entity.getOnPos(float)@0-147` (`getOnPos() = getOnPos(1.0E-5f)`, `getOnPosLegacy() = getOnPos(0.2f)`, `getBlockPosBelowThatAffectsMyMovement() = getOnPos(0.500001f)`) returns `p` on `offset <= 1.0E-5f` (`@21-26/97-98`), `FENCES` with `offset <= 0.5` (`@38-54`), or `WALLS`/`FenceGateBlock` unconditionally (`@57-74`), else `p.atY(Mth.floor(y - offset))` (`@79-96`) — never flooring `center.x/z` unless `mainSupportingBlockPos` is empty (`@99-147`), verified bit-for-bit across W1–W9 and checker adversarial A1–A5 (`dossier.md:9-40`, `check.md:18-37`). Matches `engine/law/src/movement.rs:331-345` (`on_pos`) and `engine/world/tools/ModelTable.java:113-116` (`dossier.md:46-52`, `check.md:25-27`).
- **Wiki page:** Generated via `lab-wiki.ps1` (`lab\wiki\index.md`) (`lab\wiki\port-legacy-support-overhang.md`; `check.md:44` noted no prior wiki page at check time).
- **What it changes:** Confirms `engine/law/src/movement.rs:331-345` (`on_pos`) and its callers (`350`, `365`, `431`, `762`, `976`) match the jar bit-for-bit on overhangs/fences/walls, and leaves a **lead** (`dossier.md:69`) that `applyMovementEmissionAndPlaySound@28` uses `getOnPos(1e-5f)` -> `isStateClimbable`.

### `local-20261114-1940-1-0-port-elytra-climbable-branch`
- **Lens:** `port-elytra-climbable-branch`
- **Grade:** `reproduced` (`dossier.md:1-63`, `check.md:1-108`; checker caught cosmetic label error in Limits "GRAVITY=0.0764" vs actual effective gravity `0.08`, `check.md:85-89`)
- **Settled fact:** In `server-26.2.jar` (`86765a5899bd9c96461036a628796b4245715058`), `LivingEntity.travel` tests `shouldTravelInFluid@12-23` strictly before `isFallFlying()@27-38` (fluid shadows fall-flying, `Case F`), and `LivingEntity.travelFallFlying` checks `onClimbable()@0-4` -> `travelInAir(input)@7-11` (invoking ladder clamp + `move(SELF)` while bit 7 is still `true`) **before** `stopFallFlying()@12-15` (which writes `setSharedFlag(7, true)@0-4` then `setSharedFlag(7, false)@7-11`), skipping the free-glide path `@17-73` completely; byte-spliced mutant swapping `@7-16` flips `isFallFlyingAtEntry` to `false` (`dossier.md:1,11-45`, `check.md:26-44,48-73`).
- **Wiki page:** Generated via `lab-wiki.ps1` (`lab\wiki\index.md`) (`lab\wiki\port-elytra-climbable-branch.md`; `check.md:76` noted missing before regeneration).
- **What it changes:** Completes the third elytra stop/mode-transition rule (alongside `port-elytra-delta` and `port-movement-modes`, `check.md:76-82`): fluid > fall-flying > air, and onClimbable cancel executes air+ladder `move` with flag 7 visible before `stopFallFlying` clears it (`jobs\port-backlog\report.md`).

### `local-20261114-2027-1-0-port-elytra-cosine-domain`
- **Lens:** `port-elytra-cosine-domain`
- **Grade:** `reproduced` (`dossier.md:1-72`, `check.md:1-107`; checker corrected Limits legal float count from `~1.3e9`/`0.08%` to `2,238,185,474`/`~0.045%`, and clarified Rust Windows toolchain `msvcrt.dll -> UCRT`, `check.md:75-79,83-85`)
- **Settled fact:** Over 1M seeded legal pitch floats (`SEED=0xC05E1E11DA7A4BDF`) pushed through `(double)(pitch * 0.017453292f)` in `server-26.2.jar` (`86765a5899bd9c96461036a628796b4245715058`) on MS OpenJDK 25.0.1 Win11 x64, default `Math.cos` (`-Xint` identical) disagrees with `StrictMath.cos` on `623/1M` (`0.0623%`, always `1` ulp, `[32,64)°` peak `4.098%`; checker exhaustive sweep over `[0,1) U [4,90]` confirms `max_ulp=1` and `[32,64)° = 4.2445%`), `DisableIntrinsic=_dcos` gives `0` (`hash 31de578536edde88`), `cos(-x)==cos(x)` has `0` violations, and Rust `f64::cos` (`msvcrt->UCRT`) mismatches default JVM on `67/1M` (`19` ulp at `89.99996°`) and `660/1M` vs `StrictMath`, reaching `306` ulp in checker's exhaustive `[64,90]` cancellation-zone sweep (`dossier.md:1,20-57`, `check.md:20-74`).
- **Wiki page:** Generated via `lab-wiki.ps1` (`lab\wiki\index.md`) (`lab\wiki\port-elytra-cosine-domain.md`; `check.md:94` noted missing at check time).
- **What it changes:** Reinforces `port-glide-cosine-exact` and `port-elytra-bits`: the Rust port for `engine/law/src/movement.rs:118/131` must target the default JVM `_dcos` intrinsic bits (signs freely foldable; near-±90° cancellation is worst class, `306` ulp UCRT divergence) (`dossier.md:54-57`, `check.md:92-95`).

### `local-20261114-2028-1-0-trial-boat-water-surface-bits`
- **Lens:** `trial-boat-water-surface-bits`
- **Grade:** `reproduced` (`dossier.md:1-76`, `check.md:1-43`; checker corrected splash reposition sign prose in `dossier.md:42` to `wlAbove - bbHeight + 0.101` `@52-65`, `check.md:28`)
- **Settled fact:** In `server-26.2.jar` (`86765a5899bd9c96461036a628796b4245715058`), `AbstractBoat.checkInWater` scans `y ∈ [Mth.floor(bb.minY)@25, Mth.ceil(bb.minY + 0.001)@34-38)`, computes `f = (float)y + FluidState.getHeight` (`i2f@150, fadd@162`), unconditionally updates `waterLevel = Math.max(waterLevel, (double)f)` (`@165-176`), and tests strict `bb.minY < (double)f` (`dcmpg@188 / ifge@189`): exact surface equality `S = (double)(64.0f + 8/9f)` is dry (`IN_AIR`), `nextDown(S)` (1 double ulp lower) is wet (`IN_WATER`), consumed in the same tick by `AbstractBoat.floatBoat()@150` (`d1 = (waterLevel - getY())/bbHeight`, `f = 0.9f`, buoyancy tail `(vy + d1*(0.04/0.65))*0.75`, `0.65@344, 0.75@350`), while `getStatus@0-56` gives `isUnderwater()` priority over `checkInWater` (`dossier.md:1,11-50`, `check.md:11-20`).
- **Wiki page:** Generated via `lab-wiki.ps1` (`lab\wiki\index.md`) (`lab\wiki\trial-boat-water-surface-bits.md`; `check.md:34` noted it recovers truncated `lab\wiki\port-boat-surface-transition.md`).
- **What it changes:** Recovers and strengthens `port-boat-surface-transition` for the boat/water-entry boundary in `engine/law`, adding ulp-level strictness, float widening `(double)(float)(y + getHeight)`, and same-tick `floatBoat` buoyancy arithmetic (`check.md:34-35`, `jobs\port-backlog\report.md`).

### `local-20261114-2029-1-0-port-fluid-height-witness`
- **Lens:** `port-fluid-height-witness`
- **Grade:** `reproduced` (`dossier.md:1-114`, `check.md:1-40`)
- **Settled fact:** In `server-26.2.jar` (`86765a5899bd9c96461036a628796b4245715058`), `FlowingFluid.getHeight@0-15` = `hasSameAbove@0-20 ? 1.0f : getOwnHeight@0-9` (`amount / 9.0f`), where `hasSameAbove` virtual-dispatches `Fluid.isSame` (`Fluid@116-124` reference, `WaterFluid@157-168` `WATER||FLOWING_WATER`, `LavaFluid@338-349` `LAVA||FLOWING_LAVA`), so any differing fluid above (including checker's `FLOWING_LAVA` above water) yields `own_height` not `1.0f` (`3de38e39` at amount 1, `3f638e39` at amount 8) (`dossier.md:1,37-60`, `check.md:17-33`). Re-running `oracles/1.9c-fluid-height/run.ps1`, `cargo test -p law --test fluid_height` (2 passed), and `scratch_height.rs` matches all 136 rows of `engine/law/tests/data/fluid-heights.csv` (`engine/law/src/world.rs:60-79` `surface_height`/`own_height`; production `Fluid` constructor `engine/world/src/snapshot.rs:87-91`), while the "any fluid above" mutant breaks all `34` cross-kind rows (`dossier.md:8-11,20-33,74-90`, `check.md:7-11,23-24`).
- **Wiki page:** Generated via `lab-wiki.ps1` (`lab\wiki\index.md`) (`lab\wiki\port-fluid-height-witness.md`).
- **What it changes:** Recovers evidence for lost `local-20261107-1500-1-4-port-fluid-rules`, confirms `engine/law/src/world.rs:60-79` + callers (`movement.rs:897,1041`, `world.rs:388`) match vanilla bit-for-bit, and identifies `engine/world/src/snapshot.rs:87-91` as the sole production `Fluid` seam (`dossier.md:81-90`, `check.md:24,37`).

### `local-20261114-2030-1-0-port-water-flow-axis`
- **Lens:** `port-water-flow-axis`
- **Grade:** `reproduced` (`dossier.md:1-66`, `check.md:1-94`; checker added 6 adversarial X/Z/mixed/asymmetric cases all bit-exact `check.md:45-62`)
- **Settled fact:** In `server-26.2.jar` (`86765a5899bd9c96461036a628796b4245715058`), `FlowingFluid.getFlow@0-340` for non-falling level-4 flowing water (`ownHeight = 0x3ee38e39`, 4/9f) reads strictly the four horizontal neighbor `getFluidState` entries in order `NORTH, EAST, SOUTH, WEST` with zero `getBlockState` reads, computes `g = current.getOwnHeight() - n.getOwnHeight()` in `f32`, widens `(double)(step*g)` when `g != 0.0f` (`@178-213`), and normalizes (`Vec3.normalize`: returns `Vec3.ZERO` sentinel by reference when `length < 9.999999747378752E-6d`): equal `(1,1,1,1)`, `(4,4,4,4)`, and `(7,7,7,7)` return `Vec3.ZERO` (`0x0`), while `grad71` (`7,1,4,4`) vs `grad17` (`1,7,4,4`) return `-1.0` (`0xbff0000000000000`) vs `+1.0` (`0x3ff0000000000000`), and mixed `7,1,7,1` returns `(-1/sqrt(2), 0, -1/sqrt(2))` (`0xbfe6a09e667f3bcd`) (`dossier.md:1,9-36`, `check.md:26-62`).
- **Wiki page:** Generated via `lab-wiki.ps1` (`lab\wiki\index.md`) (`lab\wiki\port-water-flow-axis.md`).
- **What it changes:** Sharpens `port-flowing-water` (`world.rs:60-137`, `movement.rs:845-926`) with exact float-subtraction/double-accumulation bit patterns, `Vec3.ZERO` sentinel threshold, and `NORTH/EAST/SOUTH/WEST` read order (`check.md:82-86`, `jobs\port-backlog\report.md`).

### `local-20261114-2031-1-0-port-fluid-tick-delay`
- **Lens:** `port-fluid-tick-delay`
- **Grade:** `reproduced` (`dossier.md:1-106`, `check.md:1-94`; checker executed previously unexecuted `LavaFluid.getSpreadDelay` adversarially over `n=10000` samples `check.md:51-70`)
- **Settled fact:** In `server-26.2.jar` (`86765a5899bd9c96461036a628796b4245715058`), `WaterFluid.getTickDelay@0-1` = constant `5` (inherited by `FlowingFluid.getSpreadDelay@0-5`), `EmptyFluid.getTickDelay@0-1` = `0`, and `LavaFluid.getTickDelay@0-14` = `LavaFluid.isFastLava(level) ? 10 : 30`, where `isFastLava@0-20` reads `LevelReader.environmentAttributes().getDimensionValue(EnvironmentAttributes.FAST_LAVA)` (`minecraft:gameplay/fast_lava`, default `false` `@938-963`, set `true` only in `data/minecraft/dimension_type/the_nether.json`), paired with `LavaFluid.getDropOff` (`1` vs `2`) and `getSlopeFindDistance` (`4` vs `2`); on rising lava spread (old/new non-empty, neither `FALLING`, `newHeight > oldHeight`), `LavaFluid.getSpreadDelay@0-93` draws `nextInt(4)` and multiplies delay by `4` with probability `3/4` (`40` fast / `120` normal), executed over `n=10000` (`base(30)=2555`, `quad(120)=7445`, `0` RNG calls when empty or `FALLING`) (`dossier.md:1,17-41,53-62`, `check.md:23-67`). (`BucketItem` uses `LiquidBlock.onPlace` -> `scheduleTick` rather than calling `getTickDelay`, `dossier.md:32-36`.)
- **Wiki page:** Generated via `lab-wiki.ps1` (`lab\wiki\index.md`) (`lab\wiki\port-fluid-tick-delay.md`; `check.md:92` noted no page yet at check time).
- **What it changes:** Supplies fluid scheduled-tick delays (`5` water, `30`/`10` lava via `the_nether` `fast_lava`, and `3/4` `×4` rising-lava RNG) missing from `port-flowing-water` (`check.md:91-94`, `jobs\port-backlog\report.md`).

### `local-20261114-2032-1-0-port-water-delay-minimum`
- **Lens:** `port-water-delay-minimum`
- **Grade:** `reproduced` (`dossier.md:1-62`, `check.md:1-33`)
- **Settled fact:** In `server-26.2.jar` (`86765a5899bd9c96461036a628796b4245715058`), `WaterFluid.getTickDelay(LevelReader)@0-1` is `iconst_5; ireturn` (zero `aload_1`, level unused), verified directly at runtime on `Fluids.WATER.getTickDelay(null) == 5` and `Fluids.FLOWING_WATER.getTickDelay(null) == 5` (`Fluids.EMPTY.getTickDelay(null) == 0`), whereas `LavaFluid.getTickDelay@0-14` loads `aload_1` immediately to call `isFastLava@0-20` -> `FAST_LAVA` (`10`/`30`) and throws NPE on `null` (`dossier.md:3,18-45`, `check.md:10-25`). (`FlowingFluid.getSpreadDelay@0-5` delegates to `getTickDelay`, `check.md:12`.)
- **Wiki page:** Generated via `lab-wiki.ps1` (`lab\wiki\index.md`) (`lab\wiki\port-water-delay-minimum.md`).
- **What it changes:** Minimal null-level pin corroborating `port-fluid-tick-delay` (`5` water cadence, `EmptyFluid = 0`, `LavaFluid` `10`/`30` `FAST_LAVA` split vs old `ultraWarm()`, `dossier.md:36,48`, `check.md:29-30`).

### `local-20261114-2033-1-0-port-water-source-neighbor-count`
- **Lens:** `port-water-source-neighbor-count`
- **Grade:** `reproduced` (`dossier.md:1-75`, `check.md:1-53`; checker noted minor citation range `isSourceBlockOfThisType @856-869` vs `@856-879`, `check.md:20,49`)
- **Settled fact:** In `server-26.2.jar` (`86765a5899bd9c96461036a628796b4245715058`), `FlowingFluid.getNewLiquid@0-284` counts horizontal `isSame && canPassThroughWall && isSource` neighbors in `Direction.Plane.HORIZONTAL` (`@15-123`) and tests earliest threshold `sourceCount >= 2` (`@126-129`: `iload 5; iconst_2; if_icmplt 185`) before `WaterFluid.canConvertToSource@0-16` (`GameRules.WATER_SOURCE_CONVERSION`, default `true` at `GameRules.<clinit>@822`) and `below.isSolid()@162-167 || isSourceBlockOfThisType(belowFluid)@170-173` (`@856-869`) (`dossier.md:1,10-31`, `check.md:17-20`). Direct jar execution across R1–R8 and checker's A1–A9 proves `1` source -> `flowing(7)` (even with a vertical source above, yielding `flowing(8, falling=true)`), `>=2` sources (`<=4`) with solid or matching source below and gamerule on -> `source`, while `lava` neighbors/below or `WATER_SOURCE_CONVERSION=false` fail conversion (`dossier.md:37-56`, `check.md:24-37`).
- **Wiki page:** Generated via `lab-wiki.ps1` (`lab\wiki\index.md`) (`lab\wiki\port-water-source-neighbor-count.md`).
- **What it changes:** Makes each predicate of the water source-conversion law (`sourceCount >= 2`, horizontal-only, same-kind `isSame`, solid-or-matching-source below, `WATER_SOURCE_CONVERSION`) individually executable for `engine/law` fluid tick recurrence (`check.md:41`, `jobs\port-backlog\report.md`).

### `local-20261114-2109-1-0-port-mining-target-switch`
- **Lens:** `port-mining-target-switch`
- **Grade:** `reproduced` (`dossier.md:1-70`, `check.md:1-42`; checker noted peripheral table wording nit on `@76-251` refusals and `@252-257` vs `253-258`, `check.md:35-36`)
- **Settled fact:** In `server-26.2.jar` (`86765a5899bd9c96461036a628796b4245715058`), on `START_DESTROY_BLOCK` (`@69-73 -> [76,487)`), `ServerPlayerGameMode#handleBlockBreakAction` overwrites `destroyProgressStart = gameTicks` first (`@252-257`), evaluates NEW pos progress (`@260-349`; insta-mine `destroyAndAck(NEW)@351-376` leaves OLD armed), checks `if (isDestroyingBlock)@379-383` with no pos-equality check, unicasts ONLY `ClientboundBlockUpdatePacket(destroyPos@398, level.getBlockState(destroyPos)@406)` to the breaker (`@386-415`, never a `-1` `ClientboundBlockDestructionPacket`), then writes `isDestroyingBlock=true@431` -> `destroyPos=pos.immutable()@439` -> `ServerLevel.destroyBlockProgress(player.getId(), NEW, state)@464` (skips breaker `@48-57`, `< 1024.0` sq dist `@99-120`) -> `lastSentState=state@481` (`dossier.md:1,15-39`, `check.md:15-24`). Deleting `@383..431` or swapping `destroyPos`/order fails `assert-target-switch.sh` (`dossier.md:42-59`, `check.md:28-31`).
- **Wiki page:** Generated in `lab\wiki\index.md` (per post-`2026-11-15 14:20` `lab-wiki.ps1` run; see Section 3).
- **What it changes:** Defines the server-side target-switch reset packet grammar and write ordering for the Rust mining port (`check.md:40`, `jobs\port-backlog\report.md`).

---

## 2. The three premise tests (`trial-food-crossing`, `trial-offset-monotone`, `port-wormhole-facts`)

### T4: `trial-food-crossing` ([`local-20261115-1224-1-0-trial-food-crossing`](file:///C:/Users/PASCAL/Desktop/path%20tracing/lab/results/local-20261115-1224-1-0-trial-food-crossing/dossier.md))
- **Confirmed:**
  - `jobs\zones-design\redteam-opus.md:16-17` crossing magnitude: flat-stone sprint-jump settles to a 12-tick cycle (`4.2761` b/cycle, `0.231` exh/cycle = `0.2f` jump + `0.1f * 31 * 0.01f` cm) and hits `foodLevel == 6` (`FoodData.hasEnoughFood` strict `foodLevel > 6` at `LocalPlayer.isSprintingPossible@29`) after **330 cycles / 3,949 ticks / ~1,407 blocks** from `(20, 5, 0)` (vs `≈320 cycles / ≈1.4k blocks` estimate), and **589 cycles / 7,057 ticks** from `(20, 20, 0)`.
  - `jobs\3.2d-3\spec.md:64-65`: Peaceful skips `foodLevel -= 1` (`FoodData.tick@57-61`), refills saturation to `20` every `20` ticks (`ServerPlayer.tickRegeneration@62-85`), and drains `exhaustionLevel > 4.0f` unconditionally (`FoodData.tick@10-28`), so Peaceful fixtures need no food-crossing inverter.
- **Refuted:**
  - `jobs\zones-design\redteam-opus.md:130` (**U2. Food is a cycle of cycles**): `(cycle phase, exhaustion bits, saturation bits)` does **not** recur cleanly before the crossing (`3,949` / `7,057` ticks). U2's exact bit-cycle memo premise fails.
- **Exact lines and replacement text:**
  - `jobs\zones-design\redteam-opus.md:16`
    - *Current:* `  - From spawn bars (20 food, 5 saturation), food reaches 6 after about 19 × 4 / 0.24 ≈ 320 cycles, so **≈1.4k blocks**.`
    - *Replacement:* `  - From spawn bars (20 food, 5 saturation, 0 exhaustion), food reaches 6 after 330 12-tick cycles (3,949 ticks, **~1,407 blocks**) at 0.231 exhaustion/cycle (589 cycles / 7,057 ticks from (20, 20, 0)).`
  - `jobs\zones-design\redteam-opus.md:130`
    - *Current:* `- **U2. Food is a cycle of cycles.** Per-phase exhaustion amounts are constants (cm rounding, \`movement.rs:1116\`), so (phase, exhaustion bits) recurs after some number of 4.0 wraps. The store keeps the super-period's debits, which is ES's cycle memo at two scales.`
    - *Replacement:* `- **U2. Food is sequential debit accumulation (T4 refuted bit recurrence).** Per-phase exhaustion uses rounded horizontal cm (\`ServerPlayer.checkMovementStatistics@294-306\`, \`movement.rs:1116\`), but \`(phase, exhaustion bits, saturation bits)\` does **not** recur before food reaches 6 (3,949 / 7,057 ticks in T4). Track per-word \`(Δexhaustion, saturation, food)\` debits monotonically instead of assuming a short periodic super-period memo.`
  - `jobs\3.2d-3\spec.md:139`
    - *Current:* `Later, after the mob port: the Normal-difficulty two-word million-block line, with its food-crossing event/U2 debit summary and actual packet re-anchor, gets its own design and gates; none is part of this revision.`
    - *Replacement:* `Later, after the mob port: the Normal-difficulty two-word million-block line, with its food-crossing event (sequential debit integration; T4 refuted U2 exact bit-recurrence) and actual packet re-anchor, gets its own design and gates; none is part of this revision.`
  - `docs\engine.md:149-152` (`FoodConstants`)
    - *Current:*
      ```markdown
      **`FoodConstants`:**
      - Sprinting needs food above 6.
      - Exhaustion: sprinting 0.1 per metre, a jump 0.05, a sprint-jump 0.2, each healed health point 6. Four exhaustion cost one food point.
      - Regeneration: one health point every 80 ticks at food ≥ 18, or every 10 ticks when saturated.
      ```
    - *Replacement:*
      ```markdown
      **`FoodConstants`:**
      - Sprinting needs food strictly above 6 (`FoodData.hasEnoughFood`: `foodLevel > 6`).
      - Exhaustion: sprinting 0.1f × rounded horizontal cm × 0.01f (0 airborne non-elytra), a jump 0.05f, a sprint-jump 0.2f, each healed health point 6; capped at 40.0f (`FoodData.addExhaustion`). `FoodData.tick` drains when `exhaustionLevel > 4.0f` strictly (on Peaceful, skips `foodLevel -= 1`; `tickRegeneration` restores saturation toward 20 every 20 ticks).
      - Regeneration: one health point every 80 ticks at food ≥ 18, or every 10 ticks when saturated.
      ```

### T7: `trial-offset-monotone` ([`local-20261115-1236-1-0-trial-offset-monotone`](file:///C:/Users/PASCAL/Desktop/path%20tracing/lab/results/local-20261115-1236-1-0-trial-offset-monotone/dossier.md))
- **Confirmed:**
  - `jobs\zones-design\redteam-opus.md:69-71,73`: Intervals of translation offset `s` are finitely many with clean **single-ulp** boundaries (`4` thresholds on stone→ice, `2` on 1-block gap), all continuous fields (`x, y, vx, vy`) are monotone within each constant-tick plateau, and axis order (`|dx| < |dz|`) / step-up caused no within-plateau violations here.
- **Refuted:**
  - `jobs\zones-design\redteam-opus.md:70`: Global monotonicity of outcome across interval boundaries is **false**. Across a 1-block gap, exit tick reverses **12 → 14 → 12** over fall-in window `s ∈ (96.336888156561680, 96.736888332719810)` (true width `0.40000017615813`), and continuous exit fields sawtooth across exit-tick breakpoints. Interval order does not imply outcome order, so bisection cannot assume a globally monotone exit map.
- **Exact lines and replacement text:**
  - `jobs\zones-design\redteam-opus.md:69-71`
    - *Current:*
      ```markdown
        - A portal is (gait, phase, an interval of the translation offset s).
        - Along one word with fixed inputs, float + and × by positive constants and the min/max clamps are monotone in s. So each law comparison flips at most once, and the intervals are finitely many.
        - Find the endpoints by float bisection (≤64 steps) and replay both.
      ```
    - *Replacement:*
      ```markdown
        - A portal is (gait, phase, a constant-outcome interval of the translation offset s).
        - T7 result: continuous exit fields are monotone **only within each constant-tick branch plateau** and sawtooth across breakpoints; globally across geometry (e.g. a 1-block gap), exit tick is **non-monotone** (reversing 12 → 14 → 12 across a 0.40-block fall-in window). Law comparisons still produce finitely many clean single-ulp plateau boundaries without ulp ripples.
        - Discover plateau/branch intervals by coarse sweep (wider than the smallest fall-in hazard) plus float bisection (≤64 steps) per comparison flip, never assuming global outcome monotonicity in s, and replay each endpoint.
      ```
  - `jobs\3.2d-3\spec.md:136`
    - *Current:* `R3 (zone tree), R5 (portals/field/segments) and R6 (cones) are LATER, OWN DESIGN DOORS: no implementation until T5–T9 have tested loaded lead, palette presence, portal monotonicity, tie plateaus and tree size. Add their own proofs where the tests refute a premise.`
    - *Replacement:* `R3 (zone tree), R5 (portals/field/segments) and R6 (cones) are LATER, OWN DESIGN DOORS: no implementation until T5–T9 have tested loaded lead, palette presence, portal monotonicity, tie plateaus and tree size. Add their own proofs where the tests refute a premise — specifically for R5, T7 refuted global outcome monotonicity in s (12→14→12 reversal at a 1-block gap and sawtooth continuous exits), so R5 must bisect/partition per-branch constant-tick plateaus rather than assuming globally monotone portal maps.`

### T11: `port-wormhole-facts` ([`local-20261115-1239-1-0-port-wormhole-facts`](file:///C:/Users/PASCAL/Desktop/path%20tracing/lab/results/local-20261115-1239-1-0-port-wormhole-facts/dossier.md))
- **Confirmed:**
  - `jobs\zones-design\redteam-opus.md:92`: Confirmed that missing client chunks skip fluids via `EmptyLevelChunk` (returning void air) and replace gravity in `LivingEntity.travelInAir@101-159` when `level.isClientSide() && !level.hasChunkAt(getBlockPosBelowThatAffectsMyMovement())`. Refined: it sets `vy := (Y > minY) ? -0.1 : 0.0` then applies `× 0.98f` drag (`-0.098/tick` steady fall), and `LocalPlayer` itself has zero chunk references.
  - `jobs\zones-design\redteam-opus.md:111-119`: Confirmed every portal & pearl claim (both-sheet portal build `3×2` pad + `4×5` frame + `2×3` interior, `8:1` X/Z origin -> `16`/`128` `PortalForcer` search by `distSqr` tie-broken by lowest `Y`, `80`-tick wait survival / `0` `abilities.invulnerable`, `10`-tick player dimension cooldown; pearl `5.0f` `ender_pearl` damage, `20`-tick cooldown, `1.5f` throw power, `0.03` gravity, `0.99f` air / `0.8f` water drag, thrower momentum with `onGround ? 0.0 : known.y`, `±0.0172275` per-axis triangular spread on pearl's own `RandomSource`, `oldPosition()` landing, `10`-tick portal cooldown).
  - `jobs\3.2d-3\spec.md:85,140`: Confirmed `loaded(p,t)` server-driven lease semantics (`EmptyLevelChunk` returns null in `ClientChunkCache(..., FULL, false)`) and pinned R7+ T11 facts.
- **Refuted:** None — all qualitative premises in `redteam-opus.md` and `spec.md` held and were pinned to exact bytecode constants.
- **Exact lines and replacement text:**
  - `jobs\zones-design\redteam-opus.md:92`
    - *Current:* `- *Why.* In a chunk the client hasn't received, vy becomes −0.1 (\`movement.rs:444-450\`), and fluids skip (\`875\`). So \`loaded(p, t)\` is a server-driven premise. The disk's far field is not what the client holds at the tick the body arrives.`
    - *Replacement:* `- *Why.* In a chunk the client hasn't received (\`!level.hasChunkAt(getBlockPosBelowThatAffectsMyMovement())\`; \`EmptyLevelChunk\` does not count), \`LivingEntity.travelInAir@101-159\` sets \`vy := (Y > minY) ? -0.1 : 0.0\` before \`0.98f\` drag (\`-0.098\`/tick steady fall; \`movement.rs:444-450\`), and \`EmptyLevelChunk\` returns void air so fluids skip (\`875\`). So \`loaded(p, t)\` is a server-driven premise. The disk's far field is not what the client holds at the tick the body arrives.`
  - `jobs\3.2d-3\spec.md:140`
    - *Current:* `R7+ holds the separately frozen, combinatorial hard-parkour extreme, tools/edits (toll PLUS restoration debt), mobile entity tools, wiggle, and portal/pearl semantics. Portal two-sheet restoration/wait/search and pearl item/health/cooldown/spread/momentum need T11/native facts; no clean 8:1 or deterministic-exit claim. Deferred nudge rejoin/width work is not silently restored.`
    - *Replacement:* `R7+ holds the separately frozen, combinatorial hard-parkour extreme, tools/edits (toll PLUS restoration debt), mobile entity tools, wiggle, and portal/pearl semantics. T11 (\`port-wormhole-facts\`) pinned the vanilla portal contract (80t survival / 0 creative wait, 10t player cooldown, 8:1 X/Z origin -> 16/128 \`distSqr\` search, 3×2 obsidian pad + 4×5 frame) and pearl contract (1.5f power, ±0.0172275 triangular spread before scaling, thrower \`(vx, onGround ? 0 : vy, vz)\`, 0.03g, 0.99f/0.8f drag, \`oldPosition()\` land, 5 HP, 20t item cooldown, 10t portal cooldown); no clean 8:1 or deterministic-exit claim. Deferred nudge rejoin/width work is not silently restored.`
  - `docs\engine.md:163-166`
    - *Current:*
      ```markdown
      **Ender pearl:**
      - thrown at 1.5 blocks per tick;
      - gravity 0.03; drag 0.99 per tick (0.8 in water);
      - 5 damage on landing, a 20-tick cooldown, and a random spread drawn from the server's RNG.
      ```
    - *Replacement:*
      ```markdown
      **Ender pearl:**
      - thrown at `1.5f` power with per-axis triangular spread `±0.0172275` (`divergence 1.0f`) drawn on the pearl entity's own `RandomSource` *before* `1.5` scaling (`Projectile.shootFromRotation@70-110`), plus thrower momentum `(known.x, onGround ? 0.0 : known.y, known.z)` (`Projectile.getMovementToShoot@4-74`);
      - gravity `0.03`; drag `0.99f` per tick in air (`0.8f` in water); lands at `oldPosition()` (`ThrownEnderpearl.onHit@137-413`);
      - `5.0f` `ender_pearl` damage on landing, a `20`-tick item cooldown (`stacksTo(16).useCooldown(1.0f)`), and sets a `10`-tick portal cooldown.

      **Nether portal:**
      - portal wait: `80` ticks survival, `0` creative (`abilities.invulnerable`, not `isCreative()`), `0` non-players (`NetherPortalBlock.getPortalTransitionTime@0-50`); post-transit cooldown: `10` ticks for players (`Player.getDimensionChangingDelay@0-2`);
      - linking: `from.coordinateScale()/to.coordinateScale()` (`8:1`) on X/Z only (`NetherPortalBlock.getPortalDestination@64-98`), nearest existing portal within radius `16` (Nether) / `128` (Overworld) by `distSqr` (tie-broken by lowest `Y`, `PortalForcer.findClosestPortalPosition@9-21,35-107`), else builds a `3×2` obsidian pad + `4×5` frame + `2×3` interior (`createPortal@83-490`).
      ```

---

## 3. Wiki status and entries to write / amend (`lab\wiki\` format)

1. **Coverage status:** `scripts\lab-wiki.ps1` was run after `2026-11-15 14:20` and already created generated pages in `lab\wiki\<lens>.md` (listed in `lab\wiki\index.md:1-209`) covering all **41** `local-20261114-*` and `local-20261115-*` runs that have `DONE` (`reproduced` and `lead`), plus listing the 5 incomplete `local-20261115-*` runs without `DONE` (`1358-1-0-port-entity-contact`, `1359-1-0-port-collapse-schedule`, `1416-1-0-port-collapse-due-tick`, `1419-1-0-port-schedule-order-tie`, `1420-1-0-port-amethyst-state`, which are skipped from this digest and require re-run).
2. **Why `lab\wiki\` drafts are still needed below:** `scripts\lab-wiki.ps1` truncates `Claim` and prints only a truncated first line of `Discrepancies`, omitting checker-won corrections that live under `Adversarial` or multi-line bullets (verified directly in [`port-elytra-bits.md`](file:///C:/Users/PASCAL/Desktop/path%20tracing/lab/wiki/port-elytra-bits.md#L6-L7) and [`port-freeze-damage.md`](file:///C:/Users/PASCAL/Desktop/path%20tracing/lab/wiki/port-freeze-damage.md#L5-L7)). Below are clean `lab\wiki\`-formatted replacement sections (matching `scripts\lab-wiki.ps1` headings/bullets without mid-sentence truncation) for the pages where the checker contradicted or materially corrected the author's claim:

```markdown
<!-- Draft wiki amendments for checker-corrected claims (scripts/lab-wiki.ps1 format) -->
# port-elytra-bits

## local-20261114-1553-1-0-port-elytra-bits (2026-11-14)
- Grade: reproduced (vanilla lift law confirmed; UCRT Rust claim refuted adversarially)
- Claim: In Mojang 26.2 (`server-26.2.jar` sha1 `86765a5899bd9c96461036a628796b4245715058`), `LivingEntity.updateFallFlyingMovement` computes lift as `Mth.square(Math.cos((double)(getXRot() * 0.017453292f)))` (JVM double cosine, identical across C2 and `-Xint`, while `Mth.sin` boost and `calculateViewVector` use the float `SIN` table). Against 4,424 adversarial rows (`check/adv.csv`), using JVM `dcos` matches 4,424/4,424, whereas rustc/MSVC `f64::cos` (UCRT) mismatches JVM `Math.cos` on 86/4,424 rows (306 ulps apart at ±90°, `be6777a5cf72cffe` vs `be6777a5cf72cecc`). Substituting `Mth.cos` or `f64` π/180 fails 4,396/4,424 and 4,388/4,424 rows.
- Falsifier: Any row in `check/adv.csv` failing JVM `dcos` transcription, or `Mth.cos` substitution passing on `42b40000`.
- Checker discrepancy: Author's 3,498-case claim that UCRT `f64::cos` is bit-exact fails on 86/4,424 adversarial rows (`check.md:6-13,65-73,86-100`); also row 1029 diff is 31 ulps (not ~7) and 541 collapsed inputs = 265/262 unique (`check.md:90-93`).
- Run folder: `local-20261114-1553-1-0-port-elytra-bits`
```

```markdown
# port-freeze-damage

## local-20261114-1633-1-0-port-freeze-damage (2026-11-14)
- Grade: reproduced (one parenthetical sub-sentence falsified by checker)
- Claim: In Mojang 26.2 (`server-26.2.jar` sha1 `86765a5899bd9c96461036a628796b4245715058`), freeze damage is server-only inside `LivingEntity.aiStep@666-779` (after `travel@617`, before `pushEntities@817`): `tickCount % 40 == 0 && isFullyFrozen() && canFreeze()` triggers `hurtServer(level, damageSources().freeze(), 1.0f)`, setting `hurtTime=10`, `hurtDuration=10`, `invulnerableTime=20`, `lastHurt=1.0`, `health -= 1.0` (bypassing armor, eating absorption first), `hurtMarked=true`, zero velocity delta, and zero difficulty scaling (1.0 across Peaceful–Hard). `tryAddFrost` applies transient `-0.05f * min(tf,140)/140` on `movement_speed`.
- Falsifier: Any hit off the 40-tick period, any `Motion`/`Pos`/`OnGround` change across freeze hits, or amount != `1.0f`.
- Checker discrepancy: Dossier's sub-sentence claiming `tf >= 140` survives up to 20 ticks outside snow is falsified (`check.md:69-76`): `Entity.baseTick@75-85` clears `isInPowderSnow` and decay (`-2`) runs at `@697-722` before the damage gate at `@733-768`, so `140 -> 138` immediately outside snow (`0` ticks survive). Minor offset nits: `DamageSources@250-256`, `PowderSnowBlock@27-39`, `Player.hurtServer@51-107`.
- Run folder: `local-20261114-1633-1-0-port-freeze-damage`
```

```markdown
# port-glide-cosine-exact

## local-20261114-1300-1-0-port-glide-cosine-exact (2026-11-14)
- Grade: reproduced
- Claim: On MS OpenJDK 25.0.1 (AMD Zen 4), `LivingEntity.updateFallFlyingMovement@52` `Math.cos((double)(xRot * 0.017453292f))` invokes HotSpot's `UseLibmIntrinsic` stub (`-Xint` byte-identical; `-XX:-UseLibmIntrinsic` collapses 100% onto `StrictMath.cos`), returning `0x3fe6e7ab883308cb` (+1 ulp vs `StrictMath.cos`/CR `0x3fe6e7ab883308ca`) on pitch `0x42312bde`, disagreeing with `StrictMath.cos` on 1,487/65,536 seeded angles (`max_ulp=1`, `0` sign flips, exact ± symmetry; exhaustive legal-pitch census confirms `max_ulp=1` and `0.062%` bit-uniform rate across 2,238,185,474 legal floats), while Rust/UCRT `f64::cos` diverges on 84/65,536 seeded rows up to 10 result-ulps near 90°.
- Falsifier: `reproduce.sh` mismatch on sha1, callsite bytecode, failure bits, or 1,487/0/84 counts.
- Checker discrepancy: Prose only — actual `cos(0x3ff921fa00000000) = 1.267590795056392e-06` (not `~6e-6`), and total legal pitch floats = `2,238,185,474` (not `~2·2^24`), making the bit-uniform rate `0.062%` (`check.md:56-64`).
- Run folder: `local-20261114-1300-1-0-port-glide-cosine-exact`
```

```markdown
# port-elytra-cosine-domain

## local-20261114-2027-1-0-port-elytra-cosine-domain (2026-11-14)
- Grade: reproduced
- Claim: Over 1M seeded legal pitch floats pushed through `(double)(pitch * 0.017453292f)` on MS OpenJDK 25.0.1 (Win11 x64), default `Math.cos` (`-Xint` identical) differs from `StrictMath.cos` on 623/1M (`0.0623%`, all 1 ulp, `[32,64)°` density `4.098%`; checker exhaustive sweep `[0,1) U [4,90]` confirms `max_ulp=1`, `[32,64)° = 4.2445%`), `DisableIntrinsic=_dcos` gives 0 (`hash 31de578536edde88`), `cos(-x)==cos(x)` holds bitwise everywhere, and Rust `f64::cos` (`msvcrt.dll -> UCRT`) differs from default JVM on 67/1M (max 19 ulp sample; **max 306 ulp** in checker's exhaustive `[64,90]` sweep) and 660/1M vs `StrictMath`.
- Falsifier: `reproduce.sh` `DIVERGED`, or anchor `42312bde` failing `3fe6e7ab883308cb` vs `...ca`.
- Checker discrepancy: Limits legal float count is `2,238,185,474` (`1M ≈ 0.045%`, not `~1.3e9`/`0.08%`), and Rust links `msvcrt.dll -> UCRT` (`check.md:75-79,83-85`).
- Run folder: `local-20261114-2027-1-0-port-elytra-cosine-domain`
```

```markdown
# port-push-order-noncommuting

## local-20261115-1340-1-0-port-push-order-noncommuting (2026-11-15)
- Grade: lead (downgraded from author's claim for nonzero initial velocity)
- Claim: `Entity.push(Entity)` (`@0-169`) commits each impulse immediately via `push(DDD)` (`@5203-5242`). While author showed `v0 == 0` 2-term/same-order batching commutes (failing at `>=3` terms `9,786/59,520` and 3-entity mutual loops `288/992`), checker proved that at real `LivingEntity.aiStep` (`travel@617 -> pushEntities@817`) where `v0 != 0`, **both** 2-term order commutativity (`264/1,260`) and same-order batching (`297/1,260`) fail by 1 ulp — so a bit-exact port must apply each per-pair push immediately in vanilla loop order.
- Falsifier: Finding commutative 2-term batching under nonzero `v0` (refuted by `check.md:11-29`).
- Checker discrepancy: Downgraded to `lead`: author's `v0 == 0` assumption omits normal post-`travel` nonzero `v0`, falsifying batch commutativity in vanilla ticks (`check.md:11-29`).
- Run folder: `local-20261115-1340-1-0-port-push-order-noncommuting`
```
