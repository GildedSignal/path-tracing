# Is a held word's outcome monotone in its start offset?

Use the staged Mojang 26.2 server and client jars under D:/path-tracing-lab (server: D:/path-tracing-lab/server/versions/26.2/server-26.2.jar; the client jar is in the staged dependency root D:/path-tracing-deps/data) and staged javap at D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe.

**Consumer:** the zones design's red-team (jobs/zones-design/redteam-opus.md, finding 7 and test T7). It proposes that a portal between two zones can be certified exactly as (gait, phase, an interval of the translation offset s). The claim: along one word with fixed inputs, float addition and multiplication by positive constants, and the min/max clamps, are monotone in s. So each comparison the law makes flips at most once, and finitely many intervals cover all entry offsets.

The red-team names its weak premise. The collision axis order (`|dx| < |dz|`) and the step-up choice compare velocities, not s, and may break monotonicity.

**Build a jar-backed sweep** with the real LocalPlayer movement code (as in this lab's earlier trial-sprint-* lenses). A steady-state sprint-jump gait along +x is started at offsets s = x0 + k·ulp and over a coarse grid, in two worlds:
1. flat stone that becomes ice at a fixed x;
2. flat stone with a one-block gap.

For each s, record the tick of the first event (landing, contact, the first tick on ice) and the full state bits there.

**Report:**
- whether the exit tick and each exit field are monotone in s;
- how many breakpoints there are, and where;
- any non-monotone case, with its witness s.

**Falsifier:** any exit field that is non-monotone in s along one word.

Leave the rig alone.
