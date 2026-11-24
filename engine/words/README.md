# Movement words

The word program definitions and joint event signatures consumed by `engine/plan`.

- **`program` (`program.rs`):** `Program`, `Arguments`, `Key`, `held_input`, and `tiny_programs` — each movement word maps facts of the state and arguments (`yaw`, `pitch`, flags) directly to a held tick input (`law::Input`).
- **`segment` (`segment.rs`):** `Event` and `EventKind` — causal boundary markers emitted on word certificates (`Tick`, `GoalBoundary`, `FoodGuard`, `Contact`, `SupportLost`, `Ceiling`).

Compiled words live in `engine/plan` (`store::Gait`, `solve::solve`, and `joints::compose_chain`). Run `cargo test --workspace` from `engine/` to verify `words` alongside `law`, `world`, and `plan`.
