# Post-mortem, 16 November: how the planner forked into an "easy" and a "hard" path

Julien (S161, S163): "we can't have two paths like that "easy" or "hard" it should be the same mechanism"; "i want to know why and when it happened".

## When

| When | Commit | What happened |
|---|---|---|
| 12–13 Nov | 4dabdc0, 83dbe82, 7bffc43, d1ec165 | The plan is whole. learning.md (approved S93) has one loop: words pulled by context, biggest tool first, sets backward with premises and prices, the point forward, decisions at switches. "Search over exact states belongs to learning." |
| 14 Nov 10:26 | c679bc3 | HANDOFF is rewritten for a new session (S104). Its roadmap line is "3.2d: sets backward, the point forward, one replay". **It is the last time the plan's design text changed.** |
| 14 Nov, afternoon | S120, S121 | His straight-line turn ("it should see a straight line and go oh i have no kit… the modulus of my best tool", "presence first", "segments in parallel") criticises the representation. The register files it as **"Talking it through (not yet built)"**. |
| **14 Nov 17:59** | **752eba2** | **The fork is born.** "plan-track now asks the plain path first and hands the click to the planner, unchanged, when it does not apply." S120 is built within hours, as a special case in front of the unchanged A\*, with `catch_unwind` as the hand-off. It was never talked through, though the register said it would be. |
| 15 Nov | plain-path judge | FAIL, but for per-tick work, an invented food bound and digest equality. **Nobody judged it for being a second path.** |
| 15 Nov 12:05 | 81452b4 | My zones design (`jobs/zones-design/design.md:111-119`) keeps the fork: "if one plain zone covers the line… closed form; otherwise… A\*… inside spans the representation cannot certify." |
| 15 Nov, afternoon | 3.2d-3 spec | Astra makes it two systems: slice E (the plain adapter) and slice H (the old Planner per "exceptional span"). "Sets backward" goes to "LATER, OWN DESIGN DOOR". I accepted it and launched slices A and B. |
| 16 Nov | S160 | I describe the plan to him as "easy vs hard" without seeing it. |

## Why

1. **The living docs stopped on 14 November.** No commit has touched representation.md, scales.md, learning.md, engine.md or vocabulary.md since then. His turns S116–S134 went into the verbatim record and the register, but were never folded into the plan: the dream, zones from need, cutting points, parallel segments, the wiggle, conditions choosing the compute, water at saddles, the two extremes. The design lived only in job files and in a worktree (`D:\path-tracing-wt\sets`).
2. **Every new context started from HANDOFF's "start here" and a summary, not from the docs.** There were several compactions and sessions since 14 November. Those carry jobs, meters and launches, not the mechanism. So each spec was checked against the previous spec and the judges' findings, never against learning.md.
3. **S120 was read as a feature request.** It was a critique of the representation (the straight line should be trivial *because the representation makes it so*). It was built as a fast path for one kind of terrain.
4. **The judges and red-teams were asked the wrong questions.** Their prompts held the spec and the code, not the docs. They checked per-tick work, tolerances and made-up numbers, and passed the fork. My own red-team question ("is anything here a second search engine?") was the right one, put to a design that already assumed the fork.
5. **The integrator became a dispatcher.** From 14 November my turns went to launches, meters, scripts and shutdown recoveries. "For a new idea, think and talk with him first, and build when he says so" (AGENTS.md) was skipped for S120. The register's status ("Talking it through") was left saying the opposite of what happened.

## What changes

- **His design turns are folded into the living docs the same day,** with the register row pointing at the doc line. A turn left only in the register counts as not done.
- **Every spec and design is checked against the docs before it is accepted:** the one-mechanism test (no terrain kind gets its own code path) and learning.md section 1. Judge and red-team prompts include representation.md, scales.md and learning.md, and ask that question.
- **A register status must match the code.** "Talking it through" and "Open" mean nothing built.
- **After a compaction or a new session, the integrator re-reads the docs, not only HANDOFF.**
