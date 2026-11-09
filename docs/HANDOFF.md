# Handoff

This file holds the state of the build, what is in flight and what comes next. It is rewritten in place; history lives in git.

## First ten minutes

0. Read "Now" below.
1. Read [PRINCIPLES.md](PRINCIPLES.md), then [AGENTS.md](../AGENTS.md), then [representation.md](representation.md) (the framework), then this file.
2. Run `git status --short`, `git log -8 --oneline` and `git worktree list`.

## Now (9 November)

**The framework is approved** (S44–S48): one law at three grains, a tool as a law input you carry, the word as the one object, one mind inside the server, missions. The whole-project roadmap below is approved (S48). **Phase 0 is under way.**

- **Consolidation.** `wt/engine` (`D:\path-tracing-wt\engine`) = `wt/l0` + `wt/world`, `wt/law`, `wt/raw-fluid-export`, `wt/glm-lab`, `wt/tracks` and `main`, all clean merges. Rust: 81 tests pass. Bot track tests pass (the mixin audit learned local capture, `f0bcea2`). Terra job [0.2](../jobs/0.2-consolidate/prompt.md) is verifying the remaining self-tests (`test-run-gate-1.2` exits 1). Then `main` fast-forwards to `wt/engine`.
- **The 100× clock** (`wt/clock`, `388a1bc`): the Sonnet judge passed it as MERGE WITH FIXES. Its report was run on uncommitted code, so the gate gets rerun on the merged HEAD. The bot follows `/tick rate` by default through the launcher, 2000 TPS is the only verified ceiling, and a guard for recordings above it is needed. After 0.2, the integrator starts `git merge wt/clock`, and Terra job [0.3](../jobs/0.3-clock-merge/prompt.md) resolves the conflicts and adds the guard.
- **Docs.** Being rewritten to the framework: AGENTS (team with Terra), representation, engine, ladder, README.
- **Deletions** wait for Julien's OK on the list (Phase 0.3).

## Roadmap (approved 9 November, S48)

Every phase ends in a **flight**, something Julien uses in the game, and has a **kill criterion** that re-derives the representation instead of patching it. After Phase 3 the scoreboard reorders the rest, largest gap first.

0. **One line of truth.**
   - Commit the pending state (done, `51cdba1`).
   - Consolidate the branches.
   - Delete what's dead.
   - Put the scaffolding on a diet: the harness ends smaller than the law it guards.
   - Rewrite the docs.
1. **The ground.** Lanes in parallel:
   - **law speed:** allocation-free, bit-identical, ≤ 1 µs per tick, from 26–59 µs today (Terra);
   - **footprints and determinism:** randomness as input; the canonical pattern key is red-teamed (Sol);
   - **world reader:** [2.1a](../jobs/2.1a-region-reader/prompt.md) (Terra);
   - **close L0:** step-ups seed 17, tick 204 (Sol);
   - **the ground layer:** GLM lanes, one Mojang rule each, batched in by Terra.

   Flight: his track runs replayed offline at ≥ 10⁴× real time. Kill: no path below about 5 µs means the `World` interface is wrong.
2. **One mind in the server, v0.**
   - Design: a Fabric server mod hosts the Rust engine in-process through Java 25's foreign-function API. Bots are real `ServerPlayer`s, and world changes stream to the engine once. This revises the 4 November "separate process" decision, so it needs Julien's yes.
   - Build: the deviation stream (every server-owned outcome predicted and compared, each mismatch a witness for GLM triage).
   - [Swarm 0](../jobs/swarm-0-headless-scale/prompt.md), rewritten to this design: 0, 1, 100 and 1,000 bots.
   - A sandbox server at 100× for R&D.

   Flight: Julien sees 100, then 1,000, bots as players.
3. **The vocabulary on legs, and the first flight.**
   - The word record, red-teamed and then frozen in `docs/vocabulary.md`.
   - The word compiler: gaits, transitions, compound words, a cache keyed by pattern.
   - The medium and fields: the light cone, relaxed bounds, landmarks as the far field.
   - The executor.

   Rungs: R1–R3, R-retarget, his tracks, R-scale, R-far 10³, and the rediscovery count starts. Flight: the bot races his tracks. Kill: words per regime ≫ 10³, or a pattern-cache hit rate under about 50% on natural terrain.
4. **Tools as carried inputs.**
   - Interaction laws; operator words carrying their obligations.
   - Discovery pulled by gaps.
   - A relaxed medium for chains.
   - An R&D slice on the sandbox.

   Rungs: R6, R7, R8 (price = T to get another), R-brach, R10, rediscovery, the Baritone benchmark. Kill: ≫ 10³ verifications per gap, or a tool needing new search code.
5. **The swarm.**
   - Missions; shared fields.
   - Reservations only for scarce things.
   - A demand threshold for infrastructure.
   - Obligations that pass between bots.
   - Congestion; lazy bots in the server.

   Rungs: 20 bots get a portal where 1 bot doesn't; a shared bucket crossing; a rolling bridge cleaned up by the last bot; 10³ and 10⁴ bots.
6. **Economy and building.**
   - Prices as fields; farms as words.
   - Blueprints as forecasts W(t).
   - Reach-spot fields; (max, +) scheduling; scaffolding words.
   - Multiple objectives (S32).
7. **The cherry.**
   - Learned ordering (Expert Iteration on logs kept from Phase 3).
   - The LLM overseer and grounded conversation; LLM-proposed words.
   - The GPU where profiling points.
   - The human-limits profile (S32).

**Parallel lanes:** Phase 0 first. Then Phase 1's lanes together (two GPT jobs plus GLM). The integrator writes the specs for 2.1 and 3.1 during Phase 1, and they're red-teamed. Then Phases 2 and 3 build side by side.

The rules that keep it converging are in [AGENTS.md](../AGENTS.md): flights, no job without a consumer, SR-71, ladders that climb until they fail.

## Scoreboard

Rows begin with Phase 1:
- T over his time, and over the bracket's upper bound;
- first-move latency at p99;
- cost per bot, and server ms per tick at N bots;
- deviation rate per capability;
- words per regime, and the pattern-cache hit rate;
- rediscovery count;
- feedback-loop time.

Each row shows its gap to its Fermi floor, in orders of magnitude.

## Tracked requests (AGENTS.md: a request made twice gets an owner)

| Request | Owner | Where it lands |
|---|---|---|
| Multipoles (FMM) for how T acts at a distance (S6, ten times in ES) | integrator, Phase 3 | the medium's far field: bounds and landmarks, with the gap between lower and upper bound as the stated error |
| Hundreds, then thousands, of bots with shared work (S18, S35–S36, S44–S45) | integrator, Phase 2 and 5 | one mind in the server |
| 100× iteration (S32, 6 November, S44) | integrator, Phase 0 and 1 | the clock (`wt/clock`) and the offline fast loop |

## When a job ends

1. Read `jobs/<id>/last.md`, then `git -C <worktree> status`.
2. Commit on its branch with `Built-by: <model>`.
3. A Sonnet judge reviews it in a fresh context.
4. Fixes come as a new job.

Launch a job with `.\scripts\gpt.ps1 -Job <id> -Worktree <path> -Model gpt-6.1-sol|gpt-6.1-terra`. The prompt is read from `jobs/<id>/prompt.md` in this checkout.

## Traps

- **Disk:** C: has about 8 GB free. The toolchain, builds and lab worlds go on D:.
- **Shared bot:** the bot's client is shared with Julien's play. Gates take the rig lock and never run while he drives the bot.
- **Vanilla judge:** Lithium and C2ME change game logic, so they never go into the bot's client or a server.
- **No Fable** (usage credits).
- **Codex sandbox:** it can't read `%LOCALAPPDATA%` (use `scripts/stage-deps.ps1`) and can't write git metadata.
- **Server stdin:** commands sent before the world loads fail (README).
- **Reference rig:** `PATHTRACING_ROOT=D:\path-tracing-lab`, port 25566. Run gate scripts with Windows PowerShell 5.1.
