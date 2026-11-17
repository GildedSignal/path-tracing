# The port, wave 2: digest today's results, then the next questions for GLM

**Julien (S207, verbatim):** "sounds good, there's still a lot to port"

**Consumers:**
- the integrator and `lab\wiki\`, the lab's settled index: what GLM settled today;
- GLM's next lanes, which run port work only (S173: "glm is not very strong"). A script copies your lenses into `lab\lenses\`, commits them and queues them when your report ends;
- the plan's later steps: step 3 (his tracks replay through the law), step 5 (editing the world) and step 8 (carried movement and wormholes) in `docs\plan.md`.

You read, and write only into `C:\Users\PASCAL\Desktop\path tracing\jobs\port-wave-2\`. Root `C:\Users\PASCAL\Desktop\path tracing\`.

## Part 1: digest the 11 results of 17 November
**Read:**
- the folders `lab\results\local-20261117-13*` through `local-20261117-16*` (not the `*-zai-limit` ones), each with `dossier.md` and `check.md`. The checker's grade wins where it disagrees with the author;
- `jobs\port-digest\report.md`, the first digest, for format and for what it already covered;
- `lab\wiki\index.md` and two existing pages;
- the engine code each result touches (`engine\law\src\`).

**Write `report.md`, part 1:**
- **per result:**
  - the lens and the checker's grade;
  - the settled fact with jar class and offsets;
  - whether the engine has it, has it partly, or lacks it (file:line);
  - what it unblocks;
  - the port delta a builder would write: file, function, the change, and the test that proves it bit-exact;
- **`port-portal-entry-and-create`** (graded short of reproduced) and **`port-lava-entity-inside`** (no check): what went wrong, and a narrower lens for each (below);
- **wiki drafts** in `jobs\port-wave-2\wiki\`, one per settled result, in `lab\wiki\`'s format;
- **the builder batches:** the ready deltas grouped so that no two jobs touch the same file.

  `engine\law\src\movement.rs` and `state.rs` are being changed on another branch today (the sets work, then the double-tap sprint port). Put deltas that touch them in a batch marked "after the sprint port".

## Part 2: the next wave of port lenses
"There's still a lot to port." Find what is left, then ask GLM for it.

**Read:**
- `jobs\port-backlog\report.md` (73 results: IN, PARTIAL, NOT IN, and nine batches);
- `jobs\flight-corpus\report.md` section 4 (what his tracks need that the engine lacks);
- `jobs\port-lenses\report.md` (wave 1);
- `lab\wiki\index.md` (what is settled: never re-ask it);
- `lab\glm.md`;
- at least four lenses in `lab\lenses\`, among them `port-sprint-window.md` and `port-breaking-threshold.md`. Their format is binding.

**Write lens files in `jobs\port-wave-2\lenses\`,** one `.md` each, named `port-<short-name>.md`, never colliding with a name in `lab\lenses\`.
- Each asks **one** jar-backed question a machine can check:
  - class, method and bytecode offsets in the staged 26.2 jars: the server jar `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`, the client jar `D:/path-tracing-deps/minecraft-26.2-client.jar`, javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`;
  - a consumer, and a falsifier.
- **Priority:**
  1. what his recorded tracks need that the engine lacks: Infinity Tower, the Diversity practices, Parkour Race's creative flight and game-mode switch;
  2. editing the world: placing, breaking, doors, trapdoors, buttons, pressure plates, water buckets;
  3. carried movement: boat, elytra, pearl, portal, including the narrower portal and lava lenses from part 1;
  4. the world's own laws that a bot meets: fluids flowing, falling blocks, scheduled ticks;
  5. what the backlog marked NOT IN that none of the above covers.
- **About 20 to 30 lenses,** each small enough for one GLM lane in under an hour. GLM is weak, so ask narrow questions. Never guess a class or method you have not seen in the jar's listing; write "to locate".

**In `report.md`, part 2:**
- the lenses in priority order, one line each: name, question, consumer;
- what is still left after this wave, as a count by area, so the integrator knows how much port remains;
- the line `queue: <name> <name> ...` with every lens name in order. The script reads this line.
