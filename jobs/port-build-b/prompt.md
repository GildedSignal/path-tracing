# Port build B: twelve settled laws into the Rust law

**Goal:** twelve rules GLM found in Mojang's 26.2 jar and a second session reproduced are in `engine/law`, each bit-exact against its witness, plus build A's one open finding fixed.

**Julien (18 November, verbatim):** "yes port build B hydrogen"

**Consumers** (`map/now.md`, the order of work): step 5, editing the world (place, break, interact), takes parts 1–6; the law under every word takes parts 7–10; step 8 (elytra, portals) takes parts 11–12.

## Where you work

- Worktree `D:\path-tracing-wt\port-build-b`, branch `wt/port-build-b`, built on port build A (`wt/port-build-a`, `055e7b7`: mining speed, boat, elytra boost, pearl), so its files are yours to extend. Build and test there with `--target-dir D:\path-tracing-target\agy-port-build-b`. Rust is at `D:\rust` (`CARGO_HOME=D:\rust\cargo`, `RUSTUP_HOME=D:\rust\rustup`, `D:\rust\cargo\bin` on `PATH`). Run `cargo test --offline -p law` from `engine\`.
- The evidence (read only), one folder per part under `C:\Users\PASCAL\Desktop\path tracing\lab\results\`. Each has `dossier.md` (the fact), `check.md` (the second session's grade, which wins where they differ), raw witness outputs, and sometimes a `work.patch` (a draft: read it, never apply it blind). **Test values are copied from the raw outputs, never retyped from prose; name the file and line in a comment.**
- Write your report as you go into `C:\Users\PASCAL\Desktop\path tracing\jobs\port-build-b\report.md`, after each part.

## The parts (independent: one that cannot hold does not stop the others)

0. **Build A's finding** (`mining.rs:149-152`): `break_ticks` is `ceil(1/p)` in f64; the game breaks at the first n with `p * n as f32 >= 1.0` (the same expression as `MiningWork::progress`). Make it that one rule; obsidian by hand becomes 5000, not 5001, and the test says why.
1. **Interaction reach and the eye ray**: `local-20261118-1012-1-0-port-interact-reach-and-clip`. Use the law's existing block traversal (`blocks.rs`); no new geometry search.
2. **Placing slabs, stairs, trapdoors**: `local-20261117-2150-1-0-port-place-slab-stair-trapdoor`.
3. **Placing doors, ladders, scaffolding**: `local-20261117-2155-1-0-port-place-door-ladder-scaffold`.
4. **The break packets' gate, creative and abort**: `local-20261117-2241-1-0-port-mining-creative-and-abort-gate`, beside build A's `mining.rs`.
5. **What a destroyed block does**: `local-20261117-2158-1-0-port-destroy-block-side-effects`.
6. **Levers and redstone-driven doors**: `local-20261117-2220-1-0-port-interact-lever-and-redstone-door`.
7. **The sprint trigger reset by food and items**: `local-20261117-2030-1-0-port-sprint-food-and-item-reset`.
8. **Lava inside the body: hurt and the collector**: `local-20261118-1014-1-0-port-lava-hurt-and-collector`.
9. **The game-mode switch's abilities**: `local-20261117-2028-1-0-port-gamemode-ability-switch`.
10. **Horizontal speed while flying**: `local-20261117-2029-1-0-port-flying-horizontal-speed`.
11. **Starting and stopping the elytra glide**: `local-20261118-1102-1-0-port-elytra-start-and-stop`, beside build A's `elytra.rs`.
12. **The new portal's base height**: `local-20261118-1013-1-0-port-portal-create-ceiling`.

New modules get only `pub mod` lines in `lib.rs`. Group the parts into modules as the law groups them (placing in one place, interaction in one place), not one file per part.

## What binds you

- **One mechanism:** no block kind gets its own code path. A rule that applies to a set of blocks reads it from the block table's data, never from names in code.
- **No patch:** no tolerance, epsilon, clamp, retry or special case to make a test pass. Mojang's oddities (NaN, signed zeros) are ported faithfully. If the port and the witness disagree, write both values and leave that part.
- **Do not touch** `movement.rs` or `state.rs` (wiring into the tick is a later job: list what each part needs there), any other crate, `map/`, or an existing test's expected values. A pinned law hash that changes is left failing and reported.
- No commits: the integrator commits.

## Done means

- `cargo test --offline -p law` passes with every new test; every earlier test result is unchanged (run the suite before your first edit and record it).
- Each new test fails under the mutant its evidence falsified; one mutant run shown per part.
- The report, per part: done or left (with both values); functions added (file:line); each test value's evidence file; the mutant run; the `movement.rs` wiring it needs. Then the full test summary line before and after, and the net lines.
