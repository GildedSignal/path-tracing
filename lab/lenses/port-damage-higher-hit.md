## port-damage-higher-hit

Use LivingEntity.hurtServer to isolate the only higher-damage transition inside the active hurt cooldown. Compare raw amount 1.0, nextUp(1.0), and 2.0 with lastHurt 1.0 and cooldown 11. Verify whether damage passed to actuallyHurt is full or incremental, and the update order for lastHurt and cooldown. No armor, effects, regeneration or live game.

First create dossier.md with an exact Claim: line at byte one and a provisional one-sentence claim. Create reproduce.sh immediately and put only evidence under your run folder. Use the staged Mojang 26.2 server jar at D:/path-tracing-lab/server/versions/26.2/server-26.2.jar and the JDK at D:/path-tracing-deps/data/java/java-runtime-epsilon/bin. Read previous results without editing them. Run the reproduction and report its exit, raw values and evidence type. No production edits, network or live game.
