# port-movement-exhaustion-branches

## local-20261117-0211-1-0-port-movement-exhaustion-branches (2026-11-17)
- Grade: reproduced
- Claim: In Mojang 26.2 (server jar sha1 `86765a5899bd9c96461036a628796b4245715058`) the exhaustion half of `ServerPlayer.checkMovementStatistics(double, double, double)` (`@0-452`) is bit-exact against `engine/law/src/movement.rs:1104-1121` for an unmounted survival/adventure player — identical branch order (`isSwimming`, `isEyeInFluid(WATER)`, `isInWater` horizontal-only, `onClimbable` stat-only, `onGround` sprint `(0.1f * cm) * 0.01f`, `isFallFlying`/airborne stat-only), `f64 -> f32` cast before `× 100.0f`, `Math.round(F)I` ties (`math.rs:45-47`), and `FoodData.addExhaustion` `min(e + a, 40.0f)` — with two contract-masked divergences: the port lacks the `isPassenger()` early-return (`@0-4`) and `Player.causeFoodExhaustion`'s `abilities.invulnerable` gate (`@0-10`, which disables all movement and jump exhaustion in Creative/Spectator).
- Falsifier: Any branch condition, multiplier constant, `f32` cast point, `sqrt` association, `Math.round` tie, or gate order differing from `checkMovementStatistics` `@0-452`, or any exhaustion-bit mismatch inside the unmounted survival/adventure domain.
- Run folder: local-20261117-0211-1-0-port-movement-exhaustion-branches
