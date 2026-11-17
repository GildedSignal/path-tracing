# Handoff

The state of the build, what is in flight and what comes next. Rewritten in place; history lives in git.

## First ten minutes

1. **Read, in order:** [AGENTS.md](../AGENTS.md); [PRINCIPLES.md](PRINCIPLES.md) (the goal, his philosophy, the register of his turns); this file; **[plan.md](plan.md), the plan**; **[clan.md](clan.md) before giving anyone a job**; [learning.md](learning.md) (the unified idea, approved 12 November); [vocabulary.md](vocabulary.md); [representation.md](representation.md) and [scales.md](scales.md); [engine.md](engine.md), [footprints.md](footprints.md), [ladder.md](ladder.md), [server.md](server.md) as the task needs.
2. **His words:** [reference/julien-2026-11-10.md](reference/julien-2026-11-10.md) (S64–S211, X7–X16; S103 was never recorded verbatim). New turns go there verbatim, with a register row in PRINCIPLES.
3. **The whole plan:** `C:\Users\PASCAL\.claude\plans\wiggly-questing-lemur.md` (outside the repo; the roadmap below is its summary).
4. **Run** `.\scripts\status.ps1` (every lane, both queues, every report not yet committed) and `.\scripts\meters.ps1` (each meter's %). Start `.\scripts\meters.ps1 -WatchAt 80` in the background: it exits, and so wakes you, when any meter reaches 80% (S106). `git log -8 --oneline`; `lab/wiki/index.md` for what GLM has settled.

## The clan in three commands (S104)

`.\scripts\launch.ps1 <job> <terra|astra|sonnet|opus|argon|hydrogen> [-Worktree D:\path-tracing-wt\<name>] [-ReadDir <dir>]` starts any companion detached, makes a GPT worktree if missing, and **refuses** to exceed a slot: four GPT (one beside Astra, S15); on Antigravity (S105) one Argon or two Hydrogen, and two Sonnet (S115: Opus is native). A job is a folder: `jobs/<id>/prompt.md` in, `jobs/<id>/last.md` out. GLM: `scripts\glm-queue.ps1 -Queue lab\queue.txt` (re-reads the file before each batch), or `scripts\glm-local.ps1 -Lenses "<a b>"` for a lane now. Capacities and the usage protocol: [clan.md](clan.md) section 4, measured 14 November.

## Start here (17 November, 22:00)

- **The plan is [plan.md](plan.md).** After every compaction re-read the docs (S164). Rules from 17 November, all in AGENTS.md, PRINCIPLES' register or memory:
  - a smaller model never writes for a bigger one (S210); Astra resumes only to build, never for sweeps (S210);
  - **thinking time is reported per real command:** a new goal each time, any direction, from standing and while moving, the world in memory, one living store. Never a repeated question labelled "warm" (S223); the harness is `engine/plan/tests/real_time.rs` in `D:\path-tracing-wt\real-time` (detached at `e6de417`, not committed), log `D:\path-tracing-evidence\real-time-20261117.log`;
  - explain what work does, not "step 3" or "pass 4" (S222); a duplicate run is called duplicate and stopped (S221);
  - **no native sub-agent until the weekly reset on Monday 23 November** unless he asks (S226); judges wait for Antigravity's Claude meter, back Thursday 20 November 23:02. Keep the integrator's own turns lean.
- **The truth about speed (S223–S225):** every new command on flat ground costs a **median 233 ms** (160–450 ms, about 4,700 law ticks re-prepared each time, nothing reused); while moving 253 ms, and **33 of 60 retargets are refused**. The 57 µs reported for days was the same goal asked twice. His word: "each knew command on the flat ground should take µs IN TOTAL".
- **Simplest case right (`wt/u2b-astra`, `e6de417`):** judged **pass with findings** (`jobs/u2b-judge/last.md`): a dominant-axis branch in `solve.rs:140`, the frozen gate's fixtures unhashed, the recurrence guard ignored. Not merged to main (nor are the core and joints and sets).
- **Armed in this session** (if the session is gone, launch by hand):
  - **00:33, Astra 6.2: every new command on flat ground in microseconds** (`jobs/warm-floor`: the judge's three findings first, then honest per-command measurement, the yaw-free generator, data shaped for the read, one walk per axis; gate: µs per new command, standing or moving, none refused). `.\scripts\launch.ps1 warm-floor astra -Worktree D:\path-tracing-wt\warm-floor -Base wt/u2b-astra`
  - **00:35, Sol 6.2: his tracks as inputs** (`jobs/loop-tracks` Part A: region terrain of any dimension, the clicked face, the settled start, a harness counting like his track clock, one record per track; no red-team, S226). `.\scripts\launch.ps1 loop-tracks sol -Worktree D:\path-tracing-wt\loop-tracks -Base wt/u2b-astra`. Closing the loop in `solve` (Part B) is Astra's after `warm-floor` is judged.
- **Antigravity runs commands** (`jobs/agy-commands/last.md`): `launch.ps1 <id> argon|hydrogen -Commands -Worktree ...`; the probe is stamped. Owed by the integrator: prompts for Argon's port tooling (the game itself as the port's judge; a coverage map from the jar), and Argon's reading of Frazzoli and the sources learning.md cites from memory. **GLM:** the queue runner now survives Zai's 5-hour limit (requeues, pauses 30 min); wave 2's 28 lenses running since 20:28 (`lab/results/queue.log`).
- **The refuge for him** (`jobs/hearth-small`, Hydrogen with its own renders, worktree `wt/hearth-small` from the renderer's branch): passes 2–4 done (pass 4 ended 21:29, not yet looked at by the integrator); **pass 5, critique and repair**, next, with his critique folded in: the rock reads as stacked boxes, so irregular edges and mixed stones, no long straight cliff faces. Placing it waits for his go and a world backup.- **Waiting for a judge** (Antigravity Sonnet from Thursday): the clicked face (`wt/track-face`), the rebuilt hard track (`wt/hard-track-2`), freeze damage (`wt/l0-veto`), the preview renderer (`wt/build-preview`, `0cd3229`). **Held:** the double-tap sprint port waits for the merges; its spec must be rewritten by the integrator (S210). **Commits after `09e1b52` are local, not pushed.**

## Now (the flights of 14 November; older session notes are in git)

**THE FIRST FLIGHT FLEW (14 November, 11:08).** `play-track grassjumpeasy` in his own world: the bot played 3.2b's certified sentence with **clock T 49 ticks = his best 49**, 0 corrections, 0 pushes, thinking 895 ms (prebuilt planner; the clock does not count it, the scoreboard must). Predicted T was 50: check whether that is the track clock's stop rule (first START touching B) against the plan's arrival row. First deviation: **tick 0, saturation 5.0 predicted, 20.0 recorded**: the planner's settled start uses a default player while arming fills to 20, which is the loader's job (3.3d). The deviation tool stops at the first field, so movement beyond it is not yet compared field by field. All 18 tracks then played live in sequence (`scripts/flight-scoreboard.ps1` over `D:\path-tracing-evidence\flight-20261114\live.txt`): **8 sentences, every one with 0 corrections and 0 pushes**, clock T against his: stonesimple 84 against 85 (**the bot ahead**), grassjumpeasy 49 = 49, flowers 48/47, fencestraightbridge 55/46, stonestaircase 131/112, mangrovejumpeasy 66/44, carpetwater 107/84, and **butoonpressureplates 198/70**: the track he built to see whether the bot worries about non-issues (S109: buttons and plates have no collision), so the 2.8x is the planner's. **Diagnosed from the plan's inputs:** a flat, straight 25 blocks; the bot sprint-jumps for about 20 ticks, then **crouch-walks (sneak on, sprint off) for about 170 ticks** over the buttons and plates, where he sprint-jumps throughout. Both have an empty collision shape (models 508 and 510, like air 378); the plate carries an inside-effect flag and both differ from air in their flags, so the class, the proposal table or a cut at each body-block change treats them as new ground where only a sneak word was found. That is the non-issue his track tests, and 3.2d's design must make such a cell provably the same as air to the law's comparisons, not merely different by name. Predicted T is the clock's T + 1 on all 8: a counting convention, to pin. No sentence: birchparkourmedium, fences, fencemaze, redfakemaze, amertystcoral (movement: the planner's), and the five that need place or interact words (S108). Thinking 0.69 to 1.29 s, outside the clock and inside the scoreboard's T. **Split, on stonesimple** (his S110 Fermi request): `cargo run` in the driver ~450 ms (removed on `wt/3.3g-play`: dry-run click 734 → 302 ms), the floor's table compile 180 ms (1.3 M landings; the harness calls it "not in thinking", which is 3.2b's FAIL point), load ~40 ms, certify ~50 ms, **search 12 ms, first sentence 6 ms**. **The Fermi floor for a straight line is ~10–100 µs** (read ~460 tube cells; ~2 decisions; certify 85 ticks at ~1 µs, or ~7 arcs at 0.7 µs): 0.002 ticks. Measured is 2 orders above it for the search, 3–4 for the click. Causes: 312 candidate replays for a 1-switch answer (28×); 12.5 µs per certified tick inside the search against the law's ~1 µs (12×); a whole-floor map rebuilt per click. Target: thinking ≪ 1 tick on paths whose map answer is trivial; above 1 tick (50 ms) is an idiot-index flag (S83). **Straight tracks (S111; `straight100/300/800` at x=300, on `wt/3.3g-play`):** T is right (0.345 b/t, the stone sprint-jump gait with its run-up), and **flown live at 15:22 (S119 "you can use it"): clock T 299, 875, 2,315 with 0 corrections and 0 pushes on all three**, live thinking 2.2, 6.3, 30.5 s cold (`D:\path-tracing-evidence\flight-20261114\live-straight.txt`; on 800 blocks thinking is 26% of T); but thinking is 0.56, 3.57, 23.1 s for 100, 300, 800 blocks, **growing as length squared**, 4 orders above the 800-block floor (~0.3–2.5 ms). Search is linear (321 ms at 800). The quadratic part is `slide` (`flight.rs:152`): on 800 blocks it saved 264 ticks and spent ~456. **Decisions: one per jump** (430 on 800 blocks) where the map's answer has about two switches: for 3.2d's design. Job **3.3h** (slide's pay rule, local replay to bit-equal rejoin, a planner that stays loaded with provenance per table) is specced and launches itself at 15:17, after Codex's 5-hour reset; worktree `D:\path-tracing-wt\thinking` (the fixture world is git-ignored and was copied in by hand: do the same for any new worktree). The stack that flies is `wt/3.3g-play` (`f0f58f9`: planner, playback, plan-track, deviation, the driver), **not on main**: it rests on 3.2b, judged FAIL on its claims. Merging it is his call. The bot's mod jar in `clients/bot/mods` is this branch's build; the 5 November jar is at `D:\path-tracing-deps\pathtracing.jar.bak-20261105`.

**The quiet machine was an assumption, and it is mostly retired (S107, measured).** L0 under full load (4 Terra, GLM, a release flight run): at Normal priority, Preflight missed the lab clock's rate floor (986 against 1,000 client ticks per second) before any movement was compared; with every lane at Idle CPU priority, Preflight and Audit passed with 0 corrections and the Full gate kept 90 of 90 runs across 11 courses, then refused **powder snow run 3 (seed 91) at tick 111, exactly as on the quiet machine on 11 November**: deterministic, not load. Lanes now run at Normal priority (S158) and are lowered to Idle only for a live gate. **L0, `wt/l0-veto` (`246ff07`):** the frozen counter's recorded server packets (1.3b) moved the gap from 3 against 4 to 4 against 5: the real client adds 2 in tick 111 (3 to 5) where the law adds 1. GLM lens `l0-freeze-tick-111` hunts the second writer. Target `D:\path-tracing-target\l0-veto-246ff07-b`, rigs `D:\path-tracing-evidence\l0-veto-20261114\{preflight,audit,full}` (delete `server/world/players/data/*.dat` before each stage); edges and corner not run yet.

**Still owed a new spec:** `roster` (stopped twice; the GLM half is in `D:\path-tracing-wt\roster-b`). The executor (re-anchors at every event, prices veto risk, S62) after 3.3c's first live deviations show what it must absorb.

**The near goal (S55):** about 5 bots with pathfinding and swarm behaviour working very well, before any serious optimization. "Vanilla in behaviour, not pure vanilla." Swarm 0 and the scale work are parked (S53).

**Where to test, each one axis of the ladder** (S97, not yet in ladder.md): parkour (width), dropper (one long arc), clutch (an operator word under a deadline), boat racing (momentum's memory), elytra (R-brach), puzzle maps (words writing what words read), PvE later. He finds the maps himself (S101).

## Anomalies (open; each blocks only the claims that depend on it)

1. **L0's step-up mutant is equivalent on block terrain** (GLM `local-20261111-1756-1-2`, reproduced: 0 of 98,871 candidates round into its window). How L0's control reported it caught: job anomaly-1. Until then that "caught" does not count.
2. **Server saturation 19.0 against 20.0 in the archived full-blocks replay: diagnosed, not closed** (`jobs/anomaly-2-saturation/last.md`). Exhaustion matches the server bit for bit; the law never calls peaceful-difficulty regeneration (`server_regeneration` exists, the movement tick omits it), and his world is peaceful. With an assumed server tick phase the whole endpoint passes. Next: GLM `port-peaceful-regen` pins the branch and how to observe its phase; then a Terra port. It bears on 3.3c's start state.
4. **Anomaly 4 narrowed** (1.4e, `5a2b81a`): `PlayerState` has no spectator input, so nothing refuses a spectator; the trapdoor-over-ladder branch has no witness. The inside-cell key gap is closed by 1.4k (13 of 13).
5. **Entity push has no producer in the law**: `push_pair` matches the jar, nothing calls it. Blocks slice C (five bots on one crossing), not slice A.
8. **The state-read hook costs the law 3–5%** (609 against 590 ns per tick).
9. **3.2a's "second column" test is near-tautological**; the class redesign replaces it.
10. **Unmodeled readers:** `LocalPlayer.aiStep:348` resets `sprintTriggerTime` (the double-tap sprint window) through `isSlowDueToUsingItem`; the law has no such state. Boat turn: the port's Mth-table speed is a record until GLM `port-boat-turn-trace` runs the jar's own table.
11. **S38's bug was never traced:** "bug, when doing a track it deletes sulfur spike from my inventory" (5 November).
12. **Sprint at an exactly axis-aligned wall depends on rounding** (17 November, `jobs/set-rederive/proposal.md`): `movement.rs:818`'s minor-collision test takes `acos` of a cosine that is 1 up to rounding, so above 1 it is NaN and sprint drops. It is Mojang's own behaviour, ported faithfully; only axis-aligned yaws hit it, and the bot's yaw grid holds them. Sets carry it as two points, never as cuts.
13. **A body that reaches a wall within 1e-7 is held, not placed on it** (17 November, `jobs/pin-rederive/proposal.md`): when another collider follows in the visit order, Mojang zeroes a remaining distance below 1e-7 (`movement.rs:613`, `shapes.rs:399`), so the exit keeps the entry's free-flight position (Astra's 1-ULP witness, `wt/u2b-astra`). It depends on how many cells the box spans at contact, not on the yaw, and lasts until the next clamp. It is nature's behaviour, ported faithfully.

**Open for Julien:** whether "i added other things" (S162) names anything beyond S116–S134 (`jobs/unified-plan/questions.md` Q12); the class (S67–S69): whether it is now where he wants it. The debug world as a one-world check of the block table (parked, S67). `omp auth import` would put Codex into `omp usage` too, but moves his OAuth tokens; not needed now that `meters.ps1` reads Codex's own logs. Two files are deleted in the working tree and not by this session: `Set GLM key.cmd` and `Stop GPT.cmd` (README and `glm-local.ps1` still name the first).

## Roadmap (approved 9 November, S48; reordered 10 November, S65, and 13 November, S95)

Every phase ends in a **flight** he uses in the game, and has a **kill criterion** that re-derives the representation instead of patching it. After Phase 3 the scoreboard reorders the rest, largest gap first.

0. **One line of truth.** Done.
1. **The ground.** Done except L0's live gate and job 1.5 (several clients at once, which removes the quiet-machine constraint). Kill: no path below about 5 µs means the `World` interface is wrong.
2. **The first flight** (slice A). The planner is the one loop of [learning.md](learning.md) section 0 (sets backward, the point forward, each worldtube checked once; steps 1–4 of [plan.md](plan.md)), then the playback (3.3c) and the executor. **Flight: one bot races his 18 tracks; count the jumps where no sentence arrives or his T beats ours; his runs replayed through the law are the diagnostic.** Kill: words per regime ≫ 10³; no failure the loop can use.
3. **Learning where the flight fails** ([learning.md](learning.md)): hand-made maps from simple to the hardest (what a hard track is: S166, learning.md section 0); one bot. Flight: the bot clears a map he picks that it could not clear before. Kill: a solved jump that does not transfer to jumps of the same shape; cost per new puzzle not falling.
4. **Slice B, one tool, words that edit the world** (S65, S95). R-lego. Needs the placement law (pinned), operator segments, the law read backwards, the archive by effect, the obligation ledger. Kill: ≫ 10³ verifications per gap, a tool needing search code, or a found word that isn't reused.
5. **Slice C, one mind in the server, five bots sharing one tool** ([server.md](server.md)). Flight: he watches five bots cooperate; also 10–50 bots as players (S53).
6. **Then scale, by the scoreboard:** the far field (multipoles), tube tickets and lazy bots, more tools, the swarm at 10³ and 10⁴, the economy and building, then the cherry (learned ordering, the LLM overseer, the GPU, the human-limits profile, his easter eggs, S66).

### The order of work

It lives in [plan.md](plan.md): the steps in plain names, what each is, its gate and where it stands (S179).

## Scoreboard

T over his time and over the bracket's upper bound; first-move latency at p99; cost per bot and server ms per tick at N bots; the deviation rate and the **veto rate** per capability (S62); the overturn rate per grain; segments per regime and the proposal hit rate; distinctions created per second; the rediscovery count; feedback-loop time. Each row shows its gap to its Fermi floor, in orders of magnitude.

## Tracked requests (a request made twice gets an owner)

| Request | Owner | Where it lands |
|---|---|---|
| Multipoles (FMM) for how T acts at a distance (S6, ten times in ES) | integrator, Phase 3 | the far field: bounds and landmarks, the gap between bounds as the stated error |
| Hundreds, then thousands, of bots with shared work (S18, S35–S36, S44–S45) | integrator, Phases 2 and 5 | one mind in the server |
| 100× iteration (S32, S44) | integrator, Phase 1 | the clock, the offline fast loop, job 1.5 |
| Creativity that arises, stays and compounds, cheaply (S32, S44, S64) | integrator | scales.md 4.1; R-lego is its falsifier |
| Explanations in his vocabulary, grounded in the whole context (S67, S75–S77) | integrator | read every doc and his turns first; speak in word, certificate, worldtube, light cone, T |
| An idiot-index pass, Fermi floors at micro, meso, macro and holistically (S83) | integrator, after the first flight | GLM `idiot-index` gives the census; talk it through with him before any build |
| Primary sources read directly; cases and extremes; his book list and Readwise (S86, S87, S89) | integrator | learning.md section 7 |
| Use every meter before the integrator's runs out (S100, S104) | integrator | `launch.ps1`, `status.ps1`, `meters.ps1 -WatchAt 80`; queue detached work before the window ends |
| Routing: Opus and Astra for the important work, Astra maximized, Terra backfills, GLM the port only, Argon and Hydrogen used fully (S70, S173, S174, S208–S211) | integrator | AGENTS.md team table; clan.md 5c |
| A smaller model never writes for a bigger one (S175, S210) | integrator | AGENTS.md "The integrator orchestrates"; every launch checks its binding inputs |
| Explain means explain: no action when he asks to explain or think first (S8, S132, S179, S181) | integrator | AGENTS.md "Julien's words" |
| Re-read the docs after compaction, and lint them against his words from time to time (S76–S77, S161–S164, S198, S211) | integrator; Argon lints | AGENTS.md first line; clan.md 5c rule 10, `jobs/doc-lint` |
| Nothing in the foreground when someone else uses the computer (S126, S135, S140, S144) | integrator | detached launches; no game or window opened unasked |
| No made-up numbers: cuts, zones and sizes emerge from the law and the need (S67, S124, S132) | integrator | every spec; each red-team checks it |
| ES-faithful: ticks are execution, never thinking (S44, S58, S83, S117, S118) | integrator | learning.md section 0; vocabulary.md section 2; every judge checks it |
| Threads and the GPU (S6, S44, S54, S118, S131): none in `engine/` today | integrator | step 9 of [plan.md](plan.md), the whole machine: threads first, the GPU for learning's batches after a measured round trip (scales.md 4.7) |
| Whether search is needed; a map clear enough to need no policy (S39, S97, X9, X15, X16, S130–S132) | integrator | learning.md section 0, "Is A* needed?"; representation.md section 11 (ridges) |
| The two extremes: a million-block straight line, a hard track as he defines it (S32, S111, S120, S131–S132, S166) | integrator; he finds the maps | learning.md section 0 and kill tests 8–10, 17; maps he finds (S101, S159) |
| Move at once unless it is hard; thinking ≪ 1 tick (S90, S110, S120, S124, S131) | integrator | learning.md section 0 ("moving at once"), kill test 15 |
| Change your mind fast near players (S6, X16, S121) | integrator | step 7 of [plan.md](plan.md), other players: cones on the timing wheel |
| Speedrun and parkour techniques, ZPD puzzles (S32, S87–S89, S97–S98, X10, S122) | integrator | ladder.md Rediscovery; learning.md Phase 3 |
| Boats and elytra (S6, S17, S96–S98, S122) and interactions ported in full (S44, S46, S51, S108) | GLM lanes, Terra | Slice B, R-brach |
| Combat, building, farms, the LLM overseer and dialogue (S23–S24, S32, S44, S46, S97, X11–X12) | integrator, later | Roadmap 6 |
| Questions he can answer, with their context (S78, S100) | integrator | each turn's one question |

## When a job ends

Read `jobs/<id>/last.md` and `git -C <worktree> status`. Judge in a fresh context by the other family (`launch.ps1 <id>-judge sonnet` for GPT's work; Terra for Claude's), verdict in `jobs/<id>-judge/last.md`. Fixes go back to the builder; squash-merge to `main` (message files, not stdin, under PowerShell 5.1); `cargo test --offline --workspace` from `engine/` with `CARGO_TARGET_DIR=D:/path-tracing-target-integrator` (Terra's builds hold the shared one); push.

## Traps

- **Disk:** toolchain, builds and lab worlds on D:. Rust: `PATH=D:\rust\cargo\bin`, `RUSTUP_HOME=D:\rust\rustup`, `CARGO_HOME=D:\rust\cargo`. `lab/results` holds 270k files (1.9 GB, git-ignored); the GLM scrubber is incremental since S104 (delete `lab/results/.scrubbed` to force a full pass).
- **Live runs need a quiet machine** until job 1.5, take the rig lock, and never run while Julien drives the bot.
- **Vanilla judge:** Lithium and C2ME never go into the bot's client or the reference rig.
- **Codex sandbox:** no `%LOCALAPPDATA%` (`scripts/stage-deps.ps1`), no git metadata, no stored GLM key.
- **Reference rig:** `PATHTRACING_ROOT=D:\path-tracing-lab`, port 25566. Gate scripts under Windows PowerShell 5.1; game files written without a BOM.
- **PowerShell 5.1:** a native command's stderr under `$ErrorActionPreference='Stop'` throws (git's progress did, in `launch.ps1`).
- **Untracked `jobs/<id>/last.md`** in main blocks squash-merging a branch that tracks it: move it aside first.
