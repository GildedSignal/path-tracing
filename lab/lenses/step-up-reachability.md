# Lens: can a step-up candidate be 0.6f + 1 ulp? (jar reader)

**Why it matters.** The Rust port of the law has a planted fault, `mutant-step-up`: the step height becomes `0.6f` plus one ulp. The gate L0 requires every planted fault to be caught by a recording of the real game. A judge argued that no vanilla geometry can produce a step candidate within one ulp of `0.6f`. If so, the fault is equivalent on the game's domain and must be replaced; if not, a course can be built that catches it. Settle it.

**Getting the jar:** as in the `hook-catalogue` lens (the official manifest, hash checked, `javap`; nothing from the jar committed).

**What to establish.**
- From `Entity.collide` and the step-up code it calls (read the bytecode: method names and offsets), exactly which values are compared against the step height, and in which precision (float or double, after which widening).
- The set of values those candidates can take in vanilla: block collision shapes' heights (from `Block.box` calls and `Shapes` in the shape-defining classes, which are in sixteenths or other fractions), the player's possible feet heights at the moment of the check, and how they combine.
- Whether any combination equals the mutant's threshold, or lies between `0.6f` and it.

**What to produce.**
- `reproduce.sh`, which extracts the shape heights from the jar's bytecode (or computes them by running the jar's own shape code on the runner, without the game's world), enumerates the candidates, and prints the closest to `0.6f` on each side, as exact bits.
- `dossier.md`: the claim in one sentence (reachable, with an example; or unreachable, with the closest candidate), and what an exhaustive enumeration would have missed.
