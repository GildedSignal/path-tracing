# Handoff

The state of the build, what is in flight and what comes next. Rewritten in place; history lives in git.

## First ten minutes

1. **Read, in order:** [AGENTS.md](../AGENTS.md); [PRINCIPLES.md](PRINCIPLES.md) (the goal, his philosophy, the register of his turns); this file; **[clan.md](clan.md) before giving anyone a job**; [learning.md](learning.md) (the unified idea, approved 12 November); [vocabulary.md](vocabulary.md); [representation.md](representation.md) and [scales.md](scales.md); [engine.md](engine.md), [footprints.md](footprints.md), [ladder.md](ladder.md), [server.md](server.md) as the task needs.
2. **His words:** [reference/julien-2026-11-10.md](reference/julien-2026-11-10.md) (S64–S105, X7–X16; S103 was never recorded verbatim). New turns go there verbatim, with a register row in PRINCIPLES.
3. **The whole plan:** `C:\Users\PASCAL\.claude\plans\wiggly-questing-lemur.md` (outside the repo; the roadmap below is its summary).
4. **Run** `.\scripts\status.ps1` (every lane, both queues, every report not yet committed) and `.\scripts\meters.ps1` (each meter's %). Start `.\scripts\meters.ps1 -WatchAt 80` in the background: it exits, and so wakes you, when any meter reaches 80% (S106). `git log -8 --oneline`; `lab/wiki/index.md` for what GLM has settled.

## The clan in three commands (S104)

`.\scripts\launch.ps1 <job> <terra|astra|sonnet|opus|argon|hydrogen> [-Worktree D:\path-tracing-wt\<name>] [-ReadDir <dir>]` starts any companion detached, makes a GPT worktree if missing, and **refuses** to exceed a slot: four GPT (two with Astra); on Antigravity (S105) one Argon or two Hydrogen, and one Opus or two Sonnet. A job is a folder: `jobs/<id>/prompt.md` in, `jobs/<id>/last.md` out. GLM: `scripts\glm-queue.ps1 -Queue lab\queue.txt` (re-reads the file before each batch), or `scripts\glm-local.ps1 -Lenses "<a b>"` for a lane now. Meters at 10:30 on 14 November: Zai 0%, Codex 5 h 7%, Codex week 11%.

**Lessons that cost slots** (memory `terra-stops-at-step-one`): Terra halts the whole job at the first open question, so resolve every classification or representation question before launching, name which steps are independent, and relaunch with a continuation section, never a bare retry. Inline the one file an Antigravity job must read (clan.md trap). A judge's verdict goes to `jobs/<id>/last.md`, never only into a chat turn.

## Now (14 November, S104–S106)

**Where we are.** The ground is built (the law at about 1 µs per tick, the world reader, footprints, the words in the code). A planner exists on `wt/planner` (job 3.2b): it certifies sentences on 8 of his 18 tracks, judged FAIL on its *claims* (the bound, thinking time, the backward method), not on its sentences, which the law replays tick by tick. **Nothing has been played in the game.** The shortest path to the first flight is therefore to fly 3.2b's certified sentences now (job 3.3c) while 3.2d replaces the planner behind the same call.

**In flight** (read each `last.md`; commit Terra's work with `Built-by:`):

| Job | Who, where | State and next action |
|---|---|---|
| 3.3c start, plan, play | Terra, `wt/playback` | Run 1 stopped at step 1 (`last-run1.md`: a field table, all useful). The continuation classifies every `PlayerState::bits()` field: observed, server-sampled, settled (two server samples ≥ 20 ticks apart, bit-identical resources, so the hurt counter is 0), dead at start (written before read), or refused. Then a `plan-track` binary, the 18-track export, `scripts/play-track.ps1` dry-run, the deviation report. **Then Julien runs the first live play on a quiet machine.** Judge: Sonnet |
| 3.2d-3 design | Opus on Antigravity, read-only | What a set *is*, the preimage, the backward solver, the token, cut into Terra-sized steps with machine gates. Next: a Terra red-team of the design, then Terra builds pass 3 on `wt/3.2d-sets` (passes 1–2 there: `0c543a3`, `32180e2`) |
| 1.4k inside-cell mutant | Terra, `wt/1.4k-inside-cell-mutant`, uncommitted | **Done:** the footprint gate is now 13 of 13; the key notices the fifth collision context (a synthetic cell, since no exported pair differs only there). Next: `launch.ps1 1.4k-judge sonnet -ReadDir D:\path-tracing-wt\inside-cell` once the Claude meter frees, then merge |
| anomaly-1 step-up | Hydrogen (its first job: the bake-off row) | how L0's control reported a mutant caught that GLM showed cannot be caught on block terrain |
| GLM | queue (6 port batches) + `port-peaceful-regen`, `port-boat-turn-trace` | read every `check.md` and `checker.log` (429) before trusting a lead; commit `lab/wiki/` |

**Merged this session:** 1.4d boats (`2a84060`, Sonnet PASS WITH FINDINGS; not connected to the player tick). Main tests: 149 passed, 11 ignored.

**The last session (S103), audited** (Argon, `jobs/s103-audit/last.md`): fifteen commits, most supported. Corrected: 1.4g (creative flight) is **not** a wall, since `abilities.flying` never holds on his survival tracks (S34), so it is unqueued; 1.4c's live half runs today. The 3.2d recheck was **8 FIXED, 8 PARTIAL** (not 7 and 9). "Anomaly 4 closed" was *narrowed*. Missing records: the 1.4c and 1.4e judges and Argon's 3.2d red-team left no `last.md`. Not a contradiction: reach 4.5 is the interaction attribute (engine.md), 5.5 the server's legality check (scales.md 49). `UseEffectsTable.java` now throws on a missing binding (1.4b judge).

**L0, `wt/l0-veto` (`246ff07`): waits only on its live gate,** which needs a quiet machine (no GLM, Terra or builds). Commands: `jobs/1.3b-frozen-counter/last.md`; a changed frozen source needs a fresh target (`oracles/1.6g-terrain-corpus/prepare.ps1 -Target <new> -ReferenceServer D:\path-tracing-lab\server`), then `run-live.ps1`. Edges and corner not run yet.

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

**Open for Julien:** the class (S67–S69): whether it is now where he wants it. The debug world as a one-world check of the block table (parked, S67). `omp auth import` would put Codex into `omp usage` too, but moves his OAuth tokens; not needed now that `meters.ps1` reads Codex's own logs. Two files are deleted in the working tree and not by this session: `Set GLM key.cmd` and `Stop GPT.cmd` (README and `glm-local.ps1` still name the first).

## Roadmap (approved 9 November, S48; reordered 10 November, S65, and 13 November, S95)

Every phase ends in a **flight** he uses in the game, and has a **kill criterion** that re-derives the representation instead of patching it. After Phase 3 the scoreboard reorders the rest, largest gap first.

0. **One line of truth.** Done.
1. **The ground.** Done except L0's live gate and job 1.5 (several clients at once, which removes the quiet-machine constraint). Kill: no path below about 5 µs means the `World` interface is wrong.
2. **The first flight** (slice A). The planner (3.2d: sets backward, the point forward, one replay), the playback (3.3c), the executor. **Flight: one bot races his 18 tracks; count the jumps where no sentence arrives or his T beats ours; his runs replayed through the law are the diagnostic.** Kill: words per regime ≫ 10³; no failure the loop can use.
3. **Learning where the flight fails** ([learning.md](learning.md)): hand-made maps from simple to the hardest; one bot. Flight: the bot clears a map he picks that it could not clear before. Kill: a solved jump that does not transfer to jumps of the same shape; cost per new puzzle not falling.
4. **Slice B, one tool, words that edit the world** (S65, S95). R-lego. Needs the placement law (pinned), operator segments, the law read backwards, the archive by effect, the obligation ledger. Kill: ≫ 10³ verifications per gap, a tool needing search code, or a found word that isn't reused.
5. **Slice C, one mind in the server, five bots sharing one tool** ([server.md](server.md)). Flight: he watches five bots cooperate; also 10–50 bots as players (S53).
6. **Then scale, by the scoreboard:** the far field (multipoles), tube tickets and lazy bots, more tools, the swarm at 10³ and 10⁴, the economy and building, then the cherry (learned ordering, the LLM overseer, the GPU, the human-limits profile, his easter eggs, S66).

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
