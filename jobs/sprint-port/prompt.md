# Port the double-tap sprint window, so his recordings replay

**Consumer:** step 3 of `docs\plan.md`, the loop on his real tracks. The bot is compared with his recorded runs, and 47 of his 48 recordings were made with `sprintWindow` 7, which the law refuses today (`engine\law\tests\replay.rs:366`, "unsupported double-tap sprint window"). Julien asked for it (S155: "Yes, queue it"). Also HANDOFF anomaly 10: `LocalPlayer.aiStep:348` resets `sprintTriggerTime` through `isSlowDueToUsingItem`.

**The spec is Hydrogen's premise check, binding:** `C:\Users\PASCAL\Desktop\path tracing\jobs\sprint-window-port\report.md`. Read all of it. Its facts come from the GLM result `lab\results\local-20261116-1008-1-0-port-sprint-window\` (graded reproduced).

**Base:** the integrator starts you on a branch cut from `main` after the sets work (`wt/u2b-astra`) is merged, because that work also changes `engine\law\src\movement.rs` and `state.rs`. The line numbers in the report are from before it: locate each edit by its function and its content, not by line number. Where the code differs from the report's "current lines", stop that edit and report the difference. A failed premise stops only what depends on it.

## Decisions (do not reopen them)
1. **`PlayerState::default().sprint_window` is 0.** Existing words, plans and headerless replays keep exactly today's behaviour. Replays set it from the recording header (0..=10).
2. **The full replay of `jobs\hard-track\julien-428.jsonl`** lives in `engine\plan\tests\hard_track.rs` as `julien_428_replay`, on the world `build()` already makes. The double-tap tests and the negative control live in `engine\law\tests\replay.rs`.
3. **`load_state` (`replay.rs`) keeps reading `initialPlayer.inputs`** for now. Note the stationary-start limitation in `last.md`.
4. **The bot client stays hold-style:** `TickInput.holdSettings` is unchanged.
5. **The state-read guard** of the report's section 2.2 is kept. With the window and the timer both 0, `state.input` is not read before it is written, so existing footprints and the words' state reads stay identical. Prove it: the words and plan test suites pass unchanged.

## Do
The report's sections 2.1–2.4, exactly:
- the two state fields with bits, `from_bits` and the state-read registration (including `encode_as!`'s `u8`);
- the previous tick's forward and sneak read before the input is overwritten;
- the countdown, the reset (anomaly 10's `isSlowDueToUsingItem && !isPassenger` included), the arm and the double-tap start, with `is_slow_due_to_using_item` shared with `canStartSprinting`;
- `recording_header` returning the window and refusing values outside 0..=10;
- `jar_sprint_latch_corpus`'s call updated.

## Gates (frozen before their first run, never weakened)
1. **`julien_428_replay`:** all 429 ticks, as the report's section 3.1 says. The law carries `sprinting` itself, never re-seeded from the recording. Bit-exact position, velocity, rotation and fall distance, plus the flags and pose, on every tick.
2. **`double_tap_sprint_window_and_negative_control`:** the report's section 3.2, all four parts:
   - the positive window at 7 (r 1..5 starts, r 6 does not);
   - the boundary windows 1 and 10;
   - the three reset gates;
   - the negative control: window 0 on a double-tap pattern never sprints.
3. **The engine workspace's tests pass,** with `CARGO_TARGET_DIR=D:\path-tracing-target\sprint-port`. LAW_HASH changes, so say which hash-pinned tests moved and why.
4. **Which of his recordings now replay:** for every one of his recordings whose world can be built offline today (`hardtrack`), and each other whose world exists, report bit-exact or the first differing tick and field. Don't fix other gaps; list them.

## Must not touch
`engine\plan\src\` and `engine\words\src\`, except a call site that a signature change forces. `bot\`, his tracks and runs, the server and clients. Launch no game. Never run `scripts\launch.ps1`, `gpt.ps1` or `follow-up.ps1`. Normal priority.

**`jobs\sprint-port\last.md`:**
- the files changed;
- each premise that differed;
- the gates with their output;
- the recordings table;
- the replay commands.

End with `STATUS: DONE` or `STATUS: BLOCKED: <why>`.
