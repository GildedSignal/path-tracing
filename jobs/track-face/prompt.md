# The clicked face is the target

**Julien (S170, verbatim):** "modify the mod so that the face that is selected with the stick is the one that is targetted instead of all faces of the block and make it clear in the feedback otherwise it'll create issues of "cheating""

**Consumer:** his races on the downloaded maps, and the bot racing the same tracks; the planner's goal must be the same face.

**Today:**
- `bot\src\pathtracing\TrackDetector.java:5-9` finishes on any face. The swept box intersects B's block inflated by `1e-3`, which is an epsilon.
- `engine\plan\src\goal.rs:19` (`touched`) is the planner's copy of the goal. Goal and touch are defined twice (zones-design D8).
- `Pointer.java` sees the clicked face (`block.getDirection()`), but `Tracks.click` (`Tracks.java:213`) stores only the block.

## Decisions already made (do not reopen them)

1. **Track file:** B gains `"bFace"`, one of `up`, `down`, `north`, `south`, `east`, `west` (Minecraft's `Direction` names), taken from the stick's hit.
   - **A must be clicked on its top face** (A is where you stand). Clicking another face of A refuses, with: "A is where you stand: click its top face."
   - **Old tracks** (no `bFace`) keep finishing on any face. Mark them "any face (set before S170)" in `/track list` and in the arming message, so he can re-create them. Their runs stay comparable to each other.
2. **One touch rule, written once in each language with the same test vectors,** with no epsilon. Let `swept` be the box from the previous tick's box to this tick's (as today), and let the face plane be at coordinate `p` on axis `n` (for example `up`: the y axis, `p = y + 1`). The face is touched when both hold:
   - on axis `n`: `swept.min[n] <= p <= swept.max[n]` (closed);
   - on each of the two tangent axes: `swept.min < face.max` and `swept.max > face.min` (open, so the overlap has positive area). An edge or a corner alone never counts, so a corner touch can't finish on two faces.

   Remove the `inflate(1e-3)`.
   - For a track without `bFace`, "any face" means any of the six under this same rule.
   - Put the test vectors in one file, read by a Java test and a Rust test: `bot\tests\fixtures\face-touch.txt`, one case per line (`previous box`, `current box`, `cell`, `face`, `expected`).
   - The cases must include: standing on the top face; a side hit; a corner-only contact (false); an edge-only contact (false); passing the plane between ticks (true); touching the wrong face (false); the old any-face rule.
3. **The planner uses the same rule:** `goal.rs` reads `bFace` and implements rule 2 exactly. `gap` measures the distance to the face's square, not to the block. Every track reader passes `bFace` through: `plan-track`, `flight.rs`, `tracks.txt` (add the face as a seventh field; old lines without it mean any face).
4. **Feedback that leaves no doubt:**
   - **On clicking B:** "B: `<block>` at x y z, `<face>` face. Only touching this face finishes."
   - **On arming a track:** the chat names B's face.
   - **In track mode, and while creating a track:** the target face's outline is drawn every 10 client ticks with client-side particles. These are particles only the player's client renders, so they are not a world edit, and the bot sees no difference. Use a vanilla particle type such as `minecraft:end_rod` or `minecraft:happy_villager`.
   - **On finishing:** the chat says "Touched B's `<face>` face at tick T."
5. **The bot's start from A** (`Pointer.java`, a stick click on A) is unchanged. `/msg PathBot track run <name>` reads `bFace` like everything else.

## Must not touch

`engine\law\`, `engine\words\`, `engine\native\`, `engine\world\src\`, any game jar, his saved runs (`lab\tracks\runs\`). His existing track files may gain no field unless he re-creates them. Launch nothing: no game, server, client or RCON. You are the Terra builder: never run `scripts\launch.ps1`, `gpt.ps1` or `follow-up.ps1`.

## Done means

- `bot\build.ps1` builds.
- The Java face-touch test and the Rust face-touch test pass on the shared vectors.
- `cargo test --offline --workspace` passes in `engine\`, with `CARGO_TARGET_DIR=D:\path-tracing-target\track-face`.
- `bot\tests\test-tracks.ps1` passes, if it exists.
- `jobs\track-face\last.md` lists the files changed and what Julien should try in game: click B's side face, touch the top (no finish), then touch the side (finish).

End with `STATUS: DONE` or `STATUS: BLOCKED: <why>`.
