# Frazzoli's maneuver automaton, read in full, for U1 and U2

**Consumer:** step U1 of the approved plan ("Frazzoli's paper read in full first", `docs\HANDOFF.md` U1) and the U2 set record. The Fable designer read only the abstracts (paywall, then HTTP 429; `jobs\unified-plan\sources.md`). Its plan rests on Frazzoli's form as "the closest prior form of the whole design": gaits as relative equilibria (trim trajectories), transitions as maneuvers.

You read and write only your report. Root `C:\Users\PASCAL\Desktop\path tracing\`.

## Read
- **Our side:** `jobs\unified-plan\plan.md` and `sources.md`; `docs\learning.md` section 0; `docs\representation.md` sections 10–11; `docs\vocabulary.md`.
- **Primary sources, in full where you can reach them:**
  - E. Frazzoli, M. A. Dahleh, E. Feron, "Maneuver-based motion planning for nonlinear systems with symmetries", IEEE Transactions on Robotics, 2005;
  - Frazzoli's PhD thesis (MIT, 2001), "Robust hybrid control for autonomous vehicle motion planning";
  - "Real-time motion planning for agile autonomous vehicles" (Frazzoli, Dahleh, Feron, 2002).

  Use open copies (MIT DSpace, author pages, arXiv, Semantic Scholar PDFs). Then read whatever they cite that matters to us: Burridge, Rizzi and Koditschek's sequential composition of funnels (1999); trim trajectories; the symmetry group's role.

## Write `jobs\frazzoli\report.md`
1. **What the papers actually say,** with page and section: the maneuver automaton's definition; trim trajectories and symmetry; how maneuvers compose; the completeness and optimality claims and their conditions; how they handle a state that is not exactly on a trim.
2. **The mapping to our design:** word ↔ trim, transition ↔ maneuver, count ↔ coasting time, our symmetry group (translations, binade-limited, yaw lattice), joints. Where the fit is exact, where it breaks (integer ticks, floats, collisions with terrain, which they lack), and what we would get wrong by copying it.
3. **What U1 and U2 should take from it, and what they should refuse,** as concrete changes or confirmations to `jobs\unified-plan\plan.md`, with line references.
4. **How far you read each source,** and whether it is primary.

Never guess: a source you could not open is "not reached".
