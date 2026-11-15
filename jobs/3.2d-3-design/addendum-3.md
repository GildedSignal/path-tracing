# Addendum 3: the zones design and its red-team (S130–S134)

Julien's turns since your last revision, verbatim in `C:\Users\PASCAL\Desktop\path tracing\docs\reference\julien-2026-11-10.md` (S130–S135):
- "the terrains complexity should be a mirror of compute's complexity here our path should take µs not tens of ms" (S131).
- Zone sizes "should arise based of needs, some regions are more compressible than others" (S132).
- "we need those two extremes": a one-million-block straight path, and the most convoluted tool-heavy 3D parkour (S131).

Read these three, in this order:
1. `C:\Users\PASCAL\Desktop\path tracing\jobs\zones-design\design.md`
2. `C:\Users\PASCAL\Desktop\path tracing\jobs\zones-design\redteam-opus.md` (Opus: FREEZE AFTER FIXES for R0, R1, R2 and R4 only)
3. The Terra judge's FAIL on the plain path: `D:\path-tracing-wt\plain-judge\jobs\plain-path-judge\verdict.md`

The plain path is built on `wt/plain-path` (`D:\path-tracing-wt\plain`, `engine/plan/src/plain.rs`). It is unmerged.

## Revise your spec `jobs/3.2d-3/last.md` to fold in R0, R1, R2 and R4

- **R0, the resident planner.**
  - One direct channel to the mod.
  - `PlayerState` from the bot's observed bits, never a re-simulated settle.
  - Fix the judge's blocking findings 1, 2 and 4: no per-tick work decides anything; food from counted debits on the observed bars; byte equality after the digest; the no-return proof (red-team finding 15).
- **R1, one role-class codec.**
  - It replaces the four keyings: footprints, pattern keys, class/table, `law_digest`.
  - It is relative to the kit's verbs (S134): legs, break, place.
  - Your section 3's four law-read roles are its legs projection.
- **R2, one generator store.**
  - It replaces `arith::Cycle`, the `reach.rs` tables, the `gait.rs` memo and the `plain.rs` fold (D3).
  - `reach.rs`'s `f64` trig goes; the law's table stays.
  - Transients are memoised under the recurrence key (T1).
  - Its key is the zone label (U3).
- **R4, the executor contract.**
  - The planner sends the word × count and its certificate. The client is the only per-tick runner.
  - At events, it checks exact bits minus the server-driven fields, with `deviation.rs`'s comparator (U9).
  - It re-anchors on the observed state.
- **Findings 1, 2, 3, 8 and 12 go in as written.**
  - Gates name their difficulty. The thinking gate runs on peaceful, Julien's world (HANDOFF anomaly 2). A non-peaceful million-block line is a two-word sentence, with the food crossing as an event.
  - Any bearing gets the two-yaw lattice closed form, not only the axes. That makes five decisions.
  - One Fermi floor per case: one zone; k events; varied terrain with a rolling horizon.
  - Ties are broken by a fixed canonical order at zero cost, and decisions are counted at switches.
  - No per-tick re-evaluation anywhere: cone checks go on the timing wheel.
- **U1 is the rule for the fold.** Binade exponent, food level, sprint allowed, loaded, zone boundary and yaw switch are all events under vocabulary §4. The fold is closed form between them.
- **R3 (zone tree), R5 (portals, field, segments) and R6 (cones) stay out of this revision.** Name them as later doors that wait on tests T5–T9. Name the hash-consed octree (finding 6), the min-plus overlay (U5) and the opening angle (finding 4) as their leads.
- **No new search code** (finding 10).

Keep the spec at 200 lines or fewer. Re-cut it into Terra slices, each with a machine gate. Mark which slices are independent. Put the tests that must pass before a slice starts (T1–T4 here) in that slice's gate. End with `STATUS: DONE`.
