## port-fall-damage-cooldown-tag

Check only whether vanilla 26.2 fall damage bypasses the hurt cooldown. Read the actual staged damage type tag data and LivingEntity.hurtServer guard. Build one reproducible pair with lastHurt=1, cooldown=11 and equal raw damage, comparing fall with a known cooldown-bound source. Distinguish tag and branch proof from runtime execution; stop before armor, enchantments or fall geometry.

Before investigating, write a provisional dossier with Claim: at byte one and create the shortest runnable reproduce.sh. Save raw output from the first run before expanding. Use staged 26.2 jars under D:/path-tracing-lab/server/versions/26.2 or D:/path-tracing-deps/data/libraries/com/mojang/minecraft/26.2 as appropriate and the JDK at D:/path-tracing-deps/data/java/java-runtime-epsilon/bin. Keep every write in your result folder. Do not launch the game, edit production, commit or use the network. Say exactly which evidence was direct runtime, bytecode, or reconstructed arithmetic. Keep the checker entry at Grade: on line one.

## Follow-on checklist

Bank the first runnable proof and raw output before continuing. The earlier stop boundaries keep that first checkpoint narrow. The additional cases below are authorized after it. Work in order, record the verdict and reproduce command for every attempted item, and continue until the checklist is exhausted or a concrete blocker prevents further work. When time is tight, leave an exact first failing command or missing input for the next item. Keep all outputs in this result folder. The independent checker should rerun the primary result and list which follow-on claims it reproduced.

1. Read every relevant damage-type tag transitively and verify which one gates the cooldown.
2. Exercise equal and stronger raw hits in the eligible branch and compare lastHurt and cooldown after each.
3. Separate source flags, amount computation and effective debit into explicit input facts.
