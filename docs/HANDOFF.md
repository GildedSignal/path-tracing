# Handoff

This file holds the state of the build, what is in flight and what comes next. It is rewritten in place; history lives in git.

## First ten minutes

0. Read "In flight" below.
1. Read [PRINCIPLES.md](PRINCIPLES.md), then [AGENTS.md](../AGENTS.md), then this file.
2. Run `git status --short` and `git log -8 --oneline`.

## Now (5 November, midday)

- **Done today:** the recorder and dumper (1.2 to 1.2e) passed gate 1.2 live over RCON and are merged into `main` (report: `lab/gates/evidence/gate-1.2-20261105.md`; full evidence on `D:\path-tracing-evidence`).
- **Running:**
  - the Sonnet judge of job 1.5, the Rust law (`ef3d04b` on `wt/law`; 21 tests pass, four mutants build). It reviews the port against the bytecode before L0 runs; its findings become a job;
  - job 1.6 (GPT Sol) in `D:\path-tracing-wt\fixtures`: the L0 fixtures. The bot's client exports the world around each recording; courses for every terrain class; one recording command;
  - job L1 (GPT Sol) in `D:\path-tracing-wt\glm-lab`: GLM's harness on Actions.
- **Next:** run `record-fixtures.ps1` (Julien's client out), then L0 by a fresh judge with the four mutants and a red-team-chosen one (ladder.md), then merge `wt/law`.
- **After L1:** the integrator writes the first lenses, only where a target exists: a pre-freeze red-team of L0-x and of the legs ladder, jar questions for the elytra port; the breaker once fixtures exist. **Julien:** run `scripts\set-glm-key.ps1 -Sync` once to set the Actions secret.- **Integrator left from 1.1:** the lab server on D: (port 25566, RCON 25576) with fixture worlds; `/tick rate` in the gate runner.
- **When a job ends:** read `jobs/<id>/last.md`, then `git -C <worktree> status`, then commit on its branch with `Built-by: gpt-6.1-sol`. Relaunch with `.\scripts\gpt.ps1 -Job <id> -Worktree <path>`; answers go into its `prompt.md`.
- **Done:** the philosophy log (step 0); toolchain (1.0); lab hygiene (1.1, apart from the above); prior art (1.4: port from the bytecode, Azalea only as a reading guide; Baritone v1.19.0 runs on 26.2 for Phase 12); the representation's red-team; the register checks; ES's testing method (`ladder.md`) and the one-liner audit (S19).
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
