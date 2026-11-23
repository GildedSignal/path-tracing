# Generator rows: the player's resource and cooldown clocks

**Goal:** one parameter row per clock the player carries in 26.2 (food, saturation, exhaustion, air, fire, freeze, effects, item cooldowns, invulnerability, regeneration), each backed by bit-exact jar witnesses, for the shared clock generator.

Health, food and the inventory are resources spent in T (`map/goal.md`). The law is being compressed onto a few shared generators (`map/now.md`, step 2); one of them is the clock: a counter that rises or falls each tick, fires an event at a threshold, and resets. This job does not port Rust; it extracts each clock's parameters and scars so the generator can be checked against Mojang. The planner today starts with saturation 5.0 while the bot has 20.0; the food guard matters.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`.

**From the bytecode** (class, method, offsets), per clock: the step per tick (and per action: sprint, jump, sprint-jump, swim, break, attack, take damage — the exhaustion costs), the threshold and what fires there, the reset value, whether the decrement happens before or after the check, the difficulty and game-rule dependencies (natural regeneration, peaceful), and the order the clocks run inside one player tick (`Player.tick`, `LivingEntity.baseTick`, `FoodData.tick`). Every irregularity is a scar row with its method and offset.

**Witnesses:** run the jar's own `FoodData`, air-supply, fire and freeze code (a tiny Java harness, no server if you can avoid it) through scripted 600-tick sequences: sprint-jumping from food 20 / saturation 20, from food 20 / saturation 5, from food 7 (the sprint guard), under water from full air, in fire and out, in powder snow and out, with Regeneration and Hunger effects; write the state of every clock per tick.

**Write in your run folder:**
- `rows.tsv`: `clock	unit	step	threshold	on_threshold	reset	check_order	depends_on	scars	source`.
- `witnesses.tsv`: `sequence	tick	food	saturation_hex	exhaustion_hex	air	fire_ticks	frozen_ticks	health_hex	effects	sprint_allowed` (floats as hex bits).
- `dossier.md` (first line `Claim: `), `reproduce.sh` regenerating both from the jar, and `wiki.md`: a short `lab/wiki/` note on the tick order of the clocks and the first tick a sprint-jump run loses sprint from each start.

**Falsifier:** any witness tick where the rows, run as one generic clock in the stated order, give a different state than the jar.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
