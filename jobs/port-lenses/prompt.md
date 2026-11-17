# Port lenses for GLM's overnight lanes

**Consumer:** GLM, which runs port work only (S173: "use glm just for the port and hydrogen for more complex questions, glm is not very strong"). The lenses you write run tonight; a script commits and queues them when your report ends. The plan's later steps need the port: step 5, editing the world, and step 8, carried movement and wormholes (`docs\plan.md`).

You read, and write only your report and the lens files. Root `C:\Users\PASCAL\Desktop\path tracing\`.

## Read
- `docs\plan.md`, the plan in plain names. Read steps 5 and 8.
- `jobs\port-backlog\report.md`: 73 port results (IN, PARTIAL, NOT IN) and nine port batches.
- `jobs\flight-corpus\report.md`, section 4: the mechanics his recorded tracks need that the engine lacks.
- `jobs\event-forms\report.md`: what each mechanic's event form needs.
- `lab\wiki\index.md` and the pages it links: what is already settled. Never re-ask a settled fact.
- `lab\glm.md`: the lab's rules.
- At least four existing lenses in `lab\lenses\`, among them `port-sprint-window.md` and `port-breaking-threshold.md`. Their format is binding.

## Write
1. **Lens files** in `C:\Users\PASCAL\Desktop\path tracing\jobs\port-lenses\lenses\`, one `.md` per lens. Each file's name is the lens name, `port-<short-name>.md`; check it doesn't collide with `lab\lenses\`.
   - Each lens asks **one** jar-backed question a machine can check: class, method and bytecode offsets in the staged 26.2 jar (`D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`, client jar in `D:/path-tracing-deps/data`, javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`).
   - Each lens names its **consumer** (the plan step, a track of his, or a port batch) and a **falsifier**.
   - Order of priority:
     1. mechanics his recorded tracks need that the engine lacks;
     2. editing the world: block placing, breaking, interacting with doors, trapdoors and buttons;
     3. carried movement: boat, elytra, pearl, portal;
     4. the law bugs the port backlog found, among them the `acos` NaN at `movement.rs:818`, `boat.rs`'s double-index trig, and `server_regeneration`'s missing health guard.
   - About 15 to 25 lenses. Each must be small enough for one GLM lane in under an hour; GLM is not strong, so ask narrow questions.
2. **`jobs\port-lenses\report.md`:**
   - the lenses in priority order, one line each: name, question, consumer;
   - what you left out, and why;
   - the line `queue: <name> <name> ...` listing every lens name in order. The script reads this line.

Never guess a class or method name you have not checked in the jar's listing; write "to locate" instead.
