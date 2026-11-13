# Job 1.4c: ground batch 2, three deltas the lab reproduced from the jar

**Why.** The port grows from what the lab has pinned (S46, S51, S81). Each delta below is reproduced by GLM against the 26.2 jar, with a dossier, a `reproduce.sh` and, where it exists, a `candidate.patch`. The law refuses or mispredicts these today.

**Consumer.** The deviation stream and the first flight on natural terrain (cactus, berry bushes and campfires sit on trips; the fluid and fall-flying gates are read by every tick in water or with an elytra).

**Read.** `AGENTS.md`, `docs/engine.md`, `lab/wiki/port-block-laws.md`, `port-fluid-rules.md`, `port-movement-modes.md`, and in the main checkout `C:\Users\PASCAL\Desktop\path tracing\lab\results\` these run folders, whole (dossier, check.md, reproduce.sh, witnesses, candidate.patch):
1. `local-20261111-1823-1-1-port-block-laws`: the `entityInside` hurt family (cactus, campfire, sweet berry bush) as one gate plus three per-state effects: one `insideContact` column in the block table, not a branch per block.
2. `local-20261111-1823-1-3-port-fluid-rules`: the whole current half of fluid contact gated on `Player.isPushedByFluid()` = `!abilities.flying`.
3. `local-20261111-1823-1-4-port-movement-modes`: the fall-flying stop, `LivingEntity.updateFallFlying()` clearing shared flag 7.

**Branch.** You are in `D:\path-tracing-wt\ground-2` on `wt/ground-2`, cut from `main`.

**Build.** One delta at a time, in the order above. For each: first a witness test that is red on main, taken from the dossier's jar-derived values bit for bit; then the port, as data where the dossier shows data (a column read from the game's dump, never a constant of ours or a per-block branch); then the witness green. A candidate patch is a lead to read, not something to apply blindly: the check.md says what its checker found. Where a dossier does not pin something you need, skip that delta, say exactly what is missing, and go on to the next.

**Must not.** Add a tolerance, an epsilon, a retry or a per-block branch. Add a rule the jar evidence does not show. Touch `bot/`, `docs/`, `engine/plan`, `engine/words` beyond what compiles. Run anything live. Commit, push or merge.

**Done means.** The jar suite still 6 of 6; `jobs/1.2-footprints/verify.ps1` still 12 of 12 mutants; `cargo test` from `engine/` passes; a final message with, per delta: ported or skipped and why, the rule (class, method, offsets from the dossier), the witness red then green, the net lines.
