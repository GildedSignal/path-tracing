# Now

What is in flight and what comes next. Rewritten freely; history lives in git.

## In flight

- **Flat ground: on `main`, flown in his game.** `plan-track --mode solve` writes the tick script the mod plays. The flights: `straight100` 288 predicted / 288 on the track clock, `straight300` 845 / 845, `straight800` 2,248 / 2,248, `stonesimple` 85 / 85 (his best 86). Per new command (`real_time.rs`, 60 each, every plan replayed by `law::try_tick` onto B at the predicted tick): from standing min 6.2 / median 13.5 / p90 18.6 / max 28.8 µs, while moving 7.0 / 12.7 / 16.1 / 23.1 µs, 0 refused, 0 law ticks prepared after the first command on a ground (119 µs, 15 law ticks). The track clock counts from the last tick on A (his 97 recorded times each +1).
- **Deltas the flights show (expected against seen):** (1) `plan-track` costs 1.8–2.3 s per click (a process and the world from disk). The stick runs `solve` in the bot's own process instead (`engine/native/src/resident.rs`: one living store per session, the corridor read from the chunks the bot has received), so a stick click costs microseconds. In his game (the bot driven by `goto` from the console): open ground reached (5 of 5, up to 46 blocks; the bot slides past B on its momentum after the touch); a way that crosses a fence is refused as `NeedsJoints` with the fence named, because a new site has no joints yet (step 3). Playback stops only on events that move the body (explosion, motion, position, rotation) or a block edit inside the plan's corridor; chunks arriving while it walks do not stop it. The corridor is the straight box between the bot and B plus 2 blocks; a click mid-jump takes the first solid block below as the ground; a click on its own block answers "Already there." Open: (a) he sees the bot lag in the air while jump-running, though the flights land on the predicted tick: unmeasured; (b) a block whose shape the law refuses ("non-local shape", stairs) panics the corridor instead of becoming an exception; (c) steps, fences and stairs are refused as `NeedsJoints`. (2) The planner's settled start has saturation 5.0, the bot has 20.0 (first deviation at tick 0 on every flight; T unaffected on these lengths, the food guard is not). (3) Ten new flat goals are not flown yet. Astra is back on the 23rd (Codex week reset, 20:34).
- **Sonnet judges on Antigravity: stopped by the budget guard.** Two at once took the Claude+GPT weekly meter from 87% to 74% left in about 10 minutes, before either reached a verdict; their sandbox cannot run commands ("non-absolute file path"). The hard-track judge's notes so far: the branch is ~450 lines of offline certificate the game does not judge plus ~56k lines of generated CSVs, and the witness T rose from 579 to 939 race ticks. The four prompts (`jobs/*-judge`) wait for his choice of judge.
- **Argon, `jobs/map-lint`:** the map against his verbatim turns, once.
- **The port branch, `wt/port-fold` (`b605c9b`, pushed to `origin`, `Sketch` state — stays off `main` until flown):** unifies the early Hydrogen builds plus the folded 26.2 port batches. Holds **32,252 / 32,366** modeled block states in `engine/world/data/census.tsv` (`99.65%`; only 114 dynamic shulker/moving-piston states remain non-local), **31,351** Rust lines in `engine/law/src/` across 42 files, and **9,117** bit-exact `server-26.2.jar` witnesses across 18 TSVs (`tests/jar.rs`). None of it is wired into `plan`/`bot/` or flown in-game yet.
- **Waiting local GLM dossiers (`lab/results/`, `jobs/glm-chain.txt` empty):** three finished dossiers wait for one closing fold onto `wt/port-fold` (`port-status-effect-tick-and-periodic` — lifting `WitherRose` — `port-item-entity-and-inventory`, and `port-shield-crit-sweep-and-mace`). Broad class-by-class GLM porting is frozen; future GLM lanes extract compact parameter rows, Mojang scars, and jar witnesses against the generator basis, plus short `lab/wiki/` notes.

## The overhaul (`jobs/overhaul/plan.md`, approved S240): state

Built and on `main`: the map (this folder; 22 KB loaded whole, history in `archive/`), AGENTS.md and the README rewritten, the hooks of `work.md` (card and his words on every message, the launch guard, the job watcher, the map and net-line commit checks, the push guard), the scoreboard (`scripts/scoreboard.ps1`), companion agency in `gpt.ps1` and `gemini.ps1`, GLM without its checker or lenses, tests cut to what the game judges, memory cut to 11 files, 220 dead worktrees and 51 branches archived (`D:\path-tracing-archive`, `archive/*` tags), history stripped from `map/detail/`. The meters: `/meters` opens a pane in every session (the mod in `C:\Users\PASCAL\.claude\mods\meters-band`, loaded by `CLAUDE_CODE_PLUGIN_DIRS` in `~/.claude/settings.json`; the app's own Context panel takes no mod content).

Checks against these fixes: `jobs/overhaul/checks.md`.

Left: delete the old A\* planner (`plan/src/flight.rs` modes 3/5/b, `planner.rs`) and dead `words` code, after Astra's job; Argon lints `map/` against his verbatim turns once.

## Next, in order

1. **Close the current port wave:** fold the 3 waiting local GLM dossiers onto `wt/port-fold` (~92% of non-mob laws, ~10k jar witnesses), then freeze speculative porting.
2. **Unbundle and compress the law onto its generator basis:** start with a `clean:` pass; collapse duplicate world traits (`World`, `BoatQuery`, `PortalWorld`, `SimWorld`) into one `World`/`WorldMut` interface that is multi-entity native, reversible via edit deltas, automatic on footprints, and observer-filterable — while keeping `Word` certificates pure (`found / forbidden / searched / refused`) so any future contact solver sits cleanly outside the crystal; factor duplicated physics onto ~5–6 shared generators (`affine_body`, `aabb_sweep`, `ray_dda`, resource/cooldown clocks, `cell_rewrite` + class-major event calendar) plus data tables and thin Mojang scars (keeping non-affine mechanisms like elytra, pistons, `PortalForcer` cleanly separate); split `PlayerState` into an interned static profile + compact hot state (~24–32 B logical view). Target: ~35K → ~10–15K lines so the whole law fits one context window and L1I cache, guarded bit-for-bit by the ~10k jar witnesses and `bench` ≤ `0.6–0.95 µs/tick`.
3. **Climb the live flight ladder on `main` (his 21 tracks + direct word runner):** delete the dead A\* planner (`flight.rs` modes 3/5/b, `planner.rs`), fix the 3 open flat-ground deltas, wire the ES audit wins (`jobs/es-audit/report.md`: arc certificates, local rejoin in `polish::slide`, `u16 model_id` corridor checks, zero-alloc compact nodes, lazy panels), and run `(word, count)` programs directly in `bot/` with event re-anchoring and mid-run retargeting (`R-retarget`).
4. **Wire compressed generators into `plan` and `bot/`, then fly tools and edits:** derive binade-exact event forms once per generator (never linear-derivative or nearest-LUT approximations), derive kit and entity classes from `(generator, params, scars)`, and merge `wt/port-fold` onto `main` slice by slice only after live flights pass (world edits & restoration → body swaps & wormholes → dynamic world & hazards).
5. **Far map, one mind, and 10,000 bots / 4D building:** medium bounds, `u64` bit-parallel light cones, lazy certified bots (`shipmem` demote/promote + class-major calendar), sparse-vs-dense switch (event certificates vs AVX-512 SIMD lockstep; Vulkan GPU only after a measured crossover), and `(max, +)` blueprint building. Once two-body flights measure the real certificate-thrashing boundary, use [pvp-strategy-2026-11-22.txt](../archive/research/pvp-strategy-2026-11-22.txt) for the two-regime contact singularity boundary, `(max, +)` hurt-time swarm scheduling, and fat-tailed supply buffering.
6. **Mobs and long-tail survival on demand:** pulled only when real flights or missions hit them, via compact parameter rows, scars, and jar witnesses.

## Derived component state (ES rule)

| Component | Derived state | Scope |
|---|---|---|
| Core `solve`, joints, flat-ground word store | `Load-bearing` | `main`, flown (`straight100/300/800`, `stonesimple`) |
| 21 tracks runner, ES audit wins, direct word executor | `Sketch` -> `main` next | closes flat deltas, removes dead planner code |
| `wt/port-fold` (26.2 non-mob laws, 9,117 jar rows) | `Sketch` (`origin/wt/port-fold`) | compresses onto generator basis, then merges per passing flight |
| Kit classes, edits & restoration, tools, dynamic world | Blocked on compression + track runner | pulled through one mechanism into `plan` and `bot/` |
| Far map, one mind (`10⁴` bots), dense SIMD/GPU, 4D build | After live tool flights | class-major calendar, lazy bots, `(max, +)` builds |

## Waiting for a judge or for him

- Branches under the judges above: the clicked face (`wt/track-face`), the rebuilt hard track (`wt/hard-track-2`), freeze damage (`wt/l0-veto`), the preview renderer (`wt/build-preview`).
- Parked: the refuge (`wt/hearth-small`, S228).

## Known anomalies (each blocks only what depends on it)

- Entity push sits unwired on `wt/port-fold` (`Sketch` in `entity_contact.rs` / `sim.rs`), not yet flown on `main` (five bots on one crossing).
- Sprint at an exactly axis-aligned wall depends on rounding in Mojang's own `acos` (`movement.rs:818`), ported faithfully.
- His bug S38: a track run deletes a sulfur spike from his inventory; never traced.
