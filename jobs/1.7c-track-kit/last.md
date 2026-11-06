Finished from `91dcf5b`; changes are uncommitted. No game launched.

- Completed kits, session recovery, survival preparation, re-arming, restoration, `/track kit`, `/track exit`, and human run hashes.
- Fixed creative-mode ordering, server-authoritative refill verification, stale callback handling, and entry failure handling.
- Recovery restores before loading tracks and prevents immediate re-entry on A.
- Expanded regression tests and mixin audit; updated [tracks.md](/D:/path-tracing-wt/tracks/bot/tracks.md) to 35 lines.
- Detector rules, T, bot toolkit and bot preparation remain unchanged.

Validation passed: build, 394 inventory/track-mode checks, 101 existing headless checks, all 24 audited injections, PowerShell 5.1 parsing, ASCII and whitespace checks.

Assumptions: inventory covers the specified 41 slots plus hotbar selection; stick removal targets those slots while preserving nested item data. The kit stays fixed throughout track mode. Restoration uses the saved mode, including adventure or spectator. Full means maximum health, 20 food and 20 saturation. Death restoration waits for respawn; preparation requires staying on A. Unconfirmed steps stop after 200 client ticks and retain the session for explicit recovery.