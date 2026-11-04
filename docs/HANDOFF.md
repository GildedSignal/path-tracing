# Handoff

This file holds the state of the build, what is in flight and what comes next. It is rewritten in place; history lives in git.

## First ten minutes

0. Read "In flight" below.
1. Read [PRINCIPLES.md](PRINCIPLES.md), then [AGENTS.md](../AGENTS.md), then this file.
2. Run `git status --short` and `git log -8 --oneline`.

## Now (4 November 2026)

- The lab works (see README). The algorithm slot holds `StraightLine`, which stays as the negative control.
- **Step 0 is done:** the philosophy log. That is PRINCIPLES (goal, principles, register), [engine.md](engine.md), [representation.md](representation.md) (a draft), [ladder.md](ladder.md), and [his words, verbatim](reference/julien-2026-11-04.md).
- **Waiting on Julien:** his approval of PRINCIPLES.md, which he owns. No builder starts before it.
- **Meanwhile:** Phase 1.0 (the toolchain) and 1.1 (lab hygiene), done by the integrator.

## Resuming after a compaction (4 November, about 23:00)

- **Running in the background** (both GPT Sol slots):
  - job 1.5, in `D:\path-tracing-wt\law`;
  - job 1.2b, in `D:\path-tracing-wt\recorder`.
- **When a job ends:** read `jobs/<id>/last.md`, then `git -C <worktree> status`. GPT can't commit, so commit its changes on its branch with `Built-by: gpt-6.1-sol`.
- **Relaunching:** run `.\scripts\gpt.ps1 -Job <id> -Worktree <path>`. Answers to a job's questions go into its `prompt.md` under "Answers…".
- **After 1.2b:**
  1. a fresh Sonnet judge;
  2. `.\scripts\setup.ps1`, which applies the bot's no-auto-input options;
  3. a live run of the 1.2 gate (`lab/gates/`);
  4. recording the L0 fixtures for 1.5;
  5. the merges.
- **Waiting on Julien:** may the bot also accept console commands over RCON for gates, or must his client be logged in?

## In flight

| Job | Who, where | Lands | Next action |
|---|---|---|---|
| Register completeness check | Haiku | done: 2 rows added (X1 "gather resources on think", S10) | — |
| Red-team of representation.md | Opus, fresh context | done: corrections approved (S14) and folded in | — |
| 1.2 Recorder and dumper | GPT gpt-6.1-sol, `D:\path-tracing-wt\recorder` on `wt/recorder` | built (ce09c2e). The Sonnet judge passed its exactness and flagged 12 fixes, now job `jobs/1.2b-recorder-fixes/` (running) | after 1.2b: a live run of the frozen gate in the rig, then the merge. **Needs Julien:** the bot only takes his whispers, so either his client is logged in during gates, or the bot also accepts console commands over RCON |
| 1.5 Movement law in Rust | GPT gpt-6.1-sol, worktree `D:\path-tracing-wt\law` on `wt/law`, job `jobs/1.5-law/` | `wt/law`, `jobs/1.5-law/last.md` | queued for 22:37 beside 1.2. Its L0 gate waits for 1.2's recorder and the fixtures |
| 1.4 Prior art | Sonnet, read-only | **done; decided: port from the bytecode, with Azalea as a reading guide** | Azalea (MIT, targets 26.2) mirrors vanilla's shape, merger and sweep code, but has gaps inside the bit-exact class: no friction or drag modifiers, 0.216/f³ applied to every block (the jar applies it only when f > 0.6), soul sand, honey and stuck factors still TODO, no exhaustion, no tick-by-tick test. Pumpkin (GPL) doesn't simulate players. Parkour Calculator (GPL) runs the game's own entity. Baritone has a 26.2 build (v1.19.0, LGPL), so Phase 12 can run |
| 1.0 Toolchain | integrator | **done:** rustc and cargo 1.99.1, stable-x86_64-pc-windows-gnu; `RUSTUP_HOME` and `CARGO_HOME` under `D:\rust`; a hello-world builds | — |
| 1.1 Lab hygiene | integrator | **done:** vanilla judge (Lithium and C2ME only in Julien's client); `$Rig` from `PATHTRACING_ROOT`; `Enter-Rig`/`Exit-Rig` lock in `scripts/common.ps1`. **Left:** the lab server on D: (port 25566, RCON 25576) with pre-generated fixture worlds, and `/tick rate` in the gate runner | next integrator step, then the 1.2 job file |

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
3. Movement only, in an unmodified world: every legal way to move (sneak edges, ladders, water, slime, crawling under existing trapdoors). This replaces edits as conjugates (S13).
4. Scale: the server observer, portal tables, Hashlife, local repair, light-cone scheduling.
5. Verticals: the bucket.
6. Momentum ground: ice, slime, soul sand, honey.
7. Tokens: pearls, food, health, durability.
8. The moving world: fluids, falling blocks, mobs, redstone.
9. Risk: edge width under jitter, Monte Carlo, irreversible losses.
10. Speed: Vulkan, threads, learned ordering that only schedules.
11. Far travel: boats, the Nether, elytra.
12. The benchmark against Baritone.

After the first flight, the scoreboard reorders phases 3–12, furthest gap first.

Every capability passes the universal gate: it enters as law and toolkit data, with no new search code.

## Decisions (Julien, 4 November)

- Port the law to Rust first. Mojang's code in the bot's client stays the judge.
- Restoration means the identical block and block state. Without Silk Touch, blocks whose drop differs are barriers.
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

## Proposals awaiting Julien

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
