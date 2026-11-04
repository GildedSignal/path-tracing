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

## In flight

| Job | Who, where | Lands | Next action |
|---|---|---|---|
| Register completeness check | Haiku | done: 2 rows added (X1 "gather resources on think", S10) | — |
| Red-team of representation.md | Opus, fresh context | proposals, below | Julien decides |
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
3. Edits as conjugates: pickaxe and blocks, identical-block restoration.
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

These come from the Opus red-team of [representation.md](representation.md) (4 November). The draft stands, challenged, until these are settled.

**Decisions only he can make:**
1. **His void bridge (S6) against "identical block" (S12).**
   - A block placed under the feet over the void drops into the void when it is broken back.
   - Stone drops cobblestone, grass drops dirt, gravel sometimes drops flint.
   - Placed leaves become persistent, so they never match the originals, even with Silk Touch.
   - Pillars taller than the 4.5-block reach can't be removed from the top.

   Strict identity makes most edits barriers in default worlds.
2. **Consumables in the scoreboard.** As written, a pearl that saves one tick is always worth throwing. Should pearls, food and durability carry a price in ticks? (Proposal: health and food become feasibility constraints, not tokens.) One more thing to weigh: a pearl spawns an endermite 5% of the time when the difficulty isn't peaceful, which is an irreversible change to the world.

**Corrections to the draft (fold in once he has read them):**
- **Claim 2 wrongly hid tool use.** Placing is "just a movement", so tool use is an ordinary priced edge. Each edit must close inside its own edge (open, cross, close), so that border states never carry open edits.
- **Claim 4 was wrong: speed matters on ordinary ground too.** Sprint-jumping (7.1 m/s) is the fastest gait on stone, and air drag is 0.91, so speed takes about 32 ticks to fade. Speed belongs in the state wherever sprint-jumping wins, which is nearly everywhere.
- **Claim 5 missed exact merge points.** `aiStep` zeroes velocity below its thresholds (about 8 ticks after the bot stops on stone), collisions clamp the position to the face, and landing snaps the height. States merge exactly at stops, contacts and landings; widths are needed only in between.
- **Claim 1: crawling is not a land mode.** It needs a trapdoor or water, so it is a toolkit entry. The body folding is the exact inflation of the collision boxes for each pose, because the bot can stand with 0.3 blocks overhanging an edge.
- **"Which math":** drop the group, commutator and Lie-bracket language. The moves are partial maps that don't commute, and T is min-plus over sequences of them: a weighted shortest path (principle 6).
- **Claim 8:** share tables at sub-chunk cells and key them by toolkit class, not by counts. Hashlife pays little on default terrain.
- **Claim 9: the lower bound must be computed per toolkit.** Ice boats, pearls thrown while moving and the Nether's ×8 each beat the current cone. Portals wait 80 ticks, and generating a new destination portal is an edit that can't be restored.
- **Claim 11: the target is a goal-rooted time-to-go field** over the tables, with greedy descent: his straight line and his mug, rather than A* over an exploding state.
- **Name the real noise before gating on it:** tick misalignment, server RNG, block-update latency. Keep R1–R3 short enough that their upper bound is exact.

## Traps

- **Disk:** C: has about 10 GB free. The toolchain, builds and lab worlds go on D:.
- **Shared bot:** the bot's client is shared with Julien's play. Gates take the rig lock and never run while he is driving the bot.
- **Vanilla judge:** Lithium and C2ME change game logic, so they never go into the bot's client or a server.
- **No Fable:** Fable sub-agents fail on this account (usage credits).
- **Codex path:** the Codex CLI lives inside the Codex app, at `%LOCALAPPDATA%\OpenAI\Codex\bin\<hash>\codex.exe`, and the hash folder changes with app updates.
- **Server stdin:** commands piped into the server's stdin before the world has loaded fail (README).
- **Planning thread:** planning on the client's main thread freezes the bot's window. The bridge moves it out.
