# Double-tap sprint: the exact port change, so his recordings replay

**Consumer:** step 3 of `docs\plan.md`, the loop on real tracks, which compares the bot with his recorded runs. Most of his runs were recorded with `sprintWindow` 7, and `engine\law\tests\replay.rs:366` refuses them ("unsupported double-tap sprint window"). Specs are the integrator's (S175). Your job is the premise check, the exact edit list, and the proof test, so that the integrator's Terra prompt leaves nothing open.

You read only, and write only your report. Root `C:\Users\PASCAL\Desktop\path tracing\`.

## Read
- **The settled facts:** `lab\wiki\port-sprint-window.md`; the result folder `lab\results\*port-sprint-window*` (its `dossier.md` and `check.md`; the checker wins); the lens `lab\lenses\port-sprint-window.md`.
- **The anomaly:** `docs\HANDOFF.md`, anomaly 10. `LocalPlayer.aiStep:348` resets `sprintTriggerTime` through `isSlowDueToUsingItem`.
- **The engine:**
  - `engine\law\src\state.rs` (the state fields), `movement.rs` (sprint, `update_sprinting`), `attributes.rs`;
  - the client-input path: how the law receives keys (`engine\law\src\state.rs` `Input`);
  - `engine\law\tests\replay.rs`, around line 366 and the recording header.
- **His recordings:**
  - `bot\src\pathtracing\Recorder.java` and `TickInput.java` (what is recorded per tick: raw keys, `isSprinting`, settings);
  - `jobs\hard-track\julien-428.jsonl`;
  - the run index `lab\tracks\runs\*.jsonl` (each run's `sprintWindow` and recording path);
  - `jobs\flight-corpus\report.md`.

## Write `jobs\sprint-window-port\report.md`
1. **Premises:** every file, field and function the change touches. Mark each EXISTS, DIFFERS or MISSING, with file:line.
2. **The exact edits:**
   - the new state field or fields (the trigger timer), where it lives and how it is reset (including anomaly 10's `isSlowDueToUsingItem` path);
   - where the law reads the forward key's press edge;
   - how `sprintWindow` from the recording header enters;
   - what `replay.rs` stops refusing.

   For each, give the current lines and what the jar says they must become: class, method and offsets from the dossier.
3. **The proof test:** replay `julien-428.jsonl` bit for bit through the law from its recorded inputs. Predicted `isSprinting` must equal the recorded one at every tick, and positions must match. Name the test, its inputs and what it asserts. Add one negative control: `sprintWindow` 0 on a double-tap pattern must not sprint.
4. **His runs:** which of his 51 tracks' recordings would replay after the change, and which still could not, with the reason (another missing port, a `recordingNote`).
5. **Open questions** for the integrator, each with the options and your recommendation.

Cite file:line for every claim. Where something is absent, write "not found in" with the files checked.
