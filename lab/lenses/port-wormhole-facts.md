# Wormhole facts: Nether portals, pearls and unloaded chunks

Use the staged Mojang 26.2 server and client jars under D:/path-tracing-lab (server: D:/path-tracing-lab/server/versions/26.2/server-26.2.jar; the client jar is in the staged dependency root D:/path-tracing-deps/data) and staged javap at D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe.

**Consumer:** the zones design's red-team (jobs/zones-design/redteam-opus.md, findings 11 and 14, test T11). Julien (S132) calls a Nether portal "a manifold" you open by building a frame, and pearls wormholes. The design must price them from the jar, not from memory.

**From the 26.2 bytecode**, state each with class, method and offsets.

1. **The portal, as the player goes through it:**
   - the portal wait in ticks (survival, creative);
   - the cooldown after a transit;
   - the coordinate scale between the sheets, and where it is applied.
2. **Linking:**
   - the exit search radius in each dimension (PortalForcer or its 26.2 successor);
   - the rule choosing among several existing portals;
   - what the game builds when none exists: the frame, the platform, the air it clears.
3. **The ender pearl:**
   - the throw's velocity and inaccuracy (the shoot call's divergence and which RNG);
   - whether and how the thrower's momentum adds in;
   - the landing's damage, cooldown and teleport rules.
4. **The client player's tick in an unloaded chunk:** the branch that sets vertical speed to −0.1 (our law has it at movement.rs:444-450), what it reads to decide "loaded", and whether horizontal motion and fluids are skipped.

**Build a jar-backed reproduction** for items 3 and 4: print a pearl's first 10 ticks of motion with and without thrower momentum, and one tick of a player in an unloaded chunk. Items 1–2 can be offsets only.

**Falsifier:** any value that differs from the red-team's statement in findings 11 and 14.

Leave the rig alone.
