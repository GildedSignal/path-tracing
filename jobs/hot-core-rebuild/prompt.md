# The player's core as a causal chain, rebuilt beside the oracle

**Goal:** the player's behaviour runs on a new core that treats Minecraft as a causal chain (causes, their effects, and a generator between them), byte-identical to the frozen old law wherever behaviour is observed, so the old hot files are deleted whole and the code and test lines fall in decades.

Builder: Opus (native sub-agent of the integrator). Worktree `D:\path-tracing-wt\port-fold`, branch `wt/port-fold` (at `38a49808` or later). Read first: `map/` (goal, principles, architecture, now, vocabulary) and `jobs/law-grouping-audit/table.md` (the groups, the basis with `ledger`, the hot/cold split, the `PlayerState` byte split), all in `C:\Users\PASCAL\Desktop\path tracing`. Commit on `wt/port-fold` in small commits; never push, never touch `main` or other branches, his game or server.

## His words (verbatim, 23 November)

"Minecraft shall be seen as a causal chain engine which simplifies a lot, ticks are not really needed at all they are a by-product of a way of thinking not of behvavior, behavior is king." — "the grouping table can change it is emergent not designed, the needs might require it to change. it should ask itself if the thing it is looking is actually useful or not for vanilla behavior and think with the constraints simple as that the how it was made before doesn't matter." — "The goal is to have to do less and less work each pass we'll make is aimed at improving scalability and scaling laws, making each subsequent things easier, easier test less lines for tests and so on." — "LOCs reduction should be measured in 1/log."

## The method

- **Behaviour is king; the law is a causal chain.** A cause (an input change, a contact, a cell crossed, a clock threshold, an edit) fires its effect; between causes a generator carries the state. The core's face is state + cause → effect and the next cause. A tick is derived only where behaviour is observed per tick: the moves the server checks, the replays, the witnesses, `try_tick` for the planner. Bit-exact arithmetic may iterate inside a generator; that is arithmetic, not a tick loop in the design.
- **Useful for vanilla behaviour, or gone.** For each thing in the oracle, ask whether it changes behaviour the player or the server can observe. If not, it is not rebuilt (sound and game events, bookkeeping nothing reads, copies kept for tests).
- **Think from the constraints, not from what is there.** Start from what behaviour demands (the witnesses, the exactness tier per group, T, the cache line, the information floor of the state) and invent the core inside them. The old law is the oracle, frozen: it says what happens, never how to write it. Do not refactor it; never re-derive from the jar what it already holds.
- **The table is emergent, not designed.** Its groups, generators and estimates change when the constraints demand; write each change and why. Find symmetry by hashing (ES 32): run candidates on the same causes; byte-equal outputs are one row class.
- **Spartan on duplication:** every fact held once, every value computed once. A field stored twice (yaw beside the input's yaw), a value recomputed that a row could hold, or two paths for one shape (two hurt pipelines, the test-only twins) is a defect, deleted.
- **Scope, the hot layer** (the table's section 5): the player's flag machines (sprint, crouch, swim, pose, flight), travel on `body` with kicks as its input term, `sweep` and shapes with the player's scars, the player's clocks, the speed attribute fold, food, exhaustion and fall damage on `ledger`, the `World` read path, the numeric scars, the early-outs of fluids and block effects. The cold layer (redstone, pistons, fluids' spread, portals, rails, cell rules) stays as it is and is called where the chain reaches it. The hot state is what a plain move touches (~96 B by the table, less if the constraints allow); output is not state.
- **Judged where behaviour is observed, only:** the 29 replays, the jar's boundary rows the hot layer reaches (sprint rows fed through the core's face, not a helper), the planner's `real_time` byte-identical (364 lines, as passes 2 and 3 ran it), and a differential test: random reachable states driven by the same causes through the oracle and the core, byte-identical. One table-driven test per generator, never a test that names an internal helper.
- **A disagreement is data** (ES method §7): each case where the oracle and the core differ is a reproducible row (state, cause, both outputs) in `jobs/hot-core-rebuild/anomalies.tsv` of the main checkout (C:/Users/PASCAL/Desktop/path tracing), explained as a bug in the core or a scar the oracle holds. None is dropped.
- **Then switch and delete:** `try_tick` runs the new core; the old hot files and their test-only twins go in one commit. What the cold layer still needs from them moves behind `WorldMut` or the core's face.
- Main moves meanwhile (Astra works on `plan` on `main` tonight). Do not merge `main`; the integrator does, after.

## Conditions

- Every commit green: `cargo test -p law --release --lib --test jar --test replay`, plus the differential test once it exists. A witness value that changes is a bug in the change; the count of witness rows never falls (rows are data; only harness lines shrink).
- Each commit message prints code lines and test lines for `engine/law` before → after, both in decades (`log10(before / after)`), lines per group against the table, and the bench ratio against `38a49808` with its spread (`scripts/bench-judge.sh`). The commit-msg hook wants a `map/` change or `Map: unchanged, because …`, and a `Bought:` line when `engine/` grows. End each message with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- Over 2× a group's estimate: stop and say why the representation is wrong. Table rows are inert data; no rule scripts in tables.
- Toolchain on D: (`export PATH=/d/rust/cargo/bin:$PATH RUSTUP_HOME=D:/rust/rustup CARGO_HOME=D:/rust/cargo`), cargo from `engine/`. Edit with the Edit/Write tools. Do not edit `map/`. Commit often: the 5-hour window may end the run.

## Output

Your final message is the report: commits (hash, one line), code and test lines before → after and in decades, files deleted and left, how the table changed and why, what was judged useless for vanilla behaviour and dropped, the causes the core knows, the hot state's size, the bench with its spread, the anomalies and how each was explained, what the cold layer still takes from the old hot files, what is left, and the lines the map should change.
