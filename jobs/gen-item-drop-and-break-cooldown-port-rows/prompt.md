# GLM 5.4 live verification: 26.2 ItemEntity pickup, drop trajectory, and client break cooldown

**Goal:** Execute live 26.2 `Block.popResource`, `ItemEntity.tick` (`10`-tick pickup delay, `-0.04` gravity, `0.98` drag, `2.6x2.8x2.6` player inflate pickup box), and `ServerPlayerGameMode` / `BlockBehaviour.getDestroyProgress` (`destroyDelay = 5` non-insta-mine vs `0` insta-mine when `>= 1.0f`) on `server-26.2.jar`.

Supplies exact witness rows needed for live `ametwoodbox` drop pickup and law compression. See `lab/results/local-20261124-130147-11356-1-0-vibe-check-block-place-break/` for clean boot.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`; java/javac: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/`.

**Live scenes to execute:**
1. `ItemEntity` drop from broken block at `BlockPos(0, 64, 0)` with seeded RNG: record `u64` IEEE bits for `(x, y, z, vx, vy, vz)` and `pickupDelay` (`10 -> ... -> 0`) on ticks `0..=15` and exact pickup tick vs player standing nearby.
2. `getDestroyProgress` & `destroyDelay` boundaries for `oak_log`, `stone`, `cherry_fence`, `cobweb`, and `slime_block` across `air`, `iron_pickaxe`, `diamond_axe` (`Efficiency V`), standing vs airborne (`0.2x`).

**Write in your run folder:**
- `rows.tsv`, `witnesses.tsv`, `dossier.md` (`Claim: ...`), `reproduce.sh`, `wiki.md`.
- Write only in your run folder; no game launch, network, or `work.patch`.
