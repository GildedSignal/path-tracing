# Lens: a player moving into a chunk that isn't live (jar reader)

**Why it matters.** `docs/server.md` plans bots that hold chunk tickets only under their path, at the lowest level that keeps behaviour vanilla. The kill line is a bot reaching a chunk before it is ready. This lens says exactly what "ready" must mean.

**The question.** In the 26.2 server jar, a `ServerPlayer`'s movement packet takes it into a chunk that is (a) not loaded, (b) loaded but below block-ticking, (c) block-ticking but not entity-ticking. For each of the three:
- do the collision queries behind `ServerGamePacketListenerImpl.handleMovePlayer` and `Entity.move` see that chunk's blocks, or treat it as empty? Name the method that decides and its offset;
- is the move accepted, corrected or ignored?
- does the player's own tick still run there (`ServerPlayer.doTick`, fall damage, fluid and `checkInsideBlocks`, `pushEntities`, item pickup)? Which part of it is gated by the chunk's level, and where?
- do a scheduled block tick, a fluid tick and a dropped item's pickup delay advance there?

**What to produce.**
- `reproduce.sh`: regenerates the bytecode evidence; a direct runtime witness for (a) is better if a bounded one exists without starting a server.
- `table.tsv`: one line per (chunk state, question): the answer, the evidence type, and class, method and offset.
- `dossier.md`: the claim in one sentence (the lowest chunk level at which a moving player behaves as in vanilla, and what breaks below it), and what the reading could have missed.

**Deliver the first proof early.** Write `dossier.md` with `Claim: ` on its first physical line and a runnable `reproduce.sh` before expanding; save the first output as soon as it exists. Use `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and the JDK in `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin`. Say which evidence is bytecode, reconstructed logic or a direct run. Write only in your run folder. No network, no live game, no production edits, no commits.
