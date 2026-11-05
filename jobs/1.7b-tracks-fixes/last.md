Implemented, uncommitted. No game was run.

- Bests now separate players and modes, with distinct human/bot lines.
- Discards close recordings and delete `.tmp`; indices are cached, and rows stay in memory until finish.
- Tools count once plus breaks; stackables count units. Held mining no longer inflates the toolkit.
- Added wall timing, tick exception containment, fallback result files, and unique definition temps with cleanup.
- Equal endpoints are refused; swept-box finish detection catches fast crossings.
- Fixed cancel suppression, `/track new` refusal during runs, bot timeout/mode preparation, departure-input recording, and action-bar-only re-arming.
- Updated documentation and regression tests.

Validation passed: build, **101 headless checks**, **19 audited injections**, PowerShell 5.1 parsing, ASCII checks. `tracks.md` is 25 lines.

Assumptions: “his best” means the fastest non-bot run on the current definition; ties use the first recorded. Without one, the bot keeps its current mode. Timeout starts at departure. `wallNanos` is elapsed time; raw endpoints are also saved. Recording names include the player; fallback names include a uniqueness suffix. The two-block fall test uses a swimming/crawling-height box.