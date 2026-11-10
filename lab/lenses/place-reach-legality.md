# Lens: when the server accepts a block placement (jar reader)

**Why it matters.** The first tool the bot will carry is a block. The wiki has pinned where a placed block goes (`lab/wiki/port-place-replaceability.md`). The next rule is whether the server accepts the click at all. A bot that places under its own feet in mid-air lives on these thresholds, so they must be exact.

**The question.** In the 26.2 server jar:
1. In `ServerGamePacketListenerImpl.handleUseItemOn`: the reach check (which attribute, which added buffer, measured from which point of the player to which point of the block), the bounds check on the hit vector, and the build-height check. Give each comparison with its constants as raw bits, and say whether it is inclusive.
2. In `BlockItem.canPlace` and what it calls: which entities obstruct a placement, with which shape test. Does the placing player's own body block a placement into the cell at its feet, the cell below its feet, and a cell its box only touches?

**What to produce.**
- `reproduce.sh`: regenerates the bytecode evidence and runs the witnesses.
- A reach witness: three adjacent representable distances around the threshold with the accept or reject outcome of each, and a mutant without the buffer that the witness rejects.
- An obstruction witness: the three cells of question 2, each accepted or refused.
- `dossier.md`: the claim in one sentence, the evidence type of each part (direct jar execution, bytecode, reconstructed arithmetic), and what was left out.

**Deliver the first proof early.** Write `dossier.md` with `Claim: ` on its first physical line and a runnable `reproduce.sh` before expanding; save the first output as soon as it exists. Use `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and the JDK in `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin`. Write only in your run folder. No network, no live game, no production edits, no commits.

**Out of scope:** item consumption, the block state chosen, neighbour updates, and the client's own pick ray.
