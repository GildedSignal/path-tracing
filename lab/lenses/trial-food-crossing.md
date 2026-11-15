# Food crossing: when a held sprint-jump on stone stops being allowed

Use the staged Mojang 26.2 server and client jars under D:/path-tracing-lab (server: D:/path-tracing-lab/server/versions/26.2/server-26.2.jar; the client jar is in the staged dependency root D:/path-tracing-deps/data) and staged javap at D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe.

**Consumer:** the zones design's red-team (jobs/zones-design/redteam-opus.md, finding 1 and test T4a). It claims that a bare-kit player sprint-jumping on flat stone from spawn bars (food 20, saturation 5, exhaustion 0) reaches food 6, where `canSprint` refuses, after about 320 jump cycles (~1.4k blocks). It also claims the per-cycle exhaustion amounts are fixed constants, so (cycle phase, exhaustion bits) recurs with a super-period. The same claims are needed with arming's fill (food 20, saturation 20).

**From the 26.2 bytecode**, state with class, method and offsets:
- every exhaustion write a held forward+sprint+jump causes on flat stone: the jump's amount; the per-distance sprint amount and how the distance is measured and rounded (Player.checkMovementStatistics, the cm rounding); which side computes it;
- FoodData.tick's order of exhaustion → saturation → food;
- the food>6 sprint guard;
- what peaceful difficulty changes (regeneration, hunger).

**Build a jar-backed reproduction** with real FoodData objects, fed the exact per-tick exhaustion sequence of one 12-tick sprint-jump cycle in steady state. Take that sequence from the server's own arithmetic, not from our law. Print:
- the tick and cycle at which food first reaches 6, from (20, 5, 0) and from (20, 20, 0), on Normal and on Peaceful;
- whether (phase, exhaustion bits, saturation bits) recurs before that, with its period.

**Falsifier:** a crossing more than 10% away from 320 cycles, or no fixed per-cycle amount.

Leave the rig alone.
