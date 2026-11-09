# Handoff

This file holds the state of the build, what is in flight and what comes next. It is rewritten in place; history lives in git.

## First ten minutes

1. Read [PRINCIPLES.md](PRINCIPLES.md), then [AGENTS.md](../AGENTS.md), then [representation.md](representation.md) (the framework), then this file.
2. Run `git status --short`, `git log -8 --oneline` and `git worktree list`.

## Now (9 November)

**The framework is approved** (S44–S48): one law at three grains, a tool as a law input you carry, the word as the one object, one mind inside the server, missions. The roadmap below is approved (S48). **Phase 0 is done; Phase 1 is under way.**

**The near goal (S55): about 5 bots with pathfinding and swarm behaviour working very well, before any serious optimization.** "Vanilla in behaviour, not pure vanilla": improve with our own changes, or find mods as needs show up. Don't invent scale problems. So the critical path is: world reader, then L0, then the planner (Phase 3), then 5 bots on the server. Swarm 0 and the scale work are parked.

- **Phase 0, done.** `main` is the one line: the law, the recorder, the tracks, and the 100× clock with its guard above 2000 TPS. Every offline suite is green.
  - The live clock gate passed on `48d2deb`: 2000 TPS movement identical on three courses, 12,000 sustained ticks identical with zero corrections, 101× active, 4000 refused by the guard.
  - Merged branches, worktrees, build folders and finished jobs are deleted, locally and on GitHub. The original L0 pairs are in `D:\path-tracing-evidence\l0-original-pairs`.
- **Phase 1.** GPT has 6% of its weekly usage left (S55): Claude sub-agents build, one at a time, with GLM lanes alongside. To build next (S59): the lab wiki (lab/glm.md) and run records with worst margins (AGENTS.md).
  - **Done: law speed (1.1).** Under 1 µs per tick on all four bench cases (`cargo run --release --bin bench` from `engine/`), about 55×, replays identical, judged MERGE.
  - **1.2 footprints (`wt/footprints`, Opus builder):** adapter, determinism (no randomness, no clock in the law) and the pattern key done, with 8 of 8 planted faults caught. The Designer: FREEZE WITH CHANGES. The key is on reads projected per query, interned u16 ids, translation scope in the entry. The cache unit is the segment (representation.md, section 3). Changes in progress, then Sonnet.
  - **1.3 L0 (Terra, `wt/l0-close`):** sprint flag and sprint modifier made independent. The saved witness now replays all 420 rows; the old derivation fails at tick 204. A fresh live capture and the frozen verdict remain.
  - **Done: world reader (2.1a)**, merged as one commit: 81,920 course cells match, it refuses rather than guesses, and tags come from the jar. **Done: 2.1b**: all 18 tracks load (one rule from the jar; 30,951 of 32,366 states modeled, each refusal named; block data 90 KB as its generator). His track runs record T only, on purpose (S56): the bar, not training data.
  - **Swarm 0 (`wt/swarm-0`, parked by S55):** server-side `ServerPlayer` bots, packed together. Median ms per tick at 20 TPS: 0.64 for 1 bot, 0.99 for 10, 6.8 for 100 (p99 15.1); all three sustain 100 TPS. That measured server-side movement, the rejected design (Designer, 9 November): continuation 2 re-measures through the client-packet path, with costs by component. **One mind and lazy bots therefore move to the start of Phase 2.** The 1,000-bot and dispersed cells remain (`D:\path-tracing-evidence\swarm-0-20261109-d`).
  - **GLM `local-20261109-1523` (`lab\results`, local):** three lanes reproduced: the swimming gate's missing passenger and flight terms (integrate first, for Phase 3); falling blocks ticking at game time + 2 (Phase 4); knockback rewriting velocity (mobs). The fluid lane left a lead with a patch. The 7 November runs were lost in the cleanup: check `git status --ignored` before removing a worktree.
  - **GLM `local-20261109-1554` (nine lanes, 24% of the 5-hour quota):**
    - Reproduced, for Terra's next batch: the exporter classifies no `stepOn`, so magma never burns on exported terrain (patch, suite green); the source-conversion rule `getNewLiquid` has no producer (the checker says the patch over-converts falling amount-8 neighbours: fix that first); rider support for boats (Phase 4); the elytra start and the movement-mode inventory (26 checks).
    - Reproduced, for Phase 3: terrain-only c is 19.81 blocks/s (blue ice under a 2.0-block ceiling); six elite techniques proven with T values.
    - Leads: use-item slowdown; three tracks (`pillarhops`, `icemegajump`, `icecorner`) whose harness didn't finish.
  - **GLM `local-20261109-1653`:** the falling-block update contract, restated and reproduced (first schedule wins per position and type). Seven checkers died on GLM's 5-hour limit (HTTP 429), so their "lead" grades mean nothing; they rerun at 20:33. About 18 lanes use one GLM window.

## Roadmap (approved 9 November, S48)

Every phase ends in a **flight**, something Julien uses in the game, and has a **kill criterion** that re-derives the representation instead of patching it. After Phase 3 the scoreboard reorders the rest, largest gap first.

0. **One line of truth** (done): one branch, dead code deleted, the scaffolding on a diet, the docs rewritten.
1. **The ground.** Lanes in parallel:
   - **law speed:** allocation-free, bit-identical, ≤ 1 µs per tick, from 26–59 µs today (Terra);
   - **footprints and determinism:** randomness as input; the canonical pattern key is red-teamed (Sol);
   - **world reader:** [2.1a](../jobs/2.1a-region-reader/prompt.md) (Terra);
   - **close L0:** step-ups seed 17, tick 204 (Sol);
   - **the ground layer:** GLM lanes, one Mojang rule each, batched in by Terra.

   Flight: the bot's runs on his tracks replayed offline at ≥ 10⁴× real time; his T is the bar (S56). Kill: no path below about 5 µs means the `World` interface is wrong.
2. **One mind in the server, v0.**
   - Design: [server.md](server.md) (draft, red-teamed 9 November): bots that don't observe, world changes streamed once, a spatial index for player queries, bulk persistence, the vanilla client's packets, a self channel, lazy bots on quiet footprints, the engine as a separate process. Julien ruled (S52): bots are players (one sleeper, phantoms kept, on the locator bar). Chunk memory follows the area covered; it freezes after the packet-path re-measure.
   - Build: the deviation stream (every server-owned outcome predicted and compared, each mismatch a witness for GLM triage).
   - A sandbox server at 100× for R&D.

   Flight: Julien sees 10–50 bots as players (S53). The chunk-ticking wall for lone bots waits for Phase 5 (server.md, "Chunks").
3. **The vocabulary on legs, and the first flight.**
   - The word record, red-teamed and then frozen in `docs/vocabulary.md`.
   - The word compiler: gaits, transitions, compound words, a cache keyed by pattern.
   - The medium and fields: the light cone (bit-parallel, ES), relaxed bounds, landmarks as the far field.
   - The executor and the allocator: one queue ordered by slack; a bot is (word, phase) (ES, S59).

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
   - Congestion; lazy bots in the server; sparse versus dense evaluation, chosen by events per cell per light-crossing; a swarm as its mission program plus scars (ES, S59).

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

## Scoreboard

Rows begin with Phase 1:
- T over his time, and over the bracket's upper bound;
- first-move latency at p99;
- cost per bot, and server ms per tick at N bots;
- deviation rate per capability;
- words per regime, and the pattern-cache hit rate, from superflat to random terrain (ES's degradation benchmark);
- distinctions created per second (ES's H_live), the memory driver at scale;
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

Launch: `.\scripts\gpt.ps1 -Job <id> -Worktree <path> -Model gpt-6.1-sol|gpt-6.1-terra` (the prompt is `jobs/<id>/prompt.md` in this checkout; `-Resume <id>` continues a session). When it ends: read `jobs/<id>/last.md` and `git -C <worktree> status`, commit with `Built-by: <model>`, have a Sonnet judge review it in a fresh context, and send fixes back as a new job.

## Traps

- **Disk:** C: has about 8 GB free. The toolchain, builds and lab worlds go on D:.
- **Shared bot:** the bot's client is shared with Julien's play. Gates take the rig lock and never run while he drives the bot. Never build or run GPT jobs during a live clock gate: CPU contention makes the client lag and the server correct it (9 November).
- **Vanilla judge:** Lithium and C2ME change game logic, so they never go into the bot's client or the reference rig. Julien's own server may take a mod that keeps behaviour vanilla, once a need shows up (S55).
- **No Fable** (usage credits).
- **Codex sandbox:** no `%LOCALAPPDATA%` (use `scripts/stage-deps.ps1`), no git metadata.
- **Reference rig:** `PATHTRACING_ROOT=D:\path-tracing-lab`, port 25566. Run gate scripts with Windows PowerShell 5.1.
