# Foundations: what the masters saw, and what it buys this project

Research memo of 5 November 2026, answering S41 (Julien: "i can't have just named things and they hapenned to be the right ones"). Written by a research sub-agent; the integrator and Julien decide what to adopt.

How to read the tags. Every factual claim carries a bracketed source number, listed at the end. **[judgement]** marks my own inference, which no source states. **[unverified]** marks something I know or believe but did not confirm in this session. Most sources are encyclopedia pages, lecture notes and arXiv papers fetched through summaries, not the original 19th-century texts. The summaries were produced by a small model, so exact dates and wordings deserve a second look before anyone quotes them.

---

## 1. The answer in one paragraph

Three ideas carry the weight; the rest are decoration or are wrong for this problem. **(a) Bellman's principle with its min-plus algebra** [18][21][23]: the least time to go is a value function that satisfies a one-step consistency equation, and composing routes is a min-plus matrix product. This is non-commutative, so "order matters" is already built in, and no Lie theory is needed to get it. **(b) Poincaré and Conley's split of a dynamical system** [7][11] into recurrent motions and the transients that drain onto them. The tick map is dissipative, so the right part of Poincaré is the return map and the attractor skeleton. His recurrence theorem is the wrong part, because it needs volume preservation [10]. **(c) The computer-science architecture for a slow map against a fast parameter**: metric-independent structure, then cheap customization (CRP, customizable contraction hierarchies) [26][27], plus symmetry quotienting by hash-consing (Hashlife) [28]. Fermat/Huygens/Hamilton give the wavefront-and-ray picture that Julien's "particle or wave" intuition wants, and the project's Bellman field already is that picture [3][5]. Gauss gives a justification for "Cartesian is only a frame" but little machinery. Lie, Chow, Noether, Feynman, Born-Oppenheimer and averaging are loose analogies or misleading as stated. Krohn-Rhodes and tropical automata [29][23] are a framing the integrator did not propose, and they fit better than Lie.

---

## 2. One section per idea

### 2.1 Gauss: intrinsic geometry (Theorema Egregium)

**What he saw.** In his 1827 memoir on curved surfaces, Gauss showed that the Gaussian curvature of a surface depends only on distances and angles measured on the surface itself [1][2]. This is surprising because he had defined curvature using how the surface bends in 3D space, via the normal vector [2]. Bend a sheet of paper without stretching and the curvature does not change [1]. A sphere has curvature 1/R² and a plane has zero, so no flat map of the Earth preserves all distances [1]. Riemann, his student, generalised this to spaces that are "surfaces" with no ambient space at all [2].

**Core insight in plain words.** Ask what an inhabitant of the space can measure with its own rulers. Facts that survive any re-embedding are the real facts. Coordinates are a description, not the thing.

**Mapping.** Real as philosophy. Julien's "changing coordinate system does not change the system" (principle 14) is this stance. The project's intrinsic ruler is the one Julien already chose: ticks. The "distance" from state a to state b is the least number of ticks over legal move sequences, which needs no coordinates.

Where it stops. The theorem itself is about smooth, symmetric (Riemannian) surfaces. The tick distance is neither smooth nor symmetric: a cliff is one-way, so T(a,b) differs from T(b,a). The right family is asymmetric Finsler-type geometry **[unverified]**, and Gauss's theorem does not transfer. I found no result of the form "curvature of the T-metric tells you X" that would give an algorithm, so I would not promise one.

**Verdict: useful analogy (justifies the stance; supplies no machinery).**

### 2.2 Fermat, Huygens, Maupertuis, Euler, Lagrange: least time and least action

**What they saw.**
- Hero of Alexandria showed reflection is the shortest path; Ibn al-Haytham extended this to refraction; Fermat in 1662 proposed least time for refraction. A contemporary, Clerselier, objected that this makes nature purposeful [3].
- Huygens's 1678 wavefront construction implies Fermat's principle, though he did not notice [3]. Rays are the lines normal to successive wavefronts [3].
- The modern statement is that the time is stationary, not necessarily minimal [3][4]. The wave explanation: nearby paths with nearly equal times reinforce each other, and a stationary time gives the widest reinforcing corridor. It is not that light "chooses" [3].
- Maupertuis, Euler and Lagrange each developed versions of "least action". The sources I fetched give no dates or formulations for them, so I cannot say who did what [4]. I did not research the original texts.

**Core insight.** Two equivalent descriptions of the same motion: a ray (point to point, one path) and a wavefront (a whole front of equal arrival time, spreading). The ray is recovered from the front by following the normal.

**Mapping.** This is real and already in the project. Julien's "path tracing" and "particle or wave" (S18, S39) are the Huygens/Fermat pair. The project's time-to-go field is the wavefront, and the greedy descent is the ray. The honest caveat is the one the register already records: for a deterministic discrete system with a true minimum, "least action" is a relabel of "shortest path". Stationary-versus-minimum is irrelevant here because we want a real minimum (Bellman, section 2.7).

**Verdict: load-bearing as the wave/particle duality that Julien asked for; no new algorithm.**

### 2.3 Hamilton and Jacobi: the characteristic function, and choosing coordinates

**What they saw.** Hamilton, in his optics, treated Fermat's least-time rays and the wavefronts as dual, and then carried the same picture to mechanics. The Hamilton-Jacobi equation has the action S as the solution; the momenta are the gradient of S [5]. Surfaces of constant S are wavefronts, perpendicular to the trajectories [5]. Jacobi's contribution, as I understand it, was using a canonical transformation chosen so that the new Hamiltonian is zero, which makes the new coordinates and momenta constants [5]. If the system is *integrable* and the energy surface is compact, one can pick action-angle variables. The actions are constant, and each angle advances at a constant rate; one full cycle advances the angle by exactly 1 [6]. That is the pendulum miracle Julien mentions: in the right coordinates, the swing is a straight line at constant speed.

**Core insight.** The difficulty of a problem lives in the coordinates. A good coordinate change removes the dynamics (everything becomes constants plus uniform rotation). Finding it is as hard as solving the problem, and it exists only for special (integrable) systems [6].

**Mapping.**
1. Real and load-bearing: *near a recurrent motion, use phase coordinates.* The 12-tick sprint-jump is a cycle. A state near it can be written as (which cycle, phase mod 12, small transverse offset that decays). That is the discrete analogue of an angle variable, and it makes "being on the cycle" a single coordinate rather than a trajectory. **[judgement]** (The decaying offset is what dissipation gives; the textbook version for dissipative systems is the "asymptotic phase" or isochron, **[unverified]**.)
2. Not applicable: Jacobi's machinery needs a conservative, smooth, integrable system [6]. Minecraft is dissipative and has snaps. No global action-angle coordinates will exist.
3. The characteristic function S is the time-to-go field. The project has already reached this (claim 13 in representation.md).

**Verdict: the idea "choose coordinates per regime so the dynamics become trivial" is load-bearing and matches claim 11. The specific theory is analogy only.**

### 2.4 Poincaré: phase space, sections, return maps, qualitative theory

**What he saw.** Poincaré's *Méthodes nouvelles de la mécanique céleste* (1892-1899) aimed to characterise all motions of mechanical systems rather than solve one, and showed that the series used by Lagrange and Laplace converge but not uniformly, so their stability proofs were in doubt [8]. His 1889 prize memoir on the three-body problem contained an error found by Phragmén; the corrected version gave the first description of homoclinic points, and the episode is called the birth of chaos theory [8]. For the restricted problem he showed no further analytic conserved quantity exists beyond the Jacobi integral [9], and he established infinitely many periodic solutions [9]. (I could not confirm the well-known remark that periodic solutions are the only breach in the fortress. It is not in the pages I read. **[unverified]**)

The **Poincaré map**: pick a surface that the flow crosses transversally, and record where each trajectory next returns to it. A periodic orbit becomes a fixed point of this map, and the orbit is stable exactly when the fixed point is [7]. The state space drops one dimension [7]. The page notes no general recipe for building a section exists [7].

**Core insight.** Replace "watch a continuous trajectory forever" by "look at it once per cycle". Long-term behaviour becomes the fixed points and their stability. The skeleton of the dynamics is the set of periodic orbits plus how they connect.

**Mapping.**
- *Return maps:* load-bearing. The 12-tick jump cycle is a periodic orbit; take the section "at the tick of landing" and the whole cycle is one fixed point (speed, offset). Stability is then a number: how fast does an off-nominal landing return to the nominal one? That number is exactly the "edge width" of claim 10. In Minecraft the section is natural (the tick), which answers the "no general recipe" caveat.
- *Poincaré recurrence theorem:* **misleading.** It says a volume-preserving flow with bounded orbits returns arbitrarily close to its start [10]. Minecraft is dissipative: friction collapses velocity to zero, landings snap height. The theorem explicitly does not apply to such systems [10]. What the tick map does instead is *forget*, which is the opposite behaviour. The integrator's "recurrence" talk should be replaced by "attractor" or "recurrent set" (see Conley).
- *Three-body lesson:* the lesson is that exact solutions are impossible when there are more degrees of freedom than conserved quantities [9], so one must work with the qualitative skeleton. Here the situation is gentler than three bodies: the law is finite, deterministic, and the skeleton is small. It is a justification of the approach, not an input to the algorithm.

**Verdict: return maps and periodic-orbit skeleton are load-bearing. Recurrence theorem: wrong. Three-body: motivation only.**

### 2.5 Birkhoff, Conley, Smale

**What they saw.**
- *Conley's fundamental theorem:* every flow on a compact phase portrait splits into a chain-recurrent part and a gradient-like part. Equivalently, there is a "complete Lyapunov function" that never increases along solutions and is constant on each isolated invariant set [11]. There are analogous results for discrete time [11]. (The page I read did not explain chain recurrence or Morse decompositions in detail, so those definitions are from memory. **[unverified]**)
- *Smale's horseshoe:* squash, stretch and fold a square; the set of points that never escape is a fractal whose dynamics is described by symbol sequences, containing infinitely many periodic orbits whose number grows exponentially with period [12]. It models what happens near a transverse homoclinic point [12].
- *Birkhoff:* I did not research his work in this session. I will not cite a claim about it.

**Core insight (Conley).** Any dynamics is "gradient flow between recurrent pieces": a hill of transients leading down to resting places, with the interesting recurrent behaviour confined to those places. Conley made this rigorous with a Lyapunov function.

**Mapping.**
- Strong fit with the proposal "a graph of recurrent motions joined by transients": recurrent set = nodes, gradient-like part = the transient edges, Lyapunov function = something like distance-to-rest.
- Adversarial caveat **[judgement]**: for a *finite* deterministic map, the content of Conley's theorem collapses to a familiar one. Every orbit ends in a cycle, the recurrent states are the states on cycles, and with controls the decomposition is the condensation of the directed graph into strongly connected components (Tarjan). The idea is correct and useful, and the algorithm is classical graph code. Do not expect new mathematics from it. Note also that the real state is floating-point, so finiteness is true but the state set is astronomically large; the useful object is the *quantised* skeleton in claim 5.
- Horseshoe: Minecraft's tick map is mostly contracting, not chaotic. Edge-of-block decisions create sensitive dependence (a hair left or right gives a different future), which is the "ridge" of claim 14, but that is a local branching, not a horseshoe. The relevant idea borrowed is *symbolic dynamics*: describe an orbit by the discrete symbol sequence (jump, land, turn...). That is the word metric in representation.md.

**Verdict: Conley's split is load-bearing as the organising picture. Smale: loose analogy.**

### 2.6 Lie, Chow-Rashevskii, Baker-Campbell-Hausdorff: composition where order matters

**What they saw.**
- BCH: for non-commuting generators X, Y of a Lie algebra, e^X e^Y = e^Z with Z = X + Y + ½[X,Y] + higher commutators. The bracket measures failure to commute; if it vanishes, effects simply add. The series converges only for small X, Y, so the statement is local [16].
- Nonholonomic systems: a wheel can only roll in the direction it points, a velocity constraint that cannot be reduced to a position constraint. The Lie bracket of two allowed motions can point in a forbidden direction, which is how a car reaches the spot beside it (parallel parking) [14]. (The page does not spell out the parking story; it only points to a separate page [14].)
- Chow-Rashevskii (Rashevsky 1938, Chow 1939): on a bracket-generating distribution any two points can be joined by an allowed path, and the Carnot-Carathéodory distance (shortest allowed path) induces the manifold's usual topology [13].
- With drift (momentum, as in Minecraft) the bracket condition is only necessary, not sufficient, for reaching everywhere; a standard example is x1' = u, x2' = x1² [15].

**Core insight.** Sequences of small moves, composed in different orders, produce net motions no single move produces, and the new motion is the commutator. Also, one can *define the distance by the allowed moves* (Carnot-Carathéodory), independent of coordinates.

**Mapping: is this the right math for "function composition, order matters"?** Only half.
- Real and load-bearing: define the metric by the admissible moves (Carnot-Carathéodory). It is the formal version of "the toolkit is part of the metric" (claim 2) and "a wall is a delay" (X4).
- Loose: the commutator story. BCH needs smooth flows and group inverses. Minecraft's moves are *partial, discrete, non-invertible* maps: friction snaps to zero, landings snap height, a fall cannot be undone. There is no group, so no commutator `[A,B]` in the Lie sense. What is true is that sequences matter, and that is simply the non-commutativity of composing maps and of min-plus matrix products (section 2.7). representation.md's "A·B·A⁻¹" is meaningful only where an explicit undo exists (pick the bucket back up); it is a specific toolkit property, not a general law.
- Chow's theorem is about *reachability*, and reachability is rarely the question here: on walkable terrain everything is reachable in some finite number of ticks. The question is cost.
- Drift matters: Minecraft has momentum, i.e. drift, which is exactly the case where the clean Chow theorem does not apply [15].

**Verdict: Lie brackets/BCH: loose analogy. Chow: loose. The Carnot-Carathéodory idea (metric from allowed moves) is the part worth keeping.**

### 2.7 Bellman, Dijkstra, Pontryagin, and the minimum-time function

**What they saw.**
- Bellman: the optimal cost-to-go from a state satisfies a one-step consistency equation. In continuous time this gives the HJB equation, whose solution is the value function V; it is necessary and sufficient for optimality [18]. Kálmán first tied it to Hamilton-Jacobi [18]. Smooth solutions may fail to exist; viscosity solutions are the repair [18]. The discrete form is the Bellman equation [18].
- Dijkstra (1956, published 1959) designed his algorithm in about twenty minutes; it needs non-negative weights, since extending a path then never shortens it. A* adds a lower bound to prune [20]. Dijkstra himself justified it by Bellman's principle [20].
- Pontryagin (1956): a necessary condition checked only along one trajectory, cheaper than HJB, which must hold over the whole state space [19]. Its first use was maximising a rocket's final speed [19].

**Core insight.** The minimum-time function is a *field* on the whole state space with a local test: at each state, T(s) equals one tick plus the best neighbour's T. The field certifies itself, and every optimal route is the greedy descent of the field.

**Mapping.** This is the central piece and it is already adopted (claims 3 and 13). Check: T only grows, so there are no negative cycles, which is Dijkstra's requirement. Triangle inequality fails on positions but holds on the full state (representation.md claim 3), which is what lets A* work. Pontryagin's trick of only checking along the route is the "lone trip uses forward search" case, but the maximum principle is a continuous-time necessary condition [19] and does not add anything over discrete Bellman here. HJB as a PDE is a relabel; the discrete Bellman equation is the working object.

**Verdict: Bellman is load-bearing. HJB/Pontryagin as PDE/continuous machinery: not needed; the discrete Bellman equation does the job.**

### 2.8 Feynman, Maslov dequantization and tropical algebra

**What they saw.**
- Dirac (1933) noticed that when the action is large against ħ, the sum over paths is dominated by paths near a stationary point of the action; Feynman (1948) gave the precise rule: each path contributes a phase e^(iS/ħ) [22]. Rotating time to imaginary time (Wick) gives e^(-tH/ħ), which looks like a statistical-mechanics partition function [22].
- Maslov dequantization: map positive numbers x to u = h ln x and define u ⊕_h v = h ln(e^(u/h) + e^(v/h)) with ordinary addition as the product. As h→0, ⊕_h tends to max, which gives the (max,+) semiring; with h negative, (min,+) [21]. Litvinov presents this with an *imaginary* Planck constant, so the Schrödinger equation becomes a heat-type equation, and as h→0 the Hamilton-Jacobi equation emerges [21]. He calls the argument heuristic and points elsewhere for rigorous versions [21]. Over this semiring, Hamilton-Jacobi is *linear*: if S1 and S2 solve it, min(λ1+S1, λ2+S2) does too (in the min version) [21]. Dynamic programming and shortest paths are the (min,+) and (max,+) cases of the Bellman equation X = H⊙X ⊕ F; Bellman-Ford corresponds to Jacobi iteration and Ford's algorithm to Gauss-Seidel [21].
- "Tropical" is named after Imre Simon, a Hungarian-born Brazilian computer scientist [23]. (Simon's own work was on min-plus automata; the page I read lists weighted finite-state transducers among applications, but I did not confirm his research area. **[unverified]**)

**Is least action really the min-plus limit of the path integral?** Yes, with two corrections. First, the limit is that of the *imaginary-time* (Euclidean, partition-function) sum, i.e. the zero-temperature limit: Σ over paths of e^(-T(path)/h) → e^(-min T / h) as h→0 [21][22]. That sum is the log-sum-exp; the "minimum" appears with no physics needed. Second, the real-time Feynman limit (stationary phase) yields a *stationary* path, not a minimum [4][22]. So "the path integral gives least action" is imprecise; "log-sum-exp at zero temperature gives min-plus" is exact. Litvinov's treatment is heuristic by his own account [21].

**What does min-plus linearity buy?** My assessment, point by point:
1. *Superposing goals by min.* Exact and true: the time-to-go for a set of goals G1 ∪ G2 is min(T_G1, T_G2) **[judgement, follows directly from the definition]**. It is also the right way to describe region goals (builders' reach regions, S23/S24) and many-to-many queries.
2. *Sparse repair.* Only half-true. The semiring has no subtraction: min cannot be undone. Lowering a cost can be folded in by one more min (cheap, exact, spreads outward), but raising a cost needs invalidation of the states that depended on the old route. This asymmetry is exactly why representation.md claim 13 needs the "shadow" logic of LPA*/D* Lite. Linearity does not make repair free. **[judgement]**
3. *Low-rank far-field compression (the fast-multipole hope).* This is the weakest. In ordinary linear algebra, far interaction matrices are low-rank, which is what FMM exploits. For min-plus there is no comparable general theory: the min-plus product is equivalent to all-pairs shortest paths and no truly subcubic algorithm for it is known in general (APSP hypothesis), the best being about n³/exp(√log n) [31]. Truly subcubic algorithms exist for structured inputs (bounded differences, low approximate rank) [31]. The most relevant structure is the Monge property: distance matrices between vertices on one face of a planar graph are Monge, and min-plus products of Monge matrices are fast (O(n²) by SMAWK-based methods) [30]. That is how planar-graph distance oracles work. **Caveat:** Minecraft's cell borders with speed attached, pearls, elytra, portals, are not obviously planar-outer-face structured, so Monge need not hold. It is an empirical question with a cheap probe (section 3, item 6).

**Verdict: min-plus as the right arithmetic: load-bearing, and it needs no physics to justify. Maslov/Feynman as "proof that least action = min-plus": overstated; it is a nice consistency story. Goal superposition: real. Repair: partial. Far-field: unproven.**

### 2.9 Noether: symmetry and conservation

**What she saw.** Every continuous symmetry of the action of a conservative system has a corresponding conserved quantity [17]. It does not apply to systems that cannot be derived from a Lagrangian alone, such as ones with a Rayleigh dissipation function, and dissipative systems with continuous symmetries need not have the conservation law [17].

**Mapping.** The literal theorem does **not** apply. Minecraft is dissipative and its symmetries are discrete: shifts by whole blocks, rotations by 90°, probably reflections **[judgement]**. No conserved quantity falls out. The usable fact is different: *symmetry lets you share work*, which is group-action quotienting, not Noether.

Two cautions **[judgement]**. (1) Translation invariance is exact in real arithmetic but the game computes in doubles at absolute coordinates; the same local situation at x=0 and at x=20,000 can round differently in the last bits. If the project insists on bit-equal replay (claim 5, "equal to Mojang's code"), table sharing across distant copies needs a test, not an assumption; this fits ES principle 32 (symmetry is found by hashing, not declared). (2) Rotation by 90° may be broken by lookup-table or ordering details; again test.

**Verdict: Noether: misleading as named. The discrete symmetry-sharing idea is load-bearing (section 2.12).**

### 2.10 Separation of timescales: Poincaré, Krylov-Bogoliubov, adiabatic ideas, Born-Oppenheimer

**What they saw.**
- Krylov and Bogoliubov: for x' = εf(x,t), average over the fast oscillation and the slow drift follows the averaged equation; the error is O(ε) over times of order 1/ε. It needs the system in standard form; applied carelessly it gives wrong answers [24].
- Born-Oppenheimer (1927): nuclei are heavy and slow, so solve the electrons with nuclei fixed, get an energy surface, and let the nuclei move on it. It breaks down when electronic surfaces approach or cross [25].

**Mapping to Julien's bottleneck.** Be careful: his case does not match these theorems in either direction.
- Averaging is about removing fast *oscillation*. The inventory does not oscillate; it jumps between combinatorially many discrete values. Nothing averages.
- In Born-Oppenheimer the fast variable is solved *given* the slow one, producing an effective landscape. Here, the slow object is the map and the fast one is the toolkit, which is the *reverse* of the usual pairing: we need the slow object's heavy precomputation done once for *every* value of the fast parameter, or lazily for the ones used. What carries over is only the design idea: *solve the heavy slow part under a parameter and keep the result as a landscape.*
- Real and load-bearing instance in a different place **[judgement]**: *within the bot's own dynamics.* Transients last 8-50 ticks and drain onto the recurrent motions; routes live on timescales of hundreds of ticks. That is genuine separation of timescales: tables need only store the recurrent skeleton and the entry/exit transients, not all intermediate states. This is the content of the "graph of recurrent motions".

**Verdict: Born-Oppenheimer/averaging for the inventory problem: loose analogy. Timescale separation of transients versus recurrent motion: load-bearing.**

### 2.11 Customizable Route Planning, contraction hierarchies: separating topology from metric

**What they did.** Delling, Goldberg, Pajor and Werneck (SEA 2011; journal version in Transportation Science 2017) split work by how often it changes. Metric-independent preprocessing partitions the graph into a multilevel overlay whose structure is the same for any cost function. Customization then computes the arc costs for a given metric. Queries run on the result. Metric changes (traffic, cost function) are far more frequent than road changes, and customization takes a second or less on continental networks [26]. Contraction hierarchies bake the weights into shortcuts, so a metric change invalidates them; customizable contraction hierarchies fix this by computing a metric-independent contraction order (nested dissection) once and re-applying it to new weights cheaply [27]. Query time for CCH is bounded in terms of tree-depth [27].

**Mapping.** This is the closest published answer to Julien's constraint, and it is the same architecture as claim 8. Map = topology (slow). Toolkit = the metric (fast, frequently changing). Correspondence is tight in structure; two caveats **[judgement]**:
1. In road networks the metric is a scalar per edge. Here the border "vertices" are *states* (position plus speed plus pose), so a cell's border is much larger than the road case, and the table is a B×B matrix of times. Whether B is small is the open "compactness" question in representation.md. CRP's published second-scale customization does not transfer automatically.
2. Road networks are nearly planar; Minecraft has pearls, elytra and portals that jump across cells.

**Verdict: load-bearing. The shape of the answer.**

### 2.12 Hashlife and symmetry sharing

**What Gosper saw.** Hashlife stores the grid as a quadtree whose identical squares are merged into one shared node (hash consing), memoises each node's future, and can jump 2^(k-2) generations from a node at level k, sometimes with exponential speedup. It uses much memory on high-entropy patterns and does poorly on chaotic rules [28]. It relies on a local deterministic rule applied identically everywhere (stated by the page only by implication) [28].

**Mapping.** Real where the terrain repeats: superflat, water, stone, air, repeated builds, tunnels. Minecraft's law satisfies the preconditions (local, deterministic, translation-symmetric) up to the floating-point caveat in 2.9. Key distinction **[judgement]**: Hashlife's time-skipping works because Life's interior evolves with no external input. Here the *bot's input* is a free choice each tick, so the memoised object is not "the future of this block" but "the table of transfers across this block", which is what claim 8 already says. The sharing is the same trick; the doubling-of-time trick is not.
Natural terrain is high-entropy at block scale, so exact sharing will pay mainly for empty, solid and fluid cells and for whatever builds repeat; this is the "High-entropy patterns" failure mode [28].

**Verdict: load-bearing for the lab and for repeated structures; modest on natural terrain. Test, don't assume.**

### 2.13 What fits better than the integrator's list

Two framings are missing from the integrator's proposals.

**A. Weighted automata and the transformation monoid (Krohn-Rhodes).** Treat the tick map as a finite automaton whose "letters" are inputs (and, in the extended version, toolkit actions). Composition of moves is composition of partial maps, non-commutative, with costs added: a *min-plus weighted automaton* (the literal home of "tropical" [23]). Krohn-Rhodes says any finite automaton is a cascade (wreath product) of two kinds of pieces: reversible permutation-like parts (simple groups) and irreversible reset parts, flip-flops that destroy a stored bit [29]. This maps unusually well **[judgement]**:
- Friction snapping velocity to zero and landings snapping height are the *resets*.
- The periodic orbits (12-tick jump cycle) are the *group parts*: a cyclic group of order 12.
- "Moves with no inverse" are just semigroup elements, no pretence of Lie algebra needed.
This is a descriptive lens and does not give an algorithm; the Krohn-Rhodes decomposition is not something to compute for a 10^15-state machine. But it states the integrator's "attractors plus transients" and "order matters" in one object, with theorems about composition, and avoids the false smoothness of Lie theory.

**B. Hybrid systems** (continuous flow between discrete jumps at contact). This is how mechanics people model a bouncing ball or a walking robot. Minecraft's air arcs plus landing/collision snaps are this shape **[judgement, unverified; I did not research the literature]**. It would justify "closed forms between events" (already adopted). I flag it only as a vocabulary to search in.

---

## 3. The inventory bottleneck: a concrete recommendation

**The constraint (S41):** a huge map that changes slowly; an inventory that changes often, with combinatorially many values.

**Principle.** Key every stored object by what it actually depends on, and let the fast-changing thing choose among stored objects rather than trigger recomputation. This is CRP's lesson [26], applied to three levels.

1. **Topology (changes never).** Fixed grid of cells aligned to blocks (sub-chunk, e.g. 16×16×16), fixed definition of border states. Metric-independent, so it is built once, as in CRP preprocessing [26].
2. **Map content (changes slowly).** Per cell, a canonical hash of its collision footprint plus a one-to-two block halo (the body inflates collision boxes, so neighbours matter). Canonicalise under the symmetry group (shifts; rotations and reflections if tested) so identical cells share a record [28]. A block edit changes one hash and re-solves at most the cells the halo touches, then the overlay above them.
3. **Toolkit (changes often): never key by raw inventory.** Key by the *capability projection*: which generators can matter inside this cell. Three ideas stack:
   - **Equivalence classes of items.** Two items are equivalent if the law treats them alike for every use the planner has. A thousand block items probably fall into a dozen-odd behaviour classes (solid, slippery, sticky, bouncy, climbable, water-placing...). Find the classes by running a probe suite and hashing the behaviour, never by hand-listing (ES 32 and the "no catalogue" rule). The count "a dozen-odd" is my guess **[unverified]**.
   - **Presence versus counts.** In a local table only whether the bot *can* use the tool matters, with a small count cap (0, 1, 2, ≥ k, where k is the most uses that fit in one cell). Across cells, counts are a *budget* tracked as a small extra dimension in the overlay search. Do not multiply the tables by counts.
   - **Lazy, affordance-gated tables.** Always have the table for the empty toolkit. Compute the table for toolkit item i in cell c *only if* the cell offers an affordance for i (a drop for a bucket, a gap for a block, a wall for a pickaxe). Combinations are computed only where two affordances coexist in one cell. The default is "table equals the base table", stored as a pointer.
4. **Monotonicity gives a bracket for free.** If the bot may ignore any item, adding items never makes T worse: T_K ≤ T_K' whenever K ⊇ K' **[judgement]**. (Conditions: counts are budgets and items can be left unused; hotbar-switching costs and the "inventory whole" rule at arrival could break it slightly, which I have not checked.) Consequences: the table for the *richest* relevant toolkit with unlimited counts is an admissible lower bound, usable as A* heuristic and as the lower end of Julien's bracket; the empty-toolkit table is an upper bound. The real answer is squeezed between them, and often the two coincide (then the toolkit does not matter on that stretch, and nothing more needs computing).
5. **Search over the overlay**, not the raw states: A* from the corridor of cells that the light cone allows, using the relaxed table for the heuristic and exact tables for cells actually expanded. Each cell is expanded with the budget vector; a pearl count of 3 means three consumable events at most along the route, so the extra dimension is tiny.
6. **Cheap probes before building.** Measure (a) B, the number of border states per cell including speed; (b) the ordinary and min-plus approximate rank of the B×B matrices on sample terrain, to decide whether low-rank/Monge compression exists (2.8, point 3); (c) the fraction of cells whose table is unchanged by each item, which is the real sparsity of the toolkit lattice.

### Fermi estimate (orders of magnitude; every input is a guess to be replaced by the probe)

Take a 10 km trip, as in rung R-far. Corridor 10,000 × 300 blocks; surface cells are about 2 per column of cells: cells ≈ (10,000/16)(300/16)(2) ≈ **2×10⁴**.

- **Table size per cell**: B border states, B² entries, 2 bytes each. B = 100 → 20 KB per cell → 0.4 GB for the corridor per toolkit class. B = 1,000 → 2 MB per cell → 40 GB. So B is the thing to measure; above a few hundred, uncompressed tables fail.
- **Customization compute per cell**: interior states ≈ 256 columns × 8 speed classes × 16 headings ≈ 3×10⁴; about 10 inputs per state gives 3×10⁵ relaxations per source; times B sources: 3×10⁷ (B=100) to 3×10⁸ (B=1,000). The project measured a stripped tick at about 10⁻⁷ s (register, S6), so about 10⁷ relaxations per second per core. That is **3 s to 30 s per cell** per core, or **~10⁵ to 10⁶ core-seconds** for the whole corridor, i.e. roughly 2 to 20 hours on 16 threads for ONE toolkit class. (The Vulkan integer path in Phase 10 could cut this by a factor I cannot estimate.)
- **Toolkit combinatorics**: with ~12 relevant generators there are 2¹² ≈ 4,000 subsets, so eager precomputation for every subset is ~10⁴ times the figure above: impossible. This is the quantitative reason for item 3 (projection, lazy, affordance-gated).
- **Lazy case**: a single trip's A* touches a tube of perhaps 10³ cells, not 2×10⁴, and each needed table is computed once and cached. At 3-30 s per cell on one core that is ~10²-10³ core-seconds spread over 16 threads, i.e. seconds to a minute per new (cell, toolkit-projection). It is workable offline or in the background for a new area, and far from "100 ms to first move", which means the main product is a *cache that persists across trips and bots*, as principle 9 says.
- **Per-event repair**: one changed block re-solves ≤ 8 cells (a halo touching a corner) = 24-240 core-seconds at the numbers above; 3-30 s on 16 threads. For a builder army placing many blocks, that is too slow per block at B ≈ 1,000 and acceptable at B ≈ 100. So the cell size and B interact: **smaller cells and smaller B help repair**, larger cells help the overlay. This trade-off needs the probe, not a decision.
- **Inventory change at run time**: if tables for the new class exist, a pointer swap (≈ free). If not, only the corridor cells that have an affordance for the new item need computing.

Takeaway **[judgement]**: the map-versus-inventory problem is tractable only if B stays in the low hundreds and the toolkit lattice is sparse. Both are measurable before any search code is written.

---

## 4. What the integrator's proposals got wrong or overstated

1. **"Poincaré recurrence."** The theorem needs volume preservation and does not apply to dissipative systems [10]. Minecraft is the opposite. Use *return maps* [7] and *Conley's recurrent set* [11].
2. **"Noether."** Applies to continuous symmetries of conservative Lagrangian systems [17]; the tick law is dissipative and has discrete symmetries. The correct statement is about *sharing across symmetric copies*, plus a floating-point caveat the proposal skipped (2.9).
3. **"Commutators and BCH generate new moves."** True for smooth invertible flows [16]. Minecraft moves are partial, non-invertible, discrete [register, representation.md]. The mathematics that survives is non-commutative composition of maps and of min-plus matrices. "A·B·A⁻¹" needs an actual undo, which only some tools supply. With drift, Chow's theorem does not give reachability [15], and reachability is not the issue anyway.
4. **"Least action is the min-plus limit of Feynman."** Only for the imaginary-time/zero-temperature version [21][22]; the real-time limit is stationary, not minimal [4]. And Litvinov calls it heuristic [21]. The stronger point is that min-plus needs no physics: shortest paths are already a min-plus matrix problem [21].
5. **"Min-plus linearity buys sparse repair and far-field compression."** Superposition of goals by min: yes. Repair: only for decreases; increases need invalidation because min has no inverse. Far-field low-rank: no general theory, and the structural results (Monge) need planarity-like structure that the cells may lack [30][31]. Treat as a hypothesis with a probe.
6. **"Born-Oppenheimer / averaging solve the inventory problem."** They describe removing a fast oscillation or solving fast variables under fixed slow ones [24][25]. The inventory is discrete and combinatorial, not oscillatory. The true answer is CRP-style customization and equivalence classes, not averaging.
7. **"Gauss gives the method."** It gives the stance, and the project already holds it. The tick metric is asymmetric, so the theorem as stated does not apply (2.1).
8. **Hamilton-Jacobi for Minecraft.** The Hamilton-Jacobi / canonical-transformation machinery requires integrable conservative systems [6]. The claim in the register that the time field "is" Hamilton-Jacobi is fine as a discrete Bellman equation [18]; no further theory transfers.
9. **Missing from the proposals:** the weighted-automaton / Krohn-Rhodes view (2.13) and the capability-projection plus monotone lattice for the toolkit (section 3).

---

## 5. Honest limits of this memo

- I read encyclopedia-grade summaries and a few arXiv abstracts, not the original papers. The CRP paper's PDF could not be opened (404), so its numbers come from the Microsoft Research page and search summaries [26].
- Not researched: Birkhoff, Lie's original work, Euler/Lagrange/Maupertuis details, Huygens beyond one sentence, adiabatic invariants as a separate topic, hybrid-systems literature, Finsler geometry, isochrons.
- Fermi estimates use guessed B, state counts and relaxation rate; they show which quantities decide feasibility, not what the answer is.

---

## 6. Sources

[1] Theorema Egregium, Wikipedia. https://en.wikipedia.org/wiki/Theorema_Egregium
[2] Some Fundamental Theorems in Mathematics (Gauss's theorem, dating, Riemann), arXiv 1807.08416. https://arxiv.org/pdf/1807.08416 (seen as a search result; also ProofWiki https://proofwiki.org/wiki/Theorema_Egregium)
[3] Fermat's principle, Wikipedia. https://en.wikipedia.org/wiki/Fermat%27s_principle
[4] Stationary-action principle, Wikipedia. https://en.wikipedia.org/wiki/Stationary-action_principle
[5] Hamilton-Jacobi equation, Wikipedia. https://en.wikipedia.org/wiki/Hamilton%E2%80%93Jacobi_equation
[6] Action-angle coordinates, Wikipedia. https://en.wikipedia.org/wiki/Action-angle_coordinates
[7] Poincaré map, Wikipedia. https://en.wikipedia.org/wiki/Poincar%C3%A9_map
[8] Henri Poincaré, MacTutor. https://mathshistory.st-andrews.ac.uk/Biographies/Poincare/
[9] Three-body problem, Wikipedia. https://en.wikipedia.org/wiki/Three-body_problem
[10] Poincaré recurrence theorem, Wikipedia. https://en.wikipedia.org/wiki/Poincar%C3%A9_recurrence_theorem
[11] Conley's fundamental theorem of dynamical systems, Wikipedia. https://en.wikipedia.org/wiki/Conley%27s_fundamental_theorem_of_dynamical_systems
[12] Horseshoe map, Wikipedia. https://en.wikipedia.org/wiki/Horseshoe_map
[13] Chow-Rashevskii theorem, Wikipedia. https://en.wikipedia.org/wiki/Chow%E2%80%93Rashevskii_theorem
[14] Nonholonomic system, Wikipedia. https://en.wikipedia.org/wiki/Nonholonomic_system
[15] Controllability with drift (search results; lecture notes and papers): https://arxiv.org/pdf/2403.04718 and https://mast.queensu.ca/~andrew/papers/pdf/2013b.pdf. I did not open the originals by Sussmann and Jurdjevic.
[16] Baker-Campbell-Hausdorff formula, Wikipedia. https://en.wikipedia.org/wiki/Baker%E2%80%93Campbell%E2%80%93Hausdorff_formula
[17] Noether's theorem, Wikipedia. https://en.wikipedia.org/wiki/Noether%27s_theorem
[18] Hamilton-Jacobi-Bellman equation, Wikipedia. https://en.wikipedia.org/wiki/Hamilton%E2%80%93Jacobi%E2%80%93Bellman_equation
[19] Pontryagin's maximum principle, Wikipedia. https://en.wikipedia.org/wiki/Pontryagin%27s_maximum_principle
[20] Dijkstra's algorithm, Wikipedia. https://en.wikipedia.org/wiki/Dijkstra%27s_algorithm
[21] Litvinov, The Maslov Dequantization, Idempotent and Tropical Mathematics: a Brief Introduction (arXiv math/0507014). https://ar5iv.labs.arxiv.org/html/math/0507014 (search also listed arXiv 0710.0377 and 1001.4247)
[22] Path integral formulation, Wikipedia. https://en.wikipedia.org/wiki/Path_integral_formulation
[23] Tropical geometry (min-plus semiring, naming after Imre Simon), Wikipedia. https://en.wikipedia.org/wiki/Tropical_geometry
[24] Method of averaging, Wikipedia. https://en.wikipedia.org/wiki/Method_of_averaging
[25] Born-Oppenheimer approximation, Wikipedia. https://en.wikipedia.org/wiki/Born%E2%80%93Oppenheimer_approximation
[26] Delling, Goldberg, Pajor, Werneck, Customizable Route Planning (SEA 2011; Transportation Science 2017). https://www.microsoft.com/en-us/research/publication/customizable-route-planning/ ; PDF listed at https://i11www.iti.kit.edu/extra/publications/dgpw-crp-11.pdf (could not be fetched)
[27] Contraction hierarchies (incl. Customizable CH), Wikipedia. https://en.wikipedia.org/wiki/Contraction_hierarchies
[28] Hashlife, Wikipedia. https://en.wikipedia.org/wiki/Hashlife
[29] Krohn-Rhodes theory, Wikipedia. https://en.wikipedia.org/wiki/Krohn%E2%80%93Rhodes_theory
[30] Monge matrices and fast min-plus products for planar-graph boundary distances (Fakcharoenphol-Rao dense distance graph): https://arxiv.org/pdf/1602.07013 , https://arxiv.org/pdf/2404.06401 , https://web3.arxiv.org/pdf/2408.04613 (secondary summaries; the original Fakcharoenphol-Rao paper was not read)
[31] Min-plus product and APSP: https://dspace.mit.edu/handle/1721.1/134750 ; Vassilevska Williams talk abstract https://phd.uniroma1.it/web/virginia-vassilevska-williams---towards-a-better-understanding-of-all-pairs-shortest-paths_nS22993EN_EN.aspx
[Project] docs/PRINCIPLES.md, docs/representation.md, docs/reference/julien-2026-11-04.md (S39 to S41).
