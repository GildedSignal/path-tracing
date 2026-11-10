# Lens: what each item writes when used (jar reader)

**Why it matters.** A tool is a law input the bot carries (`docs/representation.md`, section 4). To find which tool can help which motion without a hand-written catalogue of tricks, the engine joins two facts read from the game itself: what each law term reads (the footprints it records already), and what each item use writes. This lens produces the second, as the `hook-catalogue` lens did for blocks.

**The question.** In the 26.2 jar, which classes declare (not inherit) `Item.useOn`, `Item.use`, `Item.finishUsingItem`, `Item.releaseUsing` or `Item.interactLivingEntity`, and for each, which world writes does that method body reach inside its own class and its direct helpers: a block set or destroyed, a fluid placed or picked up, an entity added, a tick scheduled, a velocity or an effect given to the user?

**What to produce.**
- `reproduce.sh`: scans every class in the jar with `javap -c -p -constants` and prints `catalogue.tsv`.
- `catalogue.tsv`: one line per (class, method, write): the call written (`Level.setBlock`, `addFreshEntity`, `scheduleTick`, `setDeltaMovement`, `addEffect` and the like) with its offset, and the kind of thing written (block, fluid, entity, tick, the user's motion, the user's state).
- `dossier.md`: the claim in one sentence (how many item classes write the world, per kind), and why a class can't be missed.

**Deliver the first proof early.** Write `dossier.md` with `Claim: ` on its first physical line and a runnable `reproduce.sh` before expanding; save the first output as soon as it exists. Use `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and the JDK in `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin`. Write only in your run folder. No network, no live game, no production edits, no commits.

**Out of scope:** what each write does in detail, and blocks' own hooks (`hook-catalogue` has them). Don't paste method bodies; cite offsets.
