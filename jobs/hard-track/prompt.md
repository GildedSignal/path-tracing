# The hard track: the hardest 3D parkour our law can certify, for Julien to race

**Julien** (S131): "we need to hold into mind two extremes one million block straight path and the most convoluted tools use heavy 3D parkour possible with insane timings and combination of use tools that are non trivial". (S132): "for building the 3D parkour get smart about it make it branching and combinatorial but in a way that you're sure that it's hard so build from the end or something … use the tools and things that we already ported so far i would have terra though to build this not glm". (S138): "tell me when the hard track is done i'll race it then".

**Consumer:**
- Julien races it. His time becomes the human bound.
- It is the hard extreme the new planner (`jobs/3.2d-3/spec.md`) is held against.
- The current planner's failure on it is the diagnostic, not a gate.

## Decisions already made (do not reopen them)

1. **Which mechanics.** Only what the Rust law already simulates bit-exactly in `engine/law/src/movement.rs`, `blocks.rs`, `shapes.rs` and `world.rs`: movement, collision, step-up, jumping and sprint-jumping, and every block whose movement data or hook the law implements. Read the law to list them; do not assume.
   - Likely candidates: ice and blue ice (the ceiling sprint-jump on ice reaches ~19 m/s, `docs/engine.md`), slime, honey, soul sand, ladders and vines, water and fluids, powder snow, cobweb, head-hitters, neo jumps around walls, slabs, stairs, fences and walls (1.5 high), trapdoors and doors only as fixed shapes.
   - **No item use** (pearls, buckets, boats, block placing): the law and planner cannot run them yet. Name where each would fit later, in the report only.
2. **Built backward from B.** Start at B and choose each element going back. The state band (position, velocity, phase) it needs at its entry must be producible only by the element before it.
3. **A hardness certificate per element,** each from the law:
   - **(a) Reach:** a gap or height beyond what any simpler word reaches. Use the law's own reach (run it) or `lab/numbers/Speeds.java`'s figures, so a mechanic like ice momentum, a ceiling boost, a slime bounce, a ladder or water is required.
   - **(b) Timing:** a window of few ticks or few phases that the witness passes. Say how many ticks of slack it has, by running the witness with the input shifted ±1 tick.
   - **(c) Decoys:** branches that look as good where the route splits but are proven dead, by a bound or by running every word your planner's word set has from that state.
4. **Branching and combinatorial.** At least three places where the route splits. Only one combination of choices reaches B, or reaches it fastest. The others are decoys under (c).
5. **Site.** Inside x −110…−60, z −110…−60 (Julien's world spawn is (0, −60, 0); this box overlaps no track in `engine/world/tests/fixtures/tracks.txt`). The ground is the superflat floor: grass at y −61, A's block on it. Height up to y −20.
6. **The witness.** One input sentence (the existing programs and steps, or per-tick inputs) that reaches B from A's settled start under the law. It is replayed through the full law over a world built from your build list, bit for bit, in a test: `engine/plan/tests/hard_track.rs`.
   - Use only the existing law and planner code; add no search code.
   - The current planner (`plan-track`, mode 5) may be run on it with a cap. Record its result, sentence or none, as the diagnostic.

## Outputs

- `lab/tracks/hardtrack.build.txt`: the vanilla commands that build it (`/fill`, `/setblock`, block states exact), in order, with a first `/fill` that clears the box to air above the floor.
- `lab/tracks/hardtrack.json`, in the format of `lab/tracks/birchparkourmedium.json`, with A and B, and a `tracks.txt` line for the fixture.
- The world fixture can't hold it until it's built in-game. So make the test build its world from the build list over the superflat floor.
- `jobs/hard-track/report.md`, with these parts:
  1. **The elements**, in order from A, each with its certificate and its slack in ticks.
  2. **The branch points and decoys.**
  3. **The witness T in ticks.**
  4. **The current planner's result.**
  5. **Where item use, pearls, buckets and boats would extend it later.**

## Rules

- **Nothing live.** Someone is using this machine (S135), so do not run the game, the server or RCON. The integrator builds it in-game from the build list.
- **Do not touch:**
  - `engine/law/src/{pattern.rs,footprint.rs,state.rs}`, `engine/words/src/{class.rs,table.rs}`, `bot/`, `engine/plan/src/flight.rs`, `engine/plan/src/bin/`: slices A and B of 3.2d-3 are being built beside you;
  - `movement.rs`'s arithmetic.
- No new crates, no push.

## Done means

- The witness test passes.
- `cargo test --offline` from `engine/` passes.
- The report and the build list exist.
- `jobs/hard-track/last.md` lists every file and the commands run.
