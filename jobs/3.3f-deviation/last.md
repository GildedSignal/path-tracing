Implemented the [deviation library](D:/path-tracing-wt/deviation/engine/plan/src/deviation.rs) and `deviation <plan.jsonl> <recording.jsonl>` binary. The JSON report includes the first differing tick and all differing observed fields with their bits, packet counts, both ending ticks, and unobserved field names.

The comparison refuses incomplete recordings and unknown anomaly types. Position and rotation packets count as corrections; motion and explosion-knockback packets count as pushes. Ending ticks are zero-based row indices.

Verification passed:

- The L0 law replay matched its recording; an edited input produced a position or velocity deviation at that tick.
- `cargo test --offline` passed.
- The footprint verification caught **12 of 12** mutants and passed its restored checks. Its output directory was redirected because the existing log was inaccessible.

Net change: **+136 lines**. No doctest import fix was needed. Build and verification output remains in the untracked `target/` directory. No commit, push, or merge was made.