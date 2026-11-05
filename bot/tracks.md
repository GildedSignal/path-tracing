# Parkour tracks

Start both clients through scripts/launch-client.ps1. In yours, `/track new <name>` (names: [a-z0-9-]+).
Right-click A's block, then a different B block with your stick. Sneak + right-click cancels.
Green particles mark A, red particles mark B, within render distance in your client.
Use `/track list`, `/track best <name>`, `/track delete <name>` and `/track cancel`.
Stand on A to arm. Move or sprint on A freely. Leaving its support starts the clock.
Returning to A re-arms on the action bar. Cancel on A requires leaving and landing back to arm.
Stop is the first START sample whose previous/current box union touches B's cell grown by 1e-3.
Supporting A takes precedence: B counts only after leaving A. Fast non-solid B crossings count.
Chat shows your best by player and mode; `/track best` separates human and bot bests and modes.
Run rows include wallNanos (elapsed System.nanoTime), wallStartNanos and wallStopNanos.
Game-time difference is the client's own counter between time packets; it rarely shows server drift.
Death, dimension change, cancel or falling 64 blocks below the lower endpoint discards the run.
If tracks share A, the alphabetically first arms. `/track new` refuses during a run.
Send `/msg PathBot track run <name>`; enable local vanilla RCON in the rig's server/server.properties.
The bot takes the fastest human run's mode (current mode if none), your toolkit, then settles on A.
Bot runs abandon at 20 times the best human T, or 6,000 ticks without a human time.
Command-to-first-input latency includes preparation, counts turns, and is reported separately.
Stack-size-one items count once plus each break; stackables count consumed/placed/thrown/dropped units.
The bot gets each item's largest count from your finished runs on this definition of the track.
Definitions live in `lab/tracks/<name>.json`, with dimension, creator and creation date.
Runs append to `lab/tracks/runs/<name>.jsonl`; append failures save `<name>.unsaved-<timestamp>.json` there.
Each bot row stores its exact toolkit. Deleting a definition retains its run history.
Hold-style controls enable `recordings/track-<name>-<player>-<n>.jsonl`; discarded starts delete their temp.
