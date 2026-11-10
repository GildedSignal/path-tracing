# Every scale, one rule

Approved by Julien on 10 November (S64, S65). It extends [representation.md](representation.md): one rule through every scale in space and time, how creativity arises, stays and compounds, what far away means, and the swarm's local rules. Every number is a Fermi floor with its inputs stated, never a measurement. The order of work lives in [HANDOFF](HANDOFF.md); the server boundary in [server.md](server.md).

## 2. One rule at every scale

**Every claim is a certificate with a bracket:** a lower bound nothing can beat, an upper bound something has witnessed, the footprint it read, and the deadline of the decision it feeds. Compute goes to the bracket whose narrowing could change the decision due soonest (ES's slack). A trip's T already works this way. The change: a cell's crossing time, a word class's best speed, a build's makespan and an item's price work this way too.
- **How scales serve each other:** coarse hands down deadlines and where precision is needed; fine hands up exact costs, new words and surprises.
- **Creativity, the far field and testing become one activity:** closing brackets.

**Three speeds.** Information is instant (the one mind). Bodies move at c: 0.36 blocks per tick sprint-jumping, about 1 on blue ice under a ceiling. The world changes at about zero away from players. The design lives in the two gaps:
- information ≫ bodies: everything is planned before a body moves (his "turn based", S44);
- bodies ≫ world: outside loaded chunks nothing changes, so what is certified there stays certified.

**Pace layers** (Brand, *How Buildings Learn*, in his library). Ordered by how often they change: law < vocabulary < world and medium < fields < plans < controls. A change may void things only in faster layers, and only through footprints. A block edit never touches the vocabulary. An inventory change swaps a pointer to another toolkit class's medium (classes found by hashing behaviour). A law fix voids words lazily, when a replay differs.
- *Killed if:* a fast change forces a slow layer to recompute.

## 3. The ladder of scales, space and time together

c ties space to time, so the scales form one ladder. Three families sit off it: builds (small space, long time), long edges (pearls, portals: far in few ticks), knowledge (no place, the longest time).

| # | Space (blocks) | Time (ticks) | What lives there | Representation | Rule | State |
|---|---|---|---|---|---|---|
| 0 | < 1 | < 1 (machine time) | collision thresholds, the hot loop | Mojang's floats; packed integers | no allocation, no lock, integer ticks | law built; layout 4.7 |
| 1 | 1, the body | 1 | motion, packets, re-anchoring | the tick recurrence | replay; the server's veto as an event | built, L0 in flight |
| 2 | 1 to 10 | 10 (a gait cycle is 12) | a jump, a tool use | segment: class, program, footprint, width | compile once, certify by replay | 3.2a |
| 3 | 10 to 10² (reach 4.5, a site) | 10² (a chunk takes 45 at sprint-jump) | words, crowds, fluids settling, chunk tickets | chains of segment ids, worldtubes | search certified segments; tickets follow tubes | 3.2b; 4.2 |
| 4 | 10² to 10³ (view 192, a region 512) | 10³ | a trip, a live bubble, infrastructure paying or not | medium regions: hull and bracket; one field per goal | follow the field; ski rental | 4.3, 4.4 |
| 5 | 10³ to 10⁴ (Nether ×8) | 10⁴ (a day is 24,000) | long trips, supply lines, hubs, food range | opening-angle hierarchy, portals, price fields | refine by decision; auction | 4.3, 4.6 |
| 6 | a build | 10⁵ to 10⁶ | a commission, amortization | the blueprint as W(t), the build front | reverse demolition; least slack first | 4.5 |
| 7 | none | 10⁷ and more | the vocabulary, the law, game updates, our own loop | generators keyed by pattern, versioned by law | R&D; deviations; a jar diff per update | 4.1 |

## 4. The changes

### 4.1 Creativity: how it arises, stays and compounds

**Compute is not the limit.** The machine runs about 10⁷ law ticks per second. A gap at 10³ verifications of 100 ticks costs about 10 ms. The limit is who proposes and what gets kept. Poincaré said it of mathematics: invention is choosing, not combining.

- **Arises: the law read backwards.** The plan says an effect has few producers, and never says where that table comes from. A hand-written one is a catalogue of tricks (ES 4). It comes from the law itself, by the footprint machinery we have:
  - probe each law term per regime: toggle one input (a key, or a cell's fluid, shape, climbable or friction) and record which state change follows;
  - record what each item use writes (`writes` exists in the word record);
  - join them: A can help B exactly when A writes something B reads (a cell, or the body's own state) and changes its answer. The world's own laws (water spreading) are links in the same chain.
  - **Creativity lives where one word writes what another reads.** Where nothing overlaps, things commute and there is nothing to find. Working backward from a gap then branches about 5 ways, as the plan assumed, with no catalogue. This is the chemists' retrosynthesis (Corey): back from the target through known reactions to what is on the shelf. Their protecting groups are our conjugates.
- **Stays: an archive indexed by effect.** Each word is stored as a generator under (start class, effect) on a Pareto front of cost, width, resources and obligations. A dominated word is dropped by proof. That answers the known failure of learned macros: more words slowing the search (Minton's utility problem). Search retrieves by wanted effect, so the branching factor stays flat while the vocabulary grows.
- **Compounds: stable subassemblies** (Simon's two watchmakers). A depth-12 idea found from primitives costs 5¹² ≈ 2×10⁸ trials. With depth-3 words already kept it costs about 10³. So intermediate words must be kept even when nothing wants them yet. R&D's objective changes: new cells in effect space, and the gap between the law's own bound and the best witnessed word per regime, beside forecast demand (Stanley's stepping stones; quality-diversity archives). The scoreboard already counts distinctions per second.
- **Proposers, now, in the lab.** GLM isn't the scarce resource. A lens proposes programs in the frozen program language, the law scores them, the archive keeps the best and feeds them back into the next prompt (FunSearch's loop). It starts when 3.2a gives an evaluator, on legs. It pulls "LLM-proposed words" from Phase 7 to the lab, with no authority.
- **Seeing a move 37.** Any certified chain that beats the best known T for its class with a compound nobody named goes to a discoveries log with its replay, for him to watch. **A discovery is a suspected law bug until the server audits it:** quants treat a backtest that looks too good the same way.
- **The falsifier, R-lego.** One carried input (blocks), one gap past the sprint-jump limit. Pass: a crossing appears with no code naming it, in ≤ 10³ verifications, and the block comes back. It is reused on a second gap of another shape (stays). It composes with momentum: run, jump, place under the feet (S6) (compounds).
  The server's rule is pinned (GLM `place-reach-legality`, reproduced 10 November): the eye must be within 5.5 blocks of the nearest point of the clicked cell; the placer's own body blocks its feet cell, and the cell below is legal once airborne.
- *Killed if:* any of the three fails, or a tool needs search code.

### 4.2 The server: a bot pays for a tube, not a disc

On his server vanilla loads 625 chunks around a player and ticks 441, because a human is unpredictable: the disc is a wildcard's light cone. A bot's future is a certified worldtube, so its demand is the tube.
- **The worldtube is the ticket.** The engine asks the server for the chunks under each bot's footprint, ahead of time, at the level the footprint needs: read means loaded; a write, a fluid or a dropped item means ticking for as long as the obligation lasts. That is a sixth role for the footprint, not a new concept.
- **The jar agrees** (GLM `unloaded-chunk-move`, reproduced 10 November, bytecode only). A player's own move and tick are never gated by its chunk's level. At level 33 (loaded, nothing ticks) collisions see the real blocks; scheduled ticks, other entities and pickup delays are frozen. At 32 blocks tick, at 31 entities do. In an unloaded chunk the server treats the terrain as air and the player ghosts through it, so the kill line below is real.
- **Decide the meaning now, build it when a rung calls.** The server boundary is a one-way door, so tickets and presence levels go into server.md's interface before it freezes. Five bots run on plain player tickets (S55: no invented problems); the build waits for the dispersed rung.
- **Fermi.** At sprint-jump a bot enters a chunk every 2.25 s. A disc brings 25 new chunks each time (11 per second), a 3-wide tube brings 3 (1.3 per second). Fresh terrain costs maybe 0.1 to 0.3 core-seconds per chunk (a guess to measure), so an exploring bot costs 1 to 3 cores on a disc and 0.13 to 0.4 on a tube. Ticking goes from 441 chunks to 0 for a traveler, which removes the "25 to 50 lone bots fill the tick" wall of server.md.
- **Presence is a tool** (approved, S65). The live bubble (random ticks, mob spawns, spawners that read the player list) is a world edit: a swarm that grows crops and spawns creepers along every trail deforms the world, which is S13's own argument. Default: a traveler is a body, with its contact set only. The swarm switches presence on where it pays (a farm, a site), and it is always on around humans, where everything stays vanilla.
- **Pregenerate ahead of the frontier in idle time:** 10k × 10k is 3.9×10⁵ chunks, about 2 hours and 2 to 4 GB.
- *Killed if:* anything differs inside a human's bubble, or a bot reaches a chunk before it loads.

### 4.3 Far away: three meanings, one rule

- **Far in T: blurry by right.** Least-time paths are stationary (Fermat), so being wrong about where a far detail sits costs second order. With cells that grow in proportion to distance (opening angle θ, as in Barnes-Hut and ES's rings), the worst loss is about θ²/4 of T: 0.4% at θ = 1/8. A 10⁴-block trip then needs about 10³ cells in open terrain, not 3×10⁶ columns; rough terrain adds cells only where their brackets could change the route. His own highlight says it: a small heading error costs little if corrected early and often.
  - **Topology exact, geometry coarse.** A far cell must say exactly which faces connect and which way. That part is first order. Its bracket is composed once from its children's, in (min, +), and shared by content hash.
  - **A region's content is a hull, never an average.** In open ground the fastest sustained motion in each direction is the convex hull of the gait velocities, per toolkit class. That is its lower bound; a witnessed chain is its upper bound. Open regions need no table at all: rays go straight and bend at interfaces, as light does and as ant trails do (leads.md). Tables exist only at constrictions (passes, doors, portals).
  - **Many-to-many** (the scheduler's need, S24): sites talk cluster to cluster through portals, with local corrections inside a site. This is the multipole request in the form min-plus allows.
  - **Every field computed is kept as a landmark** that tightens the bounds of every later query (ALT). Hubs become landmarks by being used.
- **Far from every player: frozen.** Outside loaded chunks nothing ticks. The engine knows every block from disk, and a certificate there never expires until a loader's cone arrives.
- **Never generated:** the generator is the far field. Pregeneration covers the worlds we use. Reading only the generator's low octaves is a later option.
- **Far in time: the same rule.** Precision follows time-to-use: certify what will be executed before its certificates are likely to break; the rest stays a bracket.
- *Killed if:* the overturn rate (fine overturning coarse beyond its bracket) exceeds what the bracket states.

### 4.4 The swarm: local rules that come with proofs

| Rule | Its guarantee | From |
|---|---|---|
| Build shared infrastructure when the time already wasted equals its cost to build and restore | within 2× of knowing the future | ski rental |
| Never idle while a task is available; take the one with the least slack | makespan within 2× of optimal, before travel times | Graham; it's our allocator |
| Bid for tasks; a claimed task's price rises | near-optimal assignment; prices are the logistics problem's dual, so "prices as fields" is exact | Bertsekas |
| Queue for a scarce thing by the delay you cause, not the delay you feel | no Braess paradox | Pigou |
| Reinforce what carries flow, let the rest decay | adaptive networks | Physarum, ants |

- **Stigmergy schedules, the field decides.** Ants lock in: a late shortcut was taken in 2 of 18 runs (leads.md), and army ants can circle to death. Their only signal is traffic. Ours is T, recomputed from the law.
- **Bots are bosons.** Players overlap, so only edits and scarce items exclude, and reservations exist only there. Overlap isn't free, though: every tick two players overlap, `Entity.push` rewrites both velocities (GLM `port-entity-rules`, reproduced 10 November). The law has `push_pair` with no producer; it gets one at slice C. (The entity-cramming cap is a lead to read in the jar.)
- **Particle in cell** (plasma physics) is the compute pattern: deposit worldtubes on the grid, update the fields, bots read them. Cost is bots plus cells, never bots².
- **Fields are everything shared; particles are everything individual.** Fields: T per goal, demand, price, risk, presence, and the bracket gap (where the map of possibilities is blurry). Particles: bots, items, obligations, tasks. A multi-leg plan (fetch, then deliver) is a min-plus product of fields that already exist.
- **A swarm bracket.** Every swarm rung reports makespan over max(critical path, work ÷ bots).

### 4.5 Building: the reverse of the fastest demolition

Demolition from the outside in is always accessible. Reversed, it is a build order in which every placement had a standing spot, reach and a face to click, so no void is left (S24, S44). The build front is a wave, like a travel front: (max, +) where travel is (min, +). Termite-inspired robots (TERMES) compile a blueprint into local legality rules the same way.

### 4.6 Logistics: handling, not hauling

- **The container changed shipping because handling was the cost.** A shulker box is the container. T sits in clicks, pickups and placements.
- **The Nether is the hub:** n² routes become n.
- **Pull, not push** (Toyota): a bot restocks when a bin empties, by the price field.
- **Range.** A 10⁴-block trip eats about 5 to 20 meals, depending on a rule to read in the jar.
- **The word record is our container.** Every layer handles it without unpacking, which is why new content can travel through the whole system.

### 4.7 Micro: data shaped like its access

- **The log is the truth, state is a fold** (programs plus scars; exact replay).
- **A bot's hot state is about 24 bytes** (segment id, phase, anchor, start tick): 10⁴ bots fit in L2.
- **The planning world lives in class space:** maybe 1 to 6 KB per chunk, so 10k × 10k fits in 0.4 to 2 GB against 12 to 39 GB in the JVM. The engine holds the map; the server holds tubes.
- **Times are integer ticks,** stored as small deltas, so min-plus runs 16 lanes wide.
- **The slack queue is a timing wheel;** a block edit finds its certificates through an inverted index per section.
- **Backtest, paper, live** are our offline loop, sandbox and server. Slippage is the deviation rate.

### 4.8 Testing: checks that need no oracle

- **Always-on invariants, free to run:** LB ≤ T ≤ UB; an unused item never raises T; a block outside the bracket's ellipse never changes a plan; a shift inside the binade scope never changes T; the same plan at any thread count; the books balance (4.9).
- **The overturn rate per grain** joins the scoreboard.
- **Endgames first** (Waitzkin learned chess from king and pawn): each tool alone in the emptiest world before natural terrain.
- **Flights with telemetry are tests.** The three slices in HANDOFF's roadmap are the fastest answers to the riskiest questions.

### 4.9 Ruin: the obligation ledger

A crash mid-mission leaves water flowing and blocks placed. Obligations are written ahead of each edit, as double entry: every edit owes its counter-edit, and a mission ends when the books balance. On restart a recovery mission restores what is owed. His world is snapshotted before any flight in which bots edit.

### 4.10 What we don't build

- A mob AI port before it has a consumer (S20 stands: mobs are law; they stay cones until a mob-farm word or a fight needs more).
- A worldgen port.
- A search per bot per decision at swarm scale (shared fields, plus one replay).
- Discs, observation and validation for bots.
- Networks before the proposal table's hit rate asks for one. The GPU before a measured crossover.
- Any catalogue.

## 5. Sources (leads, cited from memory: each one adopted goes through the foundations-primary lens first)

| Who | What they saw | Lands in |
|---|---|---|
| Fermat; Barnes and Hut; ants | stationary paths forgive far errors; opening angle | 4.3 |
| Poincaré, *Science et méthode* | invention is the sieve, not the combinations | 4.1 |
| Simon, "The Architecture of Complexity" | stable subassemblies make deep things findable | 4.1 |
| Corey; Newell and Simon | work backward from the target | 4.1 |
| Silver et al.; Stanley and Lehman; Mouret and Clune; FunSearch | search grounded in outcomes; stepping stones; archives; proposers with an exact judge | 4.1 |
| Karlin et al.; Graham; Bertsekas; Pigou; Tero et al.; Bartholdi and Eisenstein | local rules with proofs | 4.4, 4.6 |
| Werfel, Petersen and Nagpal; Homem de Mello and Sanchez | a blueprint compiled into local rules; assembly by disassembly | 4.5 |
| McLean; Ohno; Goldratt | the container; pull; the bottleneck | 4.6 |
| Brand; Stommel | pace layers; the space-time diagram | 2, 3 |
| Waitzkin; Musk; Taleb; quant practice | endgames first; delete; ruin and tinkering; backtest etiquette | 4.8, 4.10, 4.9 |
