Continue the same job. Two corrections from the integrator:

1. **You are the Terra builder.** AGENTS.md's launcher rule is for the integrator launching you; it has already been applied. Never run `scripts\launch.ps1`, `gpt.ps1` or `follow-up.ps1`. Build directly in your worktree.
2. **The prompt's "a few hundred nodes" was a made-up number; ignore it.** The graph has as many supports as the geometry gives (your inventory found 2,784 candidates). Size is not a gate.

Do these steps in order, each to completion, without stopping between them:
1. **Edges:** finish `whole_box_reach_certificate` on the **old** build. Its conservative edge bounds (prompt decision 3) must find Julien's route (the decoy-lane end → the alignment-wall top → the final ladder) as a path. If they don't, the bounds are wrong: fix the bounds, not the route.
2. **Holes:** list every A-to-B path in the old build's graph that skips an intended element. These are the holes.
3. **Geometry:** change it until the certificate passes (prompt decisions 2–4).
4. **Witness:** rebuild it and its timing and controls tests for the new geometry (decision 5).
5. **Outputs:** write `lab\tracks\hardtrack.build.txt`, `hardtrack.witness.txt` and `jobs\hard-track-2\report.md`.

Run the gates in "Done means". End with `STATUS: DONE` or `STATUS: BLOCKED: <why>`, blocking only on a question the prompt and this message leave open.
