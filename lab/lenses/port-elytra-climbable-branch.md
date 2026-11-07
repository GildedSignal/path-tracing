## port-elytra-climbable-branch

Read staged LivingEntity.travelFallFlying. Pin only the onClimbable branch: when it switches to travelInAir, when it clears the flag, and whether free-glide movement executes at all. Regenerate exact offsets and produce a two-case call-order witness with a mutant that clears flight before invoking travelInAir. Leave cosine, boost and collision damage separate.

Before investigating, write a provisional dossier with Claim: at byte one and create the shortest runnable reproduce.sh. Save raw output from the first run before expanding. Use staged 26.2 jars under D:/path-tracing-lab/server/versions/26.2 or D:/path-tracing-deps/data/libraries/com/mojang/minecraft/26.2 as appropriate and the JDK at D:/path-tracing-deps/data/java/java-runtime-epsilon/bin. Keep every write in your result folder. Do not launch the game, edit production, commit or use the network. Say exactly which evidence was direct runtime, bytecode, or reconstructed arithmetic. Keep the checker entry at Grade: on line one.

## Follow-on checklist

Bank the first runnable proof and raw output before continuing. The earlier stop boundaries keep that first checkpoint narrow. The additional cases below are authorized after it. Work in order, record the verdict and reproduce command for every attempted item, and continue until the checklist is exhausted or a concrete blocker prevents further work. When time is tight, leave an exact first failing command or missing input for the next item. Keep all outputs in this result folder. The independent checker should rerun the primary result and list which follow-on claims it reproduced.

1. Verify the fluid versus fallFlying travel dispatch when both predicates are true.
2. Trace the first travel call after climb cancellation and the exact flag write.
3. Return an ordered mode-transition table and one contrasting mutant.
