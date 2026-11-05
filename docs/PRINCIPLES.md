# The goal and the philosophy

These are Julien's words. Every model reads this before acting, and every sub-agent prompt begins with its first two sections. Models propose changes and never reword them. Julien can be wrong: each statement is remembered, then either followed or disproven with evidence. Silence is never disproof. The labels are ours; the quotes are his. Keys such as (S6) and (X2) point to [his turns, verbatim](reference/julien-2026-11-04.md).

## The goal

"make minecraft's best path tracing algo in the world, which means we'll have to go first principle, it should be extremely fast to run and get to the point in the shortest amount of time T" (S6). "this is meant to be world class, garbage in, garbage out." (X1)

Here, path tracing means **pathfinding**: a bot in his lab that gets to the point he clicks with the stick in the least time.

The scoreboard, adopted with the plan on 4 November, has three parts:
- T is counted in server ticks, from his click until the bot arrives with everything restored, so thinking time counts.
- It is judged against a bracket: a proven lower bound and the best T found by an independent search.
- It is reported as a distribution, never as an average alone.

## The philosophy (always on)

1. **T is the one currency.** "basically this is an optimization problem in spacetime where we're trying to reduce T under the limits of the what the space allows us but also our health is a limiting factor too, it is a ressource that we can spend, so is hunger and what's in the inventory." (S6) "your goal is to coalesce everything into something holistically optimal, this is why i collapsed the problem down to T." (X2)
2. **First principles, by problem type.** "this project is first principle this should be kept in mind so the goal is to properly define the problem and actors and based on that find the best solution not starting from things that already exist for our particular use case but thinking in terms of problem types and solutions so this is rather polymathic." (X2) "i'm just going full first principle to not miss paradigm shifts moments." (X2)
3. **Representation first; the search must be trivial.** "the most important part so far is how we should represent things so that we can make the path search trivial and obvious" (X2). "with the wrong representation the actions will be obfuscated which is bad, like i said many times they shall be trivial, i'm pressing my words here." (X4) "with the right representation is shall be trivial and obvious." (S6)
4. **Capabilities shape the space.** "possessing the bucket changes the space itself placing the bucket is just a movement not a thinking action, very different thing. for example being on a bike changes how i look at the space effectively equivalent to changing it because i can do more." (X5) "a wall is a two second delay not a wall" (X4). "There's where the player is and where it can reach conditioned by its toolkit and how it interacts with the terrain, it's basically function composition or something." (X4)
5. **Composition, order, limits.** "it's a bunch of systems that shape the world where you can combine those system and order matters (doesn't that remind you of certain type of math?) and you have limited amount of them." (X4) "having access to a pickaxe changes many things of a certain type while having a block allows you to look further because you can use it to jump" (X4). "a bucket is something that negates downward verticality penalty and create upward ones, with the right representation it should be natural that the bot thinks of using a water bucket above him, float, take it back, put it back higher, and repeat." (X4)
6. **The simplest form.** "be careful to not overthink it and come to the simplest form possible, we're looking at many things but the resulting idea will be genuinely simple." (X4) "You're overthinking deeply, this is all the same thing just local variations but same principles." (X4)
7. **Shape the space so the problem solves itself.** "my algo for thinking is always do the least amount of compute by having the shape of the space itself solving the problem and having to do the least amount of work, a straight line, all of this cost compute but you need to spend compute in a way that changes things for the best so akin to how i can model clay in a mug shape so that it holds my coffee and doesn't spill it instead of using the clay as a scraper to move it." (X6)
8. **Everything said compounds; nothing is thrown away.** "do not underestimate any single thing that i say they are all with purpose and not thrown away." (S6) "they compound, yes the magical word, compounding. reread through all the things i said and now it'll make sense." (X6)
9. **The best compute is no compute; reuse it.** "I don't think we can just always recompute things we need to resuse that compute or we'll be not efficient." (S6) "we need to reduce updates that are not needed as much as possible" (X2). "The best compute is no compute." (X2)
10. **A causal chain, bounded by light cones.** "using the "light-cone" of the player to know what it can affect but also what can affect it, this is a dynamical problem do not forget about it but also of interactions that are bidirectional." (S6) "this is really a causal chain issue if we can run minecraft very fast we can predict what will happen in the future besides player agency which has a light cone limited by movements and ticks so anything the player does is updating something just like how it is though of in ES." (X2) "you have to think of the whole causal chain that matters and not the rest." (X2)
11. **Local repair.** "I don't want to have to recompute a whole region because one block changed that's just not going to work out." (X2)
12. **Rebuild Minecraft for us, fast, minus what doesn't matter.** "run minecraft literally but 100x faster to predict the future but the good thing is that we can do it smartly we don't care about textures we don't care about many things actually and we don't care about what's outside the light cone." (S6) "yeah we need to rebuild minecraft in a way that just works for us." (S6) "we'll need minecraft's engine in rust (i believe someone already did this and then optimize it for us and remove as much as possible)" (X2). "i want to rewrite the engine to run faster because it'll be simpler and therefore can could explore it" (X6). "Rust port first" (S11).
13. **Whatever changes the world is part of the world.** "even a zombie can be precomputed and can even help us in some niches cases that will never happen basically but still. Even redstone needs to be thought of." (X2) "The player could even create a nether portal it can go very far." (X2)
14. **Coordinates are free; the system is not.** "we can change them, changing coordinate system does not change the system be mindful of it." (X2) "Basically minecraft is 3D but do we really care about it? unless we're doing elytra and in water maybe 3D cartesian is not the best representation system we're caring about geodesics." (S6) "i think we need to compact the space so that the player is one block tall maybe?" (X4)
15. **Good enough, with feedback.** "it's also ok if we're not perfect that's how humans run they have a cheap model with error bars in their head and update based on feedback from its sensors." (X2) "jumping on a cube does not have to be precise if after you can easily correct, we need to know when we can make mistakes simple as that, it's a resource allocation problem with time, bytes and compute those are all resources that can be used." (X2)
16. **Mistakes have shapes.** "We can learn from Taleb because we will be in situations where we'll be able to make certain types of mistakes and characterizing them will help, there's a certain convexity to them." (S6) "inverting the chaotic space where you find a representation such that any ∂ (in the contexts we care about) does not generate massive variations which links a bit to my error type idea." (X2)
17. **Every scale, and the details that matter.** "The big issue is working at all scales in space and time." (S6) "given that we update often maybe a non euclidean way of seeing things where we get detail up close and big pictures from afar is what we care about" (X2). "the algo should not get lost in minute details when big ones could be better but sometimes like in chess minute details do matter." (X2) "physics has often been solved that way look at Einstein and how he got unlocked when he managed to deal with infinities" (X2).
18. **Store the generator.** "we need to store the generative function of things like a ladder has a rectangle where you can use it but you float too and shift modifies it." (S6) "block properties are very wide so we need to somehow convert them to T, ice helps us to go faster but has lower friction in a dynamical regime hence why we need a proper representation system like in emergent space (ES)." (S6) "stone and dirt travel as fast on it by foot but breaking them does not take the same time" (S6).
19. **Ready for sudden commands.** "It is to be kept in mind that multiple commands can and will be used changing plans all of the sudden and we should be ready for it." (S6)
20. **Leave no damage; keep what you have.** "i do not want the bot to destroy things and he does he should build back so a temp block has to be broken again and picked up." (X2) "you need to keep the things you have, amortization." (X4) "identical block." (S12) "image a swarm of bots, what happen with time if they can modify blocks to builds in their passage? now you get it? it compounds and deforms things in a game where you care about building, so even redstone state matters and the downstream effects too are hard to compute sometimes, even moving a trapdoor to crawl you have to put it back because it might be used decoratively. Basically don't modify the world to get around there's many ways to move in minecraft." (S13)
21. **An omniscient player.** "we suppose the player is omniscient." (S6)
22. **Steal from every field.** "we could learn A LOT from alphazero the parallel will be stronger than you'll first think of." "Ants are interesting inspirations there manage to find the geodesics with very simple local algos." "Google is the king of mapping and we can learn A LOT from them, they use crazy algos but also precompute nodes." "i'm thinking of using fast multipole method in some ways not for gravity but for how block's T can change things at a distance." "monte carlo can help us." "maybe we'll need to train a model or create heuristics if math fails us or most likely a mix of all of them." "we'll learn A LOT from my other github project emergent space engine you have to study it and its philosophy" (all S6).
23. **Every part of the machine.** "we need to run on the gpu and on threads." (S6)
24. **Think together first, then build fast.** "So the goal is to iterate together on the idea, not build for now but gather resources on think" (X1). "i'm not asking you to do it but to think first, let's talk" (S8). "We're doing the spaceX loop here." (X4) "I'm making sure you really understand what i'm saying." (X4)
25. **The right intellect for each job.** "plan for the whole build with the appropriate sub-agents doing the right things with the right intellect tackling things in the right order. properly log the philosophy of the project as it is long especially on the engine side of things." (S10) "you'll be able to use gpt as a sub-agent too it tends to be good with well formed tasks while you do better when taste is required." (S12)

## From Emergent Space

His ES philosophy ([PRINCIPLES](https://github.com/GildedSignal/emergent-space-engine/blob/main/docs/PRINCIPLES.md)) applies here wherever it doesn't conflict with the above ("my thinking in planing ES is like that too", X6).

**Carried over** (numbers are ES PRINCIPLES numbers; the glosses are mine):
- 1 less is more: delete before optimising; simplify, never dumb down;
- 2 it's a game, so error type matters more than error size. Here "exact" means equal to Mojang's code: the bot's client is the oracle and judge, and the Rust port is the implementation;
- 3 compute only surprises: a surprise is an event that breaks a certificate; what can be derived when looked at is derived then ([engine.md](engine.md));
- 4 everything emerges: no central control and no catalogue of tricks or block categories;
- 5 the player is the only wildcard, and light cones partition compute. Here the wildcards are Julien (his commands and edits) and, for now, whatever chooses (mobs) plus the bot's own execution noise ([engine.md](engine.md)). **Open:** his "even a zombie can be precomputed" (X2) may shrink that list;
- 6 store the generator, not the generated: the recurrence and its parameters, shared, with arcs derived on demand ([engine.md](engine.md), "Continuous where it helps");
- 7 spacetime: in the right frame, geodesics are trivial. Here the least-T route is not straight, because momentum is in the state; the frame is what makes the search trivial. **Multipoles are the mechanism** for far influence and causal regions (see the register's multipole row);
- 9 a failed experiment kills the configuration, never the method;
- 10 measure before claiming, test like a Spartan: climbing ladders, extreme and chaotic rungs, planted faults, anomalies as data ([ladder.md](ladder.md), "How we test");
- 11 text has a cost;
- 12 smoothness and the allocator: the p95 and p99 matter, not the average; the allocator that prevents heavy tails is the most important algorithm (first-move latency, hundreds of bots);
- 13 errors never compound; types of errors, not just errors ([engine.md](engine.md));
- 15 local laws: a cell's table depends only on its footprint and its border states, so cells can be solved anywhere, in any order, in parallel;
- 17 every part of the machine, and the best part is no part: threads, the GPU for integer work, memory traffic, idle time computing ahead;
- 18 the right question, then the representation;
- 19 a fix signals the wrong representation: any special case, tolerance or retry added to pass a check ([AGENTS.md](../AGENTS.md), "No patches");
- 20 behaviours over many seeds, no engineered outcome: a behaviour counts only if it emerges from the law, judged over many seeds (R-brach; [ladder.md](ladder.md), "How we test");
- 21 rough first, refine only where a certificate could break: narrow moves get precision, wide ones never do;
- 22 the simulation decides, heuristics schedule: pruning is deciding, so a candidate is removed only by proof ([engine.md](engine.md));
- 23 nothing past c: here c is the toolkit's top speed, the light cone that bounds both T (the bracket's LB) and where a change can matter;
- 24 the memory floor is information: bytes per bot and per cell are measured against what they must hold;
- 25 no sunk cost; avoid ruin: an irreversible loss is a barrier, never a price ([engine.md](engine.md));
- 26 steal from skin in the game;
- 27 the goal, never a proxy: progress is T, cost and scale, not gates passed ([AGENTS.md](../AGENTS.md));
- 31 the logs hold the philosophy; keep contradictions; check the incentives we create (the register, the contradictions table, the attractors);
- 32 symmetry is found, not given: identical footprints and mirrored runs are detected by hashing, never declared;
- 33 each scale resolves what matters at that scale; smaller influences enter as aggregates, never dropped;
- 34 the oracles are guards, not shepherds: Mojang's law rejects a wrong port, but never designs the representation;
- 35 start from the goal and the dynamics.

**Not carried over:**
- **14, "no ticks":** Mojang's law ticks at 20 Hz. The planner may jump from event to event, but a plan is always checked tick by tick.
- **16, "cheapest arithmetic":** the law's numbers are Mojang's floats and doubles, not ours to choose.
- **28 (the view is an instrument), 29 (history, not a winner), 30 (what must be perfect: navigation, knowledge, dynamics):** about ES as a game. The goal here is T.

**Kept side by side:** 8, "cathedral foundations, not an MVP", against "We're doing the spaceX loop here" (X4); see the contradictions table.

Integer arithmetic does carry over for time. T is a whole number of ticks, so composing times by min-plus over integers is exact and associative.

## Register of his statements

Status words: **Followed** (built, or a rule in force), **Adopted** (decided in a living doc, not built yet), **Answered** (a question he asked, answered with the reason), **Open**, **Superseded** (by his later words or his approval), **Disproven** (with evidence).

| Statement | Source | Status |
|---|---|---|
| "create a first principled path tracing algo"; set up first | S1 | **Followed:** the lab (README) |
| his own game, the bot in another game on his second monitor, a superflat world on vanilla, his own controls, pointing with a stick, everything on this computer | S1 | **Followed:** the lab |
| "by vanilla i mean just keep the perf mods from my custom modpack … i still want the game to run well" | S1 | **Followed** for his client. **Superseded** for the bot's client and the server by the approved plan (4 November): Lithium and C2ME come out so the judge runs Mojang's own code. Every other performance mod stays |
| EULA accepted; init and commit; public on GitHub; gh installed; signed in | S2–S5 | **Followed:** GildedSignal/path-tracing |
| minimise T; extremely fast to run; health, hunger and inventory are resources | S6 | **Adopted:** principle 1, the scoreboard ([ladder.md](ladder.md)) |
| 3D may not be the right representation; geodesics | S6 | **Adopted:** [representation.md](representation.md) (the bot is a point on sheets joined by arcs) |
| ants find geodesics with simple local rules | S6 | **Adopted** as stigmergy: reuse stored in the world (cached tables). Ants also refract at borders between fast and slow ground, as light does (Oettler et al., 2013; [leads](reference/leads.md)) |
| the light cone: what it can affect and what can affect it; dynamics; two-way interactions | S6 | **Adopted:** [engine.md](engine.md) (slack, dependencies), representation.md (the cone) |
| run Minecraft 100x faster; no textures; nothing outside the light cone | S6 | **Adopted:** engine.md. Measured: a stripped tick is ~10⁻⁷ s, so the margin is far beyond 100x; choosing what to simulate is the bottleneck |
| combinatorial explosion; a trained model, heuristics or a mix | S6 | **Adopted:** events instead of ticks, merged states, learned ordering that only schedules (Phase 10) |
| chess and AlphaZero; "we are whites and minecraft world is blacks" | S6 | **Answered:** the parallel holds piece by piece (transpositions, bounds, evaluation, move order, tablebases, quiescence, bitboards). Black really plays only where something chooses (mobs, his commands, our own execution noise); the deterministic core is a puzzle, so A* applies (DeepCubeA) |
| list every block property: when does it save time? dirt and stone walk alike but break differently; grass and dirt are somewhat one category | S6 | **Adopted:** properties read from the game itself (runtime dump, Phase 1.2); categories emerge per action |
| study ES and its philosophy | S6 | **Followed:** studied on 4 November; this section, engine.md |
| rebuild Minecraft for us; the player is omniscient | S6 | **Adopted:** engine.md; region files plus a server observer |
| "do not underestimate any single thing that i say" | S6 | **Followed:** this register |
| ice: faster but dynamical; convert properties into T | S6 | **Adopted:** speed in the state only where friction remembers it (representation.md). Measured: ice sprint-jumping at 9.2 m/s, 19.3 under a ceiling, against 5.6 m/s sprinting |
| store the generator: a ladder's rectangle, floating, shift | S6 | **Adopted:** the law's hooks as generators (engine.md) |
| running and shifting change things; jumping is an arc | S6 | **Adopted:** closed forms between events, checked by the ticked law |
| fast multipole for how a block's T acts at a distance | S6 | **Tracked request** (in ES he asked about ten times and it was never built; see its FMM post-mortem). Phase 4 must build far influence as a compact summary with a stated remainder bound, beside the exact portal tables, never instead of them. The design is open (Opus designer); [HANDOFF](HANDOFF.md) tracks it |
| all scales in space and time | S6 | **Adopted:** hierarchy, plus the scale rung from Phase 2 |
| "is this a negative weight triangle problem … A star?" | S6 | **Answered:** no negative cycles, because T only grows. The triangle inequality fails on positions (momentum) and holds on the full state, so A* applies over that state |
| Google precomputes nodes | S6 | **Adopted:** CRP-style tables customised per toolkit (Phase 4) |
| reuse compute; be ready for sudden commands | S6 | **Adopted:** a goal-free shaped space, a re-target rung ([ladder.md](ladder.md)) |
| the water bucket turns a dead end around; the cliff as a black hole, but not quite | S6 | **Adopted:** the bucket rewrites the vertical metric (Phase 5); a cliff is a one-way edge |
| Monte Carlo | S6 | **Adopted:** robustness under jitter, and the stochastic parts (Phase 9) |
| Taleb: kinds of mistakes, convexity | S6 | **Adopted:** edge width, irreversible losses as barriers (Phase 9) |
| run, jump into the void, place a block under your feet: trivial with the right representation | S6 | **Adopted:** an intersection of the arc with reachable faces; with no face, there is no block |
| ender pearls cost hearts and so food; food allows running | S6 | **Adopted:** tokens (Phase 7). Measured: one pearl = 5 damage = 7.5 food points to heal; sprinting needs food above 6 |
| boats nullify fall damage; lava is an issue | S6 | **Open:** the boat fall rule has not been checked in 26.2's code yet |
| GPU and threads | S6 | **Adopted:** Vulkan for integer min-plus and bitboards, 16 threads for the float law (Phase 10) |
| iterate together, not build yet; world class, garbage in, garbage out | X1 | **Followed** (memory: think before building) |
| "gather resources on think" | X1 | **Followed:** the ES study, verified 26.2 facts ([engine.md](engine.md)), [leads](reference/leads.md) |
| think first, let's talk | S8 | **Followed** (memory: think before building) |
| the causal chain; reduce needless updates; precompute zombies, even ones that help; redstone | X2 | **Adopted:** engine.md; the moving world (Phase 8) |
| "parkour calculator is local" | X2 | **Followed:** a lead, not the architecture ([leads](reference/leads.md)) |
| coordinates can change, he replied to the other model's "Cartesian coordinates still serve engine queries and collision checks"; the system doesn't change; Cartesian might still be best | X2 | **Adopted:** representation.md (coordinates per regime, each checked against the law) |
| imperfection with error bars and feedback; know when mistakes are allowed; time, bytes and compute are resources | X2 | **Adopted:** edge width, re-anchoring, work-unit budgets |
| one missing block: the representation must handle it (Einstein and infinities); an inverted chaotic space where ∂ stays small | X2 | **Adopted:** min-plus tables keep every opening; equivalence within the certified width |
| detail up close, the big picture from afar | X2 | **Adopted:** the hierarchy; refinement by slack |
| never recompute a whole region because one block changed | X2 | **Adopted:** local repair (engine.md) |
| a nether portal can take it very far | X2 | **Adopted:** the Nether as a second sheet (Phase 11) |
| Minecraft may be better thought of as continuous; blocks, but not really | X2 | **Adopted:** closed forms between events |
| not lost in minute details, though sometimes they matter, as in chess | X2 | **Adopted:** resolution follows edge width and slack |
| Minecraft's engine in Rust, optimised, stripped | X2, S11 | **Adopted:** Phase 1.5, the Rust law, judged by Mojang's code in the bot's client |
| an old route valid but poor is the causal chain he means | X2 | **Adopted:** improvement detection (Phase 4) |
| the best compute is no compute | X2 | **Followed:** principle 9 |
| never destroy; build back; pick temporary blocks up | X2 | **Adopted:** edits as conjugates (representation.md, Phase 3) |
| define the problem and the actors; problem types; polymathic | X2 | **Followed:** principle 2 |
| representation is the most important part | X2 | **Followed:** principle 3 |
| the four questions: break a wall or go around? one-high holes? when is a bucket excellent? when must it sprint? | X3 | **Answered:** in chat on 4 November; representation.md, "Four questions" |
| a wall is a two-second delay; compact the space so the player is one block tall | X4 | **Superseded** for walls by S13 (no world modification to get around); a bucket stays usable only where it leaves no trace (S14). **Adopted:** the body folded into the terrain |
| a pickaxe changes a whole type; a block lets you look further | X4 | **Adopted:** tools rewrite classes of places |
| combined systems; order matters; limited amounts ("doesn't that remind you of certain type of math?") | X4 | **Answered:** a word metric over the moves, with conjugates A·B·A⁻¹ for edits and commutators for moves no single tool makes (representation.md) |
| simplest form; not overthinking | X4 | **Followed:** principle 6; the universal gate (no new search code per capability) |
| the bucket ascent; run, jump, float with the bucket, cross, pick it back up; amortization | X4 | **Adopted:** a conjugate whose undo travels with the bot |
| the SpaceX loop | X4 | **Followed:** fly early, then iterate by bottleneck ([HANDOFF](HANDOFF.md)) |
| possessing the bucket changes the space; placing it is movement, not thought; the bike | X5 | **Adopted:** principle 4, representation.md |
| drilled on representation, transforming it and coordinates; rewrite the engine so it is simpler and explorable; compounding; reread it all | X6 | **Followed:** this document |
| the clay mug | X6 | **Followed:** principle 7 |
| "looks like you're understanding it properly" (after the four answers and the conjugate reading) | S10 | **Followed:** that understanding is what representation.md records |
| plan the whole build with sub-agents of the right intellect, in the right order; log the philosophy, especially the engine side | S10 | **Followed:** the plan (4 November); [AGENTS.md](../AGENTS.md), [engine.md](engine.md) |
| "Rust port first" | S11 | **Adopted:** Phase 1.5 |
| "identical block" | S12 | **Adopted:** restoration means the identical block and block state; without Silk Touch, blocks whose drop differs are barriers |
| GPT as a sub-agent for well-formed tasks, Claude where taste is needed | S12 | **Followed:** AGENTS.md |
| no Fable | S12 | **Followed:** AGENTS.md |
| "gpt sub-agents can be two max at the same time indenpedently of you and one if astra is being used" | S15 | **Followed:** AGENTS.md |
| speed fades faster than 32 ticks if you press the opposite key, like a brake | S13 | **Adopted:** the 32 ticks were coasting. Braking (input against the motion) is part of the law and of every local solve; the stopping time is measured in Phase 1 |
| a block that falls into the void has a price: position yourself to catch it, place it where it can be caught, or lose it for good; the drop is random | S13 | **Adopted:** a lost block is a spent token; recovering it is a cost (time, position); its scatter comes from the server's RNG |
| a swarm of bots editing as they pass would compound and deform a world built by players; redstone state, downstream effects, even a trapdoor moved to crawl; "Basically don't modify the world to get around there's many ways to move in minecraft." | S13 | **Adopted:** getting around never modifies the world. Supersedes "a wall is a two second delay" (X4) and the edit phases. Open: whether a tool use that leaves no trace (a water bucket placed and taken back) counts as a modification |
| the bucket's water must not destroy things, create sources or modify anything once removed, "same principle as earlier" | S14 | **Adopted:** a tool use is legal only if the world is identical once it is undone (representation.md, claim 6) |
| test harder things by recording him going from A to B, "for grounding in T not in method" | S14 | **Adopted:** the human baseline on every rung ([ladder.md](ladder.md)) |
| the red-team's corrections to the representation "seem sensible so approaved" | S14 | **Followed:** folded into representation.md |
| the bot may also take commands from the server console over RCON: "sure no problem" | S16 | **Adopted:** queued after job 1.2b; gates can run with his client out of the game |
| the brachistochrone should emerge with elytra under the method; its emergence "under certain conditions mean that we're doing good" | S17 | **Adopted:** rung R-brach in [ladder.md](ladder.md), a falsifier of the method: the reference is the time-optimal curve for the exact elytra law (drag, pitch-dependent lift), not the textbook cycloid |
| the planner "will be used by hundreds of bots that's why i bother so much with performances" | S18 | **Adopted:** cost is reported per bot with many bots on one machine ([ladder.md](ladder.md), scoreboard); the measure is frozen with Phase 10 |
| "i say pathtracing instead of pathfinding for reason i guess" | S18 | **Interpretation, not his words:** routes are rays of least time through a medium whose speed the toolkit sets, as in Fermat's principle. Johann Bernoulli solved the brachistochrone that way, as light refracting through layers. A field of times to one goal serves every bot heading there |
| "Yes bring the ES testing method as proposed" | S19 | **Adopted:** [ladder.md](ladder.md), "How we test": climbing ladders, extreme and chaotic rungs, planted faults chosen by the red-team, many seeds, anomalies and run records |
| check for one-liners that could be problematic, as "test like a Spartan" was | S19 | **Followed:** an audit by a Sonnet sub-agent against ES's sources (5 November) |
| think a bit about many bots now, for "methods that scale and allow swarm behaviors and emergent properties for cheap" | S19 | **Answered** in chat on 5 November; nothing built |
| push the elytra test early, "because it's just jumping between pillars and guiding it", "not too early either but remember how we test" | S18 | **Adopted:** R-brach runs right after the first flight, before Phase 3, as soon as the elytra law is ported; it enters as law data only |
| pricing consumables: "i'm unsure here honnestly", a matter of amortization and resource management without knowing the income, while nothing is gathered or regenerated | S13 | **Open.** Proposal: spending a consumable needs a budget he gives per command (zero by default); health and food regenerate, so they are feasibility constraints, not tokens |

## Contradictions (kept side by side)

| Topic | One statement | The other |
|---|---|---|
| Cartesian space | "maybe 3D cartesian is not the best representation system" (S6) | "But cartesian might be best, i'm just going full first principle to not miss paradigm shifts moments." (X2) |
| Simulate or not | "run minecraft literally but 100x faster to predict the future" (S6) | "The best compute is no compute." (X2) |
| Discrete or continuous | "it's blocks" (X2) | "we might even be better of to think of minecraft as continuous who knows" (X2) |
| Build or think | "not build for now" (X1); "think first" (S8) | "plan for the whole build" (S10) |
| Foundations or the loop | "build cathedral foundations, not an MVP" (ES 8) | "We're doing the spaceX loop here." (X4) |
| Performance mods | "just keep the perf mods" (S1) | the approved plan: a vanilla judge for the bot's client and the server |
| Editing to get around | "a wall is a two second delay not a wall" (X4); "using a water bucket above him, float, take it back, put it back higher" (X4) | "Basically don't modify the world to get around there's many ways to move in minecraft." (S13) |
