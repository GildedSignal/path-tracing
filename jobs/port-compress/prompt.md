# Untangle the port: the law onto its generator basis

**Goal:** `wt/port-fold`'s law shrinks from ~31K toward ~10–15K lines on a handful of shared generators, every jar witness and replay bit-identical, `bench` no slower.

Builder: Opus (native sub-agent of the integrator). Worktree `D:\path-tracing-wt\port-fold`, branch `wt/port-fold` (`b605c9b`, `Sketch` state, not wired into `plan`/`bot/`, not flown). This is step 2 of `map/now.md`; read `map/` (goal, principles, architecture, now) and `map/detail/engine.md` first. You commit on `wt/port-fold` yourself, in small commits; you never push, never touch `main`, other branches, his game or server.

## His words (verbatim)

- S6: "make minecraft's best path tracing algo in the world, which means we'll have to go first principle"
- S44, S49: less code is better on every level
- S235: question the requirement, delete, then simplify
- ES 19: "a fix signals the wrong representation"

## The yarn ball

`engine/law/src/` holds 31,351 lines in 42 files, ported class by class from 26.2 by many hands: `sim.rs` alone is 5,111; `movement.rs` 2,043; `world_hooks.rs` 1,627; `boat.rs` 1,451; `mining.rs` 1,432; `interact.rs` 1,413; `player.rs` 1,347. There are at least four world interfaces (`world.rs:342 World`, `boat.rs:113 BoatQuery: World`, `portal.rs:416 PortalWorld`, and `SimWorld`/`WorldOverlay` in `sim.rs`). The guards: 9,117 bit-exact `server-26.2.jar` witnesses in 18 TSVs (`engine/law/tests/jar.rs`, `tests/data`), `tests/replay.rs`, and `tests/perf_core.rs` (the law tick at 0.6–0.95 µs).

## In order

0. **Fold the three waiting GLM dossiers** first, so the compression sees the whole law once: `lab/results/local-20261122-145220-2976-1-0-port-status-effect-tick-and-periodic` (lifts `WitherRose`), `...-145222-30104-1-0-port-shield-crit-sweep-and-mace`, `...-145224-30192-1-0-port-item-entity-and-inventory` (in the main checkout, `C:\Users\PASCAL\Desktop\path tracing\lab\results\`; the `09495x` runs of the same names are earlier attempts — take the better of each pair, by its witnesses). Their witnesses go in with them, green.
1. **A `clean:` commit**: delete what nothing reaches and duplicates that the witnesses prove equal.
2. **One world interface**: collapse `World`, `BoatQuery`, `PortalWorld`, `SimWorld` into one `World` read side and one `WorldMut` edit side — multi-entity native, reversible by edit deltas (an edit is a toll with a debt), footprints recorded by the interface itself, filterable by observer. `Word` certificates stay pure (`found / forbidden / searched / refused`).
3. **Shared generators**: factor the duplicated physics onto ~5–6 generators — `affine_body` (the drag-and-input recurrence every body runs: player, boat, minecart, item, projectile, pearl), `aabb_sweep` (collision), `ray_dda` (reach, eye ray, projectiles hitting blocks), resource/cooldown clocks (food, air, fire, freeze, effects, item cooldowns), `cell_rewrite` with a class-major event calendar (scheduled ticks, redstone, fluids, random ticks) — plus data tables (parameters per entity and block class) and thin Mojang scars where the jar does something irregular. Each generator runs `k = 1` for the exact tick and `k = Δt` to its next event. True non-affine kernels stay separate: elytra, pistons, `PortalForcer`.
4. **`PlayerState` split**: an interned static profile plus a compact hot state (~24–32 B logical view).

Measure, do not guess: before and after each commit, `cargo test -p law --release --test jar --test replay` and the `perf_core` bench. Toolchain on D: (`PATH=D:\rust\cargo\bin;$PATH`, `RUSTUP_HOME=D:\rust\rustup`, `CARGO_HOME=D:\rust\cargo`). A deletion that leaves every witness and replay identical is proven sound; a witness that changes is a bug in the change, never a witness to edit.

## Conditions

- One mechanism: no per-entity or per-block special path where a generator with parameters does it.
- No made-up numbers: every constant from the jar or the law.
- The commit-msg hook wants, for a commit touching `engine/`, either a `map/` change or a line `Map: unchanged, because …`, and a `Bought:` line if `engine/` grows; end each message with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. Do not edit `map/` (the integrator does); write what the map should say in your report.
- Edit text with the Edit/Write tools (PowerShell 5.1 garbles UTF-8).

## The one stop

Stop only if the witnesses prove two ported behaviours cannot share a generator without a branch on kind. Write the case (the two behaviours, the witness rows) and keep going on the rest. Anything else you decide against the map, write down with its reason.

## Output

Write your report as you go to `D:\path-tracing-wt\port-fold\jobs\port-compress\report.md` (so it survives a quota death): per step, commits, lines before → after per file and in total, witnesses and replays (count, all identical or the deltas), bench before → after, the generators as they stand (name, parameters, what runs on it), what is left, and the lines the map should change.
