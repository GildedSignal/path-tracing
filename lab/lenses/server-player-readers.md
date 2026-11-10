# Lens: who reads the players to make the world live (jar reader)

**Why it matters.** `docs/server.md` fixes the boundary between the server and the engine, and it is a one-way door. A bot's future is known, so it should hold chunks only under its path, and the live bubble a player carries (ticking, spawning) should be something the swarm switches on. That design needs the complete list of server code that makes the world live because a player is somewhere. A site missed here is a bug later.

**The question.** In the 26.2 server jar, list every site that:
- adds, moves or removes a player's chunk ticket, or feeds a tracker of player distance (`DistanceManager`, `ChunkMap`, the natural-spawn chunk counter, the simulation-distance tracker);
- reads the level's or the server's player list, or asks for the nearest player, to decide spawning, despawning, sleeping, random ticks, entity ticks, raids or sieges (`NaturalSpawner`, each `CustomSpawner`, `Mob.checkDespawn`, `SleepStatus`, `ServerChunkCache`, `ServerLevel`, the village and raid code).

**What to produce.**
- `reproduce.sh`: scans every class in the jar with `javap -c -p` for references to the accessor methods you identify (name them), and prints `catalogue.tsv`.
- `catalogue.tsv`: one line per site: class, method, offset, what it reads (ticket tracker, player list, nearest-player query), and what that read decides, in a few words.
- `dossier.md`: the claim in one sentence (how many sites per family), and why a site can't be missed (the scan covers every caller of the listed accessors).

**Deliver the first proof early.** Write `dossier.md` with `Claim: ` on its first physical line and a runnable `reproduce.sh` before expanding; save the first output as soon as it exists. Use `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and the JDK in `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin`. Say which evidence is bytecode and which is a direct run. Write only in your run folder. No network, no live game, no production edits, no commits.

**Out of scope:** designing the mod, and what each site does beyond its one line. Don't paste method bodies; cite offsets.
