Continue the same job. The Sonnet judge's verdict is **FAIL** on one high finding; everything else passes. Read `C:\Users\PASCAL\Desktop\path tracing\jobs\maps-stick-judge\last.md` in full and fix every finding:

1. **High, reproduced:** the second import corrupts `maps.json`. PowerShell 5.1 does not unroll the array, so the first element becomes `{"value":[...],"Count":1}`, and `/map` then answers "Map: null" (`Maps.java:30`).
   - Fix it so the registry is always a flat JSON array of map objects, whatever its size (0, 1 or more maps).
   - Add the missing check to the gate: import **three** synthetic maps in a row, read `maps.json` back, and assert three entries with the right names and spawns.
   - Also assert that `Maps.java`'s reader parses that file. Use the same parsing code path in a Java or Rust unit test, or make the script's own read-back use the same shape the mod expects.
2. **The low findings:**
   - The test root is hard-coded to the maps-stick folder: make it relative to the script or a parameter.
   - The server jar default fails in a worktree: resolve it through `scripts\common.ps1`'s rig paths, as the other scripts do.
   - `bot\tracks.md` still says `[a-z0-9-]`: update it to `[a-z0-9_-]`.
   - A reset click is swallowed silently when human mode is not ready: say why in chat instead.

Rerun every gate in the prompt's "Done means" plus the new three-import check, and update `jobs\maps-stick\last.md`. You are the Terra builder: never run `launch.ps1`, `gpt.ps1` or `follow-up.ps1`. Never touch the real server world. End with `STATUS: DONE` or `STATUS: BLOCKED: <why>`.
