Implemented and left uncommitted for the integrator.

### Changes

- Added client track commands, stick selection, coloured particles and action-bar timing.
- Added the shared detector with start, re-arm, stop and abandonment handling.
- Added locked track files and JSONL run records, including settings, inventory, item usage and toolkit snapshots.
- Enabled automatic recording in both clients, with reasons recorded when capture fails.
- Added bot RCON toolkit preparation, teleport, settling, planner invocation and separate first-input latency.
- Passed `$Rig` through the launcher system property.
- Added headless tests, updated recorder documentation and wrote the 25-line [track guide](D:/path-tracing-wt/tracks/bot/tracks.md).

### Verification

- `bot/build.ps1` passes.
- 69 headless checks pass.
- All 18 new injection targets match the staged 26.2 bytecode.
- Touched scripts parse under Windows PowerShell 5.1.
- All changed files are ASCII. `git diff --check` passes.

### Assumptions and limits

- Timing samples occur at client-tick START. Seconds use 20 ticks per second.
- Abandonment includes death, dimension changes and falling 64 blocks below the lower endpoint.
- One human track is active at a time. Shared starts select alphabetically. Supporting A takes precedence over touching B.
- Item counts mean successful usage/swing events and dropped units. A right-click and its later consumption/release count once.
- Items are grouped by registry ID. Custom components and enchantments are omitted.
- Toolkit history uses the configured owner's finished runs on the current track definition. Empty history gives an empty toolkit. Toolkits exceeding 36 inventory slots are refused.
- Deletion retains history. Recreated tracks get separate best-time and toolkit comparisons. Best time includes human and bot runs.
- Track recordings live under the rig's `recordings/` directory. Discarded partial recordings retain `.tmp` files.
- Bot teleport targets the block-cell top centre, followed by five settled samples. Latency includes preparation and counts turning as input.

The bot uses the existing `StraightLine` planner. Live-game validation is pending.