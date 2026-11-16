# Sources: what each gives us, and how far it was read

Designer: Fable 5.2 sub-agent, 16 November. "Primary" means the author's own paper or text. "Read" is stated honestly: a source I recall but did not open today is marked **recalled**, and nothing in [plan.md](plan.md) rests on a recalled source without saying so (plan §11). Where a fetched PDF is named, its text was extracted with `pdftotext` from the copy the fetch saved.

## 1. Inside the project (all read in full unless noted)

| Source | Gives us | Read |
|---|---|---|
| `docs/PRINCIPLES.md` | the goal, the philosophy, the register, the contradictions | in full (343 lines) |
| `docs/reference/julien-2026-11-04.md`, `-09.md`, `-10.md` | every turn of his, S1–S164 and X1–X16 | in full. Not read: `private.md` (X13), which is kept out of git |
| `docs/reference/postmortem-2026-11-16.md` | when and why the fork happened | in full |
| `docs/learning.md`, `representation.md`, `scales.md`, `engine.md`, `vocabulary.md`, `ladder.md`, `HANDOFF.md`, `reference/leads.md` | the approved mechanism and the roadmap | in full |
| `docs/footprints.md`, `docs/server.md`, `docs/clan.md` | the footprint contract, the server boundary, the clan | **not read**; nothing in the plan changes them |
| `C:\Users\PASCAL\.claude\plans\wiggly-questing-lemur.md` | the approved whole plan | in full |
| `jobs/zones-design/design.md`, `redteam-opus.md` | the zones design and its 15 findings and 9 unifications | in full |
| `jobs/3.2d-3/spec.md` | the forked spec | in full |
| `jobs/3.2d-3-P1-judge/last.md`; `D:\path-tracing-wt\3.2d-3-P1\jobs\3.2d-3-P1\last.md` | the stored opening: 263-tick transient, 12-tick cycle, 4.276060366589263 blocks per cycle | in full |
| `jobs/hard-track/report.md`, `shortcut.md` | the hard track, and his 428-tick route its certificates missed | in full |
| `jobs/inventory-map/report.md` | 35 mechanisms (6 built, 14 partial, 15 doc only); turns with no register row | in full |
| `jobs/spec-premises/report.md` | 35 "already exists" claims of the spec: 10 exist, 17 differ, 8 are missing | in full |
| `jobs/event-forms/report.md` | each ported mechanic's events and its form between them | in full |
| `jobs/glm-digest/report.md` | what the newest lab results change | lines 1–200 of 475 (the wormhole, food, offset-monotone, push, mining, fluid and elytra-cosine results) |
| `jobs/spec-fixes/report.md`, `port-backlog/report.md`, `docs-truth/report.md` | the state of the spec's claims after slices A and B; the port backlog; stale doc lines | the first 25–60 lines of each |
| `lab/wiki/index.md` | the lab's settled facts | in full. Six lens pages opened; they hold only one-line claims |
| `engine/plan/src/goal.rs`, `reach.rs` (head) | the lower bound's premise; the per-cell backward layer | read directly. The rest of `engine/` is cited through the reports above, which cite it by line |

## 2. His ES engine (read from the clone at `D:\path-tracing-deps\emergent-space-engine`)

| Source | Gives us | Read |
|---|---|---|
| ES `docs/PRINCIPLES.md` | principles 1–35 and his register. Used here: 3 (compute only surprises), 7 (multipoles for gravity and for causality regions), 12 (the allocator), 14 (no ticks), 19 (a fix signals the wrong representation), 32 (symmetry is found), 33 (each scale resolves what matters at that scale) | in full (200 lines) |
| ES `docs/representation.md` | the thesis: "Symmetry is compression"; "Every quantity is kept in the form in which its change is an addition"; nested dolls; "a step is a certificate"; sparse and dense orders with identical bits; min-plus as causality | in full (153 lines) |
| ES `docs/fermi.md` | Fermi estimates as probes, never oracles; "Keep GPU physics unqualified until a measured crossover exists" | in full (26 lines) |
| ES `docs/law.md`, `plane.md`, `archive/engine.md` | certificates, slack, "one rule at every scale" | matching lines only (a search for certificate, light cone, multipole, worldtube) |
| ES `docs/method.md`, `test-ladder.md`, `HISTORY.md` | the testing method; the history | **not read** today; ladder.md already carries the method |

**What ES gives this plan that the path-tracing docs had not taken:** the thesis itself. The plan's §0.1 (a crystal with defects) is "symmetry is compression" applied to terrain under a kit, and §1.1's velocity-space remark is "kept in the form in which its change is an addition". ES's nested dolls, frames in which a symmetry is a translation, are what a zone is here.

## 3. Mathematics and physics

### Read today, primary

| Source | Gives us | Read |
|---|---|---|
| Abraham, Delling, Goldberg, Werneck, *A Hub-Based Labeling Algorithm for Shortest Paths on Road Networks*, MSR-TR-2010-165. https://microsoft.com/en-us/research/wp-content/uploads/2010/12/HL-TR.pdf | A distance query with **no graph search**: each vertex keeps a label of hubs with distances; the "cover property" guarantees two labels share a hub on a shortest path; the query is "a single sweep through the labels, similar to mergesort", and "each array is accessed sequentially". A random query "is as fast as five random accesses to main memory". Labels before pruning averaged "about 500" on Western Europe. This is "Google … precompute nodes" (S6) in its sharpest form, and the model for acting without search (plan §1.2, §5, §6) | the introduction (labels, the cover property), the query passage of §4 and the opening of §5; single lines of the experiments found by search (query cost, preprocessing hours). About 80 of 1,500 text lines. Not the proofs, not the tables |
| Sethian, *A fast marching level set method for monotonically advancing fronts*, PNAS 93 (1996) 1591–1595. https://cs.lbl.gov/assets/timeline/docs/1996-Fast-Marching-Level-Set-Methods.pdf | A front that only advances can be solved in **one pass**: accept the smallest trial value, update its neighbours, never go back, with a heap. The weak solution chosen is "the envelope obtained by Huygens principle". It is why a wave ordered by T never revisits a node (plan §5), and it is his "particle or a wave" (S39) made exact | the abstract and the opening, from the scan's text layer; the lines on upwinding and the heap found by search. The one-pass argument is partly recalled. The figures and the rest not read |
| Mitchell, Bayen, Tomlin, *A Time-Dependent Hamilton–Jacobi Formulation of Reachable Sets for Continuous Dynamic Games*, IEEE TAC 50 (2005) 947–957. https://www.cs.ubc.ca/~mitchell/Papers/publishedIEEEtac05.pdf | "the reachable set is the zero sublevel set of the viscosity solution of a particular time-dependent Hamilton-Jacobi-Isaacs partial differential equation", with one input driving toward the target and one away. This is "sets backward" with a wildcard as the second player: the right form for safety near real players (S121; "we are whites and minecraft world is blacks", S6). Its warning: cost rises "exponentially with dimension", dimensions 4–5 "slow but feasible". So it is used only near a cone, on the folded state, never as the planner | the abstract, the introduction and the remarks on cost; the proof not read |
| Edelsbrunner, Letscher, Zomorodian, *Topological Persistence and Simplification*, Discrete Comput. Geom. 28 (2002) 511–533. https://pub.ista.ac.at/~edels/Papers/2002-04-TopologicalPersistence.pdf | A topological feature is "a feature or noise depending on its life-time or persistence within the filtration"; simplification removes features "in the order of increasing importance". Applied to T-to-go, it measures a far feature by how much T it changes, not by its size: the criterion that keeps the thin bridge (S84) and the missing rule of scales.md:66 (plan §3, kill test 7) | the abstract and the introduction; the algorithm and its proofs not read |
| McEneaney, *A Curse-of-Dimensionality-Free Numerical Method for a Class of HJB PDE's*, IFAC 2005. https://skoge.folk.ntnu.no/prost/proceedings/ifac2005/Fullpapers/03044.pdf | Value functions as a max-plus combination of a few simple functions, propagated by a semigroup that is max-plus linear; growth "on the order of n³" in dimension, but "exponential computational growth in a certain measure of complexity of the Hamiltonian". The lesson taken: a T-field kept as a min of a few simple terms escapes the grid, and the price moves to the count of terms, which here is the defect count (plan §1.3's kit cone; §3's multipole). Gaubert's later critique of the wider claim is known from a search summary only | the abstract and the introduction |

### Read today, abstract only

| Source | Gives us | Read |
|---|---|---|
| Frazzoli, Dahleh, Feron, *Maneuver-Based Motion Planning for Nonlinear Systems with Symmetries*, IEEE Trans. Robotics 21 (2005) 1077–1091, DOI 10.1109/TRO.2005.852260; and Frazzoli's thesis, MIT 2001, http://hdl.handle.net/1721.1/8703 | The closest prior form of plan §1: for systems with symmetries, trajectories are built from **trim trajectories (relative equilibria) and maneuvers** between them, the rules of concatenation are a finite automaton, and steering reduces to "a sequence of kinematic inversion problems", with an offline computation that supports real-time planning. Gaits are trims, transitions are maneuvers, and "the modulus of my best tool" (S120) is the kinematic inversion | the index abstract of the paper and the thesis abstract. The paper is paywalled and the thesis PDF returned HTTP 429. **It should be read in full before U1 freezes** |
| Litvinov, *The Maslov dequantization, idempotent and tropical mathematics: a brief introduction*, arXiv:math/0507014 | Tropical mathematics as "a result of the so-called Maslov dequantization" of ordinary mathematics. It is the frame for X16's question, "what is the best analogue of it in another field?": the conditional T map is to the bot what the action is to a particle, and T-fields combine by min the way waves combine by sum | the abstract only. The superposition principle for Hamilton–Jacobi is recalled, not read |
| Mitchell, Papadimitriou, *The weighted region problem*, J. ACM 38 (1991) 18–73 | Shortest paths through regions of different cost per distance "obey Snell's Law of Refraction at region boundaries". It is the mathematics of "a wall is a two second delay not a wall" (X4) once a kit can cross it: obstacles become slow media and routes bend at interfaces (plan §1.3) | search summaries only; the paper was not opened. Their exact algorithm is slow (about n⁸); only the local law is used |

### Recalled, not read today

Each is a lead until read. None is a premise of the plan.

| Source | The idea | Where it would land |
|---|---|---|
| Bellman, *Dynamic Programming* (1957) | the principle of optimality; the phrase "curse of dimensionality" | plan §1.1 |
| Pontryagin, Boltyanskii, Gamkrelidze, Mishchenko, *The Mathematical Theory of Optimal Processes* (1962) | time-optimal control is bang-bang; for a linear system with real eigenvalues, at most (dimension − 1) switches | "decisions live at switches" (learning.md:17); plan §11 lists it as a prediction to test |
| Hopf (1965); Lax (1957) | the closed-form solution of a Hamilton–Jacobi equation in free space, a min-plus convolution with a kernel | a zone's kernel (plan §1.2) |
| Gauss, *Disquisitiones Arithmeticae* (1801), reduction of binary quadratic forms; Lagrange | reducing a two-vector lattice in a logarithmic number of steps | the two-yaw count problem (spec.md:55-57): which whole numbers of two gait vectors land in B's box. This is where "the prince himself" (S164) fits exactly |
| Steinhaus; Sós (1958), the three-distance theorem | the fractional parts of k·d fall into gaps of at most three lengths | why one gait has few distinct read templates along a line, so a corridor certificate is short |
| Poincaré, *Les méthodes nouvelles de la mécanique céleste* | sections and return maps | a joint is a section; a word is the return map between two (plan §1.2) |
| Lozano-Pérez, Wesley (1979) | shortest paths among obstacles bend only at obstacle corners | X16's "i draw straight lines at certain nodes and then refine them"; the joints of plan §1.2 |
| Lozano-Pérez, Mason, Taylor (1984); Burridge, Rizzi, Koditschek (1999) | preimage backchaining; controllers composed as funnels, each one's goal inside the next one's domain, chosen at run time with no search | sets backward (learning.md:19). The 1999 PDF returned HTTP 403 |
| Lee, Preparata (1984) | the funnel, or string-pulling, algorithm | the slide (plan §2) |
| Gosper (1984), Hashlife | identical regions share one memoised future | portal tables by content hash (plan §1.2) |
| Basch, Guibas, Hershberger (1997), kinetic data structures | certificates and an event queue for moving things | cones on the timing wheel (plan §1.3); already in representation.md:117 |
| Barnes, Hut (1986); Greengard, Rokhlin (1987) | the opening angle; multipole and local expansions | plan §3; scales.md:65; leads.md:30 |
| Geisberger et al. (2008); Delling, Goldberg, Pajor, Werneck (CRP); Goldberg, Harrelson (2005, ALT); Bast et al. (2007) | contraction hierarchies; topology apart from metric; landmarks; transit nodes | labels and the far map (U6); leads.md:19-21 |
| Sethian, Vladimirsky (2001–2003), ordered upwind methods | one-pass solvers when speed depends on direction | the 3D hull is anisotropic (falling is nearly free, climbing is paid); on the folded graph an exact label-setting pass already covers it |
| Lohmiller, Slotine, contraction theory | when all trajectories converge on one another | momentum's memory length (plan §1.1); leads.md:34 |

## 4. What I looked at and judged not to fit

He asked for anything stranger that genuinely fits, "octonions or whatever" (S164). Being plain about misfits is part of the answer.

- **Octonions, and the split-quaternions ES uses.** ES uses split-quaternions because its law has boosts. Minecraft's law has no boost and no exact rotation symmetry: of the eight frames, only the identity replays bit for bit (inventory report.md:48, `pattern.rs:13-14`). Octonions are not associative, and everything composed here is (min-plus, and the affine maps of the tick). I find no use for them.
- **Symplectic geometry.** It is the geometry of systems that conserve. This law dissipates on every tick, which is the very fold that makes the vocabulary small (representation.md:48). Contraction is the right lens.
- **Smooth saddle counts in high dimension** (S131: "in high D there's little saddle points"). He already granted the limit: it "holds for smooth landscapes" (S132). T in whole ticks is terraced. The discrete form that survives is the count of route classes (plan §3).
- **What does fit, and is strange enough:** tropical (min-plus) algebra, where T-fields add by `min`; persistence, which measures a far feature by its effect on T; and lattice reduction, for landing an integer combination of gaits in a box.
