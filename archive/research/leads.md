# Leads

These are the public sources found during the 4 November discussions, kept apart from [Julien's words](julien-2026-11-04.md). They are leads, not authorities: read and check each one before relying on it.

## Minecraft implementations

- **Parkour Calculator:** simulates jumps with Minecraft's own movement code, and searches with OpenCL. https://github.com/Leg0shii/ParkourCalculatorMod
- **Azalea:** a Rust bot framework with physics; it lists the mechanics it is missing. https://github.com/azalea-rs/azalea
  - Read on 4 November: MIT licence, targets 26.2.
  - Its `azalea-physics` collision shapes, mergers and sweeps follow vanilla's structure, which makes it the reading guide for job 1.5.
  - It is not forked, because of gaps listed in the handoff.
- **Pumpkin:** a Rust server, still prerelease. https://github.com/Pumpkin-MC/Pumpkin
- **MCHPRS and its Redpiler:** compiles redstone, but resets on every block edit. https://github.com/MCHPR/MCHPRS/blob/master/docs/Redpiler.md
- **Baritone's feature list:** mining, placing, ladders, bucket falls, jumping while placing. https://github.com/cabaletta/baritone/blob/1.21.4/FEATURES.md
- **Mojang on crawling:** https://www.minecraft.net/en-us/article/how-crawling-came-minecraft

## Routing and search

- **Customizable Route Planning** (Delling, Goldberg, Pajor, Werneck): https://www.microsoft.com/en-us/research/publication/customizable-route-planning/
- **Google's electric-vehicle routing:** battery level becomes part of the state, along with recharge time. https://research.google/blog/addressing-range-anxiety-with-smart-electric-vehicle-routing/
- **Transit-node routing** (Bast, Funke, Matijevic, Sanders, Schultes, 2007): long queries pass through few access nodes.
- **D* Lite and LPA\*** (Koenig and Likhachev): https://www.cs.cmu.edu/~maxim/files/DLite_aaai02_abs.htm
- **The semiring framework for shortest distances** (Mohri): https://cs.nyu.edu/~mohri/distance.html
- **Model minimisation in Markov decision processes** (AAAI 1997): merging states that have equivalent futures. https://aaai.org/Papers/AAAI/1997/AAAI97-017.pdf
- **DeepCubeA** (Agostinelli et al., 2019): a learned cost-to-go, then A*.
- **AlphaZero** (Silver et al., *Science*, 2018): https://doi.org/10.1126/science.aar6404

## Physics, control, computation

- **The fast multipole method** (Greengard and Rokhlin): https://web.stanford.edu/class/cme324/classics/greengard-rokhlin.pdf
- **Configuration space** (LaValle, *Planning Algorithms*): https://lavalle.pl/planning/node158.html
- **Backward–forward search for task and motion planning:** https://arxiv.org/abs/1604.03468. Related: PDDLStream.
- **Feedback funnels:** https://arxiv.org/abs/1601.04037
- **Contraction theory** (Lohmiller and Slotine): https://web.mit.edu/nsl/www/preprints/contraction.pdf
- **Self-adjusting computation** (Acar), and Adapton: https://www.umut-acar.org/self-adjusting-computation
- **Hashlife** (Gosper, 1984): memoised quadtree evolution.

## Ants

- **Self-organised shortcuts in the Argentine ant:** colonies chose the shorter branch in 14 of 14 runs when both were offered from the start, but only 2 of 18 when the shortcut appeared after a trail had formed. https://www.researchgate.net/publication/301232811_Self-Organized_Shortcuts_in_the_Argentine_Ant
- **Ant trails refract like light:** at the border between fast and slow ground, trails bend as Fermat's principle of least time predicts (Oettler et al., *PLoS ONE*, 2013).

## Not available here

- **The other model's research note** (`Minecraft_Minimum_Time_Research_v0_1.md`, revision 0.2) stays in that model's sandbox. If it should be kept, it needs to be pasted in.

## From the S32 research (5 November)

- **Parkour Calculator** (GPL-3.0, github.com/Leg0shii/ParkourCalculatorMod): a client mod that solves jump inputs tick by tick on the real player entity, with 26.x support. A candidate cross-check for track jumps and short-course brackets. It's an oracle, never the judge.
- **HeadlessMC** (MIT, github.com/3arthqu4ke/headlessmc): runs the vanilla client without a display. A candidate for running several bot clients at once, if job 1.8's clock works.
- **Other bots' failures** (Baritone, mineflayer-pathfinder, AltoClef, Azalea): issue lists in the research memo, section 3. They are R-far's trips and Phase 12's benchmark.
