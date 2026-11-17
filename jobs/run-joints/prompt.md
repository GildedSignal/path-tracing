# His recorded runs, broken into joints: a bar per joint, not only per track

**Consumer:** step 3 of `docs\plan.md`, the loop on real tracks. When the bot is slower than Julien, the comparison must say **where**. That needs his runs cut at their joints, which are the same objects the planner uses (`docs\representation.md` sections 10–11). On the Diversity practice tracks (neo jumps over a floor) the bot must match or beat his T (S189), so the neo joint itself must be identified.

You read and run small read-only analysis scripts (PowerShell or Git Bash over the recordings' JSON). You write only your report and its tables. Root `C:\Users\PASCAL\Desktop\path tracing\`.

## Read
- `docs\representation.md` sections 10–11: what a joint, a word and a set are.
- `docs\vocabulary.md`: the law's events (landing, takeoff, wall contact, climb, ceiling).
- `jobs\flight-corpus\report.md`: his tracks and which have recordings.
- **The run index:** `lab\tracks\runs\*.jsonl` (each run's T and recording path), and the recordings themselves: `recordings\track-*.jsonl` under the repo root (ignored by git). Each row holds per-tick inputs, position, velocity, `onGround`, `isSprinting`, collisions and pose. `jobs\hard-track\julien-428.jsonl` is one already in the repo.
- `lab\tracks\*.json`: the tracks' A and B (and `bFace`).

## Write `jobs\run-joints\report.md`
1. **The cutting rule**, stated once from the recording fields, with no number of ours. A joint is a tick where a law event changes: takeoff (`onGround` true→false), landing (false→true), a horizontal-collision change, a climb starting or ending, a pose change, a sprint change, or a key change that starts a new held word.
2. **For each of his best runs on each track** (start with the first-flight set in `flight-corpus`, then the Diversity practices, the Infinity Tower segments and the old 21): the sequence of joints, each with its tick, position, velocity, what happened (event), and the ticks since the previous joint. Write each track's table to `jobs\run-joints\tables\<track>.tsv`.
3. **The neo jumps:** on each Diversity practice track, which joint is the neo (the jump around a wall edge), and the ticks it saves against the floor route. If the floor route's time can't be derived from his runs, say so.
4. **Where his runs are not perfect** (S189: "not all my scores are perfect"): when he ran a track several times, the joints where his runs differ most in ticks. That is where the bot can beat him.
5. **Gaps:** recordings missing, unreadable, or with a `recordingNote`.

Cite the file and row for every number. Never guess.
