The fifth collision context is implemented, and the scaffolding descent witness passes. **Job 1.4e remains incomplete** because the law cannot identify spectators: `PlayerState` has no spectator-mode input, so the `LivingEntity.onClimbable` spectator branch cannot be evaluated or refused reliably.

The re-export changed collision answers for **14 of 32,366 states**: the `bottom=true`, distance 1–7 scaffolding states, with both waterlogged values. Their inside-cell shape changed from empty to the full-width, 0.125-block cap. The original 20 model columns and refusal census are unchanged; all **30,951 previously modelled states remain modelled**. The general sampling and selection are in [ModelTable.java](D:/path-tracing-wt/climbable/engine/world/tools/ModelTable.java) and [world.rs](D:/path-tracing-wt/climbable/engine/law/src/world.rs).

The existing climbable law already covers the ordinary paths:

- `LivingEntity.handleOnClimbable`, offsets 0–107: resets fall distance, bounds horizontal and downward velocity, and holds a sneaking player on a non-scaffolding climbable. The descent witness checks the widened `-0.15f` value bit for bit.
- `LivingEntity.onClimbable`, offsets 9–18 and 38–86: tests the climbable tag and the usable trapdoor-over-ladder path.
- Fall-flying is already refused before movement; its complete branch also needs the glider and relevant block-tag inputs. Ability flight is likewise refused before movement. Spectator mode remains missing and is **not** safely refused.

The original cap-shape witness failed before the change and passed afterward. The full descent witness also failed when the new sampled shape was temporarily disabled, then passed after restoration. The isolated `cargo test --offline` run passed, including the **6/6 jar suite**; the footprint verifier caught **12/12 mutants**. Fixture-dependent tests remain ignored.

Anomaly 4 is **narrowed**, not closed: the missing cap is addressed, but the requested complete climbable contract and its spectator refusal are not. Net change is **+43 lines** (+577/−534, including regenerated table rows). No live run, commit, push, or merge was performed.