# 1.9f offline integration gate

Run [run.ps1](run.ps1) with a fresh output directory. It bootstraps the pinned
Mojang 26.2 jar directly. Use its fresh flows.csv and require its SHA-256 to
match the verified flows.csv in
`D:/path-tracing-evidence/fluid-flow-eec5c28d20394f51a9f913dee1d612f2`.
The frozen hash is
`1E6FFFEE0C56DA0A0BA595CE4B2A806D644E4AF6BBFBBF040D295439F4ABF9BD`.

From the repository root, run
`cargo run --offline --manifest-path engine/Cargo.toml --example fluid-flow-oracle -- <flows.csv>`.
[compare.rs](compare.rs) decodes all 1,034 rows into the Rust world contract.
Each of the three components from the production current entry must match
the recorded binary64 bits on every row. Report row and component mismatch
counts. The generated level-gradient row also supplies the raw world for an
ordinary zero-input tick. The player starts at (0.5, 0, 0.5), in the center
cell with height 4/9 as binary32. That tick must move east by 0.014 and retain
the horizontal velocity after vanilla water drag. Its stored evaluated flow
points west. A separate evaluated snapshot must move west by 0.014 and retain
the corresponding horizontal velocity with the same drag. Check the water
contact, water height, position and horizontal velocity by exact values.

Run `cargo test --offline --manifest-path engine/Cargo.toml` for the existing
law suite and the directional face and partial-input controls.

Run the same CSV comparator with `--features mutant-fluid-wall-above` before
`--example`. Require exit code 1 with the first mismatch at
`fall-wall-above,water,4,true` (data row 9). Its oracle bits are
`[0000000000000000, bfef9089fd7aa128, bfc50b06a8fc6b70]` and the mutant must
return `[0000000000000000, 0000000000000000, bff0000000000000]`.
Keep the CSV, comparator output and exact first mismatching row.
