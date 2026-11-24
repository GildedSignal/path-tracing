# Vibe live verification: executed boats, living mounts, and explosion exposure

**Goal:** Close the `Limits` of `vibe-gen-mounts-kicks-rows` by executing `AbstractBoat` / `Raft`, `Horse`, `Camel`, `Strider`, `Pig`, and `ServerExplosion.getSeenPercent` LIVE on Unsafe `ServerLevel` + real `LevelChunk` + bound jar tags against `server-26.2.jar`.

Turns `vibe-gen-mounts-kicks-rows`'s bytecode-cited mount and explosion rows (`C:/Users/PASCAL/Desktop/path tracing/lab/results/local-20261124-132644-2896-1-0-vibe-gen-mounts-kicks-rows/`) into executed hex-exact trajectories (`tick()` / `ServerExplosion`).

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`.

**Harness requirements (mandatory executed trajectories):**
- Boot jar tags (`MappedRegistry.prepareTagReload`) + `DataComponentInitializers` (`1537/1537` item holders) + Unsafe `ServerLevel` + real `LevelChunk` + `PersistentEntitySectionManager` (see `C:/Users/PASCAL/Desktop/path tracing/lab/results/local-20261124-135031-30224-1-0-vibe-check-rails-live/`).
- Write `dossier.md` (`Claim: ...`), `rows.tsv`, and `witnesses.tsv` early once first live scenes pass, then extend cleanly.

**Live scenes to execute:**
1. **`Boat` & `Raft` (`boat.tick()`, `setInput`, `getStatus`):** on water (`IN_WATER`), submerged (`UNDER_WATER` / `UNDER_FLOWING_WATER`), on stone vs blue ice (`getGroundFriction`), and falling off ledge (`IN_AIR`), recording paddle `deltaRotation`, yaw, position, and velocity hex every tick.
2. **Saddled mounts (`ServerPlayer` passenger + input/dash/jump):** `Horse` ground trot + charged jump (charge `45` vs `90`), `Camel` dash (`dashCooldown = 55`), `Strider` on lava level `0` vs cold stone (`suffocating`), `Pig` carrot-on-stick boost start.
3. **`ServerExplosion.hurtEntities` / `getSeenPercent`:** TNT explosion next to full block vs half-slab vs open floor verifying exact `seen` fraction, `(1 - d) * seen * kbMult`, and resulting `ServerPlayer` velocity bits.

**Write in your run folder:**
- `rows.tsv`: `rule_id	scene	expected_vs_executed	constants_hex	scars	source`
- `witnesses.tsv`: `case_id	tick	entity	x_hex	y_hex	z_hex	vx_hex	vy_hex	vz_hex	flags	events`
- `dossier.md` (first line `Claim: `), `reproduce.sh`, and `wiki.md`.

**Falsifier:** Any executed trajectory hex differing from `vibe-gen-mounts-kicks-rows` or failing bit-identical `reproduce.sh` re-run.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
