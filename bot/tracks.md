# Parkour tracks

Start both clients through scripts/launch-client.ps1. In yours, `/track new <name>` (names: [a-z0-9-]+).
Right-click A's block, then a different B block with your stick. Sneak + right-click cancels.
Green particles mark A, red particles mark B, within render distance in your client.
Use `/track list`, `/track best <name>`, `/track delete <name>`, `/track cancel`, `/track exit` and `/track restore`.
Standing on A checks and saves the server's inventory and game mode before changing either.
Close inventory screens and put away cursor items and crafting ingredients before entering.
If the client and server disagree or a stack cannot be restored exactly, entry stops and chat explains why.
The first entry saves your inventory minus every stick as `lab/tracks/<name>.kit.json` and tells you.
Kits retain all main/hotbar, armour and offhand slots, selected hotbar slot and complete item data (SNBT in JSON).
`/track kit <name>` replaces a kit from your current inventory minus sticks; refused during a run or track mode.
Each arming/re-arming applies that kit, fills health/food/saturation, and switches you to survival.
Stay on A until armed; refill verification waits for a later server time packet. Leaving A starts the unchanged clock.
Returning to A re-arms on the action bar. Cancel on A requires leaving and landing back to arm.
Stop is the first START sample whose previous/current box union touches B's cell grown by 1e-3.
Supporting A takes precedence: B counts only after leaving A. Fast non-solid B crossings count.
Chat shows your best by player and mode; `/track best` separates human and bot bests and modes.
Run rows include wallNanos, wallStartNanos, wallStopNanos; game-time difference is the client's time-packet counter.
Death, dimension change, cancel or falling 64 blocks below the lower endpoint discards the run.
Finish, abandonment, `/track cancel` or `/track exit` restores your saved inventory and mode; death waits for respawn.
Restoration uses creative slot packets, then your saved mode; each step waits for server confirmation.
The session is `lab/tracks/.session-<player>.json`. The mod deletes it only after checking every slot and the selected slot against the server.
Joining with a session recovers it automatically and reports recovery in chat; completed runs are saved first.
Mode/refill commands and server verification require op permission. A restore mismatch reports the first differing slot in chat and keeps the session. Use `/track restore` to retry.
Each human run stores `kitSha256` over canonical kit JSON (sorted keys, compact UTF-8, no newline) and `gameMode`.
If tracks share A, the alphabetically first arms. `/track new` refuses during a run or track mode.
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
