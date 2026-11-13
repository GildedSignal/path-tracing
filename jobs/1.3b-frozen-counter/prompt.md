# Job 1.3b: L0's frozen counter is a server event

**Why.** L0's live gate kept 90 of 91 runs and refused powder snow run 3 (seed 91) at tick 111: `END freeze counter`, law 3, recorded 4. Pinned by GLM (read `lab/results/local-20261111-1756-1-0-l0-powder-snow-freeze/dossier.md` in the main checkout `C:\Users\PASCAL\Desktop\path tracing`, and `lab/wiki/l0-powder-snow-freeze.md`): `ticksFrozen` is synched entity data; the client's only writer is the FREEZE effect (+1, cap 140); the server alone decrements (−2) and sends every change to the player itself; `handleSetEntityData` overwrites the client's value when the packet lands. The capture records no such packet. Julien's rule (S62): what the server sends is a recorded server event, applied by the replay, never simulated and never a law mismatch.

**Consumer.** L0's pass line (docs/ladder.md), which gates the first flight.

**Read.** `AGENTS.md`, `docs/engine.md` (server-driven inputs), `docs/ladder.md` (L0), the dossier above, and how this branch already records and replays corrections (commits `4e4e122`, `b928a3e`).

**Branch.** You are in `D:\path-tracing-wt\l0-veto` on `wt/l0-veto`.

**Build.**
1. The recorder records each `SetEntityData` packet that carries `DATA_TICKS_FROZEN` for the bot's own entity: its value and its phase against the client's move pass in that tick. The hook `ClientPacketListenerMixin.handleSetEntityData` is already there.
2. The fixture schema gains that event. Adding a recorded input is not weakening the gate: pass line, mutants and comparison fields stay as frozen.
3. The replay applies the event at its recorded phase, as it applies a correction, and continues.
4. A planted control: a fixture whose recorded value, or whose phase, is edited by one must be refused. Add it as a test.
5. Offline tests: a synthetic fixture with the recorded sequence 1, 2, 4, 4, 6, 7 over ticks 109–114 replays with no refusal.

**Must not.** Run anything live or take the rig lock (the integrator runs the live gate on a quiet machine). Weaken L0, its mutants or its comparison fields. Add a tolerance or a per-block branch. Change the law's arithmetic. Touch `docs/`. Commit, push or merge (you cannot write git metadata).

**Done means.** A final message with what changed per step, the test results (`cargo test` from `engine/`, and the mod's build), what the integrator must run live and with which commands, and anything you could not do.
