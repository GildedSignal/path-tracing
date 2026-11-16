# Track face builder handoff

The clicked B face is stored in new track definitions, named in feedback, outlined locally every ten client ticks, and used by both touch detectors. Old definitions remain unchanged and explicitly say `any face (set before S170)`. A refuses a side click.

The remaining `track run` gap is resolved: `Bot.goToTrack` retains the track definition, steers its final body position to the selected face, and delegates finishing to the shared track detector. Its existing straight-line navigation is unchanged. The Rust flight and `plan-track` reader carry the face into `Goal`; optional face fields in track bounds are accepted and conflicting faces are refused.

Changed files:
- `bot/src/pathtracing/Bot.java`
- `bot/src/pathtracing/Pointer.java`
- `bot/src/pathtracing/TrackDetector.java`
- `bot/src/pathtracing/TrackStore.java`
- `bot/src/pathtracing/Tracks.java`
- `bot/tests/TrackTests.java`
- `bot/tests/test-tracks.ps1`
- `bot/tests/fixtures/face-touch.txt`
- `engine/plan/src/flight.rs`
- `engine/plan/src/goal.rs`
- `engine/plan/src/lib.rs`
- `engine/plan/src/reach.rs`
- `engine/plan/tests/face_touch.rs`
- `engine/plan/tests/hard_track.rs`
- `engine/plan/tests/planner.rs`
- `jobs/track-face/last.md`

Validation:
- `powershell -NoProfile -ExecutionPolicy Bypass -File bot/build.ps1`: PASS.
- `powershell -NoProfile -ExecutionPolicy Bypass -File bot/tests/test-tracks.ps1`: PASS, including the shared Java face vectors, six driver targets, definition round trip, and top-then-side timer check.
- From `engine`, `CARGO_TARGET_DIR=D:\path-tracing-target\track-face cargo test --offline --workspace`: PASS, including the shared Rust vectors and face-square distance check. Existing ignored tests remain ignored. Log: `D:\path-tracing-target\track-face\face-workspace-test.log`.
- `git diff --check`: PASS.
- Contact boundaries have no epsilon: exact plane contact passes, a one-ULP separation fails, and tangent-only edge/corner contacts fail. No final test refusals.

Implementation findings: removing the old inflation exposed the backward field's strict vertical comparison, which excluded standing exactly on B. Its broad candidate bound now includes equality; the exact shared touch predicate remains the arrival authority. The first workspace run failed that check; the corrected workspace passes.

Try in game:
1. Create a track. Click a side of A: expect `A is where you stand: click its top face.` Then click its top.
2. Click B's side face and check the block, coordinates and face message. Stand on A and check the arming face and particle outline.
3. Leave A and touch B's top: no finish. Then touch the selected side: finish, naming that face and tick T.
4. Check `/track list` labels for old tracks and use the stick on A or `/msg PathBot track run <name>` to start the bot with the same face target.

No game, server, client or RCON was launched. No saved track definitions or runs were edited. The pre-existing world-region changes and backup files were left untouched. No git metadata was written.

STATUS: DONE
