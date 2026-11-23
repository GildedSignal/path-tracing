# Principles: how we think, a synthesis

His words are the source; each line names its turn (the register and the verbatim turns are in `archive/`). This file is rewritten when a belief changes, and keeps only what is believed now.

## Thinking

1. **T is the one currency.** Everything is priced in ticks, thinking included; "coalesce everything into something holistically optimal" (X2).
2. **First principles, by problem type** (X2). Feasibility is argued from first principles and Fermi estimates, never from precedent (S37: "you don't need to see someone having done it").
3. **Fermi first, then measure.** The floor comes before the build; a gap of orders of magnitude is the signal that the representation is wrong, not a list of speed-ups (S83, S225, the idiot index). Code shrinks on the same scale: a reduction is reported in decades, `log10(before / after)`, against the floor.
4. **Question the requirement, delete, then simplify** (S235). "The simplest form possible, we're looking at many things but the resulting idea will be genuinely simple" (X4).
5. **Representation first, so the search is trivial** (X2, X4: "they shall be trivial"). Shape the space so the problem solves itself, like clay shaped into a mug (X6).
6. **One mechanism.** No terrain kind gets its own path; a special case means the representation is wrong (S161, ES 19).
7. **Store the generator, not the generated** (S6, ES 6). Known things are never relearned: the aim, a start speed or a place are inputs, not keys (S230). The law itself is a small basis of composable, parameterized generators plus data tables and thin Mojang scars rather than a mirror of Mojang's class tree: running a generator for `k = 1` tick gives the exact tick law, and running it to an exit event `k = Δt` gives the time-skipping word.
8. **The best compute is no compute** (X2). Compute only surprises; reuse what is known; repair locally, never a whole region for one block (X2).
9. **Capabilities shape the space.** A tool changes the space, placing it is movement, not thought (X5). Order matters and amounts are limited (X4).
10. **A causal chain bounded by light cones** (S6, X2). Only what can reach the bot, or what the bot can reach, matters.
11. **Precision only where a decision can flip** (S230, SR-71). Narrow moves are bit-exact; wide ones are never refined; the server corrects the rest (X2: "error bars … and update based on feedback").
12. **Every scale, with the details that matter** (S6, X2): near detail, far blur, kept by effect on T, never by size (S84: "i'll miss the thin bridge").
13. **Everything he says compounds** (S6, X6). Nothing is thrown away; a change of belief is recorded as a change here.
14. **Mistakes have shapes** (S6, Taleb): error type over error size, convexity, ruin excluded.

## Testing

15. **The game is the judge** (S239: "always test in game that's the only kind of test that matters"). A test the game does not judge is not written.
16. **Test like a Spartan** (S19): extremes and chaos, worst cases on purpose, distributions and tails, never an average (S6 scoreboard).
17. **Expected against seen.** A delta is data, fixed when seen: "it's literally cheaper to fix than to prevent the issue" (S235). Anomalies are the most important data (S81).
18. **Measure before claiming,** report raw numbers, and say what is not done (S223: real time per command; S220: what changed since the last report).

## Working

19. **Fly early, then iterate by the bottleneck** (X4: the SpaceX loop; S20: cathedral for the law and the representation). Every step ends in a flight he can use.
20. **Every meter works** (S241: GLM "waiting doing nothing for now which is bad"). The bottleneck on the goal comes first; idle companions take the next useful work.
21. **Less code is better on every level** (S44, S49). Smaller kernels keep the hot loop in L1/L3 cache and let the whole law fit inside one agent's context window; any deletion that leaves every jar witness, replay, and flight unchanged is proven sound (ES method). Each step ends by deleting what it made dead, and is judged by how much cheaper it makes the next: fewer lines of code and of tests, so each pass is less work than the last (his words in `archive/turns/inbox.md`).
22. **Agency for the companions** (S237): a clear job, conditions and gate; how they reach it is theirs. Only what cannot be undone stays closed.
23. **The right mind for the job; a smaller model never writes for a bigger one** (S10, S12, S174, S210).
24. **Think together first, then build fast** (X1, S8). When he asks to explain, explain and do not act (S179).
25. **His words and his vocabulary, never ours** (S75, S179); see [vocabulary.md](vocabulary.md).
26. **No made-up numbers** (S67, S132): constants come from the law or the need.
27. **Fun to make** (S36).

## From Emergent Space (ES, his engine)

Worldtubes, certificates, light cones, generators, multipoles, programs plus scars, the slack allocator, local laws, symmetry found by hashing, "a fix signals the wrong representation" (ES 19). ES's "no ticks" does not carry over: Mojang's law ticks at 20 Hz, and a plan is checked against it (ES 14). Details: `detail/engine.md`.
