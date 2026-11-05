# Lens: the speed of light (on foot, on existing terrain)

**Why it matters.** The planner's lower bound on T (the light cone, `docs/representation.md` claim 9) assumes a top speed `c` per toolkit: no trip can beat distance / c. If any arrangement of existing vanilla blocks lets a player move faster than the `c` we use, the bound is invalid and the bracket lies. Today's candidate is about 19.9 blocks per second (sprint-jumping on blue ice under a ceiling, `lab/numbers/Speeds.java`). Find out whether anything beats it, and what `c` is for each toolkit.

**The law.** The Rust port of Minecraft 26.2's movement law is on branch `wt/law` (`git fetch origin wt/law`; crate `engine/law`, stable Rust, no dependencies; `engine/README.md` explains `tick`, `World` and `BoxWorld`). It is not yet validated against the live game (rung L0), so every result here is provisional and says so. Where the law hits `unimplemented!`, that terrain is out of reach for this lens: list it.

**The toolkit here:** an empty inventory, then each of: a water bucket (used only where its water leaves no trace), an ender pearl, boots with Soul Speed, Depth Strider. The world is never modified to get around (Julien, S13): only terrain that already exists, built once by the experimenter as a `BoxWorld`.

**What to establish.**
- For each terrain class (every block with non-default friction, speed factor or jump factor; ceilings of every height a player can sprint-jump under; ladders and vines; water and bubble columns; slime; soul sand and soul soil; honey; powder snow): the steady speed and the peak speed over 200 ticks, under the best input sequence you can find (say how you searched).
- The overall maximum per toolkit, with the world and input sequence that achieves it.

**What to produce.**
- `reproduce.sh`: builds the law at the fetched commit and re-runs every simulation, printing each speed as exact bits and in blocks per second.
- `speeds.tsv`: terrain, toolkit, inputs (or their generator and seed), steady, peak.
- `dossier.md`: the claim (one sentence: the fastest speed found per toolkit, and whether it beats 19.9), the search method, and what the search could have missed.
