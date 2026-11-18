# The port, digest 3: last night's nine GLM results, made ready to build

**Julien (18 November, verbatim):** "have hydrogen work on the port"

**Consumers:**
- the integrator and `lab\wiki\`, the lab's settled index: what GLM settled last night;
- the next port builds: each settled fact becomes a delta a builder can write and prove bit-exact;
- the plan (`docs\plan.md`): step 3 (his tracks: Parkour Race's creative flight and game-mode switch), step 5 (editing the world: placing, breaking, doors, levers, tripwires, mining in creative).

You read, and write only into `C:\Users\PASCAL\Desktop\path tracing\jobs\port-digest-3\`. Root `C:\Users\PASCAL\Desktop\path tracing\`.

**Read:**
- the nine folders in `lab\results\`, each with `dossier.md` and `check.md` (the checker's grade wins where it disagrees with the author):
  - `local-20261117-2028-1-0-port-gamemode-ability-switch`
  - `local-20261117-2029-1-0-port-flying-horizontal-speed`
  - `local-20261117-2030-1-0-port-sprint-food-and-item-reset`
  - `local-20261117-2150-1-0-port-place-slab-stair-trapdoor`
  - `local-20261117-2155-1-0-port-place-door-ladder-scaffold`
  - `local-20261117-2158-1-0-port-destroy-block-side-effects`
  - `local-20261117-2220-1-0-port-interact-lever-and-redstone-door`
  - `local-20261117-2226-1-0-port-tripwire-hook-and-string` (its lane exited with an error: say what is usable)
  - `local-20261117-2241-1-0-port-mining-creative-and-abort-gate`
- `jobs\port-wave-2\report.md` Part 1: your own format from yesterday, binding;
- `lab\wiki\index.md` and two existing pages;
- the engine code each result touches (`engine\law\src\`).

**Write `report.md`** (write it as you go, result by result):
- **per result:** the lens and the checker's grade; the settled fact with jar class and offsets; whether the engine has it, has it partly, or lacks it (file:line); what it unblocks; the port delta a builder would write: file, function, the change, and the test that proves it bit-exact, naming the raw evidence file each test value comes from;
- **any result short of `reproduced`:** what went wrong, and one narrower lens to re-ask it, written as a lens file in `jobs\port-digest-3\lenses\` in the format of `lab\lenses\port-sprint-window.md` (never a name already in `lab\lenses\`);
- **wiki drafts** in `jobs\port-digest-3\wiki\`, one per settled result, in `lab\wiki\`'s format;
- **the builder batches:** the ready deltas grouped so that no two jobs touch the same file. `engine\law\src\movement.rs` and `state.rs` are being changed on another branch: put deltas that touch them in a batch marked "after the movement work". Today another builder takes `mining.rs`, `boat.rs`, and new `elytra.rs` and `pearl.rs`: mark any delta touching those as "after build A";
- the line `queue: <name> <name> ...` with any narrower lens names, or `queue:` empty.

**Rules:** one mechanism (`docs\learning.md` section 0): a delta never gives one block kind its own code path, it reads the block table's data. No patches: never propose a tolerance or a clamp to match the jar. Never guess a class or method you have not seen in the evidence.

End with `STATUS: DONE` or `STATUS: BLOCKED`.
