# Job anomaly-2: the archived server-saturation red, measured before anything is changed

**Why.** HANDOFF anomaly 2: the archived full-blocks retained replay fails `assert_server` with **server saturation, predicted `1100480512` (19.0f) against observed `1101004800` (20.0f)** (or the reverse: establish which), identically on `main` and older branches (`jobs/1.2-footprints/last.md`, "Evidence"). Saturation drops by exactly 1.0 when exhaustion crosses 4.0. The integrator's hypothesis, **to be confirmed or refuted, not assumed**: the archived settled fixtures predate the server sample, so their start `exhaustion` is the client's observation, which `bot/recorder.md` says is not server-authoritative; a wrong start exhaustion makes the law cross 4.0 where the server did not (or the reverse).

**Consumer.** Job 3.3c (running now) classifies every start-state field as observed, server-sampled, settled or refused, and puts exhaustion in "server-sampled". If the hypothesis is right, this red is that rule's first witness; if it is wrong, the law has a food defect that sprint (food > 6) and every long run will hit.

**Branch.** `D:\path-tracing-wt\anomaly-2` on `wt/anomaly-2-saturation`, from `main`. The main checkout is `C:\Users\PASCAL\Desktop\path tracing`.

**Read.** `engine/law/tests/replay.rs` (`load_state`, `assert_server`, `check_server_end`, and how the archived full-blocks recording is located), `engine/law/tests/support/server_resources.rs`, `bot/recorder.md` ("Server endpoints" and the L0 driver's resource rule), `engine/law/src/` wherever exhaustion and saturation are written.

**Do, in order:**
1. Reproduce the red; record the exact command and output.
2. Trace exhaustion tick by tick through the law from the fixture's start: the start value and its source field, every tick it changes and by what rule (file and line), and the tick where it crosses 4.0, if it does.
3. Say what start exhaustion would make the law's end state equal the server's, and whether any value the fixture or the server sample records supports it.
4. Verdict: **input defect** (the fixture's start value is not authoritative: name the field, and say what the fixture lacks), **law defect** (name the line and the jar rule it departs from, with the jar offset; then port the fix with a witness red before and green after), or **undetermined** (say what measurement would decide it).

**Must not.** Edit a fixture, a recording, or `assert_server`. Add a tolerance, skip, or `#[ignore]`. Change the law unless step 4 finds a law defect with its jar offset. Start a server or client. Commit, push or merge.

**Done means.** The four steps answered; if the law changed, `cargo test --offline` from `engine/` passing and `jobs/1.2-footprints/verify.ps1` still 12 of 12; the net lines. Under 50 lines.
