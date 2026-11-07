## port-item-cooldown-boundary

Use ItemCooldowns staged bytecode or direct runtime, starting with one item/group and duration of two ticks. Record isOnCooldown and getCooldownPercent before, at, and immediately after the expiry tick. Test exact raw bits where relevant and kill a one-tick-late removal mutant. Stop before networking or item use effects.

Before investigating, write a provisional dossier with Claim: at byte one and create the shortest runnable reproduce.sh. Save raw output from the first run before expanding. Use staged 26.2 jars under D:/path-tracing-lab/server/versions/26.2 or D:/path-tracing-deps/data/libraries/com/mojang/minecraft/26.2 as appropriate and the JDK at D:/path-tracing-deps/data/java/java-runtime-epsilon/bin. Keep every write in your result folder. Do not launch the game, edit production, commit or use the network. Say exactly which evidence was direct runtime, bytecode, or reconstructed arithmetic. Keep the checker entry at Grade: on line one.

## Follow-on checklist

Bank the first runnable proof and raw output before continuing. The earlier stop boundaries keep that first checkpoint narrow. The additional cases below are authorized after it. Work in order, record the verdict and reproduce command for every attempted item, and continue until the checklist is exhausted or a concrete blocker prevents further work. When time is tight, leave an exact first failing command or missing input for the next item. Keep all outputs in this result folder. The independent checker should rerun the primary result and list which follow-on claims it reproduced.

1. Add a re-add or override during an active cooldown and an independent other-group control.
2. Trace the server-side start/end notifications and their order relative to map writes.
3. Check whether zero-duration insertion has the same observable behavior as an absent entry.
