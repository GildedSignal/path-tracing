# Job 1.4e: climbable descent, and the collision context the exporter never samples

**Why.** Two things block the scaffolding-descent track, and they are the same bug seen from two sides: the law lets a descending body fall through scaffolding that vanilla stops, and the block table has no cap to stop it with. This is **anomaly 4** in HANDOFF, and its stated fix is a fifth sampling context in the exporter — **not** a per-block branch. A per-block branch here would be exactly the patch AGENTS forbids.

**Consumer.** The scaffolding-descent rung, which cannot run today. It also removes anomaly 4 from the list.

**Branch.** `D:\path-tracing-wt\climbable` on `wt/climbable`, cut from `main`.

**Read.** `AGENTS.md` (especially "No patches"), `docs/engine.md`, `docs/footprints.md`, `docs/HANDOFF.md` anomaly 4, and in the main checkout `C:\Users\PASCAL\Desktop\path tracing`:
- `lab\wiki\trial-scaffolding-descend.md` and the run folder `lab\results\local-20261109-2151-1-4-trial-scaffolding-descend\` (**reproduced**): a sneaking player with negative vertical velocity is held at `y=0.0` on a ladder but keeps descending at `max(y, -0.15000000596046448)` blocks/tick inside scaffolding, because `LivingEntity.handleOnClimbable` skips the hold depending on the feet block.
- **The checker's discrepancy on that run, which is part of the specification, not a footnote.** It records that for 26.2 `LivingEntity.onClimbable` the summary "feet block in tag climbable" is incomplete: it also returns false for spectators (offsets 0–8), false for fall-flying inside `BlockTags.CAN_GLIDE_THROUGH` (offsets 19–37), and **true** for `TrapDoorBlock` with `trapdoorUsableAsLadder` (offsets 58–86, the water-trapdoor case). Port what the jar does, all of it, from those offsets.
- The run folder named in HANDOFF anomaly 4 for the exporter's four collision contexts, and `engine/world`'s exporter and block-table code.

**Build.**
1. **`onClimbable` and `handleOnClimbable` as the jar has them**, including the three branches the checker named. Where a branch needs state the law does not carry (spectator mode, fall-flying with a glider), say so and stop on that branch rather than inventing the state — and say exactly what is missing.
2. **The exporter's fifth context: the feet inside the cell.** Anomaly 4 says the four sampled contexts never place the feet inside, so 14 `scaffolding[bottom=true,distance>=1]` states carry no 2-pixel cap, and vanilla stops a descending body at `y+0.125` where the law falls through. Add the context to the exporter so the cap appears in the table **for every block whose shape depends on it**, discovered by sampling, not by naming scaffolding. If adding the context changes other blocks' entries, that is a result: report the count and the diff, do not suppress it.
3. **Re-export and diff the block table.** State how many of the 32,366 states changed, and that the previously modelled 30,951 did not regress. A state that was modelled before and is refused now is a blocking failure, not a detail.

**Must not.** Branch on `scaffolding`, on any block name, or on the 14 states by identity. Add a tolerance, an epsilon or a clamp of your own — `0.15000000596046448` and `0.125` come from the jar, by offset. Touch `engine/plan`, `engine/words`, `bot/`, or the planner. Run anything live. Commit, push or merge.

**Done means.** A witness red before and green after for the descent, bit for bit against the dossier's per-tick value; the re-exported table with its diff counted; `cargo test --offline` from `engine/` passing; the jar suite still 6 of 6; `jobs/1.2-footprints/verify.ps1` still 12 of 12 mutants; and a final message with the rules as ported (class, method, offsets), the table diff, whether anomaly 4 is closed or only narrowed, the net lines, and anything left out.
