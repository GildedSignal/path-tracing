# Lens: opening a door, a trapdoor, and stepping on a pressure plate (jar reader)

**Why it matters.** Four of Julien's 18 tracks need block interaction and the bot has no word for it (HANDOFF, the planner's failure list). A door is a premise with a producer: a word writes the `open` bit that another word's tube reads. By arrival the state must be as it was (S32), so the undo is part of the word.

**The question.** In the 26.2 server jar:
1. `ServerGamePacketListenerImpl.handleUseItemOn` to `DoorBlock.useWithoutItem`, `TrapDoorBlock.useWithoutItem`, `FenceGateBlock.useWithoutItem`: every gate before the state flips (reach is already pinned in `lab/wiki/place-reach-legality.md`; sneaking with an item in hand; iron variants; the block set type's `canOpenByHand`), each comparison with its constants, and exactly which block states are written, on which cells (both halves of a door), with which update flag.
2. In which phase of the server tick the write lands against the player's move, and whether the client predicts the flip for the clicking player (on which client tick its collision shape changes).
3. `BasePressurePlateBlock.entityInside` and `checkPressed`: which entities and which box test press it, the scheduled tick that releases it and its delay, and the neighbour update that opens an adjacent door.
4. The collision shapes of the open and closed states, as the block table already exports them: confirm or contradict.

**What to produce.**
- `reproduce.sh`: regenerates the bytecode evidence and runs the witnesses.
- A door witness by direct jar execution: click, the states written on both halves, click again, the original states back bit for bit.
- A pressure-plate witness: the tick it presses, the tick it releases after the body leaves, the door's state on each.
- `dossier.md`: the claim in one sentence, the evidence type of each part (direct jar execution, bytecode, reconstructed arithmetic), and what was left out.

**Deliver the first proof early.** Write `dossier.md` with `Claim: ` on its first physical line and a runnable `reproduce.sh` before expanding; save the first output as soon as it exists. Use `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and the JDK in `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin`. Write only in your run folder. No network, no live game, no production edits, no commits.

**Out of scope:** redstone beyond the plate's direct neighbour update, villagers and mobs opening doors, sounds and game events.
