# Handoff

This file holds the state of the build, what is in flight and what comes next. It is rewritten in place; history lives in git.

## First ten minutes

0. Read "Now" below: it says what is running and what each branch needs next.
1. Read [PRINCIPLES.md](PRINCIPLES.md), then [AGENTS.md](../AGENTS.md), then this file.
2. Run `git status --short` and `git log -8 --oneline`.

## Now (5 November, 14:45)

**Running:**
- GPT job 1.5d on `wt/law` (launched 14:25);
- GPT job 1.7b on `wt/tracks` (launched 15:00): the 1.7 judge's findings ("ready with fixes", `jobs/1.7-tracks/judge.md`);
- the Sonnet judge for 1.6b (it writes `jobs/1.6b-fixture-fixes/judge.md`). If the session was restarted, check `jobs/<id>/log.jsonl` timestamps and relaunch by hand with `.\scripts\gpt.ps1 -Job <id> -Worktree <path>`. The S32 research is routed (claims 6 and 9, R6, R-far, the rediscovery rung, the speed-of-light lens). The next sweep is the GLM lens `elite-techniques` (S33).

**Queued:**
- **job 1.8** (`jobs/1.8-client-clock/prompt.md`): the 26.2 client never ticks faster than 20 per second, so `/tick rate` doesn't speed up the bot. Launch it once `wt/fixtures` is merged, in a worktree cut from `main`.

**Milestone 1, Julien races his tracks:**
1. The judges for 1.7 and 1.6b, then any fixes.
2. Merge `wt/fixtures` into `main`, then `wt/tracks`. `wt/tracks` was cut from an older `wt/fixtures`, so expect conflicts in the recorder.
3. Rebuild both clients (`scripts\setup.ps1`).
4. A live smoke test by the integrator, then tell Julien.

The fixture tooling merges once judged; L0 gates the law, not the tooling.

| Branch and worktree (`D:\path-tracing-wt\...`) | State | Next |
|---|---|---|
| `wt/law` (`law`) | the Rust law, jobs 1.5 to 1.5c committed (`0f6c901`; 39 tests pass, four mutant builds). Judged "ready with fixes"; fixes done | job **1.5d** running (the reader meets the real recordings).<br>**Field mismatch:** 1.5d's spec accepts `ticksAfterEnd`, but 1.6b named the field `serverTickOffset`: null when the read straddles ticks, plus `serverTickOffsetMin`/`Max` (`bot/recorder.md` on `wt/fixtures`).<br>1.6b also left a 101-line reader patch, `jobs/1.6-l0-fixtures/law-reader.patch` on `wt/fixtures`. After 1.5d lands, a small job reconciles both; don't apply the patch by hand.<br>Then record fixtures live, then L0 |
| `wt/fixtures` (`fixtures`) | job 1.6b committed (`684946e`): abort cleanup, rimmed water with a flow check, one-tick server reads, `ticksFrozen`, the `corner` course, edges, exporter values. It adds `EntityCollisionMixin` to observe collisions, and the judge must confirm it only observes | its judge, then merge, then `record-fixtures.ps1` live (the server is ours to use, S29), then L0 by a fresh judge |
| `wt/tracks` (`tracks`), cut from `wt/fixtures` at `b3d3297` | job 1.7 committed (`00fc6d8`): `/track` commands, one detector for both clients, run records, the bot's replay with the current planner; 69 offline checks | judged "ready with fixes"; job **1.7b** running. Merge after `wt/fixtures` |
| `main` | recorder merged (gate 1.2 passed live), GLM harness merged | — |

- **L0's planted faults:** the axis tie gets the scripted corner course (1.6b). The step-up fault (+1 ulp) looks unreachable with vanilla blocks (GLM lens `step-up-reachability` settles it); if so, the red-team replaces it at L0's freeze, which repairs the control and doesn't weaken the gate.
- **GLM:** first run (42008039888, 11:07) failed: every request got Z.ai error 1113. Probed directly (5 November): every chat endpoint (Anthropic, coding paas/v4, general paas/v4) and model (glm-5.3, glm-4.7) refuses with 1113, while the quota and subscription endpoints answer. So it is the account's entitlement, not omp, the model or the endpoint. Lead: the annual GLM Coding Pro (id 170955) shows `paymentType: WAIT_PAY` for its current period, and the two paid monthly Pro orders are scheduled from 2027-04-07. **Julien checks his Z.ai billing.** Queued lenses after that: `speed-of-light`, `track-designer`.
- **GLM harness:** merged (`.github/workflows/glm-lab.yml`, started only by hand). Three lenses exist (`lab/lenses/`): `hook-catalogue`, `step-up-reachability`, `elytra-law`, all jar questions with checkable answers. Dispatch: `gh workflow run glm-lab.yml -R GildedSignal/path-tracing -f lenses="hook-catalogue step-up-reachability elytra-law" -f minutes=60`; results come back as the artifact `glm-lab-results` (`gh run download`). The Actions secret is set. **First run dispatched 5 November 11:07** (run 42008039888, all three lenses, 60 minutes each): download its artifact, read only the `DONE` lines and the reproduced results.
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
