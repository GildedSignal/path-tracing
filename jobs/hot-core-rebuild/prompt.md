# The hot player core, rebuilt beside the oracle

**Goal:** the player's tick runs on a new core built from the approved grouping table (generators, rows, scars) over a ~96 B hot state, byte-identical to the frozen old law on every boundary witness, so the old hot files are deleted whole and the code and test lines fall in decades.

Builder: Opus (native sub-agent of the integrator). Worktree `D:\path-tracing-wt\port-fold`, branch `wt/port-fold` (at `38a49808` or later). Read first: `map/` (goal, principles, architecture, now, vocabulary) and `jobs/law-grouping-audit/table.md` (approved by him: the groups, the basis with `ledger`, the floor, the hot/cold split, the `PlayerState` byte split), all in `C:\Users\PASCAL\Desktop\path tracing`. Commit on `wt/port-fold` in small commits; never push, never touch `main` or other branches, his game or server.

## His words (verbatim, 23 November)

"The goal is to have to do less and less work each pass we'll make is aimed at improving scalability and scaling laws, making each subsequent things easier, easier test less lines for tests and so on." — "less lines the better in general" — "LOCs reduction should be measured in 1/log."

## The method

- **The old law is the oracle, frozen.** Do not refactor it. Read it to learn what the new core must do; never re-derive from the jar what the Rust already holds.
- **Think from the constraints, not from what is there.** Start from what the boundary demands (the witnesses, the exactness tier per group, T, the cache line, the information floor of the state) and invent the core inside them. The oracle tells you what; never copy its how, its types or its order of files. The table's groups and estimates are a guide, not a ceiling: beat them where the constraints allow.
- **Spartan on duplication:** every fact is held once and every value computed once per tick. A field stored twice (yaw beside the input's yaw), a value recomputed that a row could hold, or two paths for one shape (two hurt pipelines, the test-only twins) is a defect, deleted.
- **Only the hot layer** (the table's section 5): the tick core and its flag machines (sprint, crouch, swim, pose, flight), living travel on `body` with kicks as its input term, `sweep` and shapes with the player's scars, the player's `clock` rows, the speed attribute fold, food, exhaustion and fall damage on `ledger`, the `World` read path, the numeric scars, the early-outs of fluids and block effects. The cold layer (redstone, pistons, fluids' spread, portals, rails, cell rules) stays as it is and is called where the tick reaches it.
- **The hot state:** what a plain movement tick touches, ~96 B (the table's byte split). `movement_this_tick` is output, not state; yaw and pitch come from the input; flags recomputed every tick are not stored. Lukewarm and cold fields live beside it.
- **Judged at the boundary only:** the 29 replays, the jar's boundary rows the hot layer reaches (`shapes`, `rays`, `intersections`, trajectories, sprint rows fed through the tick's sprint face), the planner's `real_time` byte-identical (364 lines, as passes 2 and 3 ran it), and a differential test: random reachable states ticked by the oracle and the core, byte-identical. One table-driven test per generator, never a test that names an internal helper.
- **Symmetry is found by hashing, not by reading** (ES 32): run candidates on the same inputs; two whose output tables are byte-equal are one row class. Use it to check and refine the table's groups in the hot layer.
- **Two faces from the start** (ES 3, 14: compute only surprises): each generator's interface takes a state and returns either the next tick or the next event (the step where a certificate breaks). Build the tick face now; shape the interface so the event face fits without a rebuild.
- **A disagreement is data** (ES method §7): every case where the oracle and the core differ is logged as a reproducible row (state, both outputs) in `jobs/hot-core-rebuild/anomalies.tsv` of the main checkout (C:/Users/PASCAL/Desktop/path tracing), then explained as a bug in the core or a scar the oracle holds. None is dropped.
- **Then switch and delete:** `try_tick` runs the new core; the old hot files and their test-only twins go in one commit (the table names ~2.9K lines of witness-only twins). What the cold layer still needs from them moves behind `WorldMut` or the core's interface.
- Main moves meanwhile (Astra works on `plan` on `main` tonight). Do not merge `main`; the integrator does, after.

## Conditions

- Every commit green: `cargo test -p law --release --lib --test jar --test replay`, plus the differential test once it exists. A witness value that changes is a bug in the change; the count of witness rows never falls (rows are data; only harness lines shrink).
- Each commit message prints code lines and test lines for `engine/law` before → after, both in decades (`log10(before / after)`), lines per generator against the table's rebuilt estimate, and the bench ratio against `38a49808` with its spread (`scripts/bench-judge.sh`). The commit-msg hook wants a `map/` change or `Map: unchanged, because …`, and a `Bought:` line when `engine/` grows. End each message with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- Over 2× a group's rebuilt estimate: stop and say why the representation is wrong. Table rows are inert data; no rule scripts in tables.
- Toolchain on D: (`export PATH=/d/rust/cargo/bin:$PATH RUSTUP_HOME=D:/rust/rustup CARGO_HOME=D:/rust/cargo`), cargo from `engine/`. Edit with the Edit/Write tools. Do not edit `map/`. Commit often: the 5-hour window may end the run.

## Output

Your final message is the report: commits (hash, one line), code and test lines before → after and in decades, files deleted and left, lines per group against the table, the hot state's size, the bench with its spread, the anomalies found and how each was explained, what the cold layer still takes from the old hot files, what is left, and the lines the map should change.
