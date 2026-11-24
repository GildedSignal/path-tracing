# Yarn-ball fold: last

`wt/port-fold` `43f43730` → `28f9cbcb` (engine) + the report commit. Report: `jobs/yarn-ball-fold/report.md`.

- Shipped: the elytra `cos` correction table (252 rows) moves to `engine/law/src/dcos.bin`. Law code lines 19,047 → 18,796 raw, 23,504 → 23,259 after rustfmt; group 14 down 0.22 decades; targeted groups 2/7/12/18 down 0.
- Gates: jar 18/18, law lib 95, replay 32 + 4 ignored, real_time passes and its 364 lines are byte-identical to `43f43730`. Three workspace failures, none from this change: native `resident` (fails the same at `43f43730`), plan `deviation` (track world file missing), world `julien_tracks_load` (the known fixture anomaly). Bench against `43f43730`: run 2 at or below 1.000 everywhere (run 1's ceiling 1.006 did not repeat). Against `c392524`: sprint-jump 1.012, already in `43f43730` (1.009 from the medians).
- GLM's ground rows match the law's `models.tsv` on 668 of 1,196 blocks with 0 differences; the 528 blocks that need state properties were not checked.

STATUS: BLOCKED: the slices' duplication is already folded at 43f43730 (bodies on basis::body rows, one hurt pipeline in core, ground factors as models.tsv rows); the rest of engine/law/src is unique rules pinned by jar rows (1,887 jar-only + 668 in-file-only code lines) or frozen sim.rs kernels, so 1,500 net lines needs his choice: delete in-file-witnessed unused rules, open sim.rs's calling side, or rebuild the cell engine so rows drive through a world boundary.
