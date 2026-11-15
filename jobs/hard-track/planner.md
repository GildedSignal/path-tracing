# Current planner diagnostic

Existing mode 5 (Field, arcs and gaits on, improve off, 1,500,000 nodes), command-built world. Run: `cargo test --offline --release -p plan --test hard_track current_planner_mode_5 -- --ignored --nocapture`. Budget: 10 seconds for field plus search; reach preparation recorded separately, as in the current flight.

- Reach preparation: 0.525839 s.
- Field: 0.021011 s; (standable, reached, checks, T at A): (496, 17, 99, None).
- Search: 9.981107 s; stop Asked; nodes 161637; expansions 18800; lower 29.
- Sentence: None.
- Refusals: [].
- Uncertified candidates: [].

This is a diagnostic, not a gate or an impossibility claim. No human T exists yet.
