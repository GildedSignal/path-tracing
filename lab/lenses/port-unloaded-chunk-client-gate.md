# Port delta: LivingEntity.travelInAir unloaded-chunk client-only gate

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`, the client jar at `D:/path-tracing-deps/minecraft-26.2-client.jar`, and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`. `unloaded-chunk-move` and `port-wormhole-facts` noted the unloaded `vy = -0.1` branch; this lens isolates the exact `isClientSide` / chunk-loaded predicate vs `engine/law/src/movement.rs:444-450`.

**Consumer:** `engine/law/src/movement.rs:444-450`, `jobs/port-backlog/report.md:175-176` Conflict #8.

**From the 26.2 client and server bytecode**, state with class, method and offsets:
- `LivingEntity.travelInAir` (to locate): exact offsets of the `!level().hasChunkAt(...)` / `level().isClientSide()` check around `vy = pos.y > minY ? -0.1 : 0.0` (whether the server EVER runs `-0.1` or always runs `vy - gravity` even in an unloaded chunk, and whether `hasChunkAt` checks `below` `getBlockPosBelowThatAffectsMyMovement()` or `blockPosition()`);
- `ClientLevel.hasChunkAt` vs `ServerLevel.hasChunkAt` / `Level.isLoaded` (to locate): exact chunk ticket / section condition that returns `false` on the client vs server.

**Build a jar-backed reproduction:** run `reproduce.sh` disassembling `LivingEntity.travelInAir` (`@80-170`) in both client and server jars (confirming they are identical or showing any difference) and `Level.isLoaded` / `hasChunkAt`, plus a direct comparison showing `server` (`vy - gravity`) vs `client` (`-0.1` / `0.0`) on an unloaded chunk column. Write `dossier.md` starting with `Claim: ` on its first physical line.

**Falsifier:** any `isClientSide()` gate, queried `BlockPos`, or `vy` branch in `travelInAir` that disagrees with the 26.2 bytecode.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
