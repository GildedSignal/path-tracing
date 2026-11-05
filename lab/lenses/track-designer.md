# Lens: track designer (with a witness)

**Why it matters.** Julien runs parkour tracks on the lab's superflat world. Each has a start block A and a goal block B, and T runs from leaving A to touching B (`docs/ladder.md`, the human baseline, S25 to S28). His best time is the line the bot must reach. Tracks that each isolate one hard mechanic of the movement law are the ladder's sharpest rungs. A designed track is evidence only if it is possible, so **a track counts only with a witness:** an input sequence that reaches B under the law.

**The law.** As in the `speed-of-light` lens: branch `wt/law`, crate `engine/law`, provisional until rung L0 passes. The world must be as it was once B is reached (S32); for now tracks need no edits at all, since the law has none yet; the toolkit is what Julien would use, and the bot receives exactly the items he used (S28).

**What to produce, per track (two to four tracks, each testing a different mechanic, each something a skilled human player would find hard but fair):**
- `tracks/<name>/build.txt`: the vanilla commands that build it (`fill`, `setblock`), relative to an origin `~ ~ ~`, at most 64 × 32 × 64 blocks, ending with the coordinates of A and B relative to the origin;
- `tracks/<name>/track.md`: the mechanic it tests, in two lines, and the toolkit;
- `tracks/<name>/world.jsonl`: the same track as a `BoxWorld` description (the format in `wt/law:lab/fixtures/README.md`), built by a script from `build.txt`, never by hand;
- `tracks/<name>/witness.jsonl`: per-tick inputs (the recorder's input format, `bot/recorder.md` on `main`) from standing still on A to touching B;
- `reproduce.sh`: rebuilds each `world.jsonl` from `build.txt`, replays each witness through the law, and prints, per track, whether it touched B and the ticks from leaving A to touching B.
- `dossier.md`: the claim (one sentence: N tracks, each with its mechanic and witness time), and for each track why its witness time is or isn't near the best possible.

**Out of scope:** modifying the world during the run, mob or redstone tracks, and tracks over the size limit.
