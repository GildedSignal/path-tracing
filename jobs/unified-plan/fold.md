# The U0 fold: the approved plan written into the living docs

Fable 5.2 sub-agent, 16 November. Authority: S165 ("this is a beautiful plan i approve it") and S166 ("his recs from the questions seem sensible"; "the hard track is not even hard btw it's easy-medium"). Source: [plan.md](plan.md). No code, no commit, no launch: the integrator commits. Truth was rewritten in place; the earlier text is in git.

## Line counts

| File | Before | After | Budget |
|---|---|---|---|
| `docs/learning.md` | 76 | 129 | 200 |
| `docs/representation.md` | 167 | 199 | 200 |
| `docs/scales.md` | 145 | 166 | 200 |
| `docs/vocabulary.md` | 117 | 121 | 200 |
| `docs/engine.md` | 183 | 186 | 200 |
| `docs/HANDOFF.md` | 120 | 120 | 120 |
| `docs/PRINCIPLES.md` | 342 | 343 | (register; three status cells and one new row) |
| `jobs/unified-plan/plan.md` | 211 | 213 | 400 |
| `jobs/unified-plan/questions.md` | 128 | 130 | — |
| `jobs/unified-plan/fold.md` | new | this file | — |

## What went where

**`docs/learning.md`**
- The status line now names both approvals (S93, S165, S166).
- New **section 0, "Acting: one loop for every terrain"**: the one-mechanism rule (S161); the two extremes with his definition of a hard track and its last clause (S166), and `hardtrack` named easy-medium; why one loop serves both; `solve` in six steps; the hardness signal, presence first, biggest first, segments in parallel, moving at once, free compute while moving (S165), the slide and its local signal, changing your mind, one check per worldtube; "Is A\* needed?".
- Section 1: the "map plans" bullet no longer names a best-first reader; the "set is states" bullet points at the set record instead of "Proposed, to red-team, not built".
- Section 2: step 1 no longer replays the sentence; step 3 is scoped to one joint pattern at a time.
- **Section 6: kill tests 8–17** (the plan's ten, renumbered after the existing seven; the million-line gate added as 17).
- Section 7: a pointer to `sources.md`, and Frazzoli to be read before U1 freezes.

**`docs/representation.md`**
- A **"Thinking floors"** table under the Fermi floors, with its inputs.
- New **section 10, "The space under a kit: a crystal with defects"**: the four folds, momentum's memory derived, words in velocity space, the objects (zone, joint, set, edge, site, label, field), the kit folded in (edits, wormholes, the kit's light cone, other players).
- New **section 11, "Kept from earlier drafts"**: the four questions (X3), local consistency with repair in both directions and ridges (S23, lost on 9 November, restored), and "Still open". The register's three dead pointers now land here.
- Section 5: the far-field paragraph names portal tables, labels and the keep-by-effect-on-T rule.
- Section 8: the consumable budget.
- The set record and the label format are added to the hard-to-undo choices.

**`docs/scales.md`**
- Section 2: "the planner is this rule", with the table of what each scale hands up and down.
- Section 3: rows 3–5 of the ladder name joints, sets, zones, portal tables and labels, and steps U1–U6.
- Section 4.3: what the blur must keep (effect on T; persistence), and the multipole in min-plus form.
- Section 4.7: cones on the wheel, latency before throughput, the L3, data shaped like its access, threads then the GPU.
- Section 4.10: no second path for any kind of terrain. Section 5: one row of sources.

**`docs/vocabulary.md`** (version 3)
- Header and a version 3 entry: no decision at a body-block change, for every word on every terrain; `count` an argument of every word.
- Section 2: a worldtube is checked once, when its certificate is made, with his three turns; replay only for a first-seen joint pattern or a broken certificate.
- Section 4: the body-block rule stated for every word. Section 5: the `count` argument. Section 6: a cycle is not replayed per use. Section 8: `count` in the record. Section 10: joints and sets in acting, concrete states in learning. Section 11: the certification line. Section 12.4: closed.

**`docs/engine.md`** (only where the contract changes)
- The certificate row; the executor at events, not every tick; the engine jumps from event to event, with the tick law as oracle and S148's event forms.
- **The contract:** outputs are words with counts and certificates, not tick-stamped controls; the lower bound is the kit's (and `Goal::C` holds for bare legs only); two guarantees rewritten (one mechanism; anytime as "the first word when its bracket closes").
- The machine: the GPU takes learning's batches only.

**`docs/HANDOFF.md`**
- "Start here" is dated 16 November and opens with the approved plan, where it lives, and what is stopped. Slices A, B and P1 are stated with what comes next. `hardtrack` is named easy-medium.
- Roadmap item 2's planner line; item 3 points at his definition of a hard track.
- **The order of work U0–U9** under the roadmap, with what stands and what is replaced.
- Six tracked-request rows now land on the new docs and steps instead of `zones-design`.
- "Open for Julien" gains Q12.

**`docs/PRINCIPLES.md`** (register only)
- Rows X3 (line 153), S23 (line 185) and S23 (line 188): the status cells point at representation.md section 11. His words in column one are untouched.
- One new row, S165, **Followed**, after S149.

**`jobs/unified-plan/plan.md`, `questions.md`**: a status paragraph each (S165, S166), and `hardtrack` named easy-medium where the plan called it "the hard track".

## The twelve questions (S166: "his recs from the questions seem sensible")

| Q | Recommendation adopted | Where, marked "(S166)" |
|---|---|---|
| 1 | the mechanism; red-team the set record and the label format before building | learning.md section 0; representation.md, hard-to-undo choices; HANDOFF U2 |
| 2 | one backward wave at first sight; the first word never waits for it | learning.md section 0, "Is A\* needed?" |
| 3 | stop E and H; keep A, B, P1–P4, C, D, F, G; rewrite K's gates | HANDOFF, the order of work |
| 4 | a restorable wall is a slow medium on tracks; elsewhere only with a kit given for it, until the ledger has flown | representation.md section 10 |
| 5 | a budget per command now, the price field when sources exist | representation.md section 8 |
| 6 | his T is the bar; his inputs only as a diagnostic | learning.md kill test 10 |
| 7 | thinking gated at 10⁶ offline, 10⁴ flown live, the Normal line after the mob port | learning.md kill test 17 |
| 8 | far away is defined by effect on T | scales.md 4.3 |
| 9 | measure the GPU round trip, then learning's batches only | scales.md 4.7; engine.md, "The machine"; HANDOFF U9 |
| 10 | fold the docs; restore the two S23 claims | this fold |
| 11 | a worldtube is checked once; narrow joints stay sealed | vocabulary.md section 2; learning.md section 0 |
| 12 | (a question to him) | HANDOFF, "Open for Julien" |

## What was cut, and why

**`docs/representation.md`** (to stay under 200)
- Section 3, "The cache's unit is the segment": five lines became one. The 0.9⁶⁰ ≈ 0.2% argument and the discrete aims are in vocabulary.md sections 1 and 5. **Dropped:** the reserved "coarse tier" of model classes merged by replay and split on the first mismatch; kit classes (section 10) replace it.
- Section 6, creativity: shortened to pointers at scales.md 4.1. **Dropped from this doc:** "One cliff costs about 10³ simulated ticks this way, against about 10⁸ when tool uses are tried at every searched state", and R&D's "on the sandbox server, ranked by the T they would save if demand appeared". scales.md 4.1 holds the 10³ figure and R&D's objective.
- Section 9, building: five bullets became one paragraph; nothing was dropped.
- The hard-to-undo list and two sub-bullets of section 2 became single lines; nothing was dropped.

**`docs/vocabulary.md`**
- "Nothing reaches the executor without a replay" and "Why always replay": superseded (S117, S120, S79; Q11).
- "Each cycle is still replayed until section 12's question 2 is answered": superseded.
- Section 10's Fermi for a 1,000-block route (about 0.5 s): superseded; a pointer to the thinking floors.

**`docs/learning.md`**: "Standard pathfinding (S92) stays as the reader: best-first over the sets near": superseded by section 0. "One replay" became "each worldtube checked once".

**`docs/engine.md`**: "tick-stamped controls"; "The executor re-anchors on the observed state every tick"; "Every certificate it issues is still checked against the ticked law": each superseded.

**`docs/HANDOFF.md`** (17 lines out, 17 in)
- From "Now (14 November)": "Where we are" (it said nothing had been played, which the next paragraph contradicted), the in-flight table (both jobs finished), "Merged this session", and the S103 audit paragraph. All are session history; `jobs/docs-truth/report.md` had already marked them stale.
- From "Start here": the plain-path bullet (folded into the first bullet as "stopped") and the three sub-bullets Next, Running, Read (P1, 1.3e and spec-premises have finished).

## What I could not place

- **`docs/ladder.md` was not in the list, so it is unchanged.** It should take: the kill tests as frozen rungs with their planted faults; S97's test axes (HANDOFF still says "not yet in ladder.md"); his definition of a hard track beside the Rediscovery rung; and the latency line, which still measures "ticks from the whisper's arrival" rather than end to end from the click's server tick (S124).
- **`AGENTS.md`** still calls learning.md "the unified idea (approved 12 November, S93)". It should add 16 November, S165. It is at its 60-line budget and was not in the list.
- **The register has no rows for S150–S164 or S166**, and the rows for S120, S121, S130, S131, S132, S134, X9, X16 and X15 still read "Open" or "Talking it through", which the approved plan now answers. I was allowed three status cells and one row.
- **The numbers in the new "Thinking floors" table have no committed script** (AGENTS.md: "every number in a doc is regenerated by a committed script"). They are Fermi arithmetic from stated inputs; a small script should regenerate them, and the face count on `hardtrack` and the GPU round trip are inputs still to measure.
- **`engine.md`'s hardware line** does not state the 96 MB L3 that scales.md 4.7 now relies on.
- **`jobs/3.2d-3/spec.md`** still describes slices E, H and K as written. It is a job file, not a living doc; HANDOFF now says which slices stand. Its lines on U2's food super-period and a monotone portal map rest on refuted premises (audit D8).
- **`docs/footprints.md`, `server.md`, `clan.md`** were not read in the plan run and are unchanged. vocabulary.md's new "worldtube: the footprint, with its ticks" leans on footprints.md; someone who has read it should confirm the wording.

## The learning fold (S169: "the learning plan looks fine for now so i approve it")

Source: [learning-review.md](learning-review.md). Same rules: no code, no commit, no launch.

| File | Before | After | Budget |
|---|---|---|---|
| `docs/learning.md` | 129 | 153 | 200 |
| `docs/PRINCIPLES.md` | 343 | 344 | (one new row, S169) |
| `docs/HANDOFF.md` | 120 | 120 | 120 (unchanged: no U-step changes) |

**`docs/learning.md`**
- The status line names the third approval (S169).
- **Section 2** is retitled "Learning: one queue, one operation, one record". A preamble states the rule, the budget (free compute inside certified words, with its Fermi inputs; deadline work first, the rest R&D and play, no slice chosen), the queue, the real-time swap rule, the minimum effective dose, and that acting is never delayed. **The nine numbered steps keep their numbers,** because `goal.rs` and other docs cite them (2.3, 2.5, 2.7, 2.9); their content changed:
  - step 2: demand is a missing set or an open bracket, fed also by regret, surprises that helped, and the census;
  - step 3: a family's known member lends its breakpoints;
  - step 4: the four statuses, and learning by merging;
  - step 5: smaller circles (derive a family's envelope, then delete its stored patterns);
  - step 6: one exam, not two;
  - step 8: "kind" defined; confirmation rides on ordinary trips;
  - step 9: play as a source of queue items with skin in the game; body play only confirms, only where safe.
- **Section 4:** the three fragile rows are marked, with their guards (alternates kept at each joint; law hash and witness on every entry; play as the stressor); a new row for the store; three incentive rows (the queue, play, a slow first word); the asymmetry test with his highlight.
- **Section 5:** the "not found yet / the law lacks it" case is answered by the statuses; two new open cases (a mid-flight swap against other bots' reservations; the census's bound).
- **Section 6:** kill tests 18–24 (the review's L1–L7).
- **Section 7:** how far his highlights were read.

**Superseded in place:** "a small slice of idle time" as a chosen quantity (his 12 November decision that the slice stays is kept, with a note that its size now emerges); the second exam; "Play is the idle slice with a body".

**Not placed.** From the review, left in `learning-review.md` only: the full table mapping Waitzkin, Ferriss and Naval idea by idea; the incentive rows already guarded elsewhere (hit rate, switches, his time as the bar, lower bounds); Lindy (a modest fit); the "second mistake" (cascades), which stays "open: no model" in section 4. `scales.md` 4.1 still describes R&D's objective without saying its budget emerges; a one-line pointer to learning.md section 2 would close that, and scales.md was not on this fold's list.
