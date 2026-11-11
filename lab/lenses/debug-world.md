# Lens: the debug world type as a one-world check of every block state (jar reader)

**Why it matters.** Julien proposed (S67) using Minecraft's debug world type, which lays out every block state in one grid, to test the block table against the real game. Our table models 30,951 of 32,366 states, and one error was found only by a lab lane (scaffolding's bottom cap). One world holding every state would let a bot touch each of them on a real server. Before anything is built, the jar must say whether that is possible.

**The question.** In the 26.2 server jar (`DebugLevelSource` and whatever reads the debug flag):
1. The layout: the formula from a block state's index to its position, the spacing, the heights (the states' layer and the floor under it), and how the index order is defined. Is it deterministic across runs of the same jar?
2. What the debug world disables or changes: block updates, neighbour updates, random and scheduled ticks, fluid ticks, entity spawning, block placing and breaking, game mode, damage. Each with the bytecode site that reads the debug flag.
3. Does a survival-mode player collide with the laid-out blocks there exactly as in a normal world (is `getCollisionShape` reached with the same context)? Can a dedicated server be started on a debug world, and how (level type key, server.properties or world preset)?
4. Which states would behave differently there than in a normal world because of 2 (for instance fluids that never flow, or scaffolding with a distance that could not exist), so that a check there would mislead?

**What to produce.**
- `reproduce.sh`: the bytecode evidence and, by running the jar's own classes without a world if possible, the first and last twenty (state, position) pairs and the total count.
- `dossier.md`: the claim in one sentence (usable as a block-table check, with which limits; or not, and why), the evidence type of each part, what was left out.

**Deliver the first proof early.** Write `dossier.md` with `Claim: ` on its first physical line and a runnable `reproduce.sh` before expanding; save the first output as soon as it exists. The jar is `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`, the JDK is in `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin`. Write only in your run folder. No network, no live game, no production edits, no commits.

**Out of scope:** building the check, client rendering, any live game.
