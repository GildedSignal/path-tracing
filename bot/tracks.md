# Parkour tracks

Start both clients through scripts/launch-client.ps1 so they share the rig folder.
Type `/track new <name>` in your client. Names use lowercase letters, digits and `-`.
Right-click A's block with your stick, then B's block. Sneak + right-click cancels creation.
Green particles mark A, red particles mark B, within render distance in your client.
Use `/track list`, `/track best <name>`, `/track delete <name>` and `/track cancel`.
Stand on A to arm. Move or sprint on A freely. Leaving its support starts the clock.
Returning to A re-arms. Touching B's cell ends the run, even while airborne.
The action bar shows elapsed ticks. Chat shows ticks, seconds and the best finished time.
Client ticks are sampled at START. A different level-game-time delta is also shown.
Death, dimension changes, cancellation or falling 64 blocks below the lower endpoint discard.
If tracks share A, the alphabetically first name arms. Supporting A takes precedence over B.
Send `/msg PathBot track run <name>` to run the bot with the current planner.
The bot needs local vanilla RCON enabled in the rig's server/server.properties.
It clears its inventory, gives your used toolkit, teleports to A's top centre and settles.
Command-to-first-input latency includes preparation, counts turns, and is reported separately.
Item counts are successful use/swing events or dropped units, grouped by registry item id.
A right-click and its later consumption/release count once. Custom item components are omitted.
The bot gets each item's largest count from your finished runs on this definition of the track.
Definitions live in `lab/tracks/<name>.json`, with dimension, creator and creation date.
Finished runs append to `lab/tracks/runs/<name>.jsonl`, including settings and starting inventory.
Each bot row stores its exact toolkit. Deleting a definition retains its run history.
Hold-style controls enable automatic `recordings/track-<name>-<n>.jsonl` under the rig.
Other settings still allow timing. The run row explains any missing recording.
