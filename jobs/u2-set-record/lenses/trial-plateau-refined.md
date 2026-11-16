# Kill test 11: are plateaus finite per joint pattern under refined bisection on natural terrain?

Use the staged Mojang 26.2 server and client jars under D:/path-tracing-lab (server: D:/path-tracing-lab/server/versions/26.2/server-26.2.jar; the client jar is in the staged dependency root D:/path-tracing-deps/data) and staged javap at D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe.

**Consumer:** U2 set record red-team and Kill test 11 (`docs/learning.md` section 6, `jobs/u2-set-record/report.md`, `jobs/unified-plan/plan.md` section 10). `trial-offset-monotone` measured only two artificial patterns (stone-to-ice and a one-block gap), and its independent check (`lab/results/local-20261115-1236-1-0-trial-offset-monotone/check.md`) found both non-monotone `T` (`12->14->12`) and a sub-grid sliver (`1.2e-10` to `7.2e-5` wide) missed by a `1/2048` coarse grid. The set record freezes only if plateau counts stay small and stop growing as bisection is refined across natural-terrain joint patterns.

**Build a jar-backed sweep** with the real `LocalPlayer` movement code (exact 26.2 client jar, same harness family as `trial-offset-monotone`). Test at least 12 distinct joint patterns drawn from natural terrain and track defects (no hand-picked easy subset):
1. flat stone to ice / blue ice / soul sand / honey / slime;
2. 1-block gap, 2-block gap, 3-block gap (flat and +1 / -1 step);
3. 1-block step-up, half-slab step-up (`0.5`), stair step-up, carpet/button/pressure-plate crossing;
4. 2.0-block ceiling entry (head-hitter transition) and corner/wall grazing (`±0.3` overhang);
5. oblique yaw (e.g. `yaw = 22.5` or `35.0` deg) across a class boundary or 1-block gap.

For each joint pattern, start a steady-state sprint-jump gait (and one walking gait for comparison) over the full face span `[s_min, s_max]` at successively finer initial scan resolutions `1/64`, `1/256`, `1/1024`, `1/4096`, `1/16384`, plus analytic geometric pre-cuts (cell edges ± `0.3` body radius). At every transition in `(exit_kind, exit_block, exit_tick)`, bisect IEEE `f64` bits to single-ULP adjacency (ensure the bisection predicate compares the true exit field, avoiding the `Sweep.refineFlip` index bug of `trial-offset-monotone`). Also probe `±64` ULP around each found breakpoint for ULP-level ripples.

**Report:**
- per joint pattern and per initial grid resolution: plateau count, min/p50/p90/max plateau width, exact hex ULP bits of every boundary, and count of non-monotone `T` reversals;
- every new plateau or sub-grid sliver discovered only at `1/4096`, `1/16384`, or analytic pre-cuts, with its width and witness `s`;
- whether any ULP-level oscillation (>1 flip inside `±64` ULP) occurs anywhere.

**Falsifier:**
- plateau count grows with refinement resolution (`1/1024 -> 1/4096 -> 1/16384`) on any natural-terrain joint pattern (unbound/fractal splitting), OR
- ULP-level ripples (>1 transition inside `±16` ULP) occur at a boundary, OR
- sub-grid slivers missed by `1/1024` cannot be captured by analytic `VoxelShape` ± `0.3` geometric pre-cuts.

Leave the rig alone.
