# Job 1.6: the fixtures for L0

The recorder (jobs 1.2 to 1.2e) is merged and passed its gate live. The Rust law (job 1.5, branch `wt/law`) is waiting for its gate, rung L0 in `docs/ladder.md`: bit-identical positions and velocities, on every tick, against recordings over **every terrain class**, with many random input sequences each. This job makes those fixtures recordable with one command. You don't run the game: the integrator does, with your script.

Where something is ambiguous, take the most literal reading and write the assumption in your final message. Stop only for something that blocks the build. Do not commit; the integrator commits.

Read first: `bot/recorder.md`, `bot/src/pathtracing/` (`Recorder.java`, `BlockDumper.java`, `Bot.java`), `lab/gates/` (especially `run-gate-1.2.ps1`, whose structure you reuse), `scripts/common.ps1` and `scripts/rcon.ps1`, and the fixture format: `git show wt/law:lab/fixtures/README.md` (the replay harness that reads it is `git show wt/law:engine/law/tests/replay.rs`).

## A. The world, exported by the bot's own client

Add a bot command `export <name> <x0> <y0> <z0> <x1> <y1> <z1>`. It writes `pathtracing/fixtures/<name>.world.jsonl` (through `.tmp`, refusing to overwrite), in the README's world format, from the client's own level:
- one `block` line per position in the inclusive box whose state is not air, with its collision boxes in each context the dumper already evaluates (`boxes`, `aboveBoxes`, `belowBoxes`, `sneakBoxes`), its `insideBoxes`, friction, speed and jump factors, and the properties the README lists (climbable, fluids with height and flow, `inside`, slime, `fallOnMultiplier`, bounce, ladder and trapdoor facing...);
- a `world` line (`minY`, difficulty, natural regeneration);
- no `player` line: section C adds it.

Read every value from the game objects, as the dumper does, never from a table you write. Where the README needs a property the game doesn't expose directly (a hook such as "slime" or "honey"), derive it from the block's class (`instanceof`) and list each such derivation in `bot/recorder.md`.

## B. The courses

`lab/gates/courses.ps1` defines one course per terrain class, built with RCON `fill`/`setblock` on the lab superflat, each on its own patch far enough from the others that a 400-tick random walk never reaches a neighbour (fence or wall it in if needed, and say how). The classes are those of L0: full blocks; slabs and stairs; step-ups; low ceilings (sprint-jumps under them); ladders and vines; water (still, flowing, and a bubble column if you can); ice and blue ice; slime; soul sand; honey; cobweb; powder snow; edges, for the sneak back-off. Each course is mostly its class, with some plain ground so the bot moves in and out of it.

## C. One command: `lab/gates/record-fixtures.ps1 [-Courses <names>] [-Runs <n>] [-Ticks <k>] [-Seed <s>]`

Defaults: all courses, `-Runs 8`, `-Ticks 400`, `-Seed 1`. Reuse 1.2e's safety and rig handling: the lock, refusing while Julien is online, starting and stopping only what it started. For each course:
1. Build it, and wait for the client to see it.
2. For each run: generate inputs (`new-inputs.ps1`), replay them from the course's start, then `export` the box the recording visited plus a margin of 3 blocks.
3. Write the README's `player` line, the state **before tick zero** (after settling at the start), with every field the recorder knows. Write a `serverEnd` line from `scripts/server-state.ps1` read right after the replay ends, and say in the report how tick-aligned it is.
4. Run `check-recording.ps1`: a recording with an anomaly is reported and not kept as a fixture.
5. Copy each kept pair to `lab/fixtures/<course>/<course>-<run>.jsonl` and `.world.jsonl`.

It writes a report like 1.2e's: per course, recordings kept and rejected, and why.

## Done means

- `bot\build.ps1` builds. Every script in `lab/gates/` parses under Windows PowerShell 5.1. ASCII only.
- Offline stub tests, like 1.2e's, cover the driver's logic.
- If you can, check that a hand-written export line for one block round-trips through the Rust parser (`git show wt/law:engine/law/tests/support/json.rs`) by reading it. Don't modify `wt/law`.
- The final message lists every change, every derivation from section A, and every assumption.
