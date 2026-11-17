# Port delta: BlockState.getDestroyProgress and Player.getDestroySpeed modifiers

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`. `port-breaking-threshold`, `port-break-stop-boundary`, `port-mining-abort-target`, and `port-mining-target-switch` already settled the `MiningClock` state machine; do not re-ask those. Anchor to `engine/law/src/mining.rs:11-27` (`MiningSample`).

**Consumer:** `docs/plan.md` step 5 (`P2c`, exact breaking T calculation in `MiningSample`).

**From the 26.2 server bytecode**, state with class, method and offsets:
- `BlockBehaviour.getDestroyProgress` (to locate) -> `Player.hasCorrectToolForDrops` (`30` vs `100` divisor, hardness `-1.0f` unbreakable check) -> `Player.getDestroySpeed` (to locate):
- base tool `ItemStack.getDestroySpeed` (`DiggerItem` / tool tier speed, to locate), `Attributes.MINING_EFFICIENCY` / `Efficiency` enchantment addition (`level^2 + 1` or attribute value in 26.2, to locate), `HASTE` (`DIG_SPEED`) & `MINING_FATIGUE` (`DIG_SLOWDOWN`) multipliers (`MobEffectUtil.getDigSpeedAmplification`, to locate), `SUBMERGED_MINING_SPEED` / Aqua Affinity (`/ 5.0f` check), and `!onGround` (`/ 5.0f` check);
- exact float evaluation order and rounding/casts into `destroy_progress`.

**Build a jar-backed reproduction:** run `reproduce.sh` disassembling `getDestroyProgress` and `getDestroySpeed`, and invoke `BlockState.getDestroyProgress` directly from the jar across at least 6 cases (stone with bare hand, iron pickaxe, efficiency V pickaxe, underwater without/with Aqua Affinity, mid-air `!onGround`, and Haste II / Fatigue I), printing raw `f32` bits and exact `k_break = ceil(1.0 / p)`. Write `dossier.md` starting with `Claim: ` on its first physical line.

**Falsifier:** any modifier factor, attribute name, or float multiplication order that produces a different raw `f32` bit pattern than the jar's `getDestroyProgress`.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
