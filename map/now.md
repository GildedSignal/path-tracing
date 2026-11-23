# Now

What is in flight and what comes next. Rewritten freely; history lives in git.

## In flight

- **Flat ground: on `main`, flown in his game.** `plan-track --mode solve` writes the tick script the mod plays. The flights: `straight100` 288 predicted / 288 on the track clock, `straight300` 845 / 845, `straight800` 2,248 / 2,248, `stonesimple` 85 / 85 (his best 86). Per new command (`real_time.rs`, 60 each, every plan replayed by `law::try_tick` onto B at the predicted tick): from standing min 6.2 / median 13.5 / p90 18.6 / max 28.8 µs, while moving 7.0 / 12.7 / 16.1 / 23.1 µs, 0 refused, 0 law ticks prepared after the first command on a ground (119 µs, 15 law ticks). The track clock counts from the last tick on A (his 97 recorded times each +1).
- **Deltas the flights show (expected against seen):** (1) `plan-track` costs 1.8–2.3 s per click (a process and the world from disk). The stick runs `solve` in the bot's own process instead (`engine/native/src/resident.rs`: one living store per session, the corridor read from the chunks the bot has received), so a stick click costs microseconds. In his game (the bot driven by `goto` from the console): open ground reached (5 of 5, up to 46 blocks; the bot slides past B on its momentum after the touch); a way that crosses a fence is refused as `NeedsJoints` with the fence named, because a new site has no joints yet (step 3). Playback stops only on events that move the body (explosion, motion, position, rotation) or a block edit inside the plan's corridor; chunks arriving while it walks do not stop it. The corridor is the straight box between the bot and B plus 2 blocks; a click mid-jump takes the first solid block below as the ground; a click on its own block answers "Already there." Open: (a) he sees the bot lag in the air while jump-running, though the flights land on the predicted tick: unmeasured; (b) a block whose shape the law refuses ("non-local shape", stairs) panics the corridor instead of becoming an exception; (c) steps, fences and stairs are refused as `NeedsJoints`. (2) The planner's settled start has saturation 5.0, the bot has 20.0 (first deviation at tick 0 on every flight; T unaffected on these lengths, the food guard is not). (3) Ten new flat goals are not flown yet. Astra is back on the 23rd (Codex week reset, 20:34).
- **Astra, `jobs/flight-ladder` (tonight, after the Codex reset):** joints at a click for steps, fences and stairs; a non-local shape becomes an exception; the settled start at saturation 20.0; the dead A\* planner deleted; his tracks climbed rung by rung offline, each rung then flown in his game.
- **The port branch, `wt/port-fold` (pushed to `origin`, `Sketch` state — stays off `main` until flown):** the 26.2 port batches folded. **32,252 / 32,366** modeled block states in `engine/world/data/census.tsv` (`99.65%`; 114 dynamic shulker/moving-piston states non-local), **31,351** Rust lines in `engine/law/src/` across 42 files, **9,117** bit-exact `server-26.2.jar` witnesses across 18 TSVs (`tests/jar.rs`). Not wired into `plan`/`bot/`, not flown. **Opus, `jobs/port-compress`:** folds the three waiting GLM dossiers (status-effect tick, item entity and inventory, shield/crit/mace), then compresses the law onto its generator basis (step 2 below), report in the worktree's `jobs/port-compress/report.md`.
- **GLM 5.4, four lanes (`jobs/glm-chain.txt`):** parameter rows, scars and jar witnesses for the generators — affine bodies (`gen-affine-body-rows`), grounds (`gen-ground-rows`), steps, fences and stairs (`gen-step-fence-stair-witnesses`), clocks (`gen-clock-rows`). Class-by-class porting is frozen.

## The overhaul (`jobs/overhaul/plan.md`, approved S240): state

Built and on `main`: the map (this folder; under 40 KB loaded whole, history in `archive/`), AGENTS.md and the README rewritten, the hooks of `work.md` (card and his words on every message, the launch guard, the job watcher, the map and net-line commit checks, the push guard), the scoreboard (`scripts/scoreboard.ps1`), companion agency in `gpt.ps1`, GLM without its checker or lenses, tests cut to what the game judges, memory cut to 11 files, 220 dead worktrees and 51 branches archived (`D:\path-tracing-archive`, `archive/*` tags), history stripped from `map/detail/`. The meters: `/meters` opens a pane in every session (the mod in `C:\Users\PASCAL\.claude\mods\meters-band`, loaded by `CLAUDE_CODE_PLUGIN_DIRS` in `~/.claude/settings.json`; the app's own Context panel takes no mod content).

Checks against these fixes: `jobs/overhaul/checks.md`.

Left: delete the old A\* planner (`plan/src/flight.rs` modes 3/5/b) and dead `words` code (in Astra's `flight-ladder`); a Sonnet sub-agent lints `map/` against his verbatim turns once.

## Next, in order

The bottleneck the game shows is on `main` (his tracks are refused at steps, fences and stairs), so step 1 leads and step 2 runs beside it on `wt/port-fold`.

1. **Climb the live flight ladder on `main` (his tracks + direct word runner):** joints at a click, delete the dead A\* planner (`flight.rs` modes 3/5/b), fix the 3 open flat-ground deltas, wire the ES audit wins on the path (`jobs/es-audit/report.md`: arc certificates, no replay from tick 0 in `polish::slide`, no per-click settle), and run `(word, count)` programs directly in `bot/` with event re-anchoring and mid-run retargeting (`R-retarget`).
2. **Close the port wave and compress the law onto its generator basis:** fold the 3 waiting GLM dossiers onto `wt/port-fold`, freeze speculative porting, then a `clean:` pass; collapse duplicate world traits (`World`, `BoatQuery`, `PortalWorld`, `SimWorld`) into one `World`/`WorldMut` interface that is multi-entity native, reversible via edit deltas, automatic on footprints, and observer-filterable — while keeping `Word` certificates pure (`found / forbidden / searched / refused`) so any future contact solver sits cleanly outside the crystal; factor duplicated physics onto ~5–6 shared generators (`affine_body`, `aabb_sweep`, `ray_dda`, resource/cooldown clocks, `cell_rewrite` + class-major event calendar) plus data tables and thin Mojang scars (keeping non-affine mechanisms like elytra, pistons, `PortalForcer` cleanly separate); split `PlayerState` into an interned static profile + compact hot state (~24–32 B logical view). Target: ~35K → ~10–15K lines so the whole law fits one context window and L1I cache, guarded bit-for-bit by the ~10k jar witnesses and `bench` ≤ `0.6–0.95 µs/tick`.
3. **Wire compressed generators into `plan` and `bot/`, then fly tools and edits:** derive binade-exact event forms once per generator (never linear-derivative or nearest-LUT approximations), derive kit and entity classes from `(generator, params, scars)`, and merge `wt/port-fold` onto `main` slice by slice only after live flights pass (world edits & restoration → body swaps & wormholes → dynamic world & hazards).
4. **Far map, one mind, and 10,000 bots / 4D building:** medium bounds, `u64` bit-parallel light cones, lazy certified bots (`shipmem` demote/promote + class-major calendar), sparse-vs-dense switch (event certificates vs AVX-512 SIMD lockstep; Vulkan GPU only after a measured crossover), and `(max, +)` blueprint building. Once two-body flights measure the real certificate-thrashing boundary, use [pvp-strategy-2026-11-22.txt](../archive/research/pvp-strategy-2026-11-22.txt) for the two-regime contact singularity boundary, `(max, +)` hurt-time swarm scheduling, and fat-tailed supply buffering.
5. **Mobs and long-tail survival on demand:** pulled only when real flights or missions hit them, via compact parameter rows, scars, and jar witnesses.

## Derived component state (ES rule)

| Component | Derived state | Scope |
|---|---|---|
| Core `solve`, joints, flat-ground word store | `Load-bearing` | `main`, flown (`straight100/300/800`, `stonesimple`) |
| 21 tracks runner, ES audit wins, direct word executor | `Sketch` -> `main` next | closes flat deltas, removes dead planner code |
| `wt/port-fold` (26.2 non-mob laws, 9,117 jar rows) | `Sketch` (`origin/wt/port-fold`) | compresses onto generator basis, then merges per passing flight |
| Kit classes, edits & restoration, tools, dynamic world | Blocked on compression + track runner | pulled through one mechanism into `plan` and `bot/` |
| Far map, one mind (`10⁴` bots), dense SIMD/GPU, 4D build | After live tool flights | class-major calendar, lazy bots, `(max, +)` builds |

## Branches kept until they are needed

Each is folded when its step comes and a judge or a flight is there for it; everything else is in `main`, `wt/port-fold` or an `archive/*` tag.

- The clicked face (`wt/track-face`), the rebuilt hard track (`wt/hard-track-2`), freeze damage (`wt/l0-veto`), the preview renderer (`wt/build-preview`).
- The server mind (`wt/server-mind`: a Fabric server mod and an out-of-process Rust mind over shared memory, three-tier swarm to 10,000 bots, one GPU draw call) and 4D building (`wt/build-4d`): step 4.
- Parked: the refuge (`wt/hearth-small`, S228).
- `wt/flat-ground` stays as a worktree for its untracked `lab/fixtures/flat-ground-replay/` (15 MB of recorded runs: ice, honey, slime, slabs and stairs, ladders, powder snow …), which exists nowhere else.

## Known anomalies (each blocks only what depends on it)

- Entity push sits unwired on `wt/port-fold` (`Sketch` in `entity_contact.rs` / `sim.rs`), not yet flown on `main` (five bots on one crossing).
- Sprint at an exactly axis-aligned wall depends on rounding in Mojang's own `acos` (`movement.rs:818`), ported faithfully.
- On `main`'s checkout the test world's `r.0.0.mca` and `r.0.-1.mca` are whole untrimmed region files (3.8 and 3.4 MB), with the trimmed committed ones beside them as `.bak-20261114`; `launch.ps1` copies them into every Codex worktree. Uncommitted.
- His bug S38: a track run deletes a sulfur spike from his inventory; never traced.
