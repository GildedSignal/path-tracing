Implemented all review fixes, uncommitted.

- `Recorder.java`: `replay <name> [out]`, overwrite refusal, settings header, per-row `tickCount`, gap aborts with stopped tick, temporary outputs, plain-decimal teleport formatting, 200-tick settling timeout, rotation and explosion anomalies.
- `TickInput.java`: refuses settings incompatible with held inputs.
- `Bot.java`: updated command syntax, clears the follow path when replay starts, reports both dump counts.
- `ClientPacketListenerMixin.java`: added rotation and explosion hooks.
- `BlockDumper.java`: refuses during recording/replay, independently counts registry states, writes both counts in the summary, fails on mismatch, publishes through `.tmp`, records held items, samples scaffolding, restores the hand in `finally`.
- Added all four named scripts under `lab/gates/`: deterministic input generation, raw-bit comparison, recording validation, dump validation.
- Updated [bot/recorder.md](D:/path-tracing-wt/recorder/bot/recorder.md) with commands and formats.

Validation passed: the requested build, all four Windows PowerShell 5.1 parser checks, ASCII checks, and positive/negative helper checks covering determinism, key-run lengths, bit differences, anomalies, tick gaps, counts and constants. Live game gates remain for the integrator.

Assumptions made:

- “Three scripts” means the four explicitly named files.
- Dump counts count state rows, excluding header and summary.
- Held-item metadata means item id and stack count; scaffolding is sampled standing above.
- The first tick row establishes the `tickCount` baseline; an offending row remains in `.tmp`.
- Explicit stopping, including early replay cancellation, is clean. Failures retain `.tmp`; stale temporary recordings and existing manual recordings also cause refusal.
- The timeout counts END client ticks from sending `tp`.
- Comparison checks recursive `*Bits`, settings, tick stamps and length; decimal values, boolean flags and absolute `tickCount` are excluded.
- Generated yaw uses nonzero integer steps within ±90 degrees every 1–20 ticks; pitch stays fixed. Existing input files are replaced, and the last key run may be truncated.