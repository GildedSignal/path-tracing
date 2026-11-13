# Job 3.3a: play the planner's certified sentences in the game

**Why.** Julien wants to see the bot in the game (13 November; approved S101). The planner on this branch certifies sentences offline on his tracks (8 of 18) and nothing has ever been played. A playback is the first meeting of those sentences with the server: it shows him the bot, and every tick where the game departs from the law's prediction is a deviation to explain (S81: "a gold mine"). It is not the executor: no replanning, no re-anchoring.

**Consumer.** The first flight (HANDOFF roadmap item 2) and the deviation stream.

**Read.** `AGENTS.md`, `docs/ladder.md` (tracks: T runs from leaving A to touching B, by the same detector for him and the bot), `docs/vocabulary.md` section 7, `engine/plan` (`walk.rs`: a sentence is steps of a program at an aim for a number of ticks; `certify`), `engine/plan/tests/flight.rs`, `bot/src/pathtracing/` (`Bot.java`, `Tracks.java`, `Recorder.java`, and how the L0 gate plays an input script tick by tick: find it under `oracles/` and `jobs/1.3-l0-close/run-live.ps1`).

**Branch.** You are in `D:\path-tracing-wt\playback` on `wt/playback`, cut from `wt/planner`. Set `CARGO_TARGET_DIR=D:/path-tracing-target/playback`.

**Build.**
1. **The planner writes a sentence as a script.** For each track with a certified sentence: one file with the exact start state the sentence was certified from, and per tick the inputs the law was given (keys, sprint, sneak, jump, yaw and pitch as raw bits) and the state the law predicts after that tick (position and velocity as raw bits, on ground). Written by the flight harness beside its run record; the format is the one the mod's existing script player reads if one exists, extended only where it lacks a field.
2. **The mod plays it.** A whisper command `play <track>`: the bot goes to the track's A as `Tracks` already sets a run up (its kit, its mode), reaches the script's exact start state (if the game cannot be put in that state exactly, say what differs and stop: do not approximate), then applies the script's inputs one per client tick, through the same path the L0 gate uses.
3. **It reports.** Per run: arrived or not, T on the track's own clock, and the first tick where the game's state differs from the predicted bits, with both values and the field; every server correction as an event. Written to a file per run and summarised in chat to the player who asked.
4. **It never fights him.** `play` refuses while a track run of his is active, and any input of his cancels it, as `Tracks.cancel` does today.

**Must not.** Add a tolerance to the comparison (a difference is reported, never hidden). Re-plan, re-anchor or correct during the run. Change the law, `engine/words`, the gates, `docs/`. Run anything live, take the rig lock, or start a client or server: the integrator and Julien run it. Commit, push or merge.

**Done means.** `cargo test --offline` from `engine/` passes; the mod builds; a final message with the script format, the files written for the tracks that have a sentence, the exact command Julien types in the game, what could not be made exact about the start state, and anything left out.
