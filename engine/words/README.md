# Movement words

The word compiler's movement half (docs/vocabulary.md version 2, sections 1 to 7 and 11).
Consumer: job 3.2b, the movement planner.

- **`run_segment`** runs a program through the law, on a `RecordingWorld`, until the
  event rule fires. It returns the ticks, the exit kinds, the exit state, the footprint
  (cells, environment and the state fields read before written), and the resource
  deltas and guards. A budget that runs out returns `NoEvent`: no event is invented.
- **`class`** is the facts the law decides on, for the fields the segment read. A field
  from the block table or the toolkit enters by its bits. A field that accumulates
  enters as the answers of the law's own comparisons on it, each named in `class.rs`
  with its line in `engine/law`. No grid, no chosen number. Position leaves only the
  clamp at the world's edge: its other comparisons are against cells.
- **`ProposalTable`** keeps proposals under the class alone. A proposal is a program
  and its arguments, the outcomes its replays reached with their counts, and the
  pattern key of the cells its last filed replay read (the world part).
  - `proposals` returns every proposal of the state's classes, those whose world part
    holds here first. A miss is no proposal at all.
  - `replay` is the only way to a segment: the law runs from the concrete state. "As
    proposed" is the predicted (most frequent) outcome. A surprise leaves a witness in
    `witnesses`: the start state, the program, the arguments, both outcomes.
  - `invent` is the miss: every program at every aim to its event, each distinct piece
    learned. `regret` audits a hit against the same invention and learns nothing.
  - `counters`, `classes()` and `growth` hold section 11's counts; `walked` is the
    consumer saying which piece its route took.
  - Nothing is filed by its exact start. `Certified::key` computes a key when asked.
- **`discover_gaits`** runs each program until every field but position recurs bit for
  bit. The gait keeps the period, the displacement, the per-tick increments and the
  clamped axes. The cycle memo keeps each cycle once, under its start's bits.
- **`width`** replays a segment with its inputs one tick early and one tick late.
  Clearance is not built.

`cargo test --release -p words -- --nocapture` prints the gaits per ground type and the
microseconds per segment. `jobs/3.2c-vocabulary-v2/measure.ps1` runs the class against
version 1's on the 18 tracks and writes a run record on D:.
