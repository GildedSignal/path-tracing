# Handoff

This file holds the state of the build, what is in flight and what comes next. It is rewritten in place; history lives in git.

## First ten minutes

0. Read "Now" below: it says what is running and what each branch needs next.
1. Read [PRINCIPLES.md](PRINCIPLES.md), then [AGENTS.md](../AGENTS.md), then this file.
2. Run `git status --short` and `git log -8 --oneline`.

## Now (5 November, 15:00)

**Milestone 1 is ready: Julien can race his tracks.** `wt/tracks` (1.7, 1.7b) is merged into `main` (`794e2fe`), the mod is rebuilt and installed in both clients, and a live smoke test passed (a hand-written straight track; the bot finished in 87 ticks with the placeholder planner, run line and recording written; test files removed). His tracks and run times live in `lab/tracks/` and are committed as the human baseline; recordings are ignored.

**While Claude's weekly usage is out (from 5 November, 15:20)**
- **GPT hit its usage limit at 16:00; it resets at 19:06.** Job 1.8 stopped partway (its changes stay in `wt/clock`). The queue is reordered: 1.7e (an item lost after a track, S38), 1.7d (track names), 1.8, 2.1a. Julien double-clicks `Run queue.cmd` after 19:06.
- **GPT queue.** `Run queue.cmd` runs `jobs/queue.txt`: 1.8 (client clock, `wt/clock`) and 2.1a (region reader, `wt/world`). It never runs more than 2 GPT jobs at once, and commits each finished job on its own branch, unjudged. `Stop GPT.cmd` stops them. After a shutdown, run the queue again: finished jobs are skipped. Job 1.6c is committed on wt/fixtures (822b068). It flags one issue to fix first: it applied "hooks are unmodeled" literally, so slime, hay and beds now export inside=unmodeled although the law models their landing, and the reader rejects those fixtures. The exporter must mark only hooks the law does not model. Job 1.7c (the track kit) is complete: merged into main (e098a09), 394 + 101 offline checks and 24 audited injections, jar installed in both clients. Not yet tried live: Julien is the first to test it. Always check last.md before committing a job.
- **On return, in order:**
  1. Commit 1.6c and 1.7c.
  2. Merge 1.7c into `main` and smoke-test the kit live (Julien is waiting for it).
  3. Run a Sonnet judge on each queued job.
  4. Merge `wt/fixtures`, then record the fixtures live, then L0.
- **GLM now runs on this PC** (5 November, 16:50). GitHub's runner got "Connection error" twice (runs 42038557893 and 42043226109, no dossiers). omp v18.16.0 for Windows (SHA-256 verified) is in D:\path-tracing-deps\omp, and scripts\glm-local.ps1 runs lenses through lab/actions.sh. Julien starts the deep research with Run GLM research.cmd (lens foundations-primary, S42). Results land in lab\results\local-*: check the anchors with its reproduce.sh, then read the dossier. Re-run the earlier six lenses the same way.
- **GLM.** Run 42038557893 failed: every call got "Connection error" from omp on GitHub's runner, with the new key. That is a network failure: the same key and endpoint work from this PC, and this morning's run did reach Z.ai (error 1113). Run 42043226109 (`speed-of-light`, `track-designer`, `elite-techniques`) was dispatched at 15:15. If it fails the same way, Z.ai is probably refusing GitHub's network: run the lenses on this PC instead (`lab/run.sh` under Git Bash, with the key from `Get-GlmKey`).

**Julien's double-clicks** (README): `Play.cmd`, `Play with bot.cmd`, `Set GLM key.cmd`. The L0 courses are built in the superflat at x = 4096 + 128·i, with teleport buttons at spawn and on each roof (`scripts\place-teleports.ps1`; it sources `lab/gates/courses.ps1`, which on `main` is still 1.6's version until `wt/fixtures` merges).

**Running** (if the session was restarted, check `jobs/<id>/log.jsonl` timestamps and relaunch by hand with `.\scripts\gpt.ps1 -Job <id> -Worktree <path>`):
- GPT job **1.6c** on `wt/fixtures`: the 1.6b judge's findings ("ready with fixes", `jobs/1.6b-fixture-fixes/judge.md`). An outer corner so the axis tie can be caught, runs ending at rest, a stable start state, server-driven state recorded per tick, the obsolete reader patch deleted.
- GPT job **1.7c** on `wt/tracks` (fast-forwarded to `main`): the track kit (S34). Survival with the kit on A, his inventory and creative back at B, crash recovery. When it lands, merge it into `main`, rebuild both clients, then a smoke test. `play.ps1` sets `keep_inventory` on each start.
- GLM run 42038557893 (below).

**Queued:** job **1.8** (`jobs/1.8-client-clock/prompt.md`, the client clock), once `wt/fixtures` is merged, in a worktree cut from `main`.

| Branch and worktree (`D:\path-tracing-wt\...`) | State | Next |
|---|---|---|
| `wt/law` (`law`) | jobs 1.5 to 1.5e committed (`9fc0842`; 56 tests, four mutant builds). Server-driven state is input, `movementSpeedBase`, end-of-run replay; the axis-tie mutant now fails the outer-corner test at tick 22 | after 1.6c: record fixtures live, then L0 |
| `wt/fixtures` (`fixtures`) | job 1.6b committed (`684946e`), judged "ready with fixes" | job **1.6c** running. Then merge into `main` (conflicts expected in the recorder, since `main` now has the tracks), then `record-fixtures.ps1` live, then L0 by a fresh judge |
| `main` | recorder (gate 1.2 passed live), GLM harness, tracks (1.7, 1.7b) | — |

- **L0's planted faults:** the axis tie gets the scripted corner course (1.6b). The step-up fault (+1 ulp) looks unreachable with vanilla blocks (GLM lens `step-up-reachability` settles it); if so, the red-team replaces it at L0's freeze, which repairs the control and doesn't weaken the gate.
- **GLM:** working since 14:44 on 5 November, with Julien's new key. The first run (42008039888) had failed with Z.ai error 1113 on the old key's account. `Set GLM key.cmd` stores the key and tests it, but its copy to the Actions secret didn't take on his run (cause unknown); `scripts\set-glm-key.ps1 -Sync` fixed it. **Run 42038557893 is running** (dispatched 14:45): `hook-catalogue`, `step-up-reachability` and `elytra-law`, 60 minutes each.
  - When it ends, `gh run download 42038557893 -R GildedSignal/path-tracing`; read only the `DONE` lines and the reproduced results.
  - Next batch: `speed-of-light`, `track-designer`, `elite-techniques` (S33).
- **GLM harness:** `.github/workflows/glm-lab.yml`, started only by hand. Dispatch: `gh workflow run glm-lab.yml -R GildedSignal/path-tracing -f lenses="<names>" -f minutes=60`. Results come back as the artifact `glm-lab-results`.
- **Integrator left from 1.1:** the lab server on D: (port 25566) with fixture worlds; `/tick rate` in the gate runner, which needs job 1.8 first. The superflat server is running and ours to use (S29).
- **When a job ends:** read `jobs/<id>/last.md`, then `git -C <worktree> status`, then commit on its branch with `Built-by: gpt-6.1-sol`, then a Sonnet judge in a fresh context (one Claude sub-agent at a time), then fixes as a new job.
- **Done:** the philosophy log; toolchain (1.0); lab hygiene (1.1, apart from the above); recorder and dumper (1.2, gate passed live, report `lab/gates/evidence/gate-1.2-20261105.md`, evidence on `D:\path-tracing-evidence`); prior art (1.4); the Rust law ported and fixed (1.5 to 1.5c); ES's testing method; the one-liner audit; least-time claims 13 and 14; GLM's harness.
## The roadmap (approved 4 November; one sub-agent per family at a time)

1. **The law.**
   - toolchain;
   - lab hygiene: a vanilla judge, fixture worlds on D:, `PATHTRACING_ROOT`, a rig lock, `/tick rate`;
   - recorder and dumper in the bot mod (GPT);
   - catalogue (Haiku);
   - prior art (Sonnet);
   - the movement law in Rust (GPT, judged by Sonnet against L0).
2. **First flight, legs only.**
   - border-state probe, before any geometry;
   - bridge and region-file reader;
   - the bracket;
   - geometry v0;
   - search v0;
   - executor v0 (tick-stamped controls, local re-solve).

   Gate: R1–R3, re-targeting, and the scale rung (a baselined red until Phase 4).
2b. **The brachistochrone (S17, S18):** port the elytra law (GPT, judged against new L0 fixtures, including its `Math.cos` class), then rung R-brach. It enters as law data only. It is the method's first falsifier beyond the legs, run before scaling out.
3. Movement only, in an unmodified world: every legal way to move (sneak edges, ladders, water, slime, crawling under existing trapdoors). The simplest case of the restoration rule.
4. Scale: the server observer, portal tables, **multipoles** (a tracked request, see below), Hashlife, local repair, light-cone scheduling.
5. Edits restored by arrival (S32): the bucket (anything its water breaks repaired), trapdoors opened and put back, silk-touch breaking and rebuilding. Edits as conjugates whose undo lands before T stops.
6. Momentum ground: ice, slime, soul sand, honey.
7. Tokens: pearls, food, health, durability.
8. The moving world: fluids, falling blocks, mobs, redstone.
9. Risk: edge width under jitter, Monte Carlo, irreversible losses.
10. Speed: Vulkan, threads, learned ordering that only schedules.
11. Far travel: boats, the Nether (a portal kit, both portals taken down by arrival, S32), elytra with fireworks. R-far's 10,000-block trip is the named goal (S32).
12. The benchmark against Baritone and the other bots, on their known failure modes.

Later, logged only (S32): multiple objectives (gather x wood); an optional human-limits profile (velocities and inputs capped at what Julien can do).

After the first flight, the scoreboard reorders phases 3–12, furthest gap first.

Every capability passes the universal gate: it enters as law and toolkit data, with no new search code.

## Decisions (Julien, 4 November)

- Port the law to Rust first. Mojang's code in the bot's client stays the judge.
- The world must be the same once the bot reaches B (S32, rewording S13 and S14): edits on the way are allowed if restored by arrival, with the identical block and state.
- GPT takes well-formed tasks and Claude takes those where taste decides. No Fable.
- He approved the plan's defaults:
  - T in server ticks, judged against a bracket;
  - a vanilla judge (no Lithium or C2ME in the bot's client and the server);
  - fixture worlds on a lab server on D:;
  - the engine as a separate Rust process;
  - the GNU toolchain on D:;
  - Vulkan for integer work.

## Scoreboard

Empty until the first flight. Each row will show the goal, the value now, and the gap in orders of magnitude.

## Tracked requests (AGENTS.md: a request made twice gets an owner)

| Request | Owner | Where it lands | State |
|---|---|---|---|
| Multipoles (FMM) for how T acts at a distance (S6; about ten times in ES, never built there) | Opus designer, at Phase 4's design | far influence as a compact summary with a stated remainder bound, beside the exact portal tables | not started; never deferred "until the prerequisites", and every rung reports cost against N from the first flight |

## Proposals awaiting Julien

**Settled on 5 November (S20):** ES 8 (cathedral for the law and the representation, the SpaceX loop above them); mobs are precomputed.


**Open:** the price of consumables (S13, unsure). Until he decides, a command grants a budget, zero by default; health and food are feasibility constraints.

**Settled on 4 November (S13, S14):**
- **Getting around never modifies the world** (reworded by S32 on 5 November: the world must be the same once the bot reaches B).
- **Braking** is part of the law.
- **A block lost to the void** is a spent token.
- **The red-team's corrections** are folded into [representation.md](representation.md) (approved).
- **His own A to B runs** are the human baseline in [ladder.md](ladder.md).

## Traps

- **Disk:** C: has about 10 GB free. The toolchain, builds and lab worlds go on D:.
- **Shared bot:** the bot's client is shared with Julien's play. Gates take the rig lock and never run while he is driving the bot.
- **Vanilla judge:** Lithium and C2ME change game logic, so they never go into the bot's client or a server.
- **No Fable:** Fable sub-agents fail on this account (usage credits).
- **Codex sandbox:** it can't read `%LOCALAPPDATA%`. Builds use the copy that `scripts/stage-deps.ps1` puts in `D:\path-tracing-deps\data` (`PATHTRACING_DEPS`), and `gpt.ps1` opens that folder, `D:\rust` and `D:\path-tracing-target` to the job. Re-stage after a game or JDK update.
- **Codex path:** the Codex CLI lives inside the Codex app, at `%LOCALAPPDATA%\OpenAI\Codex\bin\<hash>\codex.exe`, and the hash folder changes with app updates.
- **Server stdin:** commands piped into the server's stdin before the world has loaded fail (README).
- **Planning thread:** planning on the client's main thread freezes the bot's window. The bridge moves it out.
