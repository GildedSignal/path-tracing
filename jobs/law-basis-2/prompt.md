# The law basis, pass 2: the player's state, then the cells

**Goal:** the player runs as a `body` row plus clocks over a compact hot state, and every block runs on one `cell` generator over the world's census table, deleting `sim.rs` and the per-class block files, so the law's code falls from ~21.9K lines toward a few thousand and the tick gets faster.

Builder: Opus (native sub-agent of the integrator). Worktree `D:\path-tracing-wt\port-fold`, branch `wt/port-fold` (at `c392524` or later). Read first: `map/` (goal, principles, architecture, now, vocabulary), `map/detail/engine.md`, `jobs/law-basis/prompt.md` (the method and the budgets, which still hold) and `jobs/law-basis/report.md` (pass 1), all in `C:\Users\PASCAL\Desktop\path tracing`. You commit on `wt/port-fold` in small commits; never push, never touch `main` or other branches, his game or server.

## His words (verbatim, 23 November)

"It is a game of cubes there must be a lot in common and then just tables for each thing based on the generators" — "The goal is massive cache hit and less loc so that it is easier for the LLMs to manage it." — "the beginning always looks messy until all of the sudden it works"

## Where it stands

`engine/law/src/basis/` holds `sweep` (138 code lines), `ray` (166), `body` (117: rows `FALLING_BLOCK`, `THROWABLE`, `ARROW`, `ITEM`, `MINECART`, `LIVING`; one `Entity.move`; one launch) and `clock` (14: food, air). Code lines in `engine/law/src`: 21,863 (30,846 total). The gate: `cargo test -p law --release --lib --test jar --test replay` (lib 100, jar 18 over 9,117 rows, replay 29 + 5 ignored). Bench median ns/tick: walk 973, sprint-jump 640, ceiling 903, bumpy 958. What is left is two piles: the player (`movement.rs`, `player.rs`, `state.rs`, ~3–4K code lines) and the cells (`sim.rs` 4,346 code lines plus `world_hooks`, `block_hooks`, `interact`, `place`, `neighbor`, `redstone`, `fluid`, `piston`: ~10K).

## In order

0. **Pin the bench first** (~an hour at most). It moves ±5% with code layout alone, so "no slower" means nothing yet. Make it a judge: interleaved base/candidate runs, pinned core and priority, enough samples to give a median with its spread, and whatever stops layout noise (for example function alignment). Report the spread you get. Every later commit reports the bench against `c392524` with that spread.
1. **The compact `PlayerState`.** Today ~100 fields, each access a `read_state!`/`write_state!`/`modify_state!` footprint call (533 of them, 419 in `movement.rs`). Make it a hot state of ~24–32 B logical (position, velocity, the flags and counters a tick reads) plus an interned static profile (attributes, gear, game mode: everything a tick does not change). Footprints are recorded per group, by the state's own interface, never by a call per field. The effect book moves inside it (`Effects.absorption` duplicates the state's): that lifts the `WitherRose` refusal. Delete the hand-written `Clone`, the field lists in `bits()` and `state_fields!`: one declaration of the fields.
2. **The player as rows.** The player's travel is a `body` row (`LIVING`, with the player's input, sprint, jump, swim, climb, ladders, powder snow and fluids as rows and named scars); its counters (fire, freeze, hurt cooldown, portal, exhaustion) are `clock` rows. Elytra stays its own small kernel on the same state. Delete what lands.
3. **`cell` on the census table.** A cell is one id into the world crate's census of 32,252 block states (`engine/world/data/census.tsv`); the table gives its boxes, friction, speed and jump factors, and the rules it fires. A rule is a local rewrite fired by an event (neighbour update, scheduled tick, random tick, entity inside, use, place, break) on one class-major calendar. One `WorldMut`: edits recorded as reversible deltas, footprints recorded by the interface. Then move each family onto it and delete its file: doors, trapdoors, gates, buttons, plates, levers, tripwire; placing and breaking; fluids; redstone (the wire graph may stay its own small kernel); pistons (their own small kernel on `WorldMut`). `sim.rs`'s private palette, `SimCell` and `sync_block` go: one representation of the world. The six edit-side traits (`PistonWorld`, `RedstoneWorld`, `BucketWorld`, `TripwireWorld`, `PortalWorld`, `Rails`) become `WorldMut`.
4. **`contact`**, if the window allows: box–box overlap (entity push, `entity_contact.rs`), box inside a cell (the table entry, replacing the enum match in `movement.rs::inside_block`), combat on it.

GLM 5.4 is producing jar rows for this in the main checkout's `lab/results/local-20261123-*-gen-*` (bodies, grounds, steps/fences/stairs, clocks). Use those that have finished as table rows and witnesses after checking each against the jar (GLM is weak, S173). The three jar mismatches pass 1 fixed (the boat's `Mth.equal` flags, the falling block's restitution, the pearl's inside-block clip) each become a witness row.

## Conditions

- Every commit green on the gate; a witness that changes is a bug in the change, never a witness to edit.
- Each commit message prints code lines before → after for `engine/law/src`, lines per generator against its budget, and the bench with its spread. The commit-msg hook wants a `map/` change or `Map: unchanged, because …`, and a `Bought:` line when `engine/` grows. End each message with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- Budgets (from `jobs/law-basis/prompt.md`): `body` ~300, `sweep` ~300, `ray` ~150, `cell` ~400, `clock` ~150, `contact` ~200 code lines. Over 2×: stop and say why the representation is wrong.
- Toolchain on D: (`export PATH=/d/rust/cargo/bin:$PATH RUSTUP_HOME=D:/rust/rustup CARGO_HOME=D:/rust/cargo`), cargo from `engine/`. Edit text with the Edit/Write tools. No made-up numbers: every constant from the jar. Do not edit `map/`.

## The one stop

Stop only if the jar proves a behaviour cannot be a row of a generator without a branch on kind. Write the case (the behaviour, the witness rows, why) and go on with the rest.

## Output

Your final message is the report: commits (hash, one line), the bench before → after with its spread, code lines before → after, lines per generator against its budget, the hot state's size, the old files deleted and those left, what is left, and the lines the map should change.
