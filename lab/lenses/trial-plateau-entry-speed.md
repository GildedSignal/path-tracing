# Do plateaus stay finite and single-ULP clean across entry velocities and gait phases?

Use the staged Mojang 26.2 server and client jars under D:/path-tracing-lab (server: D:/path-tracing-lab/server/versions/26.2/server-26.2.jar; the client jar is in the staged dependency root D:/path-tracing-deps/data) and staged javap at D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe.

**Consumer:** U2 set record red-team (`jobs/u2-set-record/report.md`, one-way-door questions 1 and 10, `docs/representation.md` section 10). Inside a **site** (joints closer together than momentum's memory: ~263 ticks on stone, ~324 on ice), arriving states do not sit on a single steady-state gait velocity — entry speed `v` and cycle phase vary. The set record needs to know whether entry speed shifts plateau breakpoints smoothly (or multiplies plateau counts), and whether a finite number of speed classes / slices suffices.

**Build a jar-backed sweep** with the real `LocalPlayer` movement code (exact 26.2 client jar, as in `trial-offset-monotone`):
1. Take three joint patterns where momentum matters:
   - stone-to-ice edge followed 4 blocks later by a 2-block gap;
   - blue-ice head-hitter (`2.0`-block ceiling) exiting into a 5-block gap;
   - stone 1-block gap after `k` ticks of sprint run-up (`k = 1, 4, 8, 12, 24, 60`).
2. Sweep start offset `s` over `[s_min, s_max]` for each entry-velocity / run-up `k` (and 8 uniformly spaced initial speeds `v` from rest to ceiling head-hitter speed), bisecting every transition in `(exit_kind, exit_block, exit_tick)` to single-ULP precision.

**Report:**
- per `(joint pattern, entry speed / k)`: plateau count, breakpoint `s` values (exact bits), `T` per plateau, and plateau widths;
- whether breakpoints move monotonically with entry speed `v` (enabling seeded bisection across a joint family at `~7x` lower cost, `jobs/unified-plan/learning-review.md` section 2), or split/create new plateaus abruptly;
- minimum entry-speed resolution (if any) at which plateau boundaries shift by `>= 1` tick.

**Falsifier:**
- plateau count explodes (`>> 10^2`) at intermediate entry speeds or head-hitter exit velocities, OR
- single-ULP breakpoint smoothness breaks down at non-steady entry velocities (chaotic ULP-level flipping).

Leave the rig alone.
