# Generator rows: block placement, breaking, and drop scatter

**Goal:** One parameter/scar table covering 26.2 block placement legality, tool breaking ticks, and dropped-item spawn/pickup rules for the shared `cell`, `clock`, and `body` generators, backed by bit-exact jar witnesses.

World edits (bridging, pillaring, mining through walls) enter `solve` only as law inputs and obligations (`R-lego`: reversible create-and-restore where required; `map/now.md`, step 3). Every scar and irregularity is a table row with its bytecode method and offset (`javap`).

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`.

**From the bytecode** (class, method, offsets):
- **Placement (`BlockItem.useOn`, `BlockPlaceContext`, `Player.blockInteractionRange`):** eye reach cap (`4.5` default / `5.5` attribute max vs AABB nearest point), clicked face + replaceable blocks (`canBeReplaced`), player AABB self-block exclusion (`canSurvive` / collision check on target cell: feet cell blocked while grounded/inside, below-feet cell legal once airborne), waterlogging / snow-layer merge edge cases.
- **Breaking (`MultiPlayerGameMode`, `ServerPlayerGameMode`, `BlockBehaviour.getDestroyProgress`):** base destroy speed by tool material/tier + `requiresCorrectToolForDrops`, `EFFICIENCY` enchantment bonus formula, `HASTE` / `MINING_FATIGUE`, `!onGround` (`0.2x`), `!isEyeInFluid(WATER)` / `AQUA_AFFINITY` (`0.2x`), exact break completion threshold (`progress >= 1.0f`) and tick count.
- **Drop spawning & pickup (`Block.popResource`, `ItemEntity`):** spawn position offset `(x + 0.5 + rand, ...)`, initial scatter velocity RNG formula, `pickupDelay = 10` ticks (`DEFAULT_PICKUP_DELAY`), `playerTouch` pickup AABB expansion (`inflate(1, 0.5, 1)`), and inventory stack merge order.

**Witnesses:** Run a tiny Java harness against the 26.2 jar covering:
1. Placement checks (valid neighbour face, airborne below-feet, blocked by own AABB, out-of-reach > 4.5, replaceable tallgrass/snow).
2. Break sequences (stone / oak planks / dirt / obsidian / iron block across bare hand, iron pickaxe, diamond pickaxe + Efficiency V, standing vs airborne vs submerged without Aqua Affinity).
3. `ItemEntity` spawn + seeded scatter velocity bits + tick-by-tick `pickupDelay` countdown and pickup tick.

**Write in your run folder:**
- `rows.tsv`: `rule_id	kind	target	condition_flags	params_hex	scars	source` (`source` = `class.method@offset`).
- `witnesses.tsv`: `case_id	tick	outcome	x_hex	y_hex	z_hex	vx_hex	vy_hex	vz_hex	progress_hex	pickup_delay	inv_count` (all floats/doubles as hex bits).
- `dossier.md` (first line `Claim: `), `reproduce.sh` regenerating both tables from the jar, and `wiki.md`: a compact `lab/wiki/` reference summarizing placement reach, breaking modifiers, and irreversible-drop / void bounds.

**Falsifier:** Any witness case or tick where the table-driven rules give a different placement verdict, break tick, or item scatter/pickup bit state than the jar.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
