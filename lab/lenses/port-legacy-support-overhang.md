## port-legacy-support-overhang

Pin Entity.getOnPosLegacy/getOnPos for a supported entity whose center crosses one horizontal cell while its feet remain on the original support. Use actual method offsets and one reproducible input where legacy support and floor(center) differ. Check only the support-column selection and the below-block fallback relevant to stepOn, with a mutant that floors the center.

Before investigating, write a provisional dossier with Claim: at byte one and create the shortest runnable reproduce.sh. Save raw output from the first run before expanding. Use staged 26.2 jars under D:/path-tracing-lab/server/versions/26.2 or D:/path-tracing-deps/data/libraries/com/mojang/minecraft/26.2 as appropriate and the JDK at D:/path-tracing-deps/data/java/java-runtime-epsilon/bin. Keep every write in your result folder. Do not launch the game, edit production, commit or use the network. Say exactly which evidence was direct runtime, bytecode, or reconstructed arithmetic. Keep the checker entry at Grade: on line one.

## Follow-on checklist

Bank the first runnable proof and raw output before continuing. The earlier stop boundaries keep that first checkpoint narrow. The additional cases below are authorized after it. Work in order, record the verdict and reproduce command for every attempted item, and continue until the checklist is exhausted or a concrete blocker prevents further work. When time is tight, leave an exact first failing command or missing input for the next item. Keep all outputs in this result folder. The independent checker should rerun the primary result and list which follow-on claims it reproduced.

1. Compare partial support at opposite sides of a seam.
2. Trace the no-support fallback and the fence or wall below-block branch, with one negative control.
3. Identify which exact existing Rust on_pos assumptions each witness exercises.
