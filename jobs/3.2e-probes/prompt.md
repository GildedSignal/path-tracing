# Job 3.2e: two probes on the planner branch, before the next spec is written

**Why.** An Opus red-team of the next planner spec asked for two cheap experiments that decide whether "cells are the bottleneck" is true. They are measurements, not features. Nothing here merges.

**Consumer.** The rewrite of `jobs/3.2d-sets/prompt.md` (the integrator).

**Branch.** You are in `D:\path-tracing-wt\probes` on `wt/probes`, cut from `wt/planner`. Set `CARGO_TARGET_DIR=D:/path-tracing-target/probes`. Read `engine/plan/src/reach.rs`, `src/search.rs`, `tests/flight.rs`, and the run records in `D:\path-tracing-evidence\planner\` (flight.txt; mode 5 is the field ordering the forward search).

## Probe 1: the walked hop's tube

`reach.rs` skips the tube check of a walked hop in two places (`if arc.grounded && !landing { return true; }` in the clear check, and the matching `continue` in the premises). Remove both skips so a walked hop's body box is checked at every tick like an arc's. Rerun mode 5 at cap 1 on `fences`, `fencemaze` and `redfakemaze`, and on the 8 tracks that had a sentence (to see what changes there).
- Report per track, before and after: sets reaching A, T at A by the sets, sentence or not, T, expansions, thinking time.
- If one of the three still stalls: take the joint where the search stood (the report's nearest node and its anchor), replay the claimed hop's program from several of the exact arriving states the forward search produced there, and log the first tick each one departs from the hop's table and what it meets (the cell, its block, the collision shape's boxes).

## Probe 2: momentum carried in

On `grassjumpeasy` the sets do not reach A ("no tool lands": stand -57.9375 to -57.0, rise 0.9375), yet the forward search finds a sentence. From the forward pass's real arriving states at that joint, run each of the reach table's tools through the law and say which land on the next set, with the arriving velocity of each. Compare with the same tools from rest at the cell's centre. One table: tool, from rest (lands or not), from the real state (lands or not), the velocity carried in.

## Must not

Add a tolerance or a special case per block. Change the law, `engine/words`, `bot/`, `docs/` or the gates. Run anything live. Commit, push or merge. Keep the edits minimal: this branch is thrown away.

## Done means

A final message with both probes' tables, the exact commands run, the run-record folders, and for each probe one sentence: what it shows and what it does not.
