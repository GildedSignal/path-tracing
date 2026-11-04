# Job 1.2b: fix the recorder after its review

You built job 1.2 (`jobs/1.2-recorder/prompt.md`) on this branch, `wt/recorder`. A judge from another model family reviewed it. Fix these points exactly. Design choices are not yours: where a fix below is ambiguous, take the most literal reading and write the assumption in your final message. Stop only for something that blocks the build. Do not commit; the integrator commits your changes.

Build dependencies are staged at `D:\path-tracing-deps\data`, and `PATHTRACING_DEPS` points there. Run the build with `powershell -ExecutionPolicy Bypass -File bot\build.ps1`.

## Fixes (the judge's file references are from the reviewed commit)

1. **Replay output (Recorder.java around lines 54 and 92).** Make the command `replay <name> [out]`. The output file is `recordings/<out or name>.jsonl`, and the replay refuses to start if that file already exists, saying so in the reply.
2. **Ticks that didn't tick (Recorder.java around 150).**
   - Record `player.tickCount` in every row.
   - If it did not advance by exactly 1 since the previous row, abort the replay or recording, and reply with the tick at which it stopped.
3. **Hold-style input (TickInput.java around 248).** At the start of a replay or recording, refuse to run unless `toggleSprint`, `toggleCrouch` and `autoJump` are all off and `sprintWindow` is 0. Write those four values into the first line of the output file.
4. **No dump while recording (Bot.java around 50; BlockDumper).** `BlockDumper.dump` refuses while a recording or a replay is active.
5. **An independent count (BlockDumper.java around 100).** Also compute the sum of `getPossibleStates().size()` over `BuiltInRegistries.BLOCK`, separately from the writing loop. Report both numbers in the reply and in a final summary line of the file, and fail if they differ.
6. **No partial files.** Write `blocks.jsonl.tmp`, and move it over `blocks.jsonl` only on success. Do the same for recordings: `.tmp` until stopped cleanly.
7. **The `tp` command (Recorder.java around 98).** Format coordinates with `BigDecimal.valueOf(v).toPlainString()`, and yaw and pitch the same way through `Float.toString` → `BigDecimal`.
8. **The held item.**
   - Write the bot's main-hand and off-hand items into the dump's first line.
   - Add one more context: "holding scaffolding". Build it with `CollisionContext.of(player)` while the main hand holds a scaffolding stack, and restore the hand in `finally`, exactly as the dumper already restores pose and position.
9. **A start that can't be reached.** If the bot hasn't settled within 200 ticks of the `tp`, abort with a reply that says so.
10. **The old follow path.** Clear the follow path when a replay starts.
11. **Anomalies.** Also flag `ClientboundPlayerRotationPacket` and the knockback in `ClientboundExplodePacket`, both for the bot, with the same mechanism.
12. **Gate scripts.** Add a folder `lab/gates/` holding three PowerShell scripts, Windows PowerShell 5.1 compatible and ASCII only:
    - `new-inputs.ps1 -Name <n> -Ticks <k> -Seed <s> -Start x,y,z,yaw,pitch`. It writes a random input file in the recorder's format into `$Rig\clients\bot\pathtracing\inputs\` (`$Rig` from `scripts/common.ps1`). Each key is held for a random run of 1–20 ticks, and yaw changes in random steps. It is deterministic for a given seed.
    - `compare-recordings.ps1 <a> <b>`. It reports the first tick where the raw-bit fields differ, or "identical", and exits non-zero on any difference.
    - `check-recording.ps1 <file>` and `check-dump.ps1 <file>`. The first reports anomaly lines and `tickCount` gaps, and exits non-zero on any. The second checks that the line count equals the summary count and asserts ice 0.98, blue ice 0.989, slime 0.8, soul sand speed 0.4, and honey speed 0.4 with jump 0.5. It exits non-zero on any failure.

Update `bot/recorder.md` to match.

## Done means

- The build passes.
- The scripts parse under Windows PowerShell 5.1: run `powershell -NoProfile -Command "[ScriptBlock]::Create((Get-Content <file> -Raw)) | Out-Null"` for each.
- The final message lists every change and every assumption you made.
