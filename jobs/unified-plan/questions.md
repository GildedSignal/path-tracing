# The decisions that are his

Designer: Fable 5.2 sub-agent, 16 November. Each question has its context, the options, what each changes, and my recommendation (S78: "properly formulate your questions"). They are ordered by what they unblock. [plan.md](plan.md) is the proposal; [audit.md](audit.md) is the evidence.

## Q1. Is this the mechanism?

**Context.** S161: "it should be the same mechanism". The proposal is one loop (plan §2): bracket; extend the biggest word until its certificate breaks; cut at the break; compose sets backward and the point forward; give compute to the bracket that could change the soonest decision. A straight line and a parkour differ only in how often a word breaks. Exact-state search survives in learning only, once per joint pattern.

**Options.**
- **A. Approve as the direction,** and send the two one-way doors (the set record of plan §1.2, and the label format) to a red-team before anything is built.
- **B. Talk it through first,** section by section, before any red-team.
- **C. Reject,** and say which sentence of plan §0 is wrong.

**Recommendation: A.** The red-team prompt must hold representation.md, scales.md and learning.md and ask the one-mechanism question (postmortem-2026-11-16.md:29).

## Q2. Does one backward wave over the joints count as "A* like"?

**Context.** S132: "i'm thinking of whether we actually need A* like algos." Plan §5 removes best-first search over exact states from acting. It keeps one thing: at the first sight of a site with no labels, a single backward pass over that site's joints. It is a shortest-path pass. It runs on the folded graph (about 400 nodes on the hard track, none on a plain), once per goal, and never revisits a node.

**Options.**
- **A. Accept it.** First sight costs a wave; after that the site has labels and acting only reads.
- **B. Forbid any wave in acting.** On unknown ground the bot moves on the coarse bracket alone while learning builds labels in the background. Purer; the first crossing of new broken ground is slower in T.
- **C. A, with B's rule for the first word:** never wait for the wave if the coarse bracket already fixes the first word.

**Recommendation: C.** It is what plan §2 says ("moving at once").

## Q3. What happens to the 3.2d-3 slices?

**Context.** Slices A and B are built and judged. P1 passed. The spec's E ("plain adapter") and H ("existing F per exceptional span") are the fork (audit A1).

**Options.**
- **A. Stop E and H. Keep A, B, P1–P4, C, D, F, G.** Rewrite K's gates as one-mechanism gates (plan §9, kill tests 1–3). Replace the "later doors" with U2, U3, U6, U7 (plan §10).
- **B. Finish the spec as written, then unify.** The plain path flies sooner; the fork is built twice over and must be removed later.

**Recommendation: A.** P2–P4 keep running: they test the kernel every zone uses.

## Q4. Is a wall a slow medium for a kit that can restore it?

**Context.** X4: "a wall is a two second delay not a wall". S13: "Basically don't modify the world to get around". S32 settled it: "the world must be the same after the bot reaches point B", and silk touch makes a stone wall restorable. Plan §1.3 treats a restorable solid as a medium with a toll and a debt, so the planner may tunnel wherever that is fastest and it can prove the restoration.

**Options.**
- **A. Yes, whenever restoration before arrival is certified.** Routes bend through walls by Fermat's rule; nothing else is needed.
- **B. Only when the kit was given for it** (on a track: the items he used, S28; elsewhere: an explicit permission per command).
- **C. Never for travel;** edits only where no other route exists.

**Recommendation: A for tracks, B elsewhere until the ledger (scales.md 4.9) has flown.** A crash mid-edit is the ruin case, and the ledger is not built.

## Q5. What does a consumable cost?

**Context.** S13: "i'm unsure here honnestly … a matter or amortization and resource management". The register still lists it Open (PRINCIPLES.md:196). The plan needs it for pearls: a pearl is an item, 5 damage (7.5 food to heal) and a 20-tick cooldown.

**Options.**
- **A. A price field:** an item costs the T to get another from its nearest source (representation.md:148). Correct in the long run; needs sources to exist.
- **B. A budget per command,** zero by default; on a track, the count he carried (S28).
- **C. B now, A when farms and chests exist.**

**Recommendation: C.**

## Q6. What is the bar on the hard track?

**Context.** He ran 469, 434, then 428 (shortcut.md:3). The witness is 579. The planner finds no sentence. Kill test 3 asks that the map hold his shortcut and that the bot's T be at most 428. His inputs cannot be replayed: the law refuses his double-tap sprint window (PRINCIPLES.md:330). S56: "ground in T but not goodharting".

**Options.**
- **A. His T is the bar; his inputs are never used.** Port the sprint window only because the law needs it (anomaly 10), not to replay him.
- **B. Also replay his run through the law as a diagnostic** (learning.md:24 allows it, "never as a hint").

**Recommendation: A, and B as a diagnostic once the window is ported.**

## Q7. How is the million-block line gated?

**Context.** S131: "we need those two extremes". Thinking for it is about 5 µs (plan §7). Running it live takes 2.8 million ticks, 39 hours, and on Normal the food runs out after about 1,407 blocks (glm-digest:11), so the honest sentence there is two words.

**Options.**
- **A. Gate the thinking at 10⁶ offline,** on a generator-backed superflat (spec.md:100), and fly 10⁴ live (R-far). The live million adds nothing the 10⁴ does not show.
- **B. Also fly the million live** under `/tick` acceleration.
- **C. A now; the Normal-difficulty two-word line after the mob port** (S136).

**Recommendation: C.**

## Q8. May "far away" be defined by its effect on T?

**Context.** S64: "what far away even mean". S84: "i'll miss the thin bridge". Plan §3 proposes: a far feature is kept if removing it would move a bracket by more than the slack of the decision it feeds. Size in blocks plays no part. No number of ours is in the rule; the slack comes from the allocator.

**Options.**
- **A. Yes.** It replaces "topology exact, geometry coarse" (scales.md:66) with a rule that can be tested (kill test 7).
- **B. Keep the opening angle alone** and test whether it loses the bridge.

**Recommendation: A.**

## Q9. Where does the GPU go?

**Context.** S118: "Are we using the gpu fully also?" S131: "run A* or something on them but on gpu in parallel". S54: "why don't we run bots on gpu?" An acting query is microseconds; one GPU round trip is about 100 µs (a Fermi input, to measure). So the GPU cannot shorten an acting query. It can shorten learning: compiling joint patterns by the hundred thousand, and waves over very large sites.

**Options.**
- **A. GPU for learning batches only,** after a measured crossover (ES fermi.md:24). Threads first.
- **B. GPU on the acting path too,** for large first-sight waves, accepting the 100 µs.
- **C. Measure the round trip now** (one small Terra job) so the choice rests on a number.

**Recommendation: C, then A.**

## Q10. May the docs be folded now?

**Context.** The post-mortem's first fix: "His design turns are folded into the living docs the same day." I did not edit any doc. Three things wait.
- learning.md, representation.md, scales.md and vocabulary.md need plan §§0–3 once he rules.
- The register points at sections that no longer exist: representation.md's claims 13 and 14 (PRINCIPLES.md:185), "Four questions" (:153), "Still open" (:188). The two least-action upgrades he approved in S23 were lost in the 9 November rewrite (audit C1).
- S131, S132, S134, S130 and X16 are "Open" in the register though this plan answers them.

**Options.**
- **A. The integrator folds after Q1,** and restores the two S23 claims.
- **B. Fold nothing until the red-team returns.**

**Recommendation: A for the register and the lost claims at once; the mechanism text after the red-team.**

## Q11. Is one check of a worldtube enough?

**Context.** vocabulary.md:18 says "Nothing reaches the executor without a replay", and :62 that each gait cycle is still replayed. S120: "why so unsure? that's not trusting your work, if it fail get better simple as that". S117: re-testing tick by tick "should never happen anyway since we're doing things ES style". S79: "of course a parabola should be one certificate".

**Options.**
- **A. A worldtube is checked once, when its certificate is made.** A replay happens only for a new joint pattern (learning) and when a certificate breaks. The server judges afterward, and every deviation is a surprise to learn from (S81).
- **B. Keep one whole-sentence replay before the first flight of each new sentence,** as a seat belt, and remove it when the deviation rate allows.

**Recommendation: A, with narrow joints still sealed by bit-exact gates** (SR-71, AGENTS.md).

## Q12. Did I miss what "i added other things" means?

**Context.** S162: "it's not as simple a backward and forward we've grown from that please read what i said". I took it to mean S116–S134: presence first, the modulus, moving at once, the hardness signal, the dream of zones and cutting points, segments in parallel, the wiggle with fat tubes, conditions choosing compute, control in the derivative, portals as manifolds, zones from need, classes by the kit's verbs. Each has a row in audit.md and a place in the plan. S103 was never recorded, and X13 is in a private file I did not read.

**The question:** is anything he added missing from that list?
