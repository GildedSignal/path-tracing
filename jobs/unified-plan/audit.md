# Audit of the current plan against his words

Designer: Fable 5.2 sub-agent, 16 November. One row per finding. "His turn" cites the verbatim files in `docs/reference/` (`04` = julien-2026-11-04.md, `09` = julien-2026-11-09.md, `10` = julien-2026-11-10.md) by line. "Where" cites the doc, spec or code line. "spec" = `jobs/3.2d-3/spec.md`; "design" = `jobs/zones-design/design.md`; "redteam" = `jobs/zones-design/redteam-opus.md`. The last column says where [plan.md](plan.md) answers it.

**Kinds.** **Forked:** a terrain kind got its own code path. **Removed:** a good thing that was approved and was taken out or sent to "later". **Missed:** his idea reached no living doc or no plan. **Badly wired:** the idea is in, connected to the wrong thing. **Stale:** a doc line his later words or a measurement overturned.

## A. The fork and what it took with it

| # | Kind | Finding | His turn | Where | Plan |
|---|---|---|---|---|---|
| A1 | Forked | The spec has two planners: slice E, a "plain adapter", and slice H, "the existing Planner per exceptional span". | S161 (10:555): "we can't have two paths like that "easy" or "hard" it should be the same mechanism" | spec:112, spec:115; design:111-119; postmortem-2026-11-16.md:12-15 | §2, §8, §10 |
| A2 | Forked | The fork goes down to the micro grain. Version 3 of the vocabulary was approved: the planner does not choose at a body-block change. The spec keeps that only for plain ground and leaves it as "debt" elsewhere. | S101 (10:203), S97 (10:181): "A* seems a bit odd" | vocabulary.md:117 (approved, "lands with job 3.2d"); spec:91 ("body-block cuts in genuinely exceptional spans remain explicit micrograin debt"); spec:115 | §1.2 (a word takes a count), §2 |
| A3 | Removed | "Sets backward, the point forward, the slide, one replay" is the approved acting mechanism. The spec moved sets, preimages and the backward solver to "LATER, OWN DESIGN DOOR". | S93 (10:163): "I approve the plan"; S162 (10:559): "it's not as simple a backward and forward we've grown from that" | learning.md:15; spec:135 | §1.2, §2 step 5 |
| A4 | Badly wired | "One rule at every scale" (compute goes to the bracket whose narrowing could change the decision due soonest) was approved on 10 November. No planner spec uses it. The spec routes by kind of ground instead ("Hot plain eligibility", "Hard"). | S65 (10:15): "i approve the plan"; S131 (10:415): "conditions to fill to use more compute or different type of compute" | scales.md:7; spec:24, spec:89 | §0.4, §2 step 6 |
| A5 | Badly wired | S120 was a critique of the representation. It was built as a fast path for one terrain. | S120 (10:345): "it should see a straight line and go oh i have no kit … so i get the modulus of my best tool" | PRINCIPLES.md:291 ("Talking it through (not yet built)") against commit 752eba2 | §0.3 |
| A6 | Badly wired | Zones were designed as a spatial partition: boxes merged bottom-up by equal description, then an octree. Both draw the zones before any need. His zones arise from need, and cells should emerge. | S132 (10:423): "they should arise based of needs, some regions are more compressible than others"; X9 (10:219): cells "should be emerging … like how doors break it" | design:83-84; redteam:58-64; spec:137 | §1.2 (a zone is a certificate's extent; the octree is only the presence index) |
| A7 | Removed | The wiggle was sent to R7+ and to job 3.3h, with no local signal for when it is useful. | S94 (10:167); S120 (10:356): "an extremely simple metric that's local perhaps that tells hey i'd like to get jiggled"; S131 (10:413): "loose in my predictions so that i can explore more paths and then wiggle the most promising ones" | spec:135, spec:140; PRINCIPLES.md:268 | §2 (the slide signal), kill test 8 |
| A8 | Removed | Segments in parallel were never specified. R5 is a later door, and `engine/` on main has no thread. | S120 (10:347): "breaking down in segments mean parallel work"; S131 (10:409) | spec:136; HANDOFF.md:27 | §2 step 4, U9 |

## B. Ideas of his that reached no living doc

The living docs stopped on 14 November (postmortem-2026-11-16.md:20). These turns sit only in the verbatim record, the register or a job file.

| # | Kind | Finding | His turn | Where | Plan |
|---|---|---|---|---|---|
| B1 | Missed | The two extremes as one test, and compute mirroring terrain. | S131 (10:409, 10:413) | PRINCIPLES.md:302 ("Open"); only design:9-16 | §0, §4, kill tests 1-2 |
| B2 | Missed | "biggest one … it breaks and then you go smallest". It is in learning.md but in no spec. | S91 (10:155) | learning.md:18; absent from spec | §0.3, §2 step 3 |
| B3 | Missed | His own account of navigating: lines between nodes, refined; many scales at once; contention points "at the edge of what i think i can do"; "a sensed notion of entropy". | X16 (10:249) | PRINCIPLES.md:272 ("Open") | §2, §3 |
| B4 | Missed | "maybe we control in derivative directly?" No register row quotes it. | S131 (10:417) | inventory-map report.md:141-142 | §1.1 |
| B5 | Missed | "a nether portal is a manifold … as if it is always there but you need to do a weird path to access it"; "pearls are wormholes". | S132 (10:423) | design:72; spec:140 ("R7+"); no living doc | §1.3 |
| B6 | Missed | Classes relative to the kit's verbs. | S134 (10:433) | spec:29-33 only; PRINCIPLES.md:305 ("Open") | §1.1, §1.3 |
| B7 | Missed | What Einstein would say: 4D heuristics. | S130 (10:402) | PRINCIPLES.md:301 ("Open") | §1.3 (cones), §2 (thinking while moving) |
| B8 | Missed | "a list of all the things in the area we care about"; "tools are mobile and terrain is not but it can be a tool also"; "a boat on the ocean somewhere". Entities as tools lying in the world. | S131 (10:409) | redteam:120 notes the gap; representation.md:103 has the principle | §1.3 (boats), §3 (prices) |
| B9 | Missed | Changing your mind fast near real players. | S121 (10:364); S6 (04:29) | spec:138 ("R6 … later"); PRINCIPLES.md:126 ("Adopted", built as a one-shot field per goal: inventory report.md:154) | §2, U7 |
| B10 | Missed | "thanks to T the mingled parkour 4D path just is not". Its consequence, that a wave ordered by T never revisits a node, is used nowhere. | S89 (10:147) | PRINCIPLES.md:262 | §5 |
| B11 | Missed | Event forms of the port. | S148 (10:503) | PRINCIPLES.md:329 ("Proposed"); `jobs/event-forms/report.md` | U1 (the kernel is the first event form), U5, U8 |
| B12 | Missed | "The representation must preserve the value of incomplete transformations." | S44 (09:23) | inventory report.md:132; representation.md:127 says it for the relaxed medium only | §1.2 (prices in sets), §1.3 (debts) |
| B13 | Missed | Crafting from what is in hand: "if you're on a frozen river and you have logs the bot need to think to make a boat". | S96 (10:175) | learning.md:20 mentions crafting in the bound; no plan | §4 (third walk) |
| B14 | Missed | "Also why don't we run bots on gpu?" has no register row. | S54 (09:156) | inventory report.md:127-128 | §6 |
| B15 | Missed | "how many of those important things have we not built yet? … we can't just omit things like that". The inventory found 15 of 35 adopted mechanisms in docs only; the spec ordered none of them. | S132 (10:423) | inventory report.md:9-12; HANDOFF.md:27 | §10 (each of U2–U9 names its doc-only mechanism) |

## C. Good things removed, and docs that point at nothing

| # | Kind | Finding | His turn | Where | Plan |
|---|---|---|---|---|---|
| C1 | Removed | He approved two upgrades from least action: local consistency with repair in both directions, and ridges. The register cites them as "claims 13 and 14" of representation.md. The doc was rewritten on 9 November and no longer has them. | S23 (04:109): "seems like an upgrade worth having to me" | PRINCIPLES.md:185; representation.md has sections 1–9 and no such claims | §3 (water and saddles); questions Q10 |
| C2 | Stale | The register points at representation.md's "Four questions" and "Still open" sections. Neither exists. | X3 (04:131); S23 (04:109) | PRINCIPLES.md:153, PRINCIPLES.md:188 | questions Q10 |
| C3 | Removed | R-brach was to run "right after the first flight, before Phase 3". The first flight flew on 14 November. Elytra is still refused by the law. | S17 (04:75): its emergence "mean that we're doing good"; S18 (04:79) | PRINCIPLES.md:195; HANDOFF.md:45; `movement.rs:130-132` (event-forms report §4.1) | U8, kill test 10 |
| C4 | Removed | The far field (multipoles), a request he has made about ten times, is in roadmap step 6, after everything. The far map is what lets a coarse bracket close and stop lower scales from opening. | S6 (04:29); S131 (10:409): "We don't play enough with all the scales in a way that tells you to stop thinking about the lower level ones" | PRINCIPLES.md:122; HANDOFF.md:78, HANDOFF.md:88 | §3, U6 |

## D. Ideas that are in, wired wrongly

| # | Kind | Finding | His turn | Where | Plan |
|---|---|---|---|---|---|
| D1 | Badly wired | The lower bound's c is terrain-only legs, with "no item" as a premise. ES 23 says c is the toolkit's top speed. With a pearl, a boat or an elytra the bound is not admissible. | S6 (04:29): pearls "are interesting too and powerful"; S132 (10:423): "pearls are wormholes" | `engine/plan/src/goal.rs:4-7`, `goal.rs:29-32`; PRINCIPLES.md:73 | §1.3 (the kit's cone), kill test 10 |
| D2 | Badly wired | The backward field is still per cell, with takeoffs from cell centres. learning.md says a set is states, not cells. | S91 (10:155): "blocks don't have intrinsic T cost"; X9 (10:219) | `engine/plan/src/reach.rs:10-11`; learning.md:16; redteam:83-85 | §1.2 (sets as plateaus) |
| D3 | Badly wired | The reader of the map is still "best-first over the sets near": a search. | S97 (10:181); X16 (10:249): "a representation where you don't need a policy because the map is clear on where to go"; S132 (10:423): "whether we actually need A* like algos" | learning.md:15; HANDOFF.md:99 | §5 |
| D4 | Stale | "Nothing reaches the executor without a replay" and "Each cycle is still replayed". | S120 (10:358): "why so unsure? that's not trusting your work, if it fail get better"; S117 (10:333); S79 (10:82): "of course a parabola should be one certificate" | vocabulary.md:18, vocabulary.md:62, vocabulary.md:113 | §1.2 (a worldtube is checked once); questions Q11 |
| D5 | Badly wired | "Far away is blurry" has no rule for what the blur must keep. An opening angle on geometry alone would lose the thin bridge. | S84 (10:124): "i'll miss the thin bridge"; S64 (10:9): "what far away even mean" | scales.md:65-66; learning.md:8 | §3 (persistence in T), kill test 7 |
| D6 | Badly wired | The hard track's certificate was argued in prose about the intended route. His shortcut used a face no certificate covered. | S150 (10:511): "Found a shortcut"; S132 (10:423): "in a way that you're sure that it's hard" | `jobs/hard-track/report.md:32`; `shortcut.md:14-16` | §1.2 (reach between all faces), kill test 3 |
| D7 | Badly wired | Editing is called an open contradiction in the design. It is settled: allowed, with restoration by arrival. An edit is a toll plus a debt. | S32 (04:187): "the world must be the same after the bot reaches point B" | design:73; PRINCIPLES.md:342; redteam:105-107 | §1.3 |
| D8 | Stale | Two spec lines rest on premises the lab has since refuted: a food super-period (U2), and an exit that is monotone in the start offset. | — | spec:139; redteam:130, redteam:69-73; `jobs/glm-digest/report.md:14`, `:24-25` | §10 |
| D9 | Badly wired | The million-line gate asks for one bearing and a food guard. The red-team showed one yaw misses B by up to 47.9 blocks and bare legs reach food 6 after about 1.4k blocks. The spec fixed both, inside the plain path only. | S131 (10:413) | redteam:13-29; spec:55-65 | §4, §7 (both are events of the one list) |

## E. Checked and found sound

- **End-to-end T** (S124): spec:16-18 holds it. It stands.
- **Presence first** (S120): spec:37-39. It stands, as step 3 of the loop.
- **Observed bits, no settling** (design D7): slice A. It stands.
- **One codec** (design D4): slice B. It stands.
- **Learning and acting are different processes** (S85): learning.md:7. It stands, and it is the only split the plan keeps.
- **Peaceful for now** (S136): spec:7. It stands.

## F. What the audit could not settle

- Whether S162's "i added other things" names anything beyond S116–S134. I read every turn and list what I found; he may mean something said outside the record (S103 was never recorded, 10:260-262).
- X13's text is in `docs/reference/private.md`, which I did not read.
