# Handoff

This file holds the state of the build, what is in flight and what comes next. It is rewritten in place; history lives in git.

## First ten minutes

0. Read "Now" below: it says what is running and what each branch needs next.
1. Read [PRINCIPLES.md](PRINCIPLES.md), then [AGENTS.md](../AGENTS.md), then this file.
2. Run `git status --short` and `git log -8 --oneline`.

## Now (6 November)

**Milestone 1 is ready: Julien can race his tracks.** `wt/tracks` (1.7, 1.7b) is merged into `main` (`794e2fe`), the mod is rebuilt and installed in both clients, and a live smoke test passed (a hand-written straight track; the bot finished in 87 ticks with the placeholder planner, run line and recording written; test files removed). His tracks and run times live in `lab/tracks/` and are committed as the human baseline; recordings are ignored.

- **Supervised jobs.** Julien wants to oversee the prepared work. The track kit (1.7c) is merged (`e098a09`) and installed. The inventory-loss fix (1.7e) is on `wt/tracks` (`7ded970`), with a live round trip on the reference rig. Track labels (1.7d) and the region reader (2.1a) are queued. Run one builder at a time.
- **Representation and pathfinding are open.** Julien is thinking through them. Codex owns the port, lab bugs, and swarm runtime work. Julien prioritizes the full port, then one headless bot. Use scripted controls for runtime tests.
- **GLM port probes.** The `local-20261107-2245` batch formally reproduced runtime pair push, scheduled-tick ties, mining target switch and larger damage inside cooldown. The cosine checker reproduced the numerical difference despite a lead grade. Six new probes are running as `local-20261107-2310` on `wt/glm-lab` (`c3d5337`).

**Julien's double-clicks** (README): `Play.cmd`, `Play with bot.cmd`, `Set GLM key.cmd`. The L0 courses are built in the superflat at x = 4096 + 128·i, with teleport buttons at spawn and on each roof (`scripts\place-teleports.ps1`; it sources `lab/gates/courses.ps1`, which on `main` is still 1.6's version until `wt/fixtures` merges).

| Branch and worktree (`D:\path-tracing-wt\...`) | State | Next |
|---|---|---|
| `wt/law` (`law`) | Grounded contact verified against standing vanilla traces (`2712dc1`). Mining clock and target switch committed at `b63ecfd` after the JVM comparison and mutant. | Live mining packet and tick ordering, and crouched and 2,000 TPS damage checks. |
| `wt/world` (`world`) | Raw current committed at `01c78f8`; ordered pair push committed at `a28f2de`. App Terra raw snapshot bridge is `2a50493` on `wt/raw-fluid-export`. Its offline and saved mixed replays passed. | Ordered entity world producer and live raw client capture. |
| `wt/l0` (`fixtures`) | [1.6g](../jobs/1.6g-l0-authoritative-corpus/prompt.md) committed (`1d7dedf`). Mixed terrain passed at both rates. Full first refused at step-ups seed 17, tick 204 speed. | App Terra built a sprint flag and attribute provenance probe. The isolated one-case reduction runs from `D:/path-tracing-target/sprint-diagnostic-source-20261107T231317`. The frozen full verdict remains refused. |
| `wt/clock` (`clock`) | 1.8 live gate passed with the L0 recorder (`820f55b`); report `lab/gates/results/20261106T1243013521080Z-clock/report.md` in the worktree | 1.8c committed (388a1bc). Focused lab passed 2,000 TPS movement and measured 100× active client throughput. 4,000 TPS stress hit vanilla corrections. Cross-family review before merge |
| `main` | recorder (gate 1.2 passed live), GLM harness, tracks (1.7, 1.7b) | — |

- **L0's planted faults:** all four existing mutants fail at their intended live movement witnesses using `lab/gates/check-l0-controls.ps1`. The step-up witness uses a half slab and a vanilla step height one ULP below the slab. Evidence is under `lab/gates/results/20261106T1237131737162Z-l0-controls` in `wt/l0`.
- **L0 preflight:** the sprint fix made all three original pairs movement-exact. The normal-difficulty full-blocks rerun also matched server resources. The peaceful full-blocks end saturation still differs. The passive probe confirmed that peaceful refills follow the server player's own tick phase. Preserve the original pairs. L0 stays open.
- **Accelerated terrain preflight:** raw recordings replayed independently using server-authoritative start saturation. See `D:/path-tracing-evidence/20261106T1256280752904Z-terrain-replay/results.csv` for exact matches and unqualified end snapshots. The edge course's motion packet also occurs at the stock rate, tracked as [1.6e](../jobs/1.6e-landing-motion/prompt.md).
- **GLM harness:** `.github/workflows/glm-lab.yml`, started only by hand. Dispatch: `gh workflow run glm-lab.yml -R GildedSignal/path-tracing -f lenses="<names>" -f minutes=60`. Results come back as the artifact `glm-lab-results`.
- **Reference rig:** `PATHTRACING_ROOT=D:\path-tracing-lab`, `PATHTRACING_SERVER_PORT=25566` on `wt/l0` and `wt/clock`. Run gate scripts with Windows PowerShell 5.1. Use the clock candidate only on the isolated rig until reviewed.
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
| Hundreds of bots on one local server with shared work (S18, S35–S36, 6 November) | Codex integrator | swarm runtime and bot server, with the vanilla lab as reference | [Swarm 0](../jobs/swarm-0-headless-scale/prompt.md) measures scripted bots spread out and packed together. Share world reads and expose tick-stamped changes for later invalidation. |
| 100× reference iteration | Codex integrator | [1.8b clock](../jobs/1.8b-clock-100x/prompt.md) | The standard and focused matrices match stock movement. [1.8c](../jobs/1.8c-clock-demand/prompt.md) qualified 100× active client throughput with the frame clamp disabled. Live report in the clock worktree at lab/gates/results/20261106T1810018200214Z-clock-full-demand/report.md. |

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
