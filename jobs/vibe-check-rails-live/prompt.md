# Vibe live verification: executed Minecart trajectories on OldMinecartBehavior & rails

**Goal:** Close the `No per-tick trajectory was executed` limit of `vibe-gen-rails-minecart-rows` by driving a real `Minecart` on Unsafe `ServerLevel` + real `LevelChunk` + bound jar tags (`#rails`, `#prevent_mob_spawning_inside`) across flat, curved, ascending/descending, powered boost/brake, detector rail, water rail, and off-rail tracks, bit-comparing every tick against `server-26.2.jar`.

Turns `vibe-gen-rails-minecart-rows`'s `ALGO_*` bytecode reads into executed hex-exact trajectories (`C:/Users/PASCAL/Desktop/path tracing/lab/results/local-20261124-131408-25860-1-0-vibe-gen-rails-minecart-rows/`). Do NOT trust any `ALGO_*` claim without running real `minecart.tick()` in `ServerLevel`.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`.

**Harness requirements (mandatory executed trajectories):**
- Boot `MappedRegistry.prepareTagReload` (binding `#minecraft:rails` and all block tags) + Unsafe `ServerLevel` + real `LevelChunk` (see `C:/Users/PASCAL/Desktop/path tracing/lab/results/local-20261124-123433-25752-1-0-vibe-check-entity-contact/` and `...-130147-...-vibe-check-block-place-break/`).
- Construct real `Minecart` + `ServerPlayer` (for ridden vs empty slowdown `0.997` vs `0.96`) and step real `minecart.tick()` and `level.tick()` (`LevelTicks<Block>` for `DetectorRailBlock`).
- Write `dossier.md` (`Claim: ...`), `rows.tsv`, and `witnesses.tsv` early after first trajectories pass, then extend cleanly.

**Live scenes to execute (recording pre/post `x,y,z,vx,vy,vz` hex bits every tick):**
1. Straight `NORTH_SOUTH` / `EAST_WEST` ridden vs empty coasting (`0.997` vs `0.96`).
2. All 4 curved corners (`SOUTH_EAST`, `SOUTH_WEST`, `NORTH_WEST`, `NORTH_EAST`) verifying exit polyline snap & direction flip.
3. Ascending & descending slopes (`ASCENDING_EAST/WEST/NORTH/SOUTH`) uphill deceleration vs downhill `0.0078125` accel + `exit.y` snap.
4. `PoweredRailBlock` powered boost (`+0.06`, standstill `±0.02` against stone wall) vs unpowered brake (`<0.03 -> 0` else `×0.5`) vs `DetectorRailBlock` `POWERED=true/false` `20`-tick schedule.
5. Waterlogged rail (`0.2` speed cap, `×0.95f` drag) + running off rail end (`comeOffTrack` `×0.5` ground friction, `0.95f` air drag).

**Write in your run folder:**
- `rows.tsv`: `rule_id	scene	expected_vs_executed	constants_hex	scars	source`
- `witnesses.tsv`: `case_id	tick	x_hex	y_hex	z_hex	vx_hex	vy_hex	vz_hex	rail_shape	events`
- `dossier.md` (first line `Claim: `), `reproduce.sh`, and `wiki.md`.

**Falsifier:** Any executed `minecart.tick()` hex output that differs from `vibe-gen-rails-minecart-rows`'s table or fails to match `reproduce.sh` bit-for-bit.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
