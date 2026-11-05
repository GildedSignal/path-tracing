# Lens: the techniques elite players use, proven under the law

**Why it matters.** Julien's litmus test (S32): the bot should rediscover what elite performers use, without being told. The rediscovery rung (`docs/ladder.md`) needs the list of those techniques. The first sweep (`docs/reference/research-2026-11-05.md`) leaned on the wiki. Julien (S33): "wiki was there to think more than spread gospell it is not known to be perfect unlike reading directly the code."

**Sources.** Go wide: speedrun guides and routing documents, speedrun.com resources and forums, parkour community guides (mcpk and others), TAS write-ups and their input files, video descriptions and transcripts, Reddit and Discord archives that are public, technical-Minecraft guides, mod and tool documentation. The wiki is one source among many, never the authority. Prefer sources made by people who compete, since they have skin in the game.

**The law.** Branch `wt/law` (`git fetch origin wt/law`), crate `engine/law`, provisional until rung L0 passes. The 26.2 jar is the final word: the setup step stages it.

**What to establish.** For each technique you find:
- what it does, and the mechanism it exploits, named as the method in the 26.2 jar that produces it (class and method);
- whether it still works in 26.2 (many guides are for 1.8 or older);
- **the proof:** a `BoxWorld` and an input sequence under the law where the technique beats the best run without it, with both T values in ticks. If the law can't simulate it yet, give the jar method and grade it a lead.

**What to produce.**
- `reproduce.sh`: re-runs every proof and prints both T values per technique.
- `techniques.tsv`: technique, mechanism (jar method), works in 26.2 (yes/no/unknown), T with, T without, source URLs.
- `dossier.md`: what you found that isn't in `docs/reference/research-2026-11-05.md`, section 2, and where the sources disagree with the jar.

**Out of scope:** techniques that need mods, client/server desyncs the server corrects, and anything that leaves the world changed at B.
