**Job 1.4 is incomplete.** Two provisional changes are in [movement.rs](D:/path-tracing-wt/ground-1/engine/law/src/movement.rs); magma remains unchanged, and there are no commits.

- **Swimming:** `update_swimming` now includes the flight and passenger terms. A regression covering the dossier’s eight oracle cases failed on the flight case before the change and passes afterward.
- **Knockback:** `ServerResourceEvent::Knockback` applies the velocity rewrite from a supplied strength, resistance and resolved direction. Its velocity witness failed before the implementation and passes afterward. This is a standalone knockback event; integration with accepted-hit and cooldown handling, the sync flag, and the dossier’s complete witness remain unfinished.
- **Magma:** the proposed exporter fix explicitly selects `MagmaBlock` by class. I stopped before applying it because the job says to stop if a delta needs a per-block branch. The exporter representation needs re-derivation under that constraint.

`cargo test --offline` passed, though recorder-dependent replay tests were ignored because their fixtures were absent. The footprint verifier caught all 12 mutants and passed its randomness audit. The release bench reported these ranges over seven samples, all in ns/tick:

| Case | Min | Median | Max |
|---|---:|---:|---:|
| Walk | 917.6 | 919.6 | 934.5 |
| Sprint-jump | 584.5 | 586.7 | 593.9 |
| Ceiling | 818.7 | 822.7 | 824.4 |
| Bumpy | 885.4 | 888.9 | 892.3 |

I could not create the requested commits: this worktree’s Git metadata is outside the writable roots and is read-only under the session’s filesystem policy. The provisional changes remain uncommitted.