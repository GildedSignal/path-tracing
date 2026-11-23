# The law basis, pass 3: the cells, then the player's travel, deleting as they land

**Goal:** every block runs on one `cell` generator over the world crate's model table behind one `WorldMut`, and the player's travel is `LIVING` rows of `body`, so `sim.rs` and the per-family block files are deleted and the law's code falls from 21,403 lines by thousands, not hundreds.

Builder: Opus (native sub-agent of the integrator). Worktree `D:\path-tracing-wt\port-fold`, branch `wt/port-fold` (at `d068260a` or later). Read first: `map/` (goal, principles, architecture, now, vocabulary), `map/detail/engine.md`, `jobs/law-basis/prompt.md` (the method and budgets), `jobs/law-basis-2/prompt.md` and `jobs/law-basis-2/report.md` (pass 2), all in `C:\Users\PASCAL\Desktop\path tracing`. You commit on `wt/port-fold` in small commits; never push, never touch `main` or other branches, his game or server.

## His words (verbatim, 23 November)

"It is a game of cubes there must be a lot in common and then just tables for each thing based on the generators" — "The goal is massive cache hit and less loc so that it is easier for the LLMs to manage it." — "the beginning always looks messy until all of the sudden it works"

## Where it stands

Pass 2 made `PlayerState` one declaration (296 B hot plus a shared 41-field profile, the effect book inside, footprints per step) and the bench a judge (`scripts/bench-judge.sh`: identical code 1.000 ± 0.3%, layout alone ±1.5%). It deleted no file: 21,863 → 21,403 code lines (`scripts/law-lines.pl`). The pile is the cells: `sim.rs` 4,324, `world_hooks` 1,390, `block_hooks` 901, `piston` 714, `interact` 490, `neighbor` 458, `place` 434, `fluid` 358, `redstone` 345: **9,414 code lines**. Then the player: `movement.rs` 1,453, `player.rs` 862. The gate: `cargo test -p law --release --lib --test jar --test replay` (lib 101, jar 18, replay 29 + 5 ignored), plus the planner's `real_time` test byte-identical to `c392524` as pass 2 ran it.

## The measure of this pass

Lines deleted, measured on a log scale (his words, 23 November: "LOCs reduction should be measured in 1/log"): report `log10(before / after)` in decades for each commit and for the pass. The floor is the budgets' sum, ~1,500 lines of generators plus tables, about one decade below 21,403; pass 2 moved 0.009 decades. A commit that adds a generator path without deleting the old path in the same commit, or the next one, is not done. Speed is held, not chased: the bench stays within the layout spread (±1.5%) of `d068260a`; do not spend the window tuning it.

## In order

1. **`cell` on the model table.** A cell is one id (`u16`) into the world crate's `models.tsv`/`blocks.tsv` (the census only lists buckets); the table gives boxes, friction, speed and jump factors, and the rules a cell fires. A rule is a local rewrite fired by an event (neighbour update, scheduled tick, random tick, entity inside, use, place, break) on one class-major calendar. One `WorldMut`: edits as reversible deltas, footprints recorded by the interface. Write it first from the jar (~400 lines), then move `sim.rs` onto it: its private palette, `SimCell` and `sync_block` go, one representation of the world. The six edit-side traits (`PistonWorld`, `RedstoneWorld`, `BucketWorld`, `TripwireWorld`, `PortalWorld`, `Rails`) become `WorldMut`.
2. **Each family onto `cell`, its file deleted:** doors, trapdoors, gates, buttons, plates, levers, tripwire (`block_hooks`, `world_hooks`, `interact`, `neighbor`); placing and breaking (`place`); fluids (`fluid`); redstone (the wire graph may stay a small kernel); pistons (a small kernel on `WorldMut`). Cheapest family first, so a file is gone by the first hour.
3. **The player's travel as `LIVING` rows:** input, sprint, jump, swim, climb, ladders, powder snow, water and lava media (each a drag/acceleration row plus named scars, e.g. gravity-after); fire, freeze, exhaustion as `clock` rows; inside-a-cell effects from the cell's table entry, replacing the enum match in `movement.rs::inside_block`. Elytra stays its own kernel. `movement.rs` shrinks to the composition or goes.
4. **`contact`**, if the window allows: box–box overlap (`entity_contact.rs`), combat on it.

The three pass-1 jar edges (the boat's `Mth.equal` flags, the falling block's restitution, the pearl's inside-block clip) become witness rows if you can run the jar (`D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`, javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`, java at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/java.exe`); otherwise list them as left. GLM's rows from today are empty (rate limits); use none.

## Conditions

- Every commit green on the gate; a witness that changes is a bug in the change, never a witness to edit. A harness assertion you rewrite is listed in the report with the reason.
- Each commit message prints code lines before → after for `engine/law/src`, the files deleted, lines per generator against its budget, and the bench ratio with its spread. The commit-msg hook wants a `map/` change or `Map: unchanged, because …`, and a `Bought:` line when `engine/` grows. End each message with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- Budgets: `body` ~300, `sweep` ~300, `ray` ~150, `cell` ~400, `clock` ~150, `contact` ~200 code lines. Over 2×: stop and say why the representation is wrong.
- Toolchain on D: (`export PATH=/d/rust/cargo/bin:$PATH RUSTUP_HOME=D:/rust/rustup CARGO_HOME=D:/rust/cargo`), cargo from `engine/`. Edit text with the Edit/Write tools. No made-up numbers: every constant from the jar or its tables. Do not edit `map/`.

## The one stop

Stop only if the jar proves a behaviour cannot be a row of a generator without a branch on kind. Write the case (the behaviour, the witness rows, why) and go on with the rest.

## Output

Your final message is the report: commits (hash, one line), code lines before → after and in decades against the floor, the old files deleted and those left with their lines, lines per generator against its budget, the bench ratio with its spread, harness assertions rewritten, what is left, and the lines the map should change.
