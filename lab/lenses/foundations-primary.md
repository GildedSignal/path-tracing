# Lens: the foundations, from primary sources

**Why it matters.** Julien wants the representation of this pathfinder to be the right one, derived from what the great minds actually showed, not from names that happen to sound right (S40, S41). A first pass (`docs/research/foundations-2026-11-05.md`) used mostly Wikipedia and abstracts. Julien (S42): "research is always from elite primary sources". This lens redoes it properly.

## Read first (all in this repository)

- `docs/PRINCIPLES.md`: "The goal", "The philosophy", and the register rows S39 to S42. These are Julien's own words.
- `docs/reference/julien-2026-11-04.md`, entries S39 to S42: his questions, verbatim.
- `docs/representation.md`: the current claims, 1 to 14.
- `docs/engine.md`: the exactness classes, and what the law is.
- `docs/research/foundations-2026-11-05.md`: the first pass. Treat it as a list of claims to verify or refute, never as a source.

## The problem, in full

A bot must go from A to B in the least time T, counted in server ticks, in Minecraft Java 26.2. The movement law is a deterministic map: state plus one tick of input gives the next state. It has been ported bit-exactly to Rust (`engine/law` on `wt/law`). Facts that constrain the representation:

1. **The state is phase space:** position (doubles), velocity, pose, contact (ground, fluid, climbable, air), the sprint latch, and server-driven attributes. The game's frame is Cartesian x, y, z plus ticks. Julien suspects it is the wrong frame for planning (S40).
2. **The law is dissipative and has resets:**
   - Horizontal velocity is multiplied each tick by 0.91 × the block's friction: 0.546 on stone, about 0.89 on ice, about 0.90 on blue ice.
   - Below |v|² < 9×10⁻⁶ it is set to exactly 0.
   - Landings snap height, and collisions clamp position.
   - So motion drains within roughly 8 to 50 ticks onto fixed points (standing) and periodic orbits (the 12-tick flat sprint-jump, the 3-tick cycle under a 2-block ceiling, the 2-tick cycle under a trapdoor ceiling).
3. **Moves compose and order matters:** sprint then jump is not jump then sprint. Many moves are partial (impossible from some states) and not invertible.
4. **The toolkit changes the space,** not just speeds:
   - A water bucket connects a cliff top to its base.
   - A nether portal is a wormhole with an 8:1 metric.
   - Blocks, pearls, elytra, wind charges and the like add new ways to move.
   - Inventory changes often, and its combinations explode.
5. **The world is huge and changes slowly.** Up to 10⁴ blocks per trip. Hundreds of bots will build in it (S23, S24), so changes concentrate where goals are. The world must be identical once the bot reaches B (S32): edits on the way are conjugates whose undo lands before arrival.
6. **Dynamic parts** (big dripleaf timers, fluids, mobs, redstone) are part of the law: periodic or event-driven.
7. **Scale targets:** a 10,000-block trip; 100 bots on one PC (S35); the first move within one tick (50 ms). The deciding unknown is B, the number of distinct border states per cell after merging. Low hundreds is fine; 10⁴ breaks the design.

## The questions

For each, answer from primary sources: the original papers and books, or the peer-reviewed papers that established the result. Then give a verdict for this problem: **load-bearing**, **useful lens**, or **misleading**, with the reason.

1. **What is the right space?** Phase space; Poincaré's surfaces of section and return maps; the qualitative theory of dynamical systems; Birkhoff; Conley's Morse decompositions for dissipative flows; the quotient of states by identical futures (bisimulation; Myhill-Nerode for weighted automata). Which of these, if any, gives "the regimes fall out" (S40)?
2. **Coordinates.** Gauss's intrinsic geometry (*Disquisitiones generales circa superficies curvas*, 1827); Jacobi's choice of coordinates that make a problem solvable (*Vorlesungen über Dynamik*); action-angle variables. Is there a choice of coordinates, such as events instead of ticks, in which this law becomes simple?
3. **Composition and order.** Semigroups and automata acting on states. Krohn-Rhodes decomposition (1965). Weighted automata over semirings (Mohri). Is "function composition where order matters" (X4, S40) best captured as min-plus matrix products, as a semigroup action, or as something else?
4. **Least action and least time.**
   - Fermat; Huygens; Maupertuis and Euler; Lagrange; Hamilton (*Theory of Systems of Rays* 1828, *On a General Method in Dynamics* 1834); Jacobi (Hamilton-Jacobi).
   - Bellman (*Dynamic Programming* 1957; "On a routing problem" 1958); Pontryagin et al. (1962); Crandall-Lions viscosity solutions (1983); Tsitsiklis (1995); Sethian's fast marching (1996).
   - Maslov and Litvinov on idempotent analysis.
   - Feynman (1948): exactly when least action is a min-plus limit, and when it is only stationary action.
5. **The inventory bottleneck** (Julien, S41: "the minecraft map doesn't change fast but it is big and you change more often your inventory with combinatorial explosion of possibilities").
   - Separating topology from metric: Delling, Goldberg, Pajor and Werneck, Customizable Route Planning (2011 and the journal version); Geisberger et al., contraction hierarchies (2008).
   - Monotone families of metrics; parametric shortest paths.
   - Item equivalence classes.
   - Separation of timescales.
   - Give a concrete recommended design and a Fermi estimate of its costs.
6. **The world changes.** Dynamic shortest paths (Ramalingam and Reps 1996); LPA* and D* Lite (Koenig and Likhachev 2002); time-dependent shortest paths and the FIFO property; periodic processes.
7. **Far-field compression** (Julien's multipoles, S6). Monge arrays; distance oracles and compressed distance matrices in planar graphs (Fakcharoenphol and Rao 2006; Klein); any min-plus analogue of the fast multipole method (Greengard and Rokhlin 1987). Is low-rank min-plus compression of cell-border tables plausible, and how would we test it?
8. **Many bots.** Many-to-many queries; shared fields; anything the classics say about many particles in one field.
9. **What a genius saw that nobody above has named, and that fits better.**

## Rules for sources

- Every claim cites a primary source: author, title, year, venue, and the exact location (page, section, theorem or equation number). Include a URL to the document itself (archive.org, Gallica, Biodiversity Heritage Library, the publisher, arXiv, or an author's page), and quote at most 25 words as the anchor.
- Wikipedia, blogs, Q&A sites, AI summaries and abstracts alone are never sources. They may only point you to the primary source, which you then read.
- If you can't reach a primary source, say so and grade that claim **unverified**.

## What to produce

- `claims.tsv`: one row per claim. Columns: question number, claim, primary source (full citation), location, URL, anchor quote, verdict for this problem.
- `dossier.md`, at most 500 lines:
  - a one-page answer: the representation you recommend and why, from the sources;
  - one section per question;
  - the inventory design with its Fermi estimate;
  - what the first pass got wrong;
  - the experiments that would falsify your recommendation cheaply. For example: a recurrence probe that searches the law for fixed points and periodic orbits per terrain; a count of B per cell; a low-rank test on real cell tables.
- `reproduce.sh`: fetches every URL in `claims.tsv` and checks that each anchor quote appears in the fetched text. It prints one line per claim, `found` or `not found`.

**The checker** runs `reproduce.sh`, reads a sample of the cited locations, and refutes any claim whose source does not say what the claim says.
