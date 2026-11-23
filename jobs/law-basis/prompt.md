# The law as generators and tables

**Goal:** the 26.2 law on `wt/port-fold` becomes a few generators plus data tables plus thin scars, written top-down from the jar, each old file deleted once its witnesses pass on the basis, toward a few thousand lines of code.

Builder: Opus (native sub-agent of the integrator). Worktree `D:\path-tracing-wt\port-fold`, branch `wt/port-fold` (at `996a652` or later). Read `map/` (goal, principles, architecture, now, vocabulary) and `map/detail/engine.md` in `C:\Users\PASCAL\Desktop\path tracing` first. You commit on `wt/port-fold` in small commits; never push, never touch `main` or other branches, his game or server.

## His words (verbatim, 23 November)

"It is a game of cubes there must be a lot in common and then just tables for each thing based on the generators i think it did not understood the mission, there's like 6DOF, drag, cubes or more like rectangles and that's it then you compose them" — "The goal is massive cache hit and less loc so that it is easier for the LLMs to manage it."

## Why the first pass missed

Pass 1 (`jobs/port-compress/report.md`) tidied the mirror of Mojang's classes: dead code out, exact duplicates merged, one recurrence for six projectile kinds. The law stayed 31,393 lines: 289 types, 1,256 functions, 80 `tick` functions, 9 world traits, collision or ray code in 7 files, drag constants hand-written in 11 files. That is bottom-up. **This pass is top-down: write the basis first, without the 42 files open, then move each thing onto it as rows and delete its file.** Do not refactor the existing files toward the basis; replace them.

## The basis (the Fermi budget is the spec)

Boxes never rotate: every collision box is axis-aligned; yaw and pitch only steer input and aim rays. A body is position (3), velocity (3), a box size, a row.

| Generator | One kernel | Budget (code lines) |
|---|---|---|
| `body` | per tick: velocity += input + gravity; move with `sweep`; velocity × drag, friction from the block that the row names; `k = 1` the exact tick, `k = Δt` to the next event | ~300 |
| `sweep` | move an AABB through a box set, axis by axis in Java's order; step-up is the same sweep with the box lifted; contact flags out | ~300 |
| `ray` | DDA through cells, clip against the cell's boxes | ~150 |
| `cell` | a block state is an id into a table; a rule is a local rewrite fired by an event (neighbour update, scheduled tick, random tick, entity inside, use, break) on one class-major calendar | ~400 |
| `clock` | a counter stepped per tick or per action, a threshold that fires, a reset | ~150 |
| `contact` | box–box overlap → push or hurt; box inside a cell → the cell's table entry | ~200 |

Plus **one** `World` read side and **one** `WorldMut` edit side (one cell representation, edits recorded as reversible deltas, footprints recorded by the interface), the arithmetic (sine table, Java float/double order), the tables (data, not code: a row per body class, block class, shape, clock, effect, item), and **scars**: a flag or a small hook named in a row where Mojang breaks its own pattern (the arrow's drag after the move, the boat's buoyancy). A scar is never a file. What truly does not fit stays its own small kernel: elytra, pistons moving block groups, the redstone wire graph, `PortalForcer`. Total target: a few thousand lines of code, plus tables. If a generator overruns its budget by more than 2×, stop and ask why: the representation is wrong (principle 2), not the code.

Java's operation order is part of the row (gravity before or after drag, which axis first, float or double); bit-exactness is never a reason to keep separate code.

## In order

1. **The basis, in its own module** (`engine/law/src/basis/`): the six generators, `World`/`WorldMut`, the table types. Written from the jar and the map; read the old files only to find the jar method a behaviour comes from.
2. **The player first** (the hot path, `movement.rs`, `player.rs`, `state.rs`): the player is a `body` row plus clocks. Gate: `cargo test -p law --release --lib --test jar --test replay` identical, and `bin/bench` no slower (today sprint-jump 628, walk 978, ceiling 902, bumpy 951 ns median). Then `PlayerState` is a compact hot state plus an interned profile, with the effect book inside (which lifts `WitherRose`).
3. **Then each family**, deleting its old files as it lands: bodies (boat, minecart, items, projectiles, falling, TNT), contact and combat, cells (redstone, fluids, world hooks, block hooks, interact, place, mining), then the kernels that stay separate, on the same `World`/`WorldMut`. `sim.rs` disappears into the calendar.
4. **Witnesses become rows:** the ~7.7K lines of in-file unit tests move to `tests/data/*.tsv` read by one table-driven test per generator.

GLM 5.4 is producing jar rows for this right now, in the main checkout's `lab/results/local-20261123-*-gen-*`: affine bodies, grounds, steps/fences/stairs, clocks. Use any that have finished as table rows and witnesses (check each against the jar before trusting it: GLM is weak, S173).

## Conditions

- Every commit green on the gate above; a witness that changes is a bug in the change, never a witness to edit.
- Each commit's message prints lines before → after for `engine/law/src` (code only, and total). The commit-msg hook wants a `map/` change or `Map: unchanged, because …`, and a `Bought:` line when `engine/` grows (the basis grows before the files go: say what it buys). End each message with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- Toolchain on D: (`export PATH=/d/rust/cargo/bin:$PATH RUSTUP_HOME=D:/rust/rustup CARGO_HOME=D:/rust/cargo`), cargo from `engine/`. Edit text with the Edit/Write tools.
- No made-up numbers: every constant from the jar.
- Do not edit `map/`; put what the map should say in your final message.

## The one stop

Stop only if the jar proves a behaviour cannot be a row of a generator without a branch on kind. Write the case (the behaviour, the witness rows, why) and go on with the rest.

## Output

Your final message is the report: commits (hash, one line), lines per generator against its budget, lines before → after (code and total), the gate and bench numbers, which old files are deleted and which remain, what is left, and the lines the map should change.
