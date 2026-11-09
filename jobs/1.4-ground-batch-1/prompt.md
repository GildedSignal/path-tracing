# Job 1.4: ground-layer batch 1, three reproduced GLM deltas

**Why.** S55: about 5 bots with pathfinding on Julien's tracks and his server. GLM lanes found these three gaps in the law. An independent GLM checker reproduced each, and each has a consumer now:

| Delta | Run folder (main checkout, `lab\results\`) | Consumer |
|---|---|---|
| Swimming-mode transition: `Player.updateSwimming` (the flight gate) over `Entity.updateSwimming` (sprint + underwater entry, exit conditions); the law lacks the passenger and flight terms | `local-20261109-1523-1-4-port-movement-modes` | water in his tracks (carpetwater) and in Phase 3's swim words |
| Knockback rewrites velocity; the law's only contact damage, HotFloor, is a no-knockback type, so this is missing | `local-20261109-1523-1-2-port-entity-rules` | bots on his server get hit by mobs; the deviation stream |
| The world exporter classifies no `stepOn`, so exported magma never burns | `local-20261109-1554-1-0-port-block-laws` (`proposed-fix.patch`) | recorder fixtures and exported terrain |

**Branch.** `wt/ground-1`, worktree `D:\path-tracing-wt\ground-1`, cut from `main`. Read each run folder's `dossier.md`, `check.md`, the witness and the patches. They're read-only for you.

## Build

- **For each delta:** port the rule into the law (or the exporter, for magma) from the jar's bytecode, as the dossier cites it. Keep the dossier's witness as a regression test: it must fail before your change and pass after. A patch in the run folder is a lead, not a spec: check it against the bytecode, and keep the smallest form that's exact.
- **No patches in the AGENTS.md sense:** no special cases, tolerances or per-block branches. If a delta needs one, stop and report why.
- **Footprints:** any new block read goes through the law's tagged reads (`hook_block`, `fluid_block`, `observe_region`; see `docs/footprints.md`), so the footprint stays complete.
- **Knockback:** model it as an input event, like `server_event` (the attacker's direction and strength recorded as values), not as randomness. The law draws no randomness and reads no clock.

## Test, fast

- Each witness, red before and green after.
- `cargo test` from `engine/` is green, and `jobs/1.2-footprints/verify.ps1` still catches every mutant.
- The bench (`cargo run --release --bin bench` from `engine/`) stays under 1 µs per tick.
- The replay suites stay bit-identical where no delta applies.

## Must not

- Touch Julien's world, run live gates, or start GLM runs.
- Push or merge.

## Done means

Three commits, one per delta, each with its witness red → green. The final message lists each delta's law change (file and function), its witness result and the bench numbers.
