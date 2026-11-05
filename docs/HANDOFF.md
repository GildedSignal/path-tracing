# Handoff

This file holds the state of the build, what is in flight and what comes next. It is rewritten in place; history lives in git.

## First ten minutes

0. Read "Now" below: it says what is running and what each branch needs next.
1. Read [PRINCIPLES.md](PRINCIPLES.md), then [AGENTS.md](../AGENTS.md), then this file.
2. Run `git status --short` and `git log -8 --oneline`.

## Now (5 November, about 11:00; written before a compaction)

**Nothing is running right now.** GPT is at its usage limit until 14:00. Two background shells in the main session will relaunch at 14:05 and 14:06; if the session was compacted or restarted, they may have died, so check `jobs/<id>/log.jsonl` timestamps and relaunch by hand with `.\scripts\gpt.ps1 -Job <id> -Worktree <path>`.

| Branch and worktree (`D:\path-tracing-wt\...`) | State | Next |
|---|---|---|
| `wt/law` (`law`) | the Rust law, jobs 1.5 to 1.5c committed (`0f6c901`; 39 tests pass, four mutant builds). Judged "ready with fixes"; fixes done | job **1.5d**: the replay reader meets the real recordings (the `end` trailer, `serverEnd` timing, new world fields, powder-snow frost). Then L0 |
| `wt/fixtures` (`fixtures`) | job 1.6, the fixture recorder (`b3d3297`). Judged **not ready** | job **1.6b** (queued for 14:06): a failed run must not end the session, water floods the start, one-tick server reads, powder snow, the corner course for the axis tie, edges, exporter values. Then its judge, then `record-fixtures.ps1` live (the server is ours to use, S29), then L0 by a fresh judge, then merge `wt/fixtures` and `wt/law` |
| `wt/tracks` (`tracks`), cut from `wt/fixtures` | job **1.7**, parkour tracks (S25 to S28), not started: GPT's limit stopped it | relaunch queued for 14:05. Merge after `wt/fixtures` |
| `main` | recorder merged (gate 1.2 passed live), GLM harness merged | — |

- **L0's planted faults:** the axis tie gets the scripted corner course (1.6b). The step-up fault (+1 ulp) looks unreachable with vanilla blocks (GLM lens `step-up-reachability` settles it); if so, the red-team replaces it at L0's freeze, which repairs the control and doesn't weaken the gate.
- **GLM:** the harness is merged (`.github/workflows/glm-lab.yml`, started only by hand). Three lenses exist (`lab/lenses/`): `hook-catalogue`, `step-up-reachability`, `elytra-law`, all jar questions with checkable answers. Dispatch: `gh workflow run glm-lab.yml -R GildedSignal/path-tracing -f lenses="hook-catalogue step-up-reachability elytra-law" -f minutes=60`; results come back as the artifact `glm-lab-results` (`gh run download`). The Actions secret is set. **First run dispatched 5 November 11:07** (run 42008039888, all three lenses, 60 minutes each): download its artifact, read only the `DONE` lines and the reproduced results.
- **Integrator left from 1.1:** the lab server on D: (port 25566) with fixture worlds; `/tick rate` in the gate runner. The superflat server is running and ours to use (S29).
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
3. Movement only, in an unmodified world: every legal way to move (sneak edges, ladders, water, slime, crawling under existing trapdoors). This replaces edits as conjugates (S13).
4. Scale: the server observer, portal tables, **multipoles** (a tracked request, see below), Hashlife, local repair, light-cone scheduling.
5. Verticals: the bucket.
6. Momentum ground: ice, slime, soul sand, honey.
7. Tokens: pearls, food, health, durability.
8. The moving world: fluids, falling blocks, mobs, redstone.
9. Risk: edge width under jitter, Monte Carlo, irreversible losses.
10. Speed: Vulkan, threads, learned ordering that only schedules.
11. Far travel: boats, the Nether, elytra with fireworks.
12. The benchmark against Baritone.

After the first flight, the scoreboard reorders phases 3–12, furthest gap first.

Every capability passes the universal gate: it enters as law and toolkit data, with no new search code.

## Decisions (Julien, 4 November)

- Port the law to Rust first. Mojang's code in the bot's client stays the judge.
- Getting around never modifies the world; a tool use is legal only if it leaves no trace (S13, S14; supersedes "identical block").
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
- **Getting around never modifies the world.** A tool use is legal only if the world is identical once it is undone: the bucket's water destroys nothing, creates no source and leaves nothing behind.
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
