# Build Report: II. The City That Borrowed a River

## Reading & Constraints Log
- Read `C:\Users\PASCAL\Desktop\path tracing\jobs\three-builds\shared.md`, `D:\path-tracing-evidence\three-builds\architecture-brief.md`, `D:\path-tracing-evidence\three-builds\taste-profile.md`, `C:\Users\PASCAL\Desktop\path tracing\engine\world\data\blocks.tsv` (all 1,140 lines including 783-805), and `C:\Users\PASCAL\Desktop\path tracing\jobs\pagoda\` (`design.md`, `report.md`, `build.txt` lines 1-60) as command-syntax reference.
- Confirmed MC Java 26.2 box and join constraints:
  - Box: `x 2300..2749`, `z -125..124`, `y -61..80`.
  - West Road enters at `(2300, -60, 0)`; North Quarry Road leaves at `(2525, -60, -125)`.
  - Verified every block ID used against `blocks.tsv`; avoided `farmland`, `wheat`, and `campfire` (not in `blocks.tsv`), using terraced `dirt`/`moss_block` plots with `sugar_cane`, `pumpkin`, `melon`, `flowering_azalea`, and `composter`s instead.
- Confirmed `run_command` is blocked by a sandbox configuration error in this harness (`readonly /path-tracing-evidence/three-builds/: non-absolute file path`), so per `shared.md` the deliverable is `gen.rs` (pure Rust `std`, deterministic, writes `build.txt` into its first argument directory) alongside `design.md` and `report.md`.

## Deliverables Written
- `C:\Users\PASCAL\Desktop\path tracing\jobs\build-river-city\design.md`: Full design rationale from geology to hand scale, three networks (water, movement, production), outdoor rooms, four districts, three coexisting historical episodes, ASCII site/district plans and longitudinal gorge section, step-by-step walk with coordinates, honest Section 6 self-tests, and constraint self-check.
- `C:\Users\PASCAL\Desktop\path tracing\jobs\build-river-city\gen.rs`: Deterministic 3D voxel generator (`450 x 142 x 250 = 15,975,000` cells) with:
  - Seeded fBm geological terrain, S-bend river gorge, Karst Spring Basin (`y=+4`), stepped cascade pools, Lower Quays Canal & Terminal Estuary Lake (`y=-58`, bed `-61`, natural levee rim `-57`), and perimeter taper down to `y=-61` at all four edges.
  - Connected movement routes: West Road (`2300,-60,0`) -> West Ferry Landing -> Lower Quays Wharf Street -> 3-flight Cliff Switchback Stairway -> Old Market Square -> Old Tuff Bridge (`2525,-18,-24`) -> Upper Terraces -> Intake Weir Footbridge -> Aqueduct Maintenance Walkway -> Obsolete Aqueduct Footpath -> Old Fulling Mill Loft -> Small Unremarkable Doorway (`2489,-4,-17`) -> Rooftop Garden (`y=-4`), plus the North Quarry Road (`2525,-60,-125`), Quarry Freight Haul Ramp, and Under-Bridge Service Ledge shortcut.
  - Four-phase greedy 3D box merger (`Terrain`, `Structure`, `Water`, `Detail`) keeping every `/fill <= 32,768` and running an automated `audit_and_repair()` pass verifying edge water exclusion, horizontal/vertical water containment, falling-block support, and lantern support before emitting `build.txt`.

## Static Audit Fixes Applied
1. **Ceiling Tie-Beams & Bracketed Door Lanterns:** Added structural `spruce_log` tie-beams at `y_top` inside `build_stone_timber_house` and `spruce_fence` exterior door brackets so `place_lantern(..., true)` always has solid support above it regardless of wall height.
2. **Cliff Stairway vs. Old Fulling Mill Separation:** Shifted the Old Fulling Mill to `x = 2480..2495` and carved the Toll House undercroft after placing piers so Flight 3 of the Cliff Switchback Stairway (`x = 2496..2507`) passes cleanly underneath the Toll House with full headroom.
3. **Rooftop Turret Stair Fix:** Replaced overlapping stair coordinates in the Old Fulling Mill loft-to-rooftop turret with a clean 7-step linear flight (`x = 2481..2487, z = -18, y = -11..-5`) leading to landing `(2488, -5, -18..-16)` and the unremarkable `spruce_door` at `(2489, -4, -17)`.
4. **Loop Stair Connectors & Phase Ordering:** Opened the Weir Footbridge west parapet gap (`z = 11..13`) for the Terrace Promenade and Aqueduct Walkway steps, added the Market-to-Aqueduct stair (`2538,-19,-47 -> 2556,-2,-40`) and Active-to-Obsolete Aqueduct step-down (`2546 -> 2540`), changed `bell` to `attachment=ceiling,facing=north` under a timber bracket, and moved irrigated terrace `dirt` from `Phase::Detail` to `Phase::Structure` so water containment is solid before Part 3 runs.

## What I Would Improve With More Time
1. **Compile-and-Run Execution & In-World Walkthrough:** Compile `gen.rs` with `rustc -O gen.rs`, inspect the generated `build.txt` command/block counts and `audit_repairs` summary, and walk the entire survival loop in-game (especially at dawn, dusk, rain, and night) to tune stair transitions and lantern spacing.
2. **Richer Building Envelope & Street-Scale Variation:** Add varied gable orientations, asymmetrical dormer lofts, recessed timber shutters (`trapdoor`s), drainage downspouts (`waxed_weathered_copper` / `tuff_brick_wall`), street-level awnings, and custom corner shapes (`shape=outer_left/inner_right`) on roofs instead of uniform rectangular `BuildingBox` shells.
3. **More Granular Interior Clutter & Narrative Traces:** Furnish every terrace cottage, warehouse upper loft, and Assay Hall council room with distinct lived-in traces (workbenches, shelves, reading nooks, crates, signage) and refine the curved aqueduct soffit arches into true stone arch rings rather than periodic piers.
