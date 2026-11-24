# Vibe live verification: Block.popResource, ItemEntity physics/pickup, and Player.drop throw velocity

**Goal:** Execute 26.2 `Block.popResource`, `ItemEntity.tick` (`spawn_velocity(rng)`, gravity `-0.04`, drag `0.98`, 4-tick rest cadence, `10`-tick pickup delay, `6000`-tick despawn, `2.6x2.8x2.6` player pickup box, `0.5` merge radius, lava/fire/cactus/void destruction), and `Player.drop` (`drop_velocity_aimed`) live on `server-26.2.jar`.

Supplies bit-exact ground truth for deterministic `ItemEntity` drop catchability over void (`C7` in `kit-edit-ledger-redteam`) and mid-sprint swarm tool handoffs without hardcoded terrain rules. See `lab/results/local-20261124-130147-11356-1-0-vibe-check-block-place-break/` for clean `DataComponentInitializers` / tag boot.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`.

**Harness requirements (mandatory executed witnesses):**
- Boot jar tags (`MappedRegistry.prepareTagReload`) + `DataComponentInitializers` + Unsafe `ServerLevel` + real `LevelChunk` + real `ItemEntity` & `ServerPlayer`/`Player` instances (with fixed/seeded `RandomSource` / `JavaRandom` inputs so RNG draws are exact).
- Inspect bytecode with `javap -c -p -constants` for `Block.popResource`, `ItemEntity.tick`, `ItemEntity.playerTouch`, `ItemEntity.mergeWithNeighbours`, `ItemEntity.hurtServer`, and `Player.drop`, then execute live on `server-26.2.jar`.
- Write `dossier.md` (`Claim: ...`), `rows.tsv`, and `witnesses.tsv` early after core spawn/tick/pickup witnesses pass, then extend cleanly.

**Live scenes to execute:**
1. **`Block.popResource` & spawn cone:** trace exact spawn center, horizontal `(rand*0.2 - 0.1)`, vertical `+0.2`, default `pickupDelay = 10`, plus `spawnAtLocation` RNG draw order.
2. **Free fall, water float, lava/cactus/void loss, and rest cadence:** execute 15-tick free fall from `BlockPos(0, 64, 0)` recording raw `u64` IEEE bits for `(x, y, z, vx, vy, vz)` per tick (verifying `y` at tick 10 vs block center, gravity `-0.04`, `0.98` drag, ground friction drag, and tick % 4 rest check), plus water buoyancy, lava `0`-tick burn vs non-fireproof, and void removal below `minY - 64`.
3. **Player pickup box (`playerTouch`):** prove live player AABB inflate `(1.0, 0.5, 1.0)` (= `2.6 x 2.8 x 2.6` around standing `0.6 x 1.8 x 0.6`), `pickupDelay` transition `10 -> 9 -> ... -> 0` (blocked at tick 9, succeeds at tick 10), and adjacent-to-void break where falling block item IS caught vs 4.5-block-far break where it drops out of reach.
4. **`Player.drop` aimed throw (`drop(ItemStack, boolean, boolean)`):** execute aimed mid-sprint throw (yaw/pitch + thrower velocity interaction, `0.3` speed + spread RNG, `40`-tick pickup delay), testing deterministic catch by a trailing/running teammate.

**Write in your run folder:**
- `rows.tsv`: `rule_id	method	gravity_or_coeff	drag_hex	delay_ticks	box_or_bounds	scars	source`
- `witnesses.tsv`: `case_id	tick	x_hex	y_hex	z_hex	vx_hex	vy_hex	vz_hex	pickup_delay	events`
- `dossier.md` (first line `Claim: `), `reproduce.sh`, and `wiki.md`.

**Falsifier:** Any executed tick where `ItemEntity` position/velocity raw double bits, pickup tick boundary, player inflate bounds, or `Player.drop` launch vector disagrees with `server-26.2.jar`.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
