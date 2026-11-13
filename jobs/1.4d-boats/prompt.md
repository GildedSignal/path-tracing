# Job 1.4d: the boat enters the law

**Why.** A boat is the clearest case of his principle 4: possessing one changes the space rather than adding a move, and it is the one tool whose momentum the ground remembers (`docs/learning.md`, "T belongs to the state a bot arrives with"). He walked the boat-on-a-frozen-river case through on paper (S98), and boat racing is one of the named axes to test on (HANDOFF, "Where to test": momentum's memory). The law cannot run any of it today.

**Consumer.** The boat-racing axis, and `learning.md`'s claim that momentum carries across words — which is untestable while the law has no boat. Not the first flight: his 18 tracks have no boats, so **nothing in Phase 2 waits on this**, and that is why it must not touch anything Phase 2 runs on.

**Branch.** `D:\path-tracing-wt\boats` on `wt/boats`, cut from `main`.

**Read.** `AGENTS.md`, `docs/engine.md` (the law's contract and "store the generator"), `docs/representation.md` (recurrent motions joined by transients), and in the main checkout `C:\Users\PASCAL\Desktop\path tracing`:
- `lab\wiki\port-boat-rider-attachment.md` and `lab\wiki\port-boat-delta.md`;
- the run folders `lab\results\local-20261113-2043-1-0-port-boat-rider-attachment\` and `lab\results\local-20261113-2043-1-1-port-boat-delta\` — each one's `dossier.md`, `reproduce.sh`, `check.md`, and `sim\boat_sim.ps1` where present. Both are **reproduced** by an independent checker and every constant is pinned to a cited offset.
- `lab\wiki\port-boat-surface-transition.md` **if it exists by the time you run** (a GLM lane was producing it tonight). If it is absent or graded `lead`, the surface transition is out of scope: say so and stop at the still-water recurrence.

**Build, in this order.**
1. **The rider's seat.** `hull.position + localOffset.yRot(-hull.yRot * 0.017453292f) - rider.VEHICLE attachment`, with the isolated yaw step from `AbstractBoat.getPassengerAttachmentPoint`. The dossier's falsifier list is your test list: a port that yields `(100.0, 63.5875, -199.85)` for hull yaw 90 or 270, that leaves a non-zero residual at 180°, that confuses 0.1875 with 0.5 on rafts, that uses `-0.4f` instead of the float add for slots, or that wraps the yaw, is **wrong** — write a test for each of those five.
2. **The occupied-boat per-tick recurrence in still water**: `floatBoat` drag and buoyancy, then `controlBoat` thrust and yaw, then `move`. The dossier calls it closed and fully bytecode-pinned, with terminal straight speed `0.39999990` b/t. Every constant (`0.9f`, `0.04f`, `0.005f`, `0.65`, `0.75`, `0.101`, `0.6`, `1.375f`, `0.5625f`) comes from its cited offset and **none is written by you**. Store it as the recurrence and its parameters, not as a table of outcomes.
3. **The turn's fixed point**, which the dossier derives analytically and independently of the tick loop. Check your port against that derivation, not against your own loop.

**Note the checker's discrepancy** on `port-boat-delta`: it calls the falsifier overstated for three of the constants in the simulation. Read what it says and do not claim a falsifier the checker weakened — port the constants from their offsets and test what actually discriminates.

**Must not.** Add a tolerance, an epsilon, a retry or a per-boat-type special case. Invent any constant, or copy one from `boat_sim.ps1` without following it to its cited offset in the jar. Touch `engine/plan`, `engine/words`, `bot/`, the gates, the fixtures, or any code the first flight runs. Model the surface transition, passengers other than the player, or collision between boats. Run anything live. Commit, push or merge.

**Done means.** A witness per delta, red before and green after, bit for bit against the dossier's values; the five falsifiers above each as a test; `cargo test --offline` from `engine/` passing; `jobs/1.2-footprints/verify.ps1` still 12 of 12 mutants; and a final message giving the rule as ported with class, method and offsets, the terminal speed your port produces against `0.39999990`, the net lines, and everything left out.
