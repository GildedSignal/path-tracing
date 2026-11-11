# Movement words

The first piece of the word compiler (docs/vocabulary.md, sections 1 to 7, movement
only). Consumer: job 3.2b, the movement planner.

- **`run_segment`** runs a program through the law, on a `RecordingWorld`, until the
  event rule fires. It returns the ticks, the exit kinds, the exit state, the footprint
  (cells, environment and the state fields read before written), and the resource
  deltas and guards. A budget that runs out returns `NoEvent`: no event is invented.
- **Events.** Any discrete fact of the state changed, or the body's block did. A
  counter cuts when it starts or stops, not on every count. Resources are guards.
- **`class`** is the words of the fields the segment read: the mask, the position
  inside the body's block, the velocity, the fall distance, the support relative to
  the block, and the other fields as bits. Resources and `t` are left out.
  - Position, horizontal: three bins per axis, cut at 0.3 and 0.7, where the body's
    box starts and stops overlapping the next block. Vertical: 1/16.
  - Velocity: four bits below the leading bit of its largest component.
  - `Grid` holds both. `tests/compiler.rs` prints the hit rate for other choices.
- **`ProposalTable`** is one `law::pattern` table whose start key is the class, so
  the world part of a class is the pattern key of the cells its proposal read.
  `proposals` looks a state up under each read mask seen. `replay` is the only way to
  a segment: it runs the law from the concrete state, keys the certificate from the
  exact start in a second table, and learns the outcome when it differs from the
  proposal.
- **`discover_gaits`** runs each program until a class recurs on the same velocity
  bits. The gait is the cycle of classes between the two visits; its orbit is the
  least period of the per-tick velocity. Position inside the block drifts from one
  orbit to the next, so a cycle of classes is several orbits long.
- **`width`** replays a segment with its inputs one tick early and one tick late.
  Clearance is not built.

Run `cargo test --release -p words -- --nocapture` for the gaits per ground type, the
head-hitter's speed, the hit rates and the microseconds per segment.
`jobs/3.2a-segments-and-gaits/verify.ps1` runs it with the law's gates and writes a
run record on D:.
