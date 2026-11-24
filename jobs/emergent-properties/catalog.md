# Catalog of Desired Emergent Properties & Controlled-Environment Tests

This catalog mines the entire `path tracing` repository (`archive/turns/`, `map/`, `map/detail/`, `archive/research/`, `jobs/`, and verified code across `main` and active worktrees) for every **emergent property** Julien has asked for across five scales. Every property below is something that **must fall out of general physics/representation/economics laws without hardcoded special-case logic**, alongside verbatim source evidence, triggering conditions, a falsifiable controlled-environment harness with an explicit anti-cheat mutant, and exact current readiness.

**Readiness Legend:**
- `flown` — Executed live in Mojang's 26.2 server + Fabric bot client with passing scoreboard logs.
- `offline-proven` — Proven in standalone Rust tests/probes against `engine/law`, jar witnesses, or `emergent-space-engine`, not yet wired/flown end-to-end in live `plan`/`bot/`.
- `blocked on <exact prerequisite>` — Spec-only (`Doc-only`) or partial sketch; blocked on named code/architectural gap.

---

## Domain 1: Single-Bot Movement, Representation & Terrain Scaling

### 1.1 Translation Collapse, Modulus-Plus-Remainder & Sub-Linear Straight-Line Scaling (`O(log D)`)

1. **Property name & mechanism**
   Over any uniform ground (or uniform medium under a fixed kit), position disappears from the dynamic problem through two folds: velocity contraction onto a periodic limit cycle (`v <- (v + input(yaw)) * friction * drag`, forgetting initial velocity to 53 IEEE bits in ~263–268 ticks on stone) and exact integer step maps per floating-point binade (`P2`). A long flat route therefore emerges automatically as a `(word, phase, count)` modulus of the kit's fastest gait plus a short remainder (`run`, `walk`, or transient brake) — never per-block A* search. Planning work scales strictly with binade crossings (`O(log D)`) and never with physical distance `D`.
2. **Verbatim quote(s) + exact source `file:line` & session tag**
   - `"we're not really doing x,y,z but something else my question is what is this something else?"` / `"i think the regimes should fall out of hand."` — `archive/turns/julien-2026-11-04.md:227` (`S40`)
   - `"it should see a straight line and go oh i have no kit but not hungry so i get the modulus of my best tool right now which is spint jump and then the remainder is run and i don't consume enough food bar to need eating."` — `archive/turns/julien-2026-11-10.md:345-360` (`S120`)
   - `"each knew command on the flat ground should take µs IN TOTAL."` — `archive/turns/julien-2026-11-10.md:857` (`S225`)
   - `"The terrains complexity should be a mirror of compute's complexity"` (`archive/turns/julien-2026-11-10.md:409`, `S131`); Kill test 17: `"Thinking is gated at 10⁶ blocks offline on a generator-backed superflat, Peaceful (S136), and 10⁴ is flown live (R-far)"` — `map/detail/learning.md:120` (`S131/S136`)
3. **Situations / conditions where it arises**
   - Any long uniform corridor (superflat or undisturbed natural strip) of `10²`, `10⁴`, or `10⁶` blocks with bare legs (`kit = ∅`) or any carried gait (`sprint-jump`, `blue_ice`, `head-hitter`), at Peaceful or Normal difficulty before the `food <= 6` horizon.
4. **Controlled-environment test harness design**
   - **Minimal arena:** Generator-backed flat stone strip (with `WorldSnapshot`/uniform-box query) spanning `D ∈ {10³, 10⁴, 10⁶}` blocks, starting from rest and from moving states, Peaceful difficulty (`difficulty = 0`).
   - **Measurable pass signal:** Log-log scaling slope of wall time and law-tick evaluations vs. `D` stays `<= 0.05` (specifically `<= 15 µs` median new command on warm store, `0` heap allocations after first command on ground, `0` refused commands), landing at `T == min` predicted by `law::try_tick` bit-for-bit across all 3 distances.
   - **Anti-cheat falsifier / mutant:** Plant a separate flat-ground fast-path branch (Kill test 8, `map/detail/learning.md:111`: `"Planted fault: a fast path for uniform ground must turn it red"`), OR switch `solve` onto an `O(D)` tick-by-tick corridor reader (`events::advance_cycles` removed in favor of linear cell walk). Must **fail** Kill test 8 trace gate or blow the `1 ms` gate at `D = 10⁶`.
5. **Current readiness**
   - `flown` on `main` (`b0d9ea3e`) for `straight100` (`288/288`), `straight300` (`845/845`), `straight800` (`2248/2248`), `stonesimple` (`85/85`), `real_time.rs` `13.5 µs` standing / `12.7 µs` moving median; open gap: `10⁶`-block offline Peaceful scaling & unified `O(1)` uniform-box spine (`inbox.md:1428-1434`, blocking `law::face::advance` merge into `plan`).

---

### 1.2 Crystal with Defects, `O(defects)` Compute & Unified Loop ("One Mechanism")

1. **Property name & mechanism**
   Under any kit, uniform terrain collapses into a crystal (`(word, phase, count)`), while obstacles, class changes, lips, and portals are **defects** where that symmetry breaks (`step 3: extend` breaks at the first defect, `step 4: cut` splits at the defect's joints). Because the exact same `solve(from, to, kit, deadline)` loop runs on every terrain without terrain-kind branches (`One mechanism`), compute scales strictly with **defects along the active worldtube**, never world size or total blocks.
2. **Verbatim quote(s) + exact source `file:line` & session tag**
   - `"the terrains complexity should be a mirror of compute's complexity here our path should take µs not tens of ms."` / `"we need to hold into mind two extremes one million block straight path and the most convoluted tools use heavy 3D parkour possible with insane timings and combination of use tools that are non trivial"` — `archive/turns/julien-2026-11-10.md:409,413` (`S131`)
   - `"what we can't have two paths like that "easy" or "hard" it should be the same mechanism"` — `archive/turns/julien-2026-11-10.md:555` (`S161`)
   - `"hard tracks are tracks that have many types of mechanics that are very precise and chained in convoluted 3D space over great spans but to the bot it should not look like that."` — `archive/turns/julien-2026-11-10.md:577` (`S166`)
   - `"i don't care where they are just if they are present first which should be cheap and reduce by a lot the considerations, it should also be broken down in segments if there's such things present like ice road."` — `archive/turns/julien-2026-11-10.md:345-360` (`S120`)
3. **Situations / conditions where it arises**
   - Comparing a `10³`-block uniform stone runway against a `10³`-block ice+stone segmented plain (1 defect = 1 class boundary) vs. a `50`-block neo-parkour track with 20 dense fence/stair/gap defects.
4. **Controlled-environment test harness design**
   - **Minimal arena:** Three test arenas served by the identical binary and `solve()` entry point: (A) `straight800` (0 interior defects), (B) `ice-transition-800` (1 surface transition halfway), (C) `hardtrack` / chained parkour (`>= 10` defects).
   - **Measurable pass signal:** Instrumented stage execution log confirms both `straight800` and `hardtrack` traverse the exact same stages (`bracket -> extend -> cut/compose`), with thinking work units (law ticks + presence queries) correlating monotonically (`R² >= 0.9` across ladder) with `(defects + events)`, and **not** growing between `straight100` (`0` interior defects) and `straight800` (`0` interior defects) (Kill tests 8 & 9, `map/detail/learning.md:111-112`).
   - **Anti-cheat falsifier / mutant:** Plant an `if is_uniform_ground()` fast-path shortcut or an A*-for-parkour branch; gate instruments call-graph stages reached by `straight800` vs `hardtrack` and **must fail red** if any solver stage is mutually exclusive.
5. **Current readiness**
   - `blocked on unifying main + wt/flight-ladder + monotone backward sets`: on `main`, `solve.rs:269` refuses any defect as `NeedsJoints`; `wt/flight-ladder` (`c74c4a5c`) solves 4 tracks (`grassjumpeasy`, `flowers`, `stonestaircase`, `fencestraightbridge`) offline via concrete forward DFS `joints/cut.rs` rather than set composition (`families/pieces/points/holds`).

---

### 1.3 Emergent Zones, Switches & Law-Induced Joint Sets (No Hardcoded Grid / No Policy)

1. **Property name & mechanism**
   Space is never partitioned in advance into fixed spatial cells or grid tiles (neither `1m` voxels nor `1/16`-block constants). **Zones** emerge dynamically from how far the held word's `Footprint::reads` holds; **joints** emerge only where a surface/class/event face cuts that word ("switches"); and **sets** at a joint partition the remaining free continuous coordinate strictly at the **law's own arithmetic comparisons** (`1e-7` approach/overlap shells, `1e-5` hit test, minor-collision `acos` test at `movement.rs:818`), forming **families → pieces → points → holds** whose threshold counts stay small (`5/7`, `8/12`) and never explode under resolution refinement. Because the backward sets carry exact `T`-to-go directly on arriving state pieces, **no separate RL/heuristic policy is needed** — acting simply reads the map.
2. **Verbatim quote(s) + exact source `file:line` & session tag**
   - `"What's that 1/16 of a block? Explain it to me because it feels like a made up number for no reason rather than emerging on properly selected constraints and laws and you know how i hate made up numbers."` — `archive/turns/julien-2026-11-10.md:28` (`S67`)
   - `"cells are red hearing i think they are an intermediate scale representation while it should be emerging based on a proper algo like how doors break it"` — `archive/turns/julien-2026-11-10.md:219` (`X9`)
   - `"don't already start with made up fake numbers they should arise based of needs, some regions are more compressible than others."` — `archive/turns/julien-2026-11-10.md:423` (`S132`)
   - `"My goal is to make a representation where you don't need a policy because the map is clear on where to go."` / `"it's just on where i park my bike that it'll matters"` — `archive/turns/julien-2026-11-10.md:249` (`X16`)
   - `"so when you're path finding there's this whole chain and some elements have wiggle room more than others"` — `archive/turns/julien-2026-11-10.md:167` (`S94`)
   - `"Pieces are few. Per joint pattern, the computed pieces equal the native partition by events and contact outcome (free, clamp, hold) at every resolution, and do not grow as it is refined (u2_pins_point_and_hold): gap 5/7 and step 8/12 at yaws 270/275.625, and 7 at yaw 270 across two wall cells"` — `map/detail/learning.md:114` (`S203`, Kill test 11)
3. **Situations / conditions where it arises**
   - Stepping up a curb/stair, crossing a 4-block gap, threading a `0.6`-block-wide gap past two wall cells, or choosing a doorway/switch at `yaw = 270` vs `yaw = 275.625` at arbitrary arrival velocities.
4. **Controlled-environment test harness design**
   - **Minimal arena:** Exact-threshold gap and step joints tested in `u2_pins_point_and_hold`: 1-block step-up and 3/4-block gap at `yaw = 270°` and `yaw = 275.625°`, plus narrow 2-wall pinch (`yaw = 270°`). Also a multi-doorway corridor (`redfakemaze`) where old cell-field A* failed (`0` anchored chains, `map/detail/learning.md:38`).
   - **Measurable pass signal:** (a) Piece count per joint pattern matches the exact analytical count (`5/7` on gap, `8/12` on step, `7` across 2 wall cells) independently of grid refinement (`epsilon -> 0`, piece count strictly invariant); (b) `0` magic constants (no `0.0625`/`1/16` grid step); (c) `T` matches brute-force tick ground truth at every velocity piece edge.
   - **Anti-cheat falsifier / mutant:** Replace analytical contact-threshold piece-splitting with uniform lattice sampling at `Δx = 1/16` or `1/64` (grid refinement grows piece count `4x`, or misses `< 2e-7` wide **hold** intervals (`representation.md:179`), causing exact-tie jump deviations against `law::try_tick`).
5. **Current readiness**
   - `offline-proven` in `jobs/set-rederive/proposal.md` & `jobs/pin-rederive/proposal.md` (`u2_pins_point_and_hold` counts verified per `learning.md:114`), **blocked on integration**: `main` lacks joint cuts entirely; `wt/flight-ladder` (`joints/cut.rs`) reverted to point-state forward DFS (`JointStore::point`) rather than `families/pieces/points/holds`; earlier unit test files were trimmed (`S235`).

---

### 1.4 Kit-Class Equivalence & Zero Unused-Item / Zero Per-Terrain Branching ("A Flower Is Air")

1. **Property name & mechanism**
   Blocks do not exist to the planner by name or ID; a block is **its projected answers to the active word's `Footprint::reads` verbs** (`shape`, `friction`, `fluid`, `inside`, `break_rate`, `sturdy_face`). Under bare-legs locomotion, grass == stone == dirt == planks, and flowers/buttons/pressure-plates (whose collision box is empty) project to **air** — until an edit word asks `replaceable?` or `places_against?` ("unless i need to place a block and it bothers me"). Because **every word checks only the masks in its own `Footprint::reads`**, carrying an unused pickaxe, silk-touch shovel, or 64 blocks **never splits uniform ground into fake defects** and adds **0 ns** overhead to unrelated gaits.
2. **Verbatim quote(s) + exact source `file:line` & session tag**
   - `""Grass and stone are the same to you when they give the same answers." unless i have silk touch pickaxe."` / `""A flower is air." unless i need to place a block and it bothers me."` — `archive/turns/julien-2026-11-10.md:433` (`S134`)
   - `"buttonpressureplates are just a bunch of random buttons and pressure plates on the floor which are there to test if the bot is worried about non issues but it needs to understand that the pressure plates do nothing or just jump over them on the buttons which have no hitbox on the player's movement i believe"` — `archive/turns/julien-2026-11-10.md:288` (`S109`)
   - `"Even just getting a silk touch shovel with efficiency V would change A LOT how T is for travel"` — `archive/turns/julien-2026-11-10.md:681` (`S189`)
   - `"an unused item never raises T"` — `map/detail/scales.md:133` & `map/detail/ladder.md:26` (`S65`)
   - `"A word's corridor check must read only the projected cell answers present in that word's own Footprint::reads. Otherwise carrying a pickaxe splits a mixed stone/dirt/planks runway into fake defects for a sprint-jump gait — violating 'an unused item never raises T' and breaking the ~13 µs floor"` — `jobs/kit-edit-ledger-redteam/report.md:14` (`C4`)
3. **Situations / conditions where it arises**
   - Traversing `flowers` / `butoonpressureplates` tracks; walking across mixed stone/cobblestone/planks terrain while holding an unused Efficiency V pickaxe or blocks; crossing a dirt hill vs stone wall with vs without Efficiency V shovel (where breaking dirt suddenly beats going around).
4. **Controlled-environment test harness design**
   - **Minimal arena:** (A) `100`-block mixed stone/dirt/planks corridor littered randomly with dandelions, oak buttons, and stone pressure plates, run twice: once with `kit = ∅`, once carrying `{Eff5_Pick, 64_Obsidian, Lava_Bucket}`; (B) `2`-block dirt wall vs `40`-block detour, run with bare legs vs `Eff5_Shovel`.
   - **Measurable pass signal:** On (A), `T`, chosen word sequence, defect count (`0`), and median `solve()` latency (`<= 15 µs`, identical within measurement noise) are invariant across `kit = ∅` vs overloaded unused kit (`unused_item_overhead_ns == 0`, `T_with_unused <= T_empty`). On (B), with `Eff5_Shovel`, the bot breaks through rather than detouring when break+restore `T < detour T` — **solely** because `BreakProjection` changes the cell's class answer.
   - **Anti-cheat falsifier / mutant:** (1) Hardcode an `IGNORE_DECORATIONS` block list (`grass`, `dandelion`, `button`) instead of reading empty `VoxelShape` from `models.tsv`/`STATE_STATIC`: fails when a novel 26.2 non-solid plant/button appears or when `place` is attempted into a flower (`replaceable == false` vs `true` in `rules.tsv`); (2) Trigger full terrain re-partitioning or global `Legs+Tools` joint-keying (`kit-edit-ledger-redteam` `C4`/`C5` mutant): causes `> 0` fake defects on the mixed stone/dirt/planks strip while holding a pickaxe.
5. **Current readiness**
   - **Mixed (`offline-proven` for flowers / `blocked` on kit projection & C1/C4/C5):** `wt/flight-ladder` (`c74c4a5c`) passes `flowers` (`48` ticks) offline (`jobs/flight-ladder/report.md:7`), not yet flown; **blocked** on `main` (`WorldSnapshot` stores lossy `ModelId` `C1` erasing `stone != dirt` for silk-touch/break rules, and `solve.rs:7-8` hardcodes `Kit::BareLegs` `C3`).

---

### 1.5 Velocity-Contraction Attractors & Speedrunner Parkour Mechanics (Unscripted)

1. **Property name & mechanism**
   Because the tick law explicitly preserves continuous velocity vector `(vx, vy, vz)` across ticks (`v <- (v + input(yaw)) * friction * drag`) and resolves sub-block bounding-box contacts (`step = 0.6`, 2-block ceilings, wall `acos` friction, axis-order sweep), elite parkour kinetics fall out **automatically from minimizing `T` over exact law rollouts**:
   - **Head-hitter ceilings:** jumping under a 2-block ceiling cuts airtime and pumps horizontal ground-acceleration frames (`7.127 -> 9.734 m/s` on stone, `9.241 -> 19.345 m/s` on ice, `19.903 m/s` on blue ice, `map/detail/engine.md:186-188`).
   - **45° diagonal / strafe speed & optimal yaw:** air input combines forward + strafe vector magnitude slightly above pure W.
   - **Neo jumps & axis-order sweeps:** curving around walls where straight sprint fails even over a safe floor (`S189` Diversity practice tracks).
   - **Active braking:** holding opposite movement keys to kill momentum faster than Mojang's ~32-tick natural decay (`S13`).
   - **Momentum run-up on A (`S27`)** and **gliding/crawling under slabs/trapdoors**.
2. **Verbatim quote(s) + exact source `file:line` & session tag**
   - `""speed takes about 32 ticks to fade." i think it can be faster if you push the opposite key like a brake."` — `archive/turns/julien-2026-11-04.md:57` (`S13`)
   - `"why are we doing a x y for heaving when we could use complex numbers? the player acts as a ball of influence not as a cube"` / `"well it doesn't matter since we can change it instantly unlike momentum"` — `archive/turns/julien-2026-11-10.md:739,747` (`S200/S202`)
   - `"the practice tracks are neo jumps so you really need to match my T or faster since there's a floor underneath"` — `archive/turns/julien-2026-11-10.md:687` (`S189`)
   - `"The map holds his shortcut. The reach relation over hardtrack's faces, built with no knowledge of his run, contains his wall-top route (jobs/hard-track/shortcut.md), and the bot's T is at most his 428."` — `map/detail/learning.md:113` (`S56/S166`, Kill test 10)
   - `Derived by Speeds.java ... sprint-jumping 7.127, or 9.734 under a 2-block ceiling; ... sprint-jumping on ice 9.241, or 19.345 under a ceiling (blue ice: 19.903).` — `map/detail/engine.md:181-188`
3. **Situations / conditions where it arises**
   - Any 2-high ceiling over stone/ice/blue-ice; Diversity neo-jumps over a floor (`S189`); stopping precisely on an edge (`B`) after high-speed ice travel (`S13`); `hardtrack` wall-top route (`T <= 428`).
4. **Controlled-environment test harness design**
   - **Minimal arena:** Four atomic endgame arenas (zero hints in code):
     1. *Braking check:* `30`-block blue-ice strip ending in exact 1-block goal `B` before lava — verify reverse-key deceleration emerges to beat pure release (`0`-input decay).
     2. *Head-hitter choice:* Parallel 50-block stone open lane vs 2-high stone ceiling lane (`+1` tick detour to enter) — verify ceiling lane is chosen (`9.734 > 7.127 m/s`).
     3. *Neo over floor:* 1x2 pillar-around neo obstacle standing on a safe floor (where jumping down-and-back-up takes `+12` ticks, neo saves `>= 6` ticks).
     4. *Wall-top shortcut (`hardtrack`)* with human benchmark `T <= 428`.
   - **Measurable pass signal:** Bot uses brake ticks, enters 2-high ceiling, executes curved neo trajectory, and discovers `hardtrack` wall-top shortcut with `T <= 428` (`<= his T`, Kill test 10) purely from raw law inputs, with **zero** named macro triggers (`"neo"`, `"headhitter"`, `"brake"`).
   - **Anti-cheat falsifier / mutant:** Restrict search state to `(x, y, z)` cell centers dropping incoming velocity `v` (Kill test mutant / `baritone` ablation), OR restrict ceiling clearance test to upright `h=1.8` collision without sneaking/jump-bonk: bot misses the ceiling boost (`T` degrades `~36%`), drops off the neo to the floor, or slides off the blue-ice stop into lava.
5. **Current readiness**
   - `offline-proven` for raw speed constants (`Speeds.java`, `law::face::advance` 0 anomalies across `31` recordings + `62` extremes including ice/blue-ice/trapdoor/slabs, `inbox.md:1418-1443`) and `stonesimple` flown (`85/85` beating his `86`); **blocked** for general neos, `hardtrack` wall-top shortcut, and 2-high ceiling discovery (`main` refuses non-flat joints; `wt/flight-ladder` only solved 4 basic tracks so far and hasn't run `hardtrack` or neo courses).

---

### 1.6 "Sandals, Not Leather" Shock Absorption, Local Slide, Two-Way Repair, Ridges & Thin-Bridge Persistence

1. **Property name & mechanism**
   Across real terrain and unexpected shocks, five tightly coupled properties emerge:
   - **"Making Sandals" (Waitzkin Soft Zone) & Anytime Re-Anchoring:** Instead of freezing or catastrophically compounding mistake #1 -> #2 -> #3 on an unexpected knockback, lag spike, or missed ledge, the planner immediately invokes `solve()` from the live observed state (`world` + `v`), absorbing perturbations locally without cascading into ruin.
   - **Local Slide (`slide_want`, `S94`/`S120`):** Individual joints inside a chain tighten via a local neighbor comparison in the continuous arrival set (`neighbor piece has smaller T-to-go reachable by upstream set`) without whole-trip replay.
   - **Two-Way Local Repair (`S23`):** An edit lowering `T` propagates solely via `(min, +)` updates; an edit raising `T` voids only certificates whose `Footprint::reads` intersect the changed cells.
   - **Saddle-Point Ridges (`S23`/`S132`):** Distinct branching paths stay cleanly separate only where `ΔT >= 1` tick; worthless sub-tick symmetries break deterministically at zero cost.
   - **Thin-Bridge Far Persistence (`S84`):** Far-map coarsening (`opening angle θ`, second-order `θ²/4` Fermat error) retains features by **effect on bracket `ΔT` vs allocator slack**, never geometric thickness — so a 1-block bridge across a 1,000-block chasm is kept at full resolution while a field of flowers blurs away.
2. **Verbatim quote(s) + exact source `file:line` & session tag**
   - `Notice how Waitzkin's "Soft Zone / Making Sandals" redefines what "good micro" even means: taming the world into compliance (trying to force every fight into a clean textbook duel) is paving the road with leather. Making sandals is building a Micro layer so calm and resilient to noise, weird terrain, and surprise shocks that taking a hit or missing a jump never cascades into error #2 and #3.` — `archive/research/pvp-strategy-2026-11-22.txt:368` (grounded in user turn `:334`)
   - `"The bot must not freeze that is important."` — `archive/research/pvp-strategy-2026-11-22.txt:1876` (also verified `now.md:8`: `Playback stops only on events that move the body ...`)
   - `"so when you're path finding there's this whole chain and some elements have wiggle room more than others so can we wiggle the path as we go maybe gradient descent like?"` — `archive/turns/julien-2026-11-10.md:167` (`S94`)
   - `"an extremely simple metric that's local perhaps that tells hey i'd like to get jiggled because here you need no jiggle."` — `archive/turns/julien-2026-11-10.md:345-360` (`S120`)
   - `"one issue is the smallest things diseapearing so i need to cross two mountains but if i look broadly i'll miss the thin bridge"` — `archive/turns/julien-2026-11-10.md:124` (`S84`)
   - `"water can branch on saddle points"` — `archive/turns/julien-2026-11-10.md:423` (`S132`)
   - `"The thin bridge (S84). A one-block bridge planted across a chasm 10³ blocks away is in the far map"` / `"The slide signal. Joints that ask for a slide gain at least a tick; a sample of those that do not gains nothing under search."` — `map/detail/learning.md:116-117` (Kill tests 13 & 14)
3. **Situations / conditions where it arises**
   - Mid-jump knockback/server teleport or last-tick block edit in front of the runner (`S121`); multi-jump parkour chains where loosening/tightening one middle landing saves a tick (`slide_want`); two-mountain `1,000`-block gorge connected by one cobble bridge (`S84`); symmetric left/right pillar detours (`ridges`).
4. **Controlled-environment test harness design**
   - **Minimal arena:**
     - *Shock absorption ("Sandals"):* Straight + gap course where an external impulse (`SetEntityMotion` horizontal knockback `0.4` b/t) or wall edit fires mid-run across `n = 50` random ticks; measure ticks-to-first-valid-control (`<= 1` tick), `freeze_count == 0`, safe-recovery rate (`100%` avoiding lethal cliff drops when a certified abort exists).
     - *Slide signal (Kill test 14):* Chained jump course comparing `slide_want == true` joints against `slide_want == false` sample.
     - *Thin bridge (Kill test 13):* Two `1,000`-block mountain plateaus separated by a 50-block void chasm spanned by one 1-block-wide stone bridge surrounded by 5,000 flowers.
   - **Measurable pass signal:** (1) `0` frozen ticks after mid-flight knockback/edit; (2) `slide_want == true` joints gain `>= 1` tick when tightened while `slide_want == false` sample gains `0`; (3) Far planner detects and routes over the 1-block bridge with `ΔT` within bracket (`<= θ²/4` Fermat error), ignoring flowers (`0` extra cells).
   - **Anti-cheat falsifier / mutant:** (1) Replace persistence-by-`ΔT` far blur with standard mipmap/quadtree spatial occupancy downsampling (`>= 50%` solid in `16×16` cell): **deletes the 1-block bridge** across the 50-block chasm, forcing an immense detour or `Impossible`; (2) Replay full corridor from tick 0 on minor shock instead of re-solving from observed state (`polish::slide` / full reset mutant): exceeds 1-tick re-anchor deadline (`> 50 ms`) and freezes/falls during shock.
5. **Current readiness**
   - Partial `flown` (`resident.rs` re-anchors instantly on `explosion`/`motion`/`position`/`rotation` and corridor block edits on flat ground, `map/now.md:8`); **blocked** on ES-audit `polish::slide` replacement (`no replay from tick 0` + continuous `families/pieces` slide metric) and Step 6 Far Map persistence (`blocked on Steps 1–5`).

---

### 1.7 Carried-Regime Time-Optimality (`R-brach` Elytra Brachistochrone), Normal Food-Guard Horizon & Anytime Dispatch (`R-retarget`)

1. **Property name & mechanism**
   When maximizing distance-per-tick (`min T`) over exact aerodynamic/physiological differential laws with zero trajectory templates:
   - **Elytra Brachistochrone (`R-brach`):** Pitching into an initial dive trades potential energy for high airspeed where lift/drag makes subsequent flattening faster than any straight glide — the dive-then-flatten curve emerges solely from minimizing `T` onMojang's `Math.cos` elytra law.
   - **Food-Guard Horizon (`P4`):** At Normal difficulty, exhaustion accumulation (`0.2` per sprint-jump) cleanly splits an otherwise single flat sprint-jump zone into `(best sprint-jump word until food <= 6, fallback walk word)` at the exact predicted exhaustion tick — and disappears automatically at Peaceful (`food never drops`).
   - **Instant Anytime Dispatch & `R-retarget`:** The body moves the instant word #1's bracket closes (`<= 1` tick latency, `p99 == 0` hold delay, Kill test 15); remaining route refinement runs asynchronously ("free compute"), and changing the goal mid-air preserves full momentum `v` cleanly.
2. **Verbatim quote(s) + exact source `file:line` & session tag**
   - `"under our method does the brachitochrone in elytra emerges naturally? elytra is for later but i suspect that the emergence of this curve under certain conditions mean that we're doing good"` — `archive/turns/julien-2026-11-04.md:75` (`S17`)
   - `"the bot should move instantly unless things are really hard imo"` — `archive/turns/julien-2026-11-10.md:345-360` (`S120`)
   - `"have a specific test where we keep changing the goal again and again."` — `archive/turns/julien-2026-11-10.md:689` (`S189`)
   - `"The brachistochrone, as a falsifier of the method (S17). Elytra with no fireworks, from rest on one pillar to a lower one, in still air. Needs only the elytra law (pitch and yaw controls; its Math.cos exactness class) added as law data, with no new search or geometry code. Pass: T within the bracket, whose UB is the independent time-optimal search over raw controls for this exact law; and the dive-then-flatten curve appears without being asked for. ... Kills: no dive means speed was dropped from the state or something other than T was minimised; a kinked dive means states were merged too coarsely at speed"` — `map/detail/ladder.md:100` (`R-brach`)
   - Kill test 15: `"p99 of (first control tick − earliest dispatch opportunity) is 0 whenever the first word's bracket is closed at the click."` — `map/detail/learning.md:118`
3. **Situations / conditions where it arises**
   - Gliding between pillars separated horizontally and vertically without rockets; long `10⁴`-block flat sprints in Survival Normal vs Peaceful; dynamic chase/target switching every ~30–100 ticks mid-jump (`R-retarget`).
4. **Controlled-environment test harness design**
   - **Minimal arena:**
     - *R-brach:* Two stone pillars separated horizontally (`Δx = 120`, `Δy = -30`), survival mode with elytra equipped and `0` firework rockets.
     - *R-retarget:* `200`-block flat/obstacle arena firing new random target `B_i` mid-sprint/mid-air every `37` ticks (`100` retargets).
     - *Food-guard split:* `2,000`-block straight run at saturation `0`, food `8` (`Normal` vs `Peaceful`).
   - **Measurable pass signal:** (1) `R-brach`: `T <= UB` (independent raw-control numerical optimizer), trajectory exhibits smooth initial negative pitch (`dive`) below straight chord before pitching up (`flatten`) with `0` curve-fitting parameters; (2) `R-retarget`: `p99` latency `<= 1` tick, incoming velocity vector preserved on tick 0 after retarget, `0` mid-air stop-and-re-settle glitches; (3) Food guard: sprint-jump ends at the exact tick food reaches `6` on Normal, continues unbroken on Peaceful.
   - **Anti-cheat falsifier / mutant:** (1) Drop velocity state at joints or optimize horizontal distance only without gravity/lift coupling (`R-brach` kill: dive disappears or kinks sharply); (2) Run settle loop at every click (`es-audit` bug fixed in Step 1) or bake a straight-line pitch heuristic: straight pitch loses `> 10%` `T` against `UB` and retarget drops momentum (`~263`-tick transient penalty).
5. **Current readiness**
   - `P4` food-guard & `wt/port-fold` elytra kernel are `offline-proven` separately (`law` elytra ported in `wt/port-fold`, `plan::P4` gate passed); `R-retarget` partially `flown` on flat ground via `resident.rs` stick (1.8–2.3s external `plan-track` issue bypassed by `resident.rs`); **blocked** on wiring `wt/port-fold` elytra kernel into `plan`/`bot/` (`Step 8`) and adding live `R-retarget` direct `(word, count)` executor (`Step 4`, `now.md:26`).

---

## Domain 2: Creativity, Tools, World Edits & "Move 37"

### 2.1 Law-Read-Backwards Causal Retrosynthesis ("Tools Shape Space", Zero Catalogue)

1. **Property name & mechanism**
   Every carried item or block interaction is treated purely as a **law input** whose execution records `Footprint::writes` (changed cells or body states, e.g., placing water, placing a block face, triggering fall-reset). When a word fails (`Footprint::reads` fails at cell `c` or body guard `g`), backward causal retrosynthesis joins **`item_use::writes ∩ gap::failed_reads`** directly from the law itself — generating bridging, water-climbing, block-clearing, or fall-resetting chains with **zero tool catalogue** and **zero per-tool search code** (Corey's retrosynthesis: "creativity lives where one word writes what another reads"). Possessing a bucket transforms the accessible geometry; placing it is merely movement (`X5`).
2. **Verbatim quote(s) + exact source `file:line` & session tag**
   - `"Think of a water bucket, it is an item that allows you to transform a dead end into something else it changes the geometry that is accessible"` — `archive/turns/julien-2026-11-04.md:29` (`S6`)
   - `"possessing the bucket changes the space itself placing the bucket is just a movement not a thinking action, very different thing."` — `archive/turns/julien-2026-11-04.md:145` (`X5`)
   - `"a bucket is something that negates downward verticality penalty and create upward ones, with the right representation it should be natural that the bot thinks of using a water bucket above him, float, take it back, put it back higher, and repeat."` — `archive/turns/julien-2026-11-04.md:135` (`X4`)
   - `"Look backward without inverting (S91: 'not doing some kind of reverse kinetic')... Tools are tried biggest first; a tool 'breaks' when its mirrored set holds no standable cell."` — `map/detail/learning.md:41` (`S91`)
   - `"join them: A can help B exactly when A writes something B reads (a cell, or the body's own state) and changes its answer. The world's own laws (water spreading) are links in the same chain. Creativity lives where one word writes what another reads."` — `map/detail/scales.md:55-56`
3. **Situations / conditions where it arises**
   - Facing a 20-block sheer wall with only 1 `water_bucket` (water-climb cycle: place above -> swim -> scoop -> repeat); 50-block drop onto stone with `water_bucket` or `powder_snow_bucket` or `hay_block`; locked corridor blocked by removable dirt/cobwebs/trapdoor; water flowing across a gap as an emerging bridge/elevator.
4. **Controlled-environment test harness design**
   - **Minimal arena:** `16`-block-tall vertical bedrock pillar (`goal B` on top, impossible for bare-legs `1.25`-block jump) with **only** 1 `water_bucket` (no blocks, no ladders) — plus a blind "invented modded physics item" test: an artificial item `ItemX` defined solely in law data that writes `SLIME` state for 10 ticks at clicked face, with **zero** string tag or special solver hint.
   - **Measurable pass signal:** Within `<= 10³` verifications (`~1 ms`), causal backward join matches `failed_reads` (`no climbable/standable face above`) against `water_bucket::writes` (`Water(level=0)` -> `swim_up` transition) AND solves the novel `ItemX` vertical shaft identically with zero code edits. Water bucket is retrieved before stepping onto `B`.
   - **Anti-cheat falsifier / mutant:** Hardcode a macro or `match item { WaterBucket => climb_wall(), ... }` branch (violating `"A hand-written one is a catalogue of tricks (ES 4)"`, `scales.md:52`): **immediately fails** on `ItemX` (never matches `ItemX::writes ∩ swim/bounce_reads`) and triggers Kill test / `scales.md:63` (`"a tool needs search code"`).
5. **Current readiness**
   - `blocked on C1 + law/codec verbs + backward join`: `engine/world/src/snapshot.rs:107` uses lossy `ModelId`; `engine/law/src/codec.rs:110-169` only has `UnportedVerb::{Break, Place}` stubs; `Footprint::writes` and backward `writes ∩ failed_reads` join are `Doc-only` (`inventory-map/report.md`, `scales.md:52-56`, `kit-edit-ledger-redteam`).

---

### 2.2 `R-lego`: Gap Crossing, Cross-Gap Transfer & Momentum-Assisted Mid-Air Block Placing

1. **Property name & mechanism**
   Three cumulative creative behaviors under one falsifier (`R-lego`):
   1. **Arises:** With carried blocks facing a gap beyond max bare-legs jump (`> 4` blocks), the planner discovers placing against the takeoff face (`eye <= 5.5` blocks, `body_box` legal once airborne) in `<= 10³` law verifications and re-mines/recovers the block if required.
   2. **Stays (Pareto Effect Archive):** Saved as a generator keyed by `(start class, effect)` in a Pareto archive of `(T, width, resources, obligations)`, the compound transfers immediately (`O(1)` lookup, beating Minton's utility collapse) to a **second gap of different width/height**.
   3. **Compounds:** Because words compose in 4D velocity space, the system chains high horizontal run-up -> sprint-jump over void -> mid-air downward placement under feet (`body_box` clears feet cell while rising/falling) -> continued jump **without** ever explicitly scripting "run-jump-place" (`S6`).
2. **Verbatim quote(s) + exact source `file:line` & session tag**
   - `"Running and then jumping into the void to then place a block under your feet is not simple in how we should think about it you have to have a certain understanding but with the right representation is shall be trivial and obvious."` — `archive/turns/julien-2026-11-04.md:29` (`S6`)
   - `"My litmus test for the right implementation of my overall idea is that it would be fast and with emergent properties always breaking our tests because it thinks outside the box because it is able to combine functions like legos, alien like thinking and rediscovering things elite performers use."` — `archive/turns/julien-2026-11-04.md:191` (`S32`)
   - `"The falsifier, R-lego. One carried input (blocks), one gap past the sprint-jump limit. Pass: a crossing appears with no code naming it, in ≤ 10³ verifications, and the block comes back. It is reused on a second gap of another shape (stays). It composes with momentum: run, jump, place under the feet (S6) (compounds). The server's rule is pinned (GLM place-reach-legality, reproduced): the eye must be within 5.5 blocks of the nearest point of the clicked cell; the placer's own body blocks its feet cell, and the cell below is legal once airborne."` — `map/detail/scales.md:61-62` (`R-lego`)
3. **Situations / conditions where it arises**
   - Any void/lava gap of `5..12` blocks with `1+` placeable blocks and pickaxe; elevated 1-block-higher 5-block gap (second shape); sprint-bridging over deep gaps where placing mid-air extends jump reach (`R-lego` compounds).
4. **Controlled-environment test harness design**
   - **Minimal arena (exact `R-lego` ladder):** Superflat void gap #1 (`width = 5` blocks, stone pillars, `kit = {2 cobblestone, Eff5 pickaxe}`), followed immediately in the same persistent store by gap #2 (`width = 6` blocks, `+1` higher landing pillar), audited live by Mojang 26.2 server (`place-reach-legality` + `ItemEntity` drop check).
   - **Measurable pass signal:**
     1. Gap #1 solved with `verifications <= 10³`, zero "bridge" code string in binary, all placed blocks back in inventory upon touching `B`.
     2. Gap #2 solved in significantly fewer verifications (`<= 1/10` of cold #1 search) using archived generator from #1; branch retrieval stays `O(1)` when `100` unrelated effect entries sit in archive.
     3. Momentum-clutch variant (only `1` block available across a `7`-block gap requiring sprint-jump + mid-air block clutch at tick `k` on gap face) emerges automatically.
   - **Anti-cheat falsifier / mutant:** (1) Hardcode a `"gap > 4 -> bridge_step()"` branch or store unparameterized absolute-coordinate recordings (`no generalization` mutant -> **fails** gap #2 shape transfer); (2) Permit placement inside own occupied feet cell (`place-reach-legality` violation -> **fails** live server audit).
5. **Current readiness**
   - `blocked on Step 5 prerequisites`: `place-reach-legality` rule pinned (`GLM` verified in `scales.md:62`); blocked on replacing `ModelId` (`C1`), implementing `Place`/`Break` in `law` + `WorldMut` (`WorldDelta`), Pareto effect archive (`inventory-map`), and backward `writes ∩ failed_reads`.

---

### 2.3 Speedrunner Creative Maneuvers (`S122` Rediscovery Set & `research` Tech)

1. **Property name & mechanism**
   None of these elite techniques are named or scripted; each falls out solely from composing exact 26.2 mechanics inside `(min, +)` / Pareto archives when terrain geometry + kit pressures create an otherwise unreachable or slow gap:
   - **Speed bridging / God-bridging / Eagle (`S108`/`S122`):** Un-sneaking between backward edge placements at exact sneak-off-edge timing windows (`sneak speed 0.3` vs sprint).
   - **Water / Powder-Snow Bucket Clutch (including Nether `water_evaporates`):** Placing bucket 1 tick before fatal ground hit, scooping on next tick (`Powder snow ... fall damage reset, also in the Nether`, `research-2026-11-05.md:59`).
   - **Boat Clutch over Lava & Lava Lake Burning-Boat Recovery Cycle (`S122`):** Placing boat -> mounting (resets fall) -> surviving/riding while boat takes lava/burn ticks -> exiting/breaking -> **recapturing `ItemEntity` boat mid-air before fire consumes it** -> repeating across a vast lava ocean.
   - **Momentum-Carried Ender Pearls (`S122`):** Sprinting/jumping/gliding before pearl throw so `thrower.velocity` adds (`51.5 -> ~90` blocks range, `research-2026-11-05.md:99`).
   - **Mid-Fall Boat Exit & Re-Enter (`S122`):** Dismounting and re-entering a falling boat to convert vertical momentum / reset horizontal trajectory over a huge void fall.
   - **Wind Charge / Mace / Spear-Lunge / Dripleaf Crawling (`research-2026-11-05.md:54,95-97`):** Using wind-charge upward burst + fall-reset, mace smash onto an armor-stand/mob target at bottom of a 300-block cliff, spear lunge air-dashes, or big dripleaf tilt to enter 1-high crawl openings.
2. **Verbatim quote(s) + exact source `file:line` & session tag**
   - `"i've studied things from speedrunners and speed bridging, boat clutch over lava, using momentum to make pearls go further or even having the boat in lava being burned and recapturing the item before it burns repeating the cycle to bridge long distances on lava or even more crazy getting out of the boat and in again to change the momentum and having more horizontal distance while falling should emerge in the right conditions."` — `archive/turns/julien-2026-11-10.md:368` (`S122`)
   - `"My litmus test ... always breaking our tests because it thinks outside the box because it is able to combine functions like legos, alien like thinking and rediscovering things elite performers use."` — `archive/turns/julien-2026-11-04.md:191` (`S32`)
   - Exact `S122` Rediscovery matrix: `map/detail/ladder.md:109-119` (`Speed bridging`, `Boat clutch over lava`, `Pearls carried further by momentum`, `The burning-boat lava bridge`, `Leaving and re-entering a boat mid-fall`).
3. **Situations / conditions where it arises**
   - `40+` block void chasm with 1 pearl (forces run-up momentum throw); `50`-block Nether cliff drop (`water_evaporates == true`, forces `powder_snow_bucket` or `hay` or `wind_charge`); `30`-block lava ocean with **1 boat and 0 blocks** (forces boat -> drop -> recapture cycle); `80`-block diagonal drop with 1 boat (forces dismount/remount horizontal steering).
4. **Controlled-environment test harness design**
   - **Minimal arena:** Five decoupled endgame arenas matching `map/detail/ladder.md:113-119` + Nether `powder_snow` cliff (`R6`), each holding **only** the minimal item required (e.g., 1 boat on lava lake, 1 ender pearl at 80-block canyon, 1 powder-snow bucket in Nether).
   - **Measurable pass signal:** On each arena, bot crosses `A -> B` on Mojang 26.2 server with `T < ∞`, `0` death/ruin, `0` failed server teleports, and **automatically logs the discovered compound sequence to the Rediscovery / Move-37 log**, with **zero** technique names or bespoke heuristics in source (`Rediscovery` score grows monotonically).
   - **Anti-cheat falsifier / mutant:** Disable thrower-momentum addition on pearls (`pearl.rs` mutant) or force `water_evaporates = false` / disable boat fire-drop capture: the 80-block pearl test or Nether cliff or 1-boat lava lake must cleanly report `forbidden`/`searched` rather than falsely claiming success. Conversely, check binary/AST has zero special-case boat/pearl/clutch templates.
5. **Current readiness**
   - Physics kernels (`boat`, `pearl`, `water_evaporates`, `item_entity` on `wt/port-fold`) are largely ported (`99.65%` block states, `9,117` jar rows, `now.md:10`), **blocked** on wiring `wt/port-fold` into `plan`/`bot/`, adding action verbs (`use`, `mount`, `dismount`, `throw`), Pareto causal retrosynthesis, and RNG-pinned `ItemEntity` physics. (Note: `wind_charge` / `spear lunge` / `mace` items exist in `use-effects.tsv` / GLM dossiers, not yet active solver words.)

---

### 2.4 Double-Entry Obligation Ledger, Reversible Edits, Slow-Media Refraction & Deterministic `ItemEntity` Void Catch

1. **Property name & mechanism**
   Every world edit is a **toll with a debt** (`T_edit + T_undo`) recorded double-entry in a mission obligation ledger before execution (`every edit owes its counter-edit`). Two emergent behaviors follow strictly from physics:
   - **Slow-Media Fermat Refraction (`X4`, `S185`):** With silk-touch/pickaxe restoration available, a wall ceases to be infinite cost; it behaves as an optical medium of thickness `d` with effective speed determined by `(break_T + walk_T + restore_T)`. Shortest-time rays automatically bend at wall faces according to Fermat's principle — cutting through thin soft walls (dirt/sand/glass) while refracting around obsidian/bedrock.
   - **Physics-Driven Void Catch vs. Ruin Barrier (`S13`, `C7`):** Instead of hardcoding `"breaking a block over void = irreversible ruin"`, the engine simulates exact deterministic `ItemEntity` physics (`vy0 = +0.2`, gravity `-0.04`, drag `0.98`, `10`-tick pickup delay -> only `-0.24` blocks vertical drop at tick 10 inside `2.6 × 2.8 × 2.6` pickup box). The planner discovers on its own when an adjacent bot pose (or teammate) catches the drop mid-air vs when unobserved RNG/unreachable geometry triggers a hard `Irreversible` ruin barrier (same for unmodeled block-entity NBT like decorated pots or spawners, `C1`/`C7`).
2. **Verbatim quote(s) + exact source `file:line` & session tag**
   - `"the world must be the same after the bot reaches point B so water can destroy things but it has to be repaired if so, a trapdoor can be used but it has to be put back into its state once done."` — `archive/turns/julien-2026-11-04.md:187` (`S32`)
   - `""A block placed under your feet over the void falls into the void when broken back, so it can’t be recovered." yes and that's hard to deal with as it is random too so it has a price, the one to position yourself to get it back or to place it where you'll get it back or you can just leave it"` / `"even moving a trapdoor to crawl you have to put it back because it might be used decoratively."` — `archive/turns/julien-2026-11-04.md:57` (`S13`)
   - `"a wall is a two second delay not a wall"` — `archive/turns/julien-2026-11-04.md:135` (`X4`)
   - `"Obligations are written ahead of each edit, as double entry: every edit owes its counter-edit, and a mission ends when the books balance. On restart a recovery mission restores what is owed."` — `map/detail/scales.md:139`
   - `"With vy_0 = +0.2, gravity -0.04, drag 0.98, and 10-tick pickup delay, an item drops only 0.24 blocks below block center at tick 10 while the player's (1.0, 0.5, 1.0) expanded pickup box is 2.6 × 2.8 × 2.6. Therefore a bot (or a teammate in the swarm) adjacent to a void bridge block can catch the drop mid-air. Never hardcode 'void = irreversible'; let the law's own ItemEntity trajectory + RNG input (or bounded ±0.1 b/t scatter cone when RNG is unobserved) decide catchability vs ruin."` — `jobs/kit-edit-ledger-redteam/report.md:17` (`C7`)
3. **Situations / conditions where it arises**
   - Dirt vs obsidian corridor wall bypasses (`Eff5_Shovel` vs bare hands); flipping a decorative trapdoor to crawl through a 1-high tunnel (`S13`); water bucket spilled across redstone dust or carpet (`R6` requires replacing swept dust/carpet or avoiding the spill); breaking temporary stone blocks back over a void bridge (`C7`); crash/kill recovery mid-mission (`scales.md:139`).
4. **Controlled-environment test harness design**
   - **Minimal arena:**
     - *Refraction check:* `30`-block detour corridor partitioned by `1`-block vs `6`-block dirt wall (`kit = Eff5_Shovel`), plus obsidian wall (`requires_correct_tool`).
     - *Ledger + Void-Catch check:* `5`-block void bridge with **exact 1 cobblestone** (`kit = {1 cobble, pickaxe}`) where crossing requires placing at pos `p`, crossing, breaking while standing `<= 1.1` blocks away on far ledge to catch `ItemEntity` inside tick 10–15 (`JavaRandom` fixed seed vs worst-case `±0.1` scatter cone), plus a crash-restart test (`SIGKILL` bot immediately after placing block `p` or flipping decorative trapdoor).
   - **Measurable pass signal:** Bot breaches thin dirt wall (`detour T > edit+restore T`), skips thick dirt wall (`edit+restore T > detour T`), refuses obsidian if unbreakable/restorable cleanly, **catches the void-bridge drop cleanly** when catchable across worst-case scatter (`0` blocks lost, `books_balance == true`, `world_hash_after == world_hash_before`), refuses if drop cone leaves pickup box (`Irreversible`), and on crash-restart auto-generates recovery mission restoring the exact missing obligation (`StateId` identical, including trapdoor `open/half` bits).
   - **Anti-cheat falsifier / mutant:** (1) Hardcode `"water pickup undoes water placement"` (ignores torches/redstone/crops washed away by spreading water: **fails** exact post-world `StateId` equality check!); (2) Hardcode `"void below block => cannot break"`: **fails** the 1-cobble void-bridge catch test even though physics proves pickup succeeds in 100% of `±0.1` scatter trajectories.
5. **Current readiness**
   - `blocked on Pillar 1-3 ledger implementation`: `WorldSnapshot` (`engine/world/src/snapshot.rs:107`) loses block identity (`C1`), `wt/port-fold` `SimWorld` has sticky flags/double storage (`C2`, `sim.rs:95-98,561-562`), and the double-entry obligation ledger + crash-recovery runner is `Doc-only` (`kit-edit-ledger-redteam/report.md`).

---

### 2.5 Incomplete Transformation Valuation, Shadow-Price Token Retention ("Keep It for There") & Resource Crafting Optionality

1. **Property name & mechanism**
   Because forward search carries exact state (inventory, durability, remaining materials) while backward sets attach **marginal T-savings shadow prices (`λ`)** to holding each item downstream ("it's an information problem", `S96`), the planner avoids greedy premature tool spending AND automatically invents **multi-step crafting/staging transformations** (e.g., chopping/carrying logs -> crafting a boat upon reaching a frozen river, `S96`; hoarding planks to swap into boats to survive cliffs or flood enemy inventories, `pvp-strategy:1405`) whose intermediate steps preserve value even before reaching `B` (`S44`).
2. **Verbatim quote(s) + exact source `file:line` & session tag**
   - `"The representation must preserve the value of incomplete transformations."` — `archive/turns/julien-2026-11-09.md:19` (`S44`)
   - `"a tool in your hand could be useful later so the backward is good here to tell hey keep it for there"` / `"if you're on a frozen river and you have logs the bot need to think to make a boat."` — `archive/turns/julien-2026-11-10.md:171,175` (`S95–S96`)
   - `"if you carry let's say planks so that you can make boats? does the cliff idea emerge? asymmetry?"` — `archive/research/pvp-strategy-2026-11-22.txt:1405`
   - `"So the backward pass carries, beside its sets, a price in T for each part of the state: a block in hand is worth the T it saves at the place that needs it ('keep it for there'). A price orders and bounds; the forward replay decides. ... the sign that one exists is the gap between the sentence's T and the bound for everything the bot could hold, crafting included, each at its price in T."` — `map/detail/learning.md:43` (`S96`)
3. **Situations / conditions where it arises**
   - Carrying 1 precious block/pearl early near a small optional shortcut (`saves 5 ticks`) vs saving it for an unavoidable late `30`-tick cliff (`shadow price = 30 > 5`); starting with `oak_log`s in inventory before a `500`-block frozen river (`crafting 1 boat saves > 200 ticks`); holding stacks of `planks` (compressed 5-plank-per-boat optionality) approaching a cliff drop or PvP clash.
4. **Controlled-environment test harness design**
   - **Minimal arena:**
     - *Keep-it-for-there:* Corridor with Gate 1 (optional 1-block gap shortcut saving `8` ticks if block is consumed/left behind, or breakable wait) followed by Gate 2 (`6`-block canyon requiring `1` placed block where detour costs `80` ticks); starting kit = `{1 stone_block}` (no pickaxe).
     - *Craft-on-need:* Starting kit = `{5 oak_logs}`, terrain = `20` blocks grass -> `400`-block frozen river -> `goal B`.
   - **Measurable pass signal:** (1) On Gate 1->2, backward pass assigns shadow price `80 > 8` to the stone block, causing the bot to take the 8-tick bypass at Gate 1 and cross Gate 2 at minimum total `T` (preserving the block for Gate 2 without forward exhaustive combinatorial search); (2) On frozen river, `KitCone` lower bound includes craftable reachable gaits (`logs -> planks -> boat`), gap between bare-legs `T` and craft-bound triggers retrosynthesis crafting the boat (`S96`).
   - **Anti-cheat falsifier / mutant:** Zero out backward item shadow prices (`λ = 0`, greedy myopic horizon) or compute `KitCone` solely over currently-held inventory items ignoring crafting transforms: bot **greedily burns its sole block at Gate 1** and gets stuck taking the 80-tick detour at Gate 2, or walks the 400-block frozen river on foot (`~2.5x` slower `T`).
5. **Current readiness**
   - `blocked on backward price fields + crafting table integration`: `Plan` currently lacks carried item shadow prices `λ`, crafting transitions (`logs -> planks -> boat`), and non-trivial `KitCone` lower bounds (`Kit::BareLegs` only, `solve.rs:7-8`, `C3`).

---

### 2.6 Macro Wormholes, Stasis Networks & Death-as-Teleportation

1. **Property name & mechanism**
   Any transition violating foot-speed `c` enters the same `(min, +)` / `(max, +)` representation uniformly as a **wormhole** edge with `(entry mouth, latency/toll, exit manifold, obligations)`:
   - **Nether Portal 8:1 Manifold + Swarm Pearl Forwarding (`S44`/`S132`):** A bot builds/enters a portal (`80`-tick wait + obsidian kit), compresses `8:1` Overworld travel, clears the Nether chunk/path, and either constructs return portals or coordinates pearl throws bypassing slow chunk load penalties for arriving teammates (`S44`).
   - **Ender-Pearl Stasis Chamber Division Summoning (`S190`):** A pearl suspended in an upward bubble column (`water + soul_sand`) acts as an armed spatial bookmark; when a scout, redstone clock, or trapdoor flips, an entire army teleports instantaneously to the front without marching.
   - **Death-as-Teleport (`S189`):** Death is simply a zero-distance wormhole back to spawn/bed whose toll is respawn ticks + dropped items (zero item loss if stored in ender chest / keepInventory or picked up by nearby partner), becoming `T = ∞` **only** where game rules or PvP stakes install an absorbing ruin barrier.
2. **Verbatim quote(s) + exact source `file:line` & session tag**
   - `"a bot could make a nether portal go into it clear the path while the rest of the bots just come into range throwing ender pearls that load instantly bypassing the loading time wihtout that method to go faster towards a destination, those things shall be obvious to them thanks to the right representation"` — `archive/turns/julien-2026-11-09.md:23` (`S44`)
   - `"oh also a nether portal is a manifold btw which requires to build a certain structure to open as if it is always there but you need to do a weird path to access it."` / `"pearls are wormholes."` — `archive/turns/julien-2026-11-10.md:423` (`S132`)
   - `"death is a tool until you impose some constraints for game mechanics like a fight you don't want to die so T=∞ there for death. Death is a form of teleportation"` — `archive/turns/julien-2026-11-10.md:683` (`S189`)
   - `"there could be a network of them [ender-pearl stasis chambers] to summon bots on demand like a whole army or division at once as needed from either certain tech or a single bot or redstone clocks and so on, very powerful and cheap tool which is an alien like behavior that i love"` — `archive/turns/julien-2026-11-10.md:695` (`S190`)
3. **Situations / conditions where it arises**
   - `10⁴`-block round-trip mining/delivery mission (`Nether 8:1` beats walking past break-even threshold, or fast return home via empty-inventory `/kill`/lava suicide when bed is at home and `T_walk_back > T_death_toll`); staged siege/rescue where 10 bots throw pearls into home/forward stasis chambers before deploying locally (`(max, +)` summon).
4. **Controlled-environment test harness design**
   - **Minimal arena:** (A) `2,000`-block asymmetric delivery (`Pick up empty-slot flag at +2000`, `Deliver at Spawn(0)` with an Ender Chest at `+2000` and barrier flag `allow_death_toll = true` vs `false`); (B) Pre-built stasis chamber (`soul_sand` + water column + trapdoor) at forward base `(500, 64, 0)` beside Goal B, triggered remotely by 1 scout bot / redstone wire when 5 bots wait at `(0, 64, 0)`.
   - **Measurable pass signal:** (A) With `allow_death_toll = true` and Ender Chest available, bot stashes flag/items in Ender Chest and uses fall/lava suicide edge (`T_return << 2000/c`), whereas with `allow_death_toll = false` (PvP/hardcore barrier) death is strictly pruned (`0` self-harm attempts); (B) With pre-armed stasis chambers, scout triggers trapdoor at `(max, +)` optimal tick so all 5 bots arrive at `(500, 64, 0)` simultaneously for minimum mission `T`.
   - **Anti-cheat falsifier / mutant:** Hardcode a whitelist of legal teleports or omit `PlayerDeath -> RespawnPos` and `TrapdoorFlip -> ProjectileCollision(Pearl)` from the causal `writes ∩ reads` graph: the planner marches 2,000 blocks back on foot in both modes and never discovers either suicide-banking or scout-triggered stasis summons.
5. **Current readiness**
   - `PortalForcer`, `pearl`, and `bubble columns` exist as `law` ports in `wt/port-fold` (`now.md:10`), **blocked** on Step 8 (`Wormholes`, `plan.md:39,52,57`) and full server-coupled multi-dimension/inventory-safe ledger integration.

---

### 2.7 Redstone as Compounding Intelligence Litmus & Periodic `WorldGait` Folding

1. **Property name & mechanism**
   Redstone circuits form deep, indirect spatial-temporal causal graphs (`button -> dust -> repeater -> observer -> sticky_piston -> slime/door`). Under the same law-driven causal engine:
   - **Intelligence Litmus (`S190`):** Because `writes ∩ reads` links cell transitions recursively and Pareto subassemblies compress multi-step latches/gates, multi-stage redstone puzzles (hidden doors, repeater-locked timing bridges, piston doors, slime launchers) emerge from composition rather than scripted templates.
   - **Periodic `WorldGait` Folding (`C8`):** Once running, undisturbed redstone clocks, pistons, and flying machines collapse into a periodic `(cycle_program, phase, period P, Δpos)` gait stored off the hot tick wheel (`0` `calendar::Wheel` wake-ups while undisturbed, `O(1)` lookup `W(t) = phase((t - t0) % P)`), waking and re-materializing exact block states **only** when a bot or arrow footprint intersects the circuit (`SpaceTimeIndex`).
2. **Verbatim quote(s) + exact source `file:line` & session tag**
   - `"i'm curious on what will emerge out of redstone, it is convoluted and indirect but if words are properly able to compound then we should see more and more complex things emerge? that's the perfect litmus test for proper intelligence of our architecture rather than just mere efficiency"` — `archive/turns/julien-2026-11-10.md:697` (`S190`)
   - `"looping redstone/piston/flying machines must fold into periodic world gaits (the biggest repeating cycle (cycle_program, phase, period P, Δpos)) rather than ticking per cycle forever — answering W(t) in O(1) and waking via the same space-time footprint index."` — `jobs/kit-edit-ledger-redteam/report.md:18` (`C8`)
   - `"Check 2: Single block edit shock across N = 10^3 -> 10^9 + arrow crossing + undisturbed WorldGait ... PASS ... 0 Wheel wake-ups over 100k ticks"` — `jobs/es-1b-probe/report.md:10`
3. **Situations / conditions where it arises**
   - Hidden iron door behind 3 chained logic gates (lever + pressure plate + calibrated pulse extender); climbing across a reciprocating slime-block piston elevator / flying machine bridge; large server worlds containing hundreds of running ambient farm clocks while bots travel nearby.
4. **Controlled-environment test harness design**
   - **Minimal arena:**
     - *Redstone puzzle:* 3-stage puzzle box (`wall_lever -> repeater_delay -> piston_retract -> bridge_exposed`) with **1** movable `redstone_torch` or lever toggle needed to reach `goal B`, with zero mentions of redstone in planner source.
     - *WorldGait scaling:* A 4-tick clock (`clock_4`) left undisturbed for `100,000` ticks beside a stream of traveling bots, followed at `t = t*` by placing 1 block intersecting its wire (`SpaceTimeIndex` wake test).
   - **Measurable pass signal:** (1) Puzzle solved via backward `(gate_open_reads ∩ piston_writes ∩ dust_writes ∩ lever_writes)` causal join (`0` brute-force random lever flipping across unreachable combinations); (2) Undisturbed clock incurs `0` `Wheel` wake-ups across `100,000` ticks and answers `W(t)` bit-identically at arbitrary `t`, then wakes **exactly 1** `WorldGait` handle upon wire interruption (`es-1b-probe` Check 2).
   - **Anti-cheat falsifier / mutant:** Poll redstone blocks on every tick (`no WorldGait` folding -> blows `0` wake-up invariant to `25,000` wake-ups per `100k` ticks), OR sever `world_mut` / wire-graph footprint `writes` tracking (`causal chain breaks at repeater -> piston`): solver fails the multi-hop iron door puzzle.
5. **Current readiness**
   - **Split:** `WorldGait` (`0` wake-ups undisturbed + `1` wake-up on `SpaceTimeIndex` hit) is `offline-proven` on ES `Wheel` (`jobs/es-1b-probe/report.md:10`); 26.2 redstone/piston `law` rules are `offline-proven` on `wt/port-fold`; compounding redstone puzzle planning (`S190`) is `blocked on Step 5/8` (`W(t)`, `WorldMut` in `plan`, backward causal composition, and Pareto library).

---
[END OF PART 1/3]

## Domain 3: Learning, Forgetting, Play & Statistical Invariants

### 3.1 Emergent Zipf Usage & Heaps Vocabulary-Growth Bending

1. **Property name & mechanism**
   Neither Zipf's rank-frequency law on word/joint usage nor Heaps' sublinear curve on new pattern invention is targeted or engineered. Both emerge naturally from crystal geometry + one demand-driven learning queue (`one record` per unique pattern key): a small core of gaits/transitions carries heavy-tailed traffic along natural terrain, while new distinct pattern keys per thousand encountered joints bend smoothly downward (`sublinear Heaps exponent < 1`) as reusable generative words cover more local terrain combinations.
2. **Verbatim quote(s) + exact source `file:line` & session tag**
   - `"we need to keep in mind the dynamics of inventing new words they are log shaped and also will be zipf's law coded. they will be amortized. The issue to look for is whether words generalize or not that would be a failure mode creating a lot of data for no good reason."` — `archive/turns/julien-2026-11-10.md:96` (`S81`)
   - `"also with need not to design the zipf's law and the other one they are emergent on power laws."` — `archive/turns/julien-2026-11-10.md:147` (`S89`)
   - `"Zipf and Heaps emerge (S89). They are read, never targeted."` — `map/detail/learning.md:35`
   - Kill test 12: `"Patterns bend (S81). New joint patterns per thousand joints fall along a long walk."` — `map/detail/learning.md:115`
3. **Situations / conditions where it arises**
   - Across long natural-world traversal (`10⁴` blocks over varied biomes) or across a sequence of tracks (`21 tracks -> random seeds`), accumulating distinct joint keys in the shared store.
4. **Controlled-environment test harness design**
   - **Minimal arena:** Default seeded natural world `10⁴`-block expedition crossing varied plains, forests, hills, and ravines (`R-far` sampler), recording pattern creation timestamps and word execution counts (`walked`, `walked_ticks`) across `10` checkpoints (`1k, 2k, ..., 10k`).
   - **Measurable pass signal:** (1) New distinct pattern keys per `1,000` joints crossed falls strictly across the run (`Heaps slope < 1`, Kill test 12); (2) Rank-ordered execution frequency follows a heavy-tailed Zipf distribution without explicit frequency eviction or Zipf weighting; (3) Held-out biome `B` solved predominantly via pattern keys learned in biome `A` (Kill test 7: `words learned on parkour, tried on terrain`, `learning.md:107`).
   - **Anti-cheat falsifier / mutant:** (1) Key joint records by absolute `(x, y, z)` coordinates or unparameterized world state instead of canonical relative patterns + `Footprint::reads` (`no generalization` mutant, `S81`): Heaps discovery rate stays flat (`~1000` new patterns per `1000` joints, linear growth -> Kill test 12 failure); (2) Artificially prune/promote words to hit a hardcoded Zipf exponent.
5. **Current readiness**
   - `partial schema only`: `engine/words/src/table.rs:27-28,66-94` tracks `walked`, `walked_ticks`, and surprise/regret counters on `main`, **unused by `solve.rs`**; full joint-pattern learning store & `Kill test 12` measurement are `blocked on Steps 2–4`.

---

### 3.2 Nerode / Byte-Equal Pattern Merging & Unsupervised Distinction Erasure

1. **Property name & mechanism**
   Equivalent environments collapse automatically from physics downward without human classification rules. Whenever two distinct pattern keys under a kit produce **byte-identical `families / pieces / points / holds` transition tables** (exact Nerode equivalence — equal futures under the law), hashing the serialized piece table merges them into a single equivalence class (`ES 32`). Thus arbitrary block distinctions (`grass vs stone`, `dandelion vs fern vs button`, `torch vs air`) erase themselves automatically in stored memory.
2. **Verbatim quote(s) + exact source `file:line` & session tag**
   - `""Grass and stone are the same to you when they give the same answers." unless i have silk touch pickaxe."` / `""A flower is air." unless i need to place a block and it bothers me."` — `archive/turns/julien-2026-11-10.md:433` (`S134`)
   - `"Two pattern keys whose piece tables are byte-equal under a kit merge into one class, found by hashing the tables (ES 32): learning also removes distinctions ('A flower is air.', S134)."` — `map/detail/learning.md:62`
   - Kill test 21: `"Merging pays. Stored bytes per distinct behaviour fall along a long walk."` — `map/detail/learning.md:129`
3. **Situations / conditions where it arises**
   - Long mixed-terrain traversal crossing many cosmetic variants (granite/diorite/andesite/oak_planks/birch_planks, poppies/dandelions/tall_grass/buttons) where different `BlockId`s map to identical transition tables under bare legs or pickaxe.
4. **Controlled-environment test harness design**
   - **Minimal arena:** `500`-joint mixed parkour track shuffling `20` distinct solid `BlockId`s + `10` non-solid plant/button `BlockId`s sharing the same physics shapes under bare legs, re-run a second time with a `wooden_pickaxe` + `Place` enabled.
   - **Measurable pass signal:** Under bare legs, all `20` solids merge into `1` behavior class (`stored bytes per distinct behavior falls`, Kill test 21) and `10` plants merge with `air` with `0` bit-level differences on replay; under `Place`, `flower` vs `air` splits cleanly only where placement queries `replaceable` vs solid `clicked_face` (`0` false merges).
   - **Anti-cheat falsifier / mutant:** Disable table-hash merging (always store per-raw-pattern-key) OR force lossy semantic merging (e.g., merging on approximate spatial bounding box rather than byte-exact piece tables, `Go-Explore` anti-pattern noted in `learning.md:136`): the former fails Kill test 21 compression, while the latter merges stairs with slabs and triggers server/law deviations.
5. **Current readiness**
   - `blocked on joint piece store implementation`: `model_digest` (`world/tests/pattern.rs` on `main`, cited in `footprints.md:120-121`) exists for block models, while full Nerode byte-equal pattern table merging (`learning.md:62`) is `Doc-only` (awaits Step 2 `families/pieces/points/holds` implementation).

---

### 3.3 Stable Subassemblies Compounding (Simon Watchmakers) & "Making Smaller Circles" (Envelope Derivation)

1. **Property name & mechanism**
   Two complementary compression emergences keep search depth shallow and memory bounded as capabilities scale:
   - **Stable Subassemblies + Pareto Effect Archive (`scales.md:57-58`):** Keeping intermediate `depth-3` compounds keyed on Pareto `(cost, width, resources, obligations)` fronts collapses a `depth-12` search (`5^12 ≈ 2×10^8` primitive trials) into `~10^3` trials over subassemblies — without Minton's utility slowdown (`O(1)` effect lookup).
   - **"Making Smaller Circles" (Waitzkin / Closed-Form Family Derivation, `S90`, `learning.md:63`):** Once a family of joints accumulates high `T × traffic`, learning tests whether its empirical breakpoints equal the closed-form ballistic/gait envelope (`arc-certificate`, `gait-modulo`). When they match exactly, **the entire lookup pattern family collapses into 1 compact generator row and all cached individual patterns are deleted** (`"smaller circles"`). Meanwhile, idle real-time compute inside safe long words (`260`-tick ladder = `91` core-seconds `~7,500` patterns, `S165`) runs continuously without blocking execution (`Kill test 18`).
2. **Verbatim quote(s) + exact source `file:line` & session tag**
   - `"we should not learn jumps but classes of jumps and uncover the generative function otherwise it'll get expensive or maybe you find a function and you see where it solves things?"` — `archive/turns/julien-2026-11-10.md:151` (`S90`)
   - `"we might be so fast that a ladder climb grants us compute to learn so much that it can improve itself real time."` — `archive/turns/julien-2026-11-10.md:573` (`S165`)
   - `"With depth-3 words already kept it costs about 10³. So intermediate words must be kept even when nothing wants them yet."` — `map/detail/scales.md:58`
   - `"Smaller circles (S90: 'we should not learn jumps but classes of jumps and uncover the generative function'): for the joint family carrying the most T × traffic, check whether its breakpoints are the closed-form envelope's; when they are, the family is one generator row and its stored patterns are deleted."` — `map/detail/learning.md:63`
   - Kill test 24: `"Families. The cost of the n-th member of a joint family falls toward a bisection (about 1,500 law ticks against about 10⁴ cold: a Fermi figure), then toward zero when its envelope is derived."` — `map/detail/learning.md:132`
3. **Situations / conditions where it arises**
   - Solving `n = 1..50` parameterized gap+height jump combinations of the same ballistic family (`1-up 3-across`, `1-down 4-across`, `2-up 2-across`), OR encountering a novel `12`-step multi-tool canyon after having explored depth-3 primitives separately (`water-bucket`, `block-step`, `pearl-throw`), OR spending free core-seconds while climbing a `260`-tick ladder on `hardtrack` (`learning.md:53`).
4. **Controlled-environment test harness design**
   - **Minimal arena:**
     - *Family collapse (Kill test 24):* Sequence of `30` distinct `(Δx, Δy)` free-arc jump gaps drawn from the ballistic family `(yaw=0, flat landing)`. Track law-ticks searched per member `n = 1..30` and pattern-store byte footprint.
     - *Real-time free-compute ladder (Kill test 18):* Start run with `260`-tick vertical ladder followed immediately by 3 uncached parkour joints; compare `p99` acting tick latency with background learner threads `ON` vs `OFF`, and measure whether suffixes committed after the ladder improve `>= 1` tick over initial fallback without stalling movement.
   - **Measurable pass signal:** (1) Search work drops from `~10⁴` law ticks (`n=1`) -> `~1,500` bisection (`n=2..5`, seeded from breakpoints) -> `0` (`O(1)` envelope check once family promotes to 1 generator row, deleting stored member tables -> byte count drops); (2) Depth-12 chained course succeeds within `~10³`–`10⁴` trials with subassemblies vs `> 10^6` timeout on primitives alone; (3) `p99` acting tick latency identical with learner threads on/off (`Kill test 18`).
   - **Anti-cheat falsifier / mutant:** Disable breakpoint seeding or block generator-row promotion (`"smaller circles"` disabled -> `n=30` still pays `~10⁴` cold search and stored bytes never collapse); OR run synchronous learner on acting thread (`Kill test 18` fails `p99`).
5. **Current readiness**
   - `blocked on learning queue & generator envelope wiring`: `arc-certificate` / `gait-modulo` arithmetic foundations (`P2`) exist, while the learning queue (`one queue, one operation`), Pareto subassembly archive, `Smaller circles` pattern-to-row deletion, and background pointer-swap updater are `Doc-only` (`inventory-map/report.md`, `learning.md §2`).

---

### 3.4 Non-Forgetting Rare Blockers, Favourable-Surprise Discovery Mining & Demand Census

1. **Property name & mechanism**
   Three complementary memory dynamics prevent silent blindness without arbitrary decay timers:
   - **Non-Forgetting Rare Blockers (`S88`):** Because `nothing is removed for disuse` and priority scales by **consequence size (`T saved × traffic`)** rather than frequency alone, a rare puzzle blocker (e.g., an essential bridge/door crossed once per `10⁵` ticks whose absence adds a `5,000`-tick detour) stays permanently in the store and receives high queue priority.
   - **Favourable-Surprise Discovery (`X2`/`S81`):** Every server deviation where arrival beats expectation (`T_seen < T_predicted`, e.g., an accidental zombie knockback boost, collision clip, or sulfur launch) is logged automatically as a candidate mechanism rather than discarded as "noise".
   - **Proactive Demand Census (`learning.md:60`):** Because the map is on disk (`omniscient world`), joint patterns along high-traffic corridors are counted by content hash and queued *before* any bot hits them.
2. **Verbatim quote(s) + exact source `file:line` & session tag**
   - `"one issue will be forgetting, something that is important but appears rarely matters."` — `archive/turns/julien-2026-11-10.md:140` (`S88`)
   - `"even a zombie can be precomputed and can even help us in some niches cases that will never happen basically but still."` — `archive/turns/julien-2026-11-04.md:127` (`X2`)
   - `"surprise also come from mistakes in our port or icompleteness or server things we did not considered and so on and are therefore a gold mine for fixing issues that we always catch."` — `archive/turns/julien-2026-11-10.md:92` (`S81`)
   - `"Nothing is removed for disuse (S88). What matters is the size of the consequence, not how often it comes. A refuted word is tagged under the law version that refuted it, not deleted."` — `map/detail/learning.md:33`
   - `"surprises that helped, since an arrival sooner than predicted is a candidate word as well as a deviation ('even a zombie can be precomputed and can even help us', X2); and the census: the world is on disk (S6: 'we suppose the player is omniscient'), so the joint patterns along likely corridors are counted by content hash before any bot fails there."` — `map/detail/learning.md:60`
3. **Situations / conditions where it arises**
   - Running `10,000` trips across plain ground containing **1 rare choke-point cave** seen only once at tick `1,000` and revisited at tick `10⁶` (`detour = +2,000` ticks if missing); unexpected mob hit/fluid glitch speeding the bot onto a high ledge `3` ticks early during live flight (`Move 37` / deviation log).
4. **Controlled-environment test harness design**
   - **Minimal arena:** Mixed world loop: run `1` traversal across rare high-consequence joint `J_rare` (`saves 1,500` ticks vs detour), then execute `1,000` unrelated flat/parkour trips touching `J_common`, then revisit `J_rare`; separately inject `1` positive deviation (`T_seen < T_predicted`) during playback via subtle environmental impulse/block interaction.
   - **Measurable pass signal:** (1) On revisit after `1,000` non-touching trips, `J_rare` executes **at `0` cold re-search cost** (still present in store, LRU eviction count `== 0`) and outperforms `J_common` if re-queued under equal traffic×detour budget; (2) Positive anomaly is auto-emitted to anomaly/discovery log with replay input rather than suppressed; (3) Corridors inspected by census pre-solve bottlenecks before first traversal failure (`Kill test 19`: queue rank correlates positively with realized `T` saved).
   - **Anti-cheat falsifier / mutant:** Attach an LRU / TTL disuse eviction timer (`evict after 500 unused queries`) or filter deviation logging strictly to `T_seen > T_predicted` (`errors only`): the former forces expensive cold re-search on `J_rare`, while the latter misses favourable physical boosts.
5. **Current readiness**
   - `blocked on unified learner queue & deviation discovery classifier`: anomaly logging exists on `main` (`engine/plan/src/bin/deviation.rs` + `engine/words/src/table.rs` surprise counters), while LRU-free storage guarantees, census corridor scanner, and positive-surprise word proposer are `Doc-only`.

---

### 3.5 Skin-in-the-Game Play, Boundary-Edge ZPD Puzzles & Hormetic Stressor Against Decay

1. **Property name & mechanism**
   To prevent **fragility to success (silent decay without stressor)** (`learning.md:86`) and escape rigid static overfitting, **Play (`S97–S98`)** creates puzzles automatically from the system's own frontier (`zero-width piece edge`, `empty set`, `open bracket`), pushing one physical parameter toward the law's bound until a solution breaks — defining the **Zone of Proximal Development (`ZPD`)** strictly from law envelopes vs vocabulary (`no made-up percentage`). Crucially, **Play obeys "skin in the game" (`Kill test 20`)**: idle bodies risk only joints with certified safe exits (`never over void without bail`), perturbations hit **the most-used trusted gaits first** (`hormesis`), and play's queue budget rises **only in proportion to real downstream trip demand hits** — preventing play from farming useless easy puzzles (`learning.md:67,88`).
2. **Verbatim quote(s) + exact source `file:line` & session tag**
   - `"R&D could be exibited as playing behavior that's how childs get better and test themselves instead of doing nothing"` / `"good point on play but it could also create its own puzzles which is important and how humans learn and test."` — `archive/turns/julien-2026-11-10.md:181,189` (`S97–S98`)
   - `"we need to find a zone of proximal development."` — `archive/turns/julien-2026-11-10.md:151` (`S90`)
   - `"Play is a source of queue items, with skin in the game (S97, S98)... Its puzzles sit at the edge the plan makes exact: an empty set, a piece whose width goes to zero, a bracket that stays open; each is a pattern trips did meet, with one thing moved toward the law's bound until the set vanishes. It is also the stressor for trusted words: the most-used gaits are perturbed first... A play result earns nothing by being solved: its share of the queue follows the demand that later hits its patterns (kill test 20)."` — `map/detail/learning.md:67`
   - Kill test 20: `"Play transfers. The share of demand joints first learned in play is above zero and rises; otherwise play is fake demand (S87)."` — `map/detail/learning.md:128`
3. **Situations / conditions where it arises**
   - Idle bots waiting between commands (`free body play` + `free compute`); after mastering 3-block flat jumps (`width > 0`), play moves destination `1` block wider or `0.5` blocks higher toward exact ballistic boundary to probe `width -> 0`; after `10⁴` ticks of peaceful stone sprinting, play perturbs high-traffic sprint-jump gait with ice patches/side-pushes to catch silent brittleness.
4. **Controlled-environment test harness design**
   - **Minimal arena:** Phase 1: Train in safe stone-floor play room (`fall <= 3` safe abort guaranteed) + synthetic "fake trap puzzle" branch (disconnected from any trip corridor, zero future demand); Phase 2: Send real trips over progressively harder unseen obstacles drawn near law limit (`3+1` step, `4`-block ice gap, head-hitter drops).
   - **Measurable pass signal:** (1) Play expands boundaries outward (`3 -> 4`-block jumps) while `0` play bots enter lethal uncertified falls; (2) Real trip demand hits patterns pre-discovered in play (`share > 0` and rising, Kill test 20); (3) Fake disconnected puzzle branch drops monotonically in queue share (`rewarded only by later demand hits`).
   - **Anti-cheat falsifier / mutant:** Reward play strictly by `puzzles_solved_per_second` ignoring downstream trip hits (`fake demand` mutant, `learning.md:88`): queue floods with trivial 1-step variants and transfer hit-rate (`Kill test 20`) collapses to `~0%`.
5. **Current readiness**
   - `blocked on executor + one-queue learner`: explicitly marked `"Not built; it needs the executor"` in `map/detail/learning.md:67`.

---

### 3.6 Bidirectional Co-Adaptation, Taleb Antifragile Asymmetry, Blind Protean Forge & Non-Deterministic `Zheng/Qi` Mixing

1. **Property name & mechanism**
   Across longer epochs and adversarial shocks, four tightly linked adaptive invariants must emerge rather than brittle fixed policies:
   - **Bidirectional Layer Shifting (`pvp-strategy:334,370`):** Top-down **"Making Smaller Circles"** (turning frequent micro joints into automatic generator rows) works simultaneously with bottom-up **Double-Loop Shock Bubbling** (surprises/deviations bubbling upward to break faulty abstractions — Taleb & Waitzkin).
   - **Antifragile Asymmetry (`Kill test 22`, `learning.md:91`, Lakatos lemma-incorporation):** Shocks (law deltas, unexpected corridor edits, pushes, enemy probes) leave the system **stronger on repeats**: deviation -> witnessed law delta -> tightened preconditions (`not monster-barring`), so time-to-recover drops sharply when a shock class recurs.
   - **DNA-Helix Red Queen + Blind `Protean Forge` (`∂World/∂θ_agent = 0`, `pvp-strategy:454-493`):** Self-play paired with an **exogenous blind generator** of mutated terrain, asymmetric kits, and extreme shocks (including Extremistan heavy-tailed variance per Jensen's inequality) forces genuine predictive coherence (`epicycloidal` co-evolution) while making trainer co-adaptation/reward-hacking impossible.
   - **Non-Deterministic Micro-Seam Mixing (`Zheng/Qi`, `pvp-strategy:1565,1578`):** Across near-equal optimal responses to probes/feints, deterministic choice creates a **Rep-#4 conditioning trap**. Mixing over certified equivalents prevents opponent legibility without compromising proven bounds.
2. **Verbatim quote(s) + exact source `file:line` & session tag**
   - `"Maybe we could have trickled down learning and bottom-up learning at the same time somehow where all the layers can shift so that with time they are complementary to each others fulfilling the right roles at the right layers in a complementary and even compounding manner."` — `archive/research/pvp-strategy-2026-11-22.txt:334`
   - `"there's the protean forge that i named it is a trainer that is blind to the trainee to solve those kinds of attractors."` — `archive/research/pvp-strategy-2026-11-22.txt:454`
   - `"learning should happen at the highest surprise because it is indicative at a failure in prediction and therefore intelligence"` — `archive/research/pvp-strategy-2026-11-22.txt:517`
   - `"if a bot's certified words are 100% deterministic in how they respond to a feint, a smart human (or mirror self-play agent) will condition and trap it on rep #4."` — `archive/research/pvp-strategy-2026-11-22.txt:1565`
   - Kill test 22: `"The asymmetry. Planted shocks on the sandbox (a law fault, edits, pushes): the time to recover falls when a class of shock repeats. Killed if every repeat costs the same."` — `map/detail/learning.md:130`
3. **Situations / conditions where it arises**
   - Repeated encounters with the same class of unexpected push/edit/law bug (`Kill test 22`); evolutionary self-play arena (`Protean Forge`) shifting terrain/kits unpredictably without inspecting agent state; repeated duel feints across 5+ exchanges where deterministic dodging/countering would be predicted by rep #4.
4. **Controlled-environment test harness design**
   - **Minimal arena:**
     - *Kill test 22 (Antifragile Shock):* Inject `10` instances of a novel shock class (e.g., surprise mid-stride cobweb placements or novel velocity impulse) into live sandbox episodes; measure recovery time (ticks + compute) on shock `1` vs shocks `5..10`.
     - *Protean Forge Blindness + Conditioning Trap:* Test trained agent against (a) a "conditioner" script throwing 3 identical feints then punishing the exact deterministic counter on rep #4; (b) blind ecological mutation suite (`0` feedback from agent to Forge RNG).
   - **Measurable pass signal:** (1) Recovery cost (ticks lost + replan work) decreases strictly from shock #1 to shock #5–10 (class absorbed into store/footprint premises -> Kill test 22 pass); (2) Rep-#4 survival rate against adaptive feint-trapper stays high via mixed equivalent responses rather than locking into `100%` deterministic counter.
   - **Anti-cheat falsifier / mutant:** (1) Discard/never index post-shock repair records (`every repeat costs the same` mutant -> fails Kill test 22); (2) Expose agent win-rate/state weights to the scenario generator (`∂World/∂θ != 0` curriculum gaming) or freeze deterministic tie-breaking on feint branches (trapped `100%` on rep #4).
5. **Current readiness**
   - `blocked on unified learner + PvP / Step 7`: `Antifragile asymmetry` (Kill test 22) awaits unified store + repair loop (`Steps 2–5`); `Protean Forge` and `Zheng/Qi` adversarial mixing belong to future Step 7+ self-play (`pvp-strategy-2026-11-22.txt:2069-2071`, intentionally deferred until Step 3 clears).

---

### 3.7 Law-Native Difficulty Ranking by Error Margin (`width`)

1. **Property name & mechanism**
   Because every certified word record carries its geometric/temporal **`width`** (`lateral clearance` and `timing/delay tolerance`), the system organically discovers an **objective, law-grounded ranking of Minecraft's hardest maneuvers** (`narrowest tolerance` closest to zero-width contact singularities) — often completely inverting what feels hard to human muscle memory.
2. **Verbatim quote(s) + exact source `file:line` & session tag**
   - `"we could literally create a ranking of the hardest maneuvers in the game in terms of error margin, not that it would be the hardest for a human though."` — `archive/turns/julien-2026-11-10.md:693` (`S190`)
   - `"Where to refine is where a word's width goes to zero (his 'contention points ... at the edge of what i think i can do'); the record already holds clearance and delay tolerance"` — `map/detail/learning.md:40`
3. **Situations / conditions where it arises**
   - Auditing a complete corpus of 21 tracks + speedrunner maneuvers (e.g., comparing 1-tick edge momentum clip / boat-lava burn timing against a visually dramatic 4-block jump with wide timing margin).
4. **Controlled-environment test harness design**
   - **Minimal arena:** Run unified `solve` / evaluator over a suite of 10 diverse maneuvers (`2-block step`, `4-block head-hitter`, `pixel-edge sneak`, `single-tick lava boat grab`, `50-block pearl`), perturbing entry position/tick `±1 ULP`, `±1 ms`, `±1° yaw` around each witness.
   - **Measurable pass signal:** Maneuver rank computed automatically from stored `width` matches empirical sensitivity cliff measured under injected law/timing perturbations (`Spearman ρ >= 0.9` with measured failure rate under tiny noise), surfacing hidden narrow maneuvers (`width ≈ 1e-7` holds) above wide forgiving jumps.
   - **Anti-cheat falsifier / mutant:** Rank difficulty by number of human clicks or path length (`ticks * actions`) instead of `width`: misclassifies long trivial ice bridges as "harder" than 1-tick/sub-pixel edge pins (`Spearman ρ` collapses).
5. **Current readiness**
   - `blocked on populated joint/word width fields across tracks`: `width` schema exists in `map/detail/vocabulary.md` / `learning.md:40`, pending full `families/pieces/points/holds` (`Step 2`) and multi-track learning corpus (`Step 3/Later`, `plan.md:56`).

---

## Domain 4: Swarm, Cooperation, Logistics & PvP

### 4.1 System-Level Tropical Needs/Prices Eusocial Organism ("Swarm as One Mind, Decentralized Execution")

1. **Property name & mechanism**
   Bots never receive scripted roles (`"Builder"`, `"Courier"`, `"Guard"`, `"Vanguard"`). Every bot acts as a local particle reading **shared global fields** (`T-to-go` per goal, item `PriceField`, build-frontier critical slack, threat cones, Pigou congestion delay) and deciding solely from local economics `max(P_task - T_arrive - C_risk)` (tropical `(min, +)` routing inside `(max, +)` causal sync). **Functional specialization** (e.g., fast-kit bots seizing vanguard gaps while overloaded bots haul bulk, or eusocial self-sacrifice vs cowardly thrashing) emerges for free out of prices and capability asymmetry (`"the less smart we have to be the better because it means the architecture is doing its job for free"`, `inbox.md:217`). Because knowledge is shared across the one mind (`"eusocial in system 1 human in system 2"`), there is zero individual thundering-herd coordination deadlock (`pvp-strategy:1223`).
2. **Verbatim quote(s) + exact source `file:line` & session tag**
   - `"they act as particles that can couple"` — `archive/turns/julien-2026-11-09.md:15` (`S44`)
   - `"the way bots move and act should be emergent on price and needs not the other way around. it's why there's tropical algebra (min and max plus they are both useful for different things) in the project."` — `archive/research/pvp-strategy-2026-11-22.txt:1174`
   - `"eusocial in system 1 human in system 2"` / `"centralizing knowledge is like the web and then based on your unique circumstances you take action"` — `archive/research/pvp-strategy-2026-11-22.txt:1132`
   - Rejecting individual-agent thundering herd / nobody-goes-first trap: `"no because they share information"` / `"no you think too much in terms of individuals instead of a system"` — `archive/research/pvp-strategy-2026-11-22.txt:1223`
3. **Situations / conditions where it arises**
   - Heterogeneous swarm (`10` bare-legs bots, `5` silk-touch Eff5 tool bots, `5` pearl-armed bots) receiving a joint multi-site build/fetch mission; 20 bots encountering a blocked front where 2 have shovels and 18 carry blocks; multi-squad tactical reaction where shared system fields prevent 20 bots jumping through the same 1-wide choke simultaneously.
4. **Controlled-environment test harness design**
   - **Minimal arena:** Dual-need mission in open yard: Task A = clear `10` stone wall (`10` ticks bare hand, `1` tick with `Eff5_Pick`), Task B = carry `100` dirt across `80` blocks. Spawn `4` bots: `2` holding `{Eff5_Pick}`, `2` holding `{64_Dirt, 64_Dirt}`, with **zero role assignments or hardcoded worker type enums**.
   - **Measurable pass signal:** Under uniform Bertsekas bid pricing (`P(task) - T_bot(task)`), `Pick` bots spontaneously win Task A and `Dirt` bots win Task B (`100%` self-sorted role match), achieving makespan `<= 1.1x` optimal oracle split; swap inventories (`give Pick to bot 3,4`) mid-run and specialization **flips instantaneously** with zero code changes. Also verify `<= 1` bot queues unnecessarily on a single-slot bottleneck (no thundering-herd crash).
   - **Anti-cheat falsifier / mutant:** Hardcode `if bot.has_pickaxe() { assign_role(Miner) }` or strip Bertsekas `PriceField` updates (constant static assignment): swaps fail to re-allocate dynamically when two jobs share identical picks or when travel distances reverse economic optimality (`pick` bot is 150 blocks away vs `bare` bot 1 block away).
5. **Current readiness**
   - `blocked on swarm fields + Step 4 integration`: `wt/server-mind` (`36aa5f99`) proved `10,000` live player bots under one Rust process (`engine/mind/`), using fixed parade/grid targets (`StepCurve`) rather than emergent tropical price/task-auction fields (`Bertsekas`, `min-plus × max-plus` are `Doc-only` in `scales.md §4.4`).

---

### 4.2 `(max, +)` 4D Blueprint Building via Reverse Demolition & Emergent Scaffolding / Container Economics

1. **Property name & mechanism**
   To build any complex 3D blueprint (e.g., the `7,951`-block, `797`-command pagoda benchmark) in minimum makespan without creating sealed interior voids or trapped workers:
   - **Reverse Demolition (`S24`/`S44` -> `scales.md §4.5`):** Simulating the fastest **outside-in demolition** from bot exterior frames (`holes passable if ender pearl/reach exists`) guarantees every removed block had a standing pocket, `4.5`-block eye reach, unoccluded ray, and exterior exit route. Reversing that partial order yields an exact **Bernstein causal footprint DAG `W(t)`** with `(max, +)` critical tail `D(p)` (`max(support exists, worker arrives) + place_time`, zero voids, zero sealed-in traps).
   - **Emergent Scaffolding & Shulker Containerization (`inbox.md:189,217`):** Scaffolding blocks (climbable towers that collapse cleanly from the bottom) and shulker boxes (batch bulk handling) aren't prescribed; they emerge whenever their marginal reduction on `(max, +)` critical-path makespan (`new productive faces opened` vs `jump+pillar` or `resupply walk`) exceeds their placement+cleanup toll.
2. **Verbatim quote(s) + exact source `file:line` & session tag**
   - `"picking up the fastest building spot and that is that you can create voids so it's more holistic than that"` / `"the bots build it so they have the overall goal in mind and it will be broken up smartly so that the time to completion is minimized, they gather ressources"` — `archive/turns/julien-2026-11-04.md:113` (`S24`)
   - `"inverse process feels the same like particles removing all the matter in an efficient manner"` / `"scaffoldings really make things way easier, actually i'll know if I have a very good algo when they'll use scaffoldings in crazy and fast and modular ways."` — `archive/turns/julien-2026-11-09.md:59,61` (`S44`)
   - `"So the reverse demolition is from the frame of the bot to start with, a hole leading to the inside is the same as the outside when you have an enderpearl for example. The goal minimizing T construction time that's it (supposing you have the materials for now) but even logistic of shulker boxes and chests matters."` — `archive/turns/inbox.md:185` (`2026-11-19`)
   - `"scafoldings are scafoldings but so are the blocks of builds, in a vertical tower maybe a scafolding is faster than jumping and placing the block underneath you. 'e 9-slot hotbar specialization vs. staging shulkers as tolls with a debt' this should be emergent on proper pricing the less smart we have to be the better because it means the architecture is doing its job for free."` — `archive/turns/inbox.md:217` (`2026-11-19`)
3. **Situations / conditions where it arises**
   - Building tall towers (`Sky Needle 32`), solid multi-layer monoliths (`8×6×8` 3D-printer peel wave), enclosed vaults with interior chandeliers (`5`-block vault opening a temporary breach tunnel then sealing behind exit), and the `7,951`-block Pagoda (`jobs/pagoda/build.txt`) at `N ∈ {1, 4, 16, 64}` bots.
4. **Controlled-environment test harness design**
   - **Minimal arena:** Four benchmarks (already defined in `wt/build-4d` + `plan.md:59`):
     1. `Sky Needle 32` (`1×1×32` vertical stone pillar, given `scaffolding` vs `no scaffolding`);
     2. `Monolith 8×6×8` solid cube with `N = 8` workers;
     3. `Sealed Vault + Chandelier` (closed hollow shell with interior target block; test with `pearl`/`breach-seal`);
     4. `Pagoda` (`797` cmds / `7,951` blocks, `N = 1, 8, 64` bots) in real game via `(min, +)` travel + inventory slots.
   - **Measurable pass signal:** (1) `compile_4d` + live server run completes every block identically (`blueprint_diff == 0`) with `0` stranded/trapped bots and `0` leftover scaffolding; (2) Vault compiles a breach-then-seal wave automatically (or pearls out before seal); (3) Scaffolding auto-selected on tall spire when faster than jump-pillar, and omitted when step-by-build blocks are free/faster; (4) Pagoda makespan bracket ratio `T / max(critical_path, total_work / N)` bounded cleanly across `N` without void deadlocks (`scales.md:104`).
   - **Anti-cheat falsifier / mutant:** (1) Naive greedy "bottom-up layer-by-layer" or `"nearest empty block"` heuristic (`greedy_nearest` mutant): traps interior workers inside the `Sealed Vault`, stalls on overhang/reach cliffs in `Pagoda`, or seals interior faces of the `8×6×8` monolith leaving permanent air voids; (2) Disable travel-time `(min, +)` & hotbar swap tolls (`wt/build-4d` old fly/teleport bug noted in `inbox.md:209,217`): achieves fake impossible `T` violating Mojang speed limits.
5. **Current readiness**
   - **Split (`offline-proven` abstract 4D DAG / `blocked` on real physics & live swarm execution):** `wt/build-4d` (`087b18ba`, `engine/build/src/waves.rs`) compiles `compile_4d` & `verify_4d` on `Sky Needle 32`, `8×6×8` monolith, `Sealed Vault`, and `797`-cmd `Pagoda` DAGs offline; **blocked** because `BuildKit` uses abstract DAG ticks disconnected from `law` physics, multi-bot `(min, +)` travel, 9-slot hotbar / shulker staging costs, and `server-mind` execution (`inbox.md:209,217,225`).

---

### 4.3 Shared Infrastructure Amortization & Mission-Owned `(max, +)` Cleanup

1. **Property name & mechanism**
   Because obligations belong to the **mission** rather than individual bots (`scales.md §4.9`, `representation.md:152`) and infrastructure is evaluated by total swarm `T` via backward `(max, +)` `release_tick`, behaviors resembling ant self-sacrifice emerge strictly from `T`-minimization:
   - **Sacrificial Bridge Clearing (`S44`):** One vanguard bot builds a temporary bridge and stays behind to tear it down **after** the `last_user` (`release_tick`) crosses — "wasting" individual effort to crush total swarm makespan `T`.
   - **Pre-Emptive Self-Growing Vine Ladder (`S44`):** A fast scout places 1 vine high on a cliff early because vine random growth creates a full ladder cheaply for a slow trailing column (`high yield for swarm`).
   - **Transferable Cleanup Auction:** Nearest capable bot absorbs pending restore debt if the original placer moved far downstream, minimizing total completion + restore time.
2. **Verbatim quote(s) + exact source `file:line` & session tag**
   - `"using a vine way in advance for an incoming swarm could be high yield action as a self growing ladder that needed only a single block to be placed"` — `archive/turns/julien-2026-11-09.md:41` (`S44`)
   - `"Using blocks to bridge and having a bot clearing the bridge as the others go in the front will waste resources but it'll be a damn worthy 'waste'"` — `archive/turns/julien-2026-11-09.md:43` (`S44`)
   - `"by completed it is conditional on the cleanup part too."` — `archive/turns/julien-2026-11-09.md:124` (`S46`)
   - `"Restoration is per mission: at completion the world is as it was (identical block states, fluids, redstone; no item left lying; inventories whole), so obligations pass to the nearest capable bot."` — `map/detail/representation.md:152`
3. **Situations / conditions where it arises**
   - `20`-bot convoy crossing a 6-block lava chasm with **only `6` stone blocks + `1` pickaxe** held by Bot #1, where individual bridge-and-clear serializes at `20 × (place + cross + break)` unless Bot #1 leaves the bridge standing for Bots #2..#20 (`release_tick`) and tears it down once #20 passes; `10`-bot convoy lagging 400 ticks behind 1 fast pearl-scout at a `20`-block cliff (`1` vine placed at top grows downward before slow rear arrives).
4. **Controlled-environment test harness design**
   - **Minimal arena:** `10`-bot team starting at `x = 0`, `goal B` at `x = 120` behind a `6`-block void/lava gap, total swarm kit = `{6 cobblestone, 1 Eff5_Pickaxe on Bot #1}`, rule = full world restoration required before mission timer `T` stops. Compare against baseline of Bot #1 building-crossing-breaking alone and tossing/holding blocks per individual bot.
   - **Measurable pass signal:** Swarm converges spontaneously to **Leave-Stand + Trail-Cleanup**: Bots `#2..#10` run across at full sprint-jump speed while Bot `#1` executes cleanup strictly at `t >= release_tick(#10)` (`0` double builds, `6/6` cobble recovered, `mission T <= 1.2 × max(single_traversal + single_build_break)` instead of `10 × build_break`). If Bot `#1` is artificially teleported past `B` after building and Bot `#10` carries a pickaxe, Bot `#10` automatically clears the bridge (transferable mission debt).
   - **Anti-cheat falsifier / mutant:** Enforce **per-bot ownership of debts** (`each bot must personally break every block it placed` mutant): forces all 9 following bots to re-bridge / transfer pickaxe individually or wait on slow serial loops, multiplying mission `T` dramatically; OR permit leaving blocks uncleaned (`obligations disabled`).
5. **Current readiness**
   - `blocked on Step 5 ledger + Step 4 multi-bot W(t) scheduler`: obligations, `release_tick`, and transferable mission cleanup ledger are `Doc-only` (`kit-edit-ledger-redteam/report.md`, `scales.md §4.9`).

---

### 4.4 Ski-Rental Infrastructure Thresholds, Pigou Congestion Pricing & Particle-in-Cell Crowd Coupling

1. **Property name & mechanism**
   Multi-bot spatial economics stabilize through three mathematical laws without centralized traffic cops:
   - **Ski-Rental Infrastructure Rule (`scales.md:94`):** When crossing an obstacle repeatedly (detour costing `ΔT` vs building permanent bridge/ice road/tunnel costing `C_build + C_restore`), the swarm builds the shortcut **at the exact tick cumulative detour waste equals `C_build + C_restore`**, guaranteeing competitive makespan within `2×` of a clairvoyant oracle knowing future demand (`Karlin et al.`).
   - **Pigou Marginal-Delay Pricing (`scales.md:97`):** Bots enter bottlenecks (`1`-wide doorway, `push_pair` crowd zone) by **total marginal delay imposed on the swarm** rather than greedy private time (`d(k·delay)/dk`), eliminating **Braess's paradox** where an unpriced shortcut slows the whole group.
   - **Particle-in-Cell (`PIC`) Crowd Field (`scales.md:102-103`):** Individual worldtubes deposit occupancy onto shared grid fields; bots read fields (`O(bots + cells)`, never `O(bots^2)` pairwise conflict checks).
2. **Verbatim quote(s) + exact source `file:line` & session tag**
   - `"cooperation must come from local rules, look at ants ... ants can bridge but no one told them to."` / `"there will be some traffic paradoxes i suppose ... solutions ant bridge like can appear because thresholds have been crossed"` — `archive/turns/julien-2026-11-09.md:17,19` (`S44`)
   - `"Build shared infrastructure when the time already wasted equals its cost to build and restore | within 2× of knowing the future | ski rental"` / `"Queue for a scarce thing by the delay you cause, not the delay you feel | no Braess paradox | Pigou"` — `map/detail/scales.md:94,97`
   - `"Particle in cell (plasma physics) is the compute pattern: deposit worldtubes on the grid, update the fields, bots read them. Cost is bots plus cells, never bots²."` — `map/detail/scales.md:102`
3. **Situations / conditions where it arises**
   - Unplanned stream of arrivals trickling across a `1`-block step/river detour (`detour = 10` ticks/bot, `bridge build+restore = 100` ticks -> automatically built after `10` bots' cumulative waste); parallel highways with a narrow high-friction chokepoint bridge (`push_pair` slowdown if `> k` bots crowd it -> excess bots spill naturally onto slightly longer clean route avoiding gridlock).
4. **Controlled-environment test harness design**
   - **Minimal arena:**
     - *Ski-rental:* Unknown Poisson stream of `M ∈ {2, 10, 100}` independent courier commands from `A -> B` separated by `12`-tick detour vs `60`-tick build+restore bridge cost.
     - *Pigou / Braess:* Two routes `A -> B`: Route 1 (wide `20`-tick stone road), Route 2 (`5`-tick base path containing a `1`-wide mud/water pinch where `push_pair` adds `2 * n_present` delay). Send `40` simultaneous bots `A -> B`.
   - **Measurable pass signal:** (1) On Ski-rental across unknown stream sizes `M ∈ {2, 10, 100}`, cumulative swarm `T / T_clairvoyant <= 2.0` (`<= 2×` worst case, threshold fires strictly once accumulated detour `>= build+restore`); (2) On Braess/Pigou arena, average arrival `T` beats selfish Wardrop equilibrium (`total T_Pigou < total T_greedy`) and removing/penalizing Route 2 cannot make swarm *faster* (no Braess paradox); (3) Field computation time scales linearly `O(bots + cells)` across `N ∈ {100, 1000, 10000}`.
   - **Anti-cheat falsifier / mutant:** (1) Disable marginal Pigou congestion term (`each bot minimizes private expected T` assuming no crowd delay): `40/40` bots jam Route 2, suffering massive `push_pair` blowup worse than removing Route 2 (`Braess paradox observed`); (2) Hardcode `"build bridge if count > 5"` offline constant instead of cumulative `wasted_T >= build_cost`: breaks `2×` ratio when `detour_delta` changes from `12` to `2`.
5. **Current readiness**
   - `blocked on wiring push_pair & PIC fields`: `law::entity_contact::push_pair` exists (`wt/port-fold`, proven at 500-bot `Battle` in `es-1b-probe`), **but has zero live planner producer** (`now.md:61`, `scales.md:101`); Ski-rental threshold, Pigou marginal field, and PIC tube-deposit grid are `Doc-only`.

---

### 4.5 Mid-Sprint Item/Tool Handoffs & Supply-Chain Stockpile Buffering

1. **Property name & mechanism**
   Because items exist as deterministic law entities (`ItemEntity` pickup `2.6×2.8×2.6` box after 10-tick drop delay, `C7`) and logistics runs by **Pull (`Toyota`)** across shared `PriceField`s:
   - **Mid-Sprint Tool/Block Handoffs:** Two running bots compute that throwing an item (`DropItem` with player momentum / facing) ahead into a partner's trajectory allows pickup without stopping (`0`-stall transfer beats detouring to chest or sharing 1 bot's trip).
   - **Wartime Stockpile Buffering (`(max, +) -> (min, +)`):** Because long sequential single-line supply chains obey fragile `(max, +)` synchronization (`delay on 1 link stalls everyone`, `pvp-strategy:1824`), building intermediate forward stockpiles breaks the dependency graph cleanly — turning brittle wartime `(max, +)` logistics back into resilient parallel `(min, +)` routing under fat-tailed delays (`representation.md:190`).
2. **Verbatim quote(s) + exact source `file:line` & session tag**
   - `"The flow of tools between bots that minimises total T is prices as fields"` — `map/detail/plan.md:53` (`S96`)
   - `"Pull, not push (Toyota): a bot restocks when a bin empties, by the price field."` — `map/detail/scales.md:114`
   - `"one line in the supply chain is fragile"` — `archive/research/pvp-strategy-2026-11-22.txt:1824`
   - `"stockpile buffers / redundant routes turn brittle wartime (max, +) supply chains into resilient (min, +) choices under fat-tailed delays."` — `map/detail/representation.md:190`
   - `"Therefore a bot (or a teammate in the swarm) adjacent to a void bridge block can catch the drop mid-air."` — `jobs/kit-edit-ledger-redteam/report.md:17` (`C7`)
3. **Situations / conditions where it arises**
   - Relay race across `300`-block gap where Bot A sprint-jumps with a sole pearl/pickaxe to `x=150` while Bot B waits/sprints past `x=150` ready for last `150` blocks (relay handoff in motion); active multi-front construction/siege facing variable courier delays where single-courier JIT stalls vs a 2-shulker forward depot keeps all builders fed (`(min, +)` failover).
4. **Controlled-environment test harness design**
   - **Minimal arena:**
     - *Moving Handoff Relay:* Bot A starts at `x=0` (fatigue / slow boots) carrying `1` essential `pearl` / `key_block`; Bot B starts at `x=60` (Speed II / clean sprint-jump corridor toward `B = 180`). Compare swarm `T` vs Bot A walking whole path or stopping for manual chest drop.
     - *Fat-Tailed Supply Buffer:* `4` builders consuming `10` stone/sec at build site, fed along `2` noisy supply lanes where random mob/ambush shock delays Lane 1 by `500` ticks once per run. Compare `JIT zero-buffer` vs `PriceField-priced forward shulker buffer`.
   - **Measurable pass signal:** (1) On relay, Bot A uses `DropItem` at tick `t_drop` while moving so `ItemEntity`'s `10`-tick pickup timer expires *inside* Bot B's moving `2.6×2.8×2.6` box (`0` tick deceleration stall on Bot B, beating non-handoff baseline cleanly); (2) Under fat-tailed supply delay, risk-adjusted price field stages `>= 1` local buffer depot so builder throughput stays smooth (`no 500-tick starvation stall`).
   - **Anti-cheat falsifier / mutant:** Exclude `DropItem` velocity transfer from `Footprint::writes` / `(min, +)` handoff graph (bots never discover moving passes, forced to walk full solo leg or stop motionless for pickup) OR zero out delay variance in supply cost pricing (`buffer size -> 0`, total stall under `500`-tick disturbance).
5. **Current readiness**
   - `ItemEntity` physics & exact throw kinematics ported in `wt/port-fold` (`items.rs:400-484`, `kit-edit-ledger-redteam/report.md:17`); **blocked** on wiring `DropItem`/`Pickup` verbs into `plan`, shared `PriceField`s, and variance-aware swarm buffering (`Step 5/8`).

---

### 4.6 Two-Regime PvP Contact Singularity, Physical Light-Cone Fog of War, Feints & `(max, +)` Hurt-Time Swarm Melee

1. **Property name & mechanism**
   Rival autonomous players break standard static `(word, count)` terrain compression inside contact (~`6` blocks, `12`-tick cooldown branching), giving rise cleanly to a **two-regime split (`architecture.md:27`, `representation.md:188`, `pvp-strategy:2046,2067`)**:
   - **Pure Crystal Outside Contact + Physical Light-Cone Horizon:** Beyond reach cones (`bow 4D trajectory cone`, `sprint+cooldown intercept`), enemies act strictly as moving **cone boundaries**. The crystal pre-builds terrain/chokes, chooses *where and when* contact happens (Napoleon's Central Position & *Bataillon Carré* encirclement timing), conceals hidden tool tricks (`B_self(opp)` vs `B_opp(self)` timed reveal, `pvp-strategy:1498`), and cancels only certificates whose worldtube meets the cone (`one entry on calendar::Wheel`).
   - **Dense `k = 1` Singularity Solver Inside Contact:** Crossing inside `~6` blocks hands off to fast exact `k=1` law rollouts + forced-kinematics reach-avoid (`10`-tick hurt-time, knockback arcs, eat/draw slowdowns) + probe feints to infer unknown opponent kit (`faints ... gain information about their kit`, `pvp-strategy:1565`).
   - **`(max, +)` 10-Tick Hurt-Time Staggering & Lanchester Geometry:** Because Mojang's `10`-tick invulnerability frame discards simultaneous hits on one target, a swarm fighting one/few enemies **spontaneously staggers strikes at `(max, +)` intervals (`+10` ticks apart)** (`pvp-strategy:2046`) while forcing wide engagement vs narrow choke splits (`Square` vs `Linear` Lanchester attrition, `:1617`).
2. **Verbatim quote(s) + exact source `file:line` & session tag**
   - `"Compute the wavefront of everything they could do." yes this is planned, what they can do is limited and bounded it's the light cone idea` — `archive/research/pvp-strategy-2026-11-22.txt:185`
   - `"a bow in a player's has a light cone that's limited especially when you think in 4D"` — `archive/research/pvp-strategy-2026-11-22.txt:248`
   - `"4D combos and chains sharing the same mind"` / `"armies of bots fighting each others"` — `archive/research/pvp-strategy-2026-11-22.txt:963`
   - `"faints can also be used to gain information about their kit too."` — `archive/research/pvp-strategy-2026-11-22.txt:1565`
   - `"don't show what they can do to surprise opponents when they are distracted"` — `archive/research/pvp-strategy-2026-11-22.txt:1498`
   - `"Lanchester's Laws"` — `archive/research/pvp-strategy-2026-11-22.txt:1617`
   - `"treating humans as something special will help we don't have to do the crystal defect to humans they are sort of singularities"` — `archive/research/pvp-strategy-2026-11-22.txt:2067` (and `representation.md:188-190`: `"two-regime singularity boundary ... Minecraft's 10-tick hurt-time turns swarm melee into (max, +) staggered scheduling"`)
3. **Situations / conditions where it arises**
   - Evading an archer across open terrain by crossing perpendicular to 4D arrow time-of-flight cones (`4D bow cone`); `4v1` / `10v10` swarm skirmishes over uneven bridges vs open fields (wide vs choked geometry, hurt-time staggering); delaying approach to ambush when rival is eating/turning away; probing hidden `wind_charge`/`pearl`/`potions` on opponent hotbar via zero-commitment feint steps.
4. **Controlled-environment test harness design**
   - **Minimal arena:**
     - *Two-regime & 4D Bow Cone:* `60`-block valley with `1` scripted sniper holding loaded bow (`dt_flight`, gravity/drag cone); compare non-contact path checks per second and arrow-dodge evasion rate without per-tick search outside cone.
     - *Hurt-time `(max, +)` Swarm Stagger:* `5` sword bots vs `1` heavily armored target (`100` HP, `10`-tick hurt immunity), measuring total hits landed vs damage dealt (`damage / hits`, wasted simultaneous hits) with vs without `(max, +)` hurt-timer coupling.
     - *Central Position / Lanchester Split:* `6` bots vs `8` split rivals (`4` + `4` separated by `40`-block gap with 1 cobble wall/water chokepoint available).
   - **Measurable pass signal:** (1) Beyond ~6–10 blocks / outside arrow flight cones, `solve()` runs `<= 1` certificate check per cone-intersection tick on `Wheel` (zero crystal thrashing); (2) In `5v1` melee, `>= 85%` of attacks land outside target `hurtTime > 0` (`~1` strike every `10` ticks), cutting TTK sharply vs naive simultaneous dogpile; (3) In `6v4+4` battle, swarm screens Squad B with `1–2` bots/water cut (`inflating T_B`) while `4–5` crush Squad A inside `T_A < T_B` cleanly without hardcoded "Napoleon macro".
   - **Anti-cheat falsifier / mutant:** (1) Try certifying `60`-tick deterministic melee words inside `6`-block contact (`Stage A melee word` mutant): causes severe certificate thrashing (`replans/sec` explodes, bot pauses/dies during stale word replay); (2) Run all `5` swarm bots on greedy individual chase ignoring shared `(max, +)` `hurtTime` field: `> 60%` of swings hit immune frames (`TTK` degrades `~2–4×`).
5. **Current readiness**
   - `blocked on Step 3 clearing + two-body PvP flight + combat wiring`: `26.2` shield/crit/mace/hurt kernels exist in `wt/port-fold` GLM dossiers (`now.md:10`), `ServerEntityMixin` hurt/death hooks exist in `wt/server-mind`; **intentionally held out of Rust word structs (`plan.md:38`, `pvp-strategy:2046,2071`)** until Step 3 (`NeedsJoints`) and stick latency clear and a live two-body flight empirically measures the contact thrashing radius.

---

### 4.7 Emergent Tactical Geometries, "Death Ground" Ergodicity Flip & Inventory Flooding Conjecture

1. **Property name & mechanism**
   Strategic maxims emerge from exact 26.2 physics + Taleb/Peters ergodicity boundaries without hardcoded psychological scripts:
   - **Cliff Herding & Asymmetric Attrition (`T_empty`):** Under survival constraints (`health`, `food`, `totem/pearl stocks`, `escape edges`), maneuvering to keep a rival near a cliff edge (`Shi` = high local survival derivative per knockback tick) or bleeding their consumables while protecting one's own `τ_ruin` arises from `(min, +)` / minimax horizon inequality.
   - **Golden Bridge vs. Death Ground Ergodicity Flip (`pvp-strategy:334-342,1954`):** Because Taleb/ergodicity flips any player whose escape survival hits `0` (`Death Ground`) into **variance-seeking desperation** (mutual lava dump, cliff suicide), an asymmetric richer/stronger leader preserves ergodicity for the weaker opponent (`leaving a taxed corridor`) *unless* the leader itself faces hopeless ruin (`when trapped in Death Ground`, self-play maximizes variance instead of timidly fleeing — `"never underestimate a cornered enemy"` + `"no golden bridge, I don't want a wussy warrior"`!). Crucially, finite death price stays strictly outside pure `Word` contracts (`mission` / `contact-solver` layer only, `representation.md:189`).
   - **VLM/Causal System-2 Asymmetric Tricks (`pvp-strategy:1405`):** Packing raw `planks` (`5` -> `1` boat cliff save OR drop-spammed to flood an enemy's hotbar/inventory pickups during loot scrambles!) arises once inventory slot limits (`36` slots) + item pickup priority enter causal `writes ∩ reads` + System-2 hypothesis generation.
2. **Verbatim quote(s) + exact source `file:line` & session tag**
   - `"never underestimate a cornered enemy without a way out"` — `archive/research/pvp-strategy-2026-11-22.txt:334`
   - `"backing up to not take a hit and building to get the high ground"` — `archive/research/pvp-strategy-2026-11-22.txt:5`
   - `"with this system could the fill the other person's inventory emerge? if you carry let's say planks so that you can make boats? does the cliff idea emerge? asymmetry?"` — `archive/research/pvp-strategy-2026-11-22.txt:1405`
   - `"self fighting should happen till death not till T"` — `archive/research/pvp-strategy-2026-11-22.txt:1498`
   - `"Sun Tzu no golden bridge, I don’t want a wussy warrior."` — `archive/research/pvp-strategy-2026-11-22.txt:1954`
3. **Situations / conditions where it arises**
   - Unwinnable corner (`health = 4 HP, armored hunter blocking only doorway`): cowardly deterministic retreat loses `100%` of rollouts, while lava/mutual-cliff variance play wins `15–30%`; asymmetric 1v1 where leader (`full health`) vs fleeing weak opponent trades `0`-risk taxed pursuit vs reckless cage-fighting; chaotic PvP over loot piles with open inventory slots (`inventory flooding`).
4. **Controlled-environment test harness design**
   - **Minimal arena:**
     - *Death Ground Variance Flip:* Enclosed pit (`12×12` bedrock arena, corner pool of lava/void chasm nearby) pitting weak Bot `W` (`6 HP`, wooden sword, 1 lava bucket) against geared Bot `S` (`20 HP`, diamond sword, no escape exit vs taxed 1-block corridor escape exit).
     - *High Ground / Knockback Cliff:* Elevated narrow ridge (`height = +30`, fatal without clutch) with `1` knockback sword + blocks.
   - **Measurable pass signal:** (1) When `W` is boxed in (`0%` deterministic flee survival), `W` shifts spontaneously to high-variance tactics (lava dump / cliff knockback lunge) improving non-zero win probability over pure retreat; (2) Geared leader `S` leaves a taxable attrition exit when `W`'s cornered variance (`e.g., lava bucket held`) puts `S`'s survival `< 100%`, yet closes relentlessly when `W` holds zero lethal variance shock (resolving the Golden-Bridge vs Non-Wussy tension strictly from the physics payoff matrix!); (3) High-ground bots use block placement to seize `+2` elevation reach/knockback advantage cleanly.
   - **Anti-cheat falsifier / mutant:** Hardcode a static `"always flee below 30% HP"` (`wussy warrior` mutant -> `0%` survival in Death Ground) OR allow pure `Word` search to bypass hard ruin barriers globally on peaceful traversal (`violates goal.md` absorbing-barrier separation).
5. **Current readiness**
   - `blocked on Step 7 PvP solver & System-2 Proposer`: pure conceptual spec (`pvp-strategy-2026-11-22.txt`); strictly excluded from `Word` structs (`representation.md:188-191`) until Step 3 + first two-body flight pass.

---
[END OF PART 2/3]

## Domain 5: ES `10⁹` Scale, Observer Light Cones & Server Interface

### 5.1 `Form-P` Stream Folding, `Ef` Scars & `seal()` Reabsorption (`10⁹` Bots in L3 Cache)

1. **Property name & mechanism**
   Undisturbed cohorts sharing a trajectory generator (`Form-P` stream: `(gait_program, phase, cohort_size)`) pay **nearly `0 B/bot`** instead of per-entity `~24–32 B` structs (`24–32 GB` at `10⁹` bots -> exceeds RAM). Temporary divergences (knockback, momentary slowdown, split edits) append compact sorted Elias–Fano scars (`Ef`) alongside a small `hot` map. Whenever displaced bots finish rejoining the underlying gait stream, running `seal()` **reabsorbs zero-delta scars straight back into `Form-P`**, driving working memory downward automatically (`0.0045 -> 0.0025 B/bot`) so `10⁹` live bots stay wholly resident inside the Ryzen 7800X3D's `96 MB` L3 cache (`~4.28 MB` for `10^9` bots + `10^6` scars).
2. **Verbatim quote(s) + exact source `file:line` & session tag**
   - `"100 bots is not even a lot, pretty sure i can run complex simulations on millions of particles"` — `archive/turns/julien-2026-11-09.md:21` (`S44`)
   - `"how ships are stored ... less than a byte ... generative functions of famillies were stored rather than per case"` — `archive/turns/julien-2026-11-09.md:168` (`S57`)
   - `"would be funny if I manage ES style billions of entities across the whole minecraft map"` — `archive/turns/inbox.md:345` (`2026-11-22`)
   - `"Check 1: 10^9 bots across 1,000 LIVE ES Program streams (100,000 scars) vs LIVE law::try_tick ... 1,000 stratified samples, 0 bit-exact mismatches"` / `"Check 3: N = 10^9, k = 1,000,000: Before 50% rejoin seal(): 4.282 MB (4.49 B/scar, 0.0045 B/bot); After 50% rejoin seal(): scars 1000000 -> 500000, bytes 4.282 MB -> 2.371 MB"` — `jobs/es-1b-probe/report.md:9,72-74`
3. **Situations / conditions where it arises**
   - Scaling a swarm from `10^3` -> `10^6` -> `10^9` traveling bots across highways/expeditions under memory limits (`< 32 MB`), with up to `10^6` local transient disturbances that periodically resolve back into steady travel.
4. **Controlled-environment test harness design**
   - **Minimal arena (`es-1b-probe` Check 1 & Check 3):** `1,000` live ES `Program` streams totaling `N ∈ {10^6, 10^7, 10^8, 10^9}` bots calibrated on Mojang `26.2` flat-stone sprint-jump (`264`-tick settle, `12`-tick gait period, `total_dz = 4.276060`), populated with `k = 10^6` non-trivial `Ef` scars, then settling `50%` back onto the stream (`rejoin`) and invoking `seal(&mut Enc::new(false))`.
   - **Measurable pass signal:** (1) `0` bit-exact position/velocity mismatches across `1,000` stratified samples vs fresh `law::try_tick` rollouts; (2) Sub-32 MB total working set (`<= 5 MB` observed across `N = 10^9, k = 10^6`, `1B / 1M` byte growth `~1.26x` at fixed `k=100k`); (3) `seal()` shrinks `scars: 1,000,000 -> 500,000` and total bytes `4.282 MB -> 2.371 MB` in median `~76 µs` per stream.
   - **Anti-cheat falsifier / mutant (`--mutant no-reabsorb`):** Write zero-delta identity entries into `Ef` instead of erasing rejoined entries against the base `Form-P` curve during `seal()`: `scars` stays `1,000,000 -> 1,000,000`, `reabsorbed = false`, exits `1` (`jobs/es-1b-probe/report.md:103`).
5. **Current readiness**
   - `offline-proven` in `jobs/es-1b-probe` (`live law` + `plan::events::Map` + live ES `shipmem` `Program`/`Prog`/`Ef`); **blocked** on integrating ES `shipmem` + multi-word chains into `engine/mind/` (`wt/server-mind`) + fixing 4 ES integration findings (`Enc::default()` panic, `Program::hot` `u32` truncation, `belt.rs` coupling, `es-1b-probe/report.md:112-125`).

---

### 5.2 `O(1)` Space-Time Inverted Footprint Wake-Ups & Stream Splitting

1. **Property name & mechanism**
   Because every running certificate (folded bot stream, flying projectile arrow/pearl worldtube, periodic redstone machine, plan prefix) registers **its `Footprint::reads` into one inverted 4D sparse `SpaceTimeIndex`** (`16³` section + `64`-bit `4×4×4` subcell occupancy mask), block edits and entity arrivals trigger **only exact intersecting handles (`O(k_intersecting)`, strictly `O(1)` with global bot count `N`)**. When an edit hits a stream covering `[0..1B)` bots, `stream.split()` bisects the stream at the spatial impact ordinal (`~172 µs`), leaving `99.9999%` of undisturbed bots sleeping without re-checking a single coordinate (`never per-bot loops`, `never 6-neighbor hacks`, `C6`).
2. **Verbatim quote(s) + exact source `file:line` & session tag**
   - `"A block edit only breaks the tubes whose footprint reads the block (scales.md section 4.7)"` (and `"The best compute is no compute ... never a whole region for one block (X2)"` — `map/principles.md:14`)
   - `"All causal wake-ups must pass through one space-time inverted footprint index, never a 6-neighbor shortcut."` — `jobs/kit-edit-ledger-redteam/report.md:16` (`C6`)
   - `"Check 2: Single block edit shock across N = 10^3 -> 10^9 + arrow crossing + undisturbed WorldGait ... Log-log median slope = 0.0038 (< 0.05); wake stream/bot=1, arrow=1, wg=1"` — `jobs/es-1b-probe/report.md:10`
3. **Situations / conditions where it arises**
   - Placing 1 cobblestone block at `[4, 0, 200]` at tick `t*` across the path of `1` traveling stream (among `1,000` streams / `10^9` bots), intersecting `1` flying arrow in flight (`t* - 5`) and `1` nearby redstone wire (`WorldGait`), while all distant streams/machines keep running untouched. Also non-local causal edits: piston quasi-connectivity `2` blocks away, `42`-block tripwire string, `7`-block scaffolding stack (`C6`).
4. **Controlled-environment test harness design**
   - **Minimal arena (`es-1b-probe` Check 2):** 4-point population ladder `N ∈ {10^3, 10^5, 10^7, 10^9}` bots across streams (`1,000` scars + `1` unsealed hot entry on impacted stream), plus a flying arrow launched at `t* - 5` whose swept cells pass `[4, 0, 200]` (deduplicated per cell), plus a local `clock_4` touching `[4, 0, 200]`. At `t*`, place block at `[4, 0, 200]` and time `notify_edit + seal + first_cross + split` over `n = 100` trials per rung.
   - **Measurable pass signal:** (1) `wake_stream == 1`, `wake_arrow == 1`, `wake_worldgait == 1` at every `N`; (2) `index_bytes == 11,431 B` invariant across `N = 10^3 -> 10^9`; (3) Log-log latency slope across ladder `< 0.05` (observed: `0.0038`, median `163..172 µs`, max `< 250 µs`).
   - **Anti-cheat falsifier / mutant (`--mutant per-bot-index`):** Register bots individually into spatial cells rather than streams: at `N = 100,000`, `index_bytes` explodes (`5.25 MB`), `wake_count` jumps to `50,001`, log-log slope jumps to `0.1072` -> exits `1` (`es-1b-probe/report.md:102`). Second mutant: naive 6-neighbor lookup without `SpaceTimeIndex` misses a 2-block-away quasi-connected piston or flying arrow `40` blocks out.
5. **Current readiness**
   - `offline-proven` on `SpaceTimeIndex` + live `plan::events::Map::first_cross` + ES `Program::split()` (`es-1b-probe` Check 2); **blocked** on fixing `Program::split()`'s `O(k)` cursor scan & unsealed-`hot` caveat (`es-1b-probe/report.md:120-122`), wiring live Mojang non-local footprints (`piston`/`tripwire`/`scaffolding` in `wt/port-fold`), and integrating into `server-mind`.

---

### 5.3 `WorldGait` Zero-Wake-Up Periodic Machine Folding & Exact Edit Materialization

1. **Property name & mechanism**
   Any undisturbed repeating world automation (redstone clocks, kelp/bamboo piston farms, flying machines bridging canyons, bubble elevators) collapses cleanly into a periodic **`WorldGait`** `(cycle_program, phase_0, period P, Δpos)` computed via modular arithmetic (`W(t)` answered in `O(1)` at `(t - t0) % P`), generating **zero `calendar::Wheel` wake-ups across arbitrarily long undisturbed spans**. The machine re-enters event-by-event `law` simulation at exact `(phase, state)` **only** when a `SpaceTimeIndex` write breaks its cycle footprint (`C8`).
2. **Verbatim quote(s) + exact source `file:line` & session tag**
   - `"looping redstone/piston/flying machines must fold into periodic world gaits (the biggest repeating cycle (cycle_program, phase, period P, Δpos)) rather than ticking per cycle forever — answering W(t) in O(1) and waking via the same space-time footprint index."` — `jobs/kit-edit-ledger-redteam/report.md:18` (`C8`)
   - `"Minecraft shall be seen as a causal chain engine which simplifies a lot, ticks are not really needed at all they are a by-product of a way of thinking not of behvavior, behavior is king."` — `archive/turns/inbox.md:1137` (`2026-11-23 18:06`)
   - `"Undisturbed looping redstone / flying machine | 0 ticks/tick | 0 simulation cost; O(1) W(t) lookup via (t - t_0) % P"` — `jobs/kit-edit-ledger-redteam/report.md:45`
3. **Situations / conditions where it arises**
   - Redstone repeaters clocking every 4 ticks for 100,000 ticks (`1.4` real hours / `100` game-minutes) undisturbed on disk/engine; a slime-block flying machine ferry crossing a void gap while distant bots time arrivals against `W(t)` years ahead.
4. **Controlled-environment test harness design**
   - **Minimal arena (`es-1b-probe` Check 2 + phase materialization):** Deploy a repeating `clock_4` redstone circuit + periodic platform gait; advance `calendar::Wheel` for `100,000` ticks with zero external intersections (`undisturbed phase`), verify `W(t)` queries at arbitrary future ticks match brute-force tick simulation bit-exact, then inject 1 block break inside wire footprint at random tick `t_hit`.
   - **Measurable pass signal:** `wheel_wakeups == 0` over all `100,000` undisturbed ticks (`0` tick-by-tick callbacks), `W(t)` query constant time (`O(1)`), and post-interrupt state materializes bit-identically (`state_delta == 0`) with `wake_wg == 1` (`es-1b-probe` Check 2).
   - **Anti-cheat falsifier / mutant:** Re-schedule every cycle `P = 4` onto `calendar::Wheel` (`reschedule next wakeup at t + 4` mutant): fires `25,000` wakeups instead of `0` over `100k` ticks, drowning quiet background worlds in event-queue churn.
5. **Current readiness**
   - `offline-proven` for `clock_4` + ES `Wheel` (`0` wakeups over `100k` ticks, `1` wakeup on edit, `es-1b-probe/report.md:10,53`); **blocked** on automatic runtime cycle-detector collapsing `wt/port-fold` redstone/piston graphs into `WorldGait` certificates and writing live materialized states back to the Mojang server.

---

### 5.4 Pure `O(visible)` Observer Light-Cone Extraction & Unified GPU Instancing (`< 1 ms`, Zero Mutation)

1. **Property name & mechanism**
   Humans only see inside their visual/tracking/locator light cone (`e.g., 192-block frustum` + real player locator ranges). Instead of materializing bots everywhere or mutating simulation caches during rendering/packet emission, a **pure read-only observer pass** walks only streams/blocks intersecting active real-player cones (`O(visible)`, never `O(N_total)`), emits exact `PlayerState` snapshots/locator coordinates with **zero mutation of underlying simulation digests**, and batches heterogeneous bots into **one unified GPU instance draw call (`SwarmRenderer`)**. Thus `10⁷`–`10⁹` simulated bots look completely alive on the server (`real players + skins + locator bar`) while frame extraction stays sub-millisecond (`max < 1 ms`).
2. **Verbatim quote(s) + exact source `file:line` & session tag**
   - `"you really don't need to store that much especially given how cheap it is to rederive anything, you only care about the blocks you see and the causal chains of things hapenning."` — `archive/turns/julien-2026-11-09.md:156` (`S54`)
   - `"when I'm in the server I want to see many bots as real players"` — `archive/turns/julien-2026-11-09.md:124` (`S46`)
   - `"Bots are players, they appear on the locator bar"` — `archive/turns/julien-2026-11-09.md:148` (`S52`)
   - `"Check 4: Pure observer 192-block flyover across 10M-bot highway (500 frames) ... Frame extract p99 = 104.300 us, max = 129.800 us (both < 1000 us); digest_equal = true, bit_mismatches = 0"` — `jobs/es-1b-probe/report.md:12`
3. **Situations / conditions where it arises**
   - Human player flying in creative/spectator along a `10,000,000`-bot highway seeing `~5,400` bots at once in a `192`-block frustum (`es-1b-probe` Check 4), OR hovering above a `10,000`-bot dense/dispersed Minecraft server world (`wt/server-mind`).
4. **Controlled-environment test harness design**
   - **Minimal arena (`es-1b-probe` Check 4 + `server-mind` live flyover):** Fly a `192`-block moving frustum over `10,000,000` active folded bots across `500` consecutive ticks (`~5,372–5,400` bots visible per frame), logging pre-extract and post-extract full-world simulation hashes, plus a live Fabric client rendering `10,000` bots.
   - **Measurable pass signal:** (1) Worst-case extraction latency `p99 < 1,000 µs` AND `max < 1,000 µs` (observed: `p99 = 104.3 µs`, `max = 129.8 µs`); (2) `bit_mismatches == 0` against brute-force rollout; (3) `digest_equal == true` (`observation_never_changes_the_digest` invariant: running observer `ON` vs `OFF` yields bit-identical future world states).
   - **Anti-cheat falsifier / mutant (`--mutant dirty-observer`):** Allow frame extractor to update/cache/seal `stream.hot` or advance a simulation timer during read-only extraction: immediately flips `digest_equal = false` -> exits `1` (`jobs/es-1b-probe/report.md:104`). Second mutant: linear `for bot in 0..N_total` frustum cull (`O(N_total)` instead of `O(visible)`): blows `< 1 ms` max frame gate past `100 ms` at `N = 10^7`.
5. **Current readiness**
   - **Dual-proven separately, unmerged:** `offline-proven` pure observer extraction at `10M` bots (`< 130 µs`, `es-1b-probe` Check 4) AND `flown` Stage 3 live observer tiering (`Virtual`/`Visual`/`Materialized`) + 18-skin GPU `SwarmRenderer` at `10,000` bots (`1.286 ms` median tick, `wt/server-mind`); **blocked** on merging `es-1b-probe` `Form-P` read-only extractor directly into `wt/server-mind` `mind.exe` (`Step 4`, `now.md:29`).

---

### 5.5 Dynamic Tier-0 <-> Tier-2 Dense `Battle` Promotion & Dissolution (Knudsen Transition)

1. **Property name & mechanism**
   Because folded certificates (`Tier-0 Form-P`) dominate sparse regimes while `Entity.push` collisions break certificates every tick in dense crowds (`Tier-2`), the engine triggers an automatic **sparse <-> dense phase transition** (`representation.md §7`, `scales.md:101`) only inside active contact zones: `Form-P` unfolds intersecting bots into dense tick-by-tick SIMD simulation using Mojang's exact canonical `law::entity_contact::push_pair`, runs `Battle` until cohorts exit push radii, and **re-folds (dissolves) them deterministically back into outgoing `Form-P` + `Ef` scars** (`0 B` leftover dense arena memory, bit-exact positions, `100%` inventory conservation).
2. **Verbatim quote(s) + exact source `file:line` & session tag**
   - `"Sparse (≪ 1): certificates, the causal frontier and the slack queue, on the CPU. This covers lone and lazy bots. Dense (near 1): cells in lockstep, on SIMD or the GPU. This covers crowds pushing each other, fights and mob swarms, where certificates break every tick and stop paying. Both give the same bits."` — `map/detail/representation.md:145-147`
   - `"Bots are bosons. Players overlap, so only edits and scarce items exclude, and reservations exist only there. Overlap isn't free, though: every tick two players overlap, Entity.push rewrites both velocities (GLM port-entity-rules, reproduced)."` — `map/detail/scales.md:101`
   - `"Check 5: Lazy Tier-0 -> Dense Battle (500 bots, real law::entity_contact::push_pair) -> Form-P dissolution ... 36,174 push pairs; 0.418 ms/tick median (0.640 ms max); oracle_mismatches = 0, dissolved_mismatches = 0, inventory = 32000/32000"` — `jobs/es-1b-probe/report.md:13`
3. **Situations / conditions where it arises**
   - Two `250`-bot columns converging through a cramped pit (`50` crossing streams hitting at `tick = 23`), interacting densely for `60` ticks via `push_pair`, then dispersing along separate clean highways (`t > 83`).
4. **Controlled-environment test harness design**
   - **Minimal arena (`es-1b-probe` Check 5):** `500` staggered bots across `50` `Form-P` streams (each carrying `64` items = `32,000` total) steered into mutual bounding-box overlap (`promote_tick = 23`), stepped `60` ticks under Mojang canonical `push_pair` (`a < b` index order), then dispersed cleanly back to non-overlapping separation and folded back into `Form-P + Ef`. Compare against a `t = 0` brute-force full tick-by-tick `law` simulation of all `500` bots.
   - **Measurable pass signal:** (1) `push_pairs_applied > 0` (`36,174` recorded) with `oracle_mismatches == 0` AND `dissolved_mismatches == 0` across every single bot/tick; (2) Dense working memory `battle_bytes` drops strictly back to `0 B` after dissolution; (3) Exact inventory conservation (`32,000 / 32,000`); (4) Dense step `<= 1 ms/tick` (`0.418 ms` median measured).
   - **Anti-cheat falsifier / mutant:** (1) Skip `push_pair` in `Form-P` or apply unordered pairwise push (`b < a` race) -> breaks bit-exact equality against brute-force Mojang reference (`oracle_mismatches > 0`); (2) Never dissolve `Battle` back into `Form-P` after separation (`battle_bytes` stays permanently allocated after `t = 83`, failing the `0 B` teardown invariant).
5. **Current readiness**
   - `offline-proven` for `500`-bot Tier-0 -> `Battle` (`push_pair`) -> `Form-P` round-trip (`es-1b-probe` Check 5); **blocked** on automatic runtime spatial detector promoting/dissolving overlapping live worldtubes in `server-mind` (`scales.md:101`: `push_pair has no producer yet`, `now.md:61`).

---

### 5.6 Worldtube Chunk Tickets (`33 / 32 / 31`) & Presence as a Tool

1. **Property name & mechanism**
   Vanilla loads `625` chunks and ticks `441` per lone player (`25×25` wild-card disc -> hits server CPU wall at `~25–50` dispersed bots, `server.md:99`). Because every bot follows a certified **worldtube** rather than unpredictable human whims (`scales.md §4.2`, `server.md:102-108`), two emergent server-scaling behaviors replace the disc:
   - **Worldtube Chunk Tickets (`33 / 32 / 31`):** Chunks along the certified worldtube receive **exact-level tickets ahead of arrival** strictly matching footprint causal needs: `33` (`loaded`, collisions see real blocks while random/scheduled ticks stay frozen `0` cost) for read-only transit; `32` (`block/fluid ticking`) where edits/fluids happen; `31` (`entities tick`) where dropped items/mobs enter (`GLM unloaded-chunk-move`).
   - **Presence as a Tool (`S65`, `scales.md:72`, `server.md:106`):** Being a "real ticking player" (spawning mobs, ticking crops, holding mob-spawners active) is itself an **activated tool**. Traveling bots default to quiet contact-only bodies (`zero ambient disturbance to crops/creepers`), switching full presence ON only at farms/sites where paying for presence unlocks yield — and keeping **vanilla `100%` untouched inside every human player's bubble** (`server.md:108`).
2. **Verbatim quote(s) + exact source `file:line` & session tag**
   - `"what far away even mean i guess it is dependent on player's chunk loading in some way."` — `archive/turns/julien-2026-11-10.md:15` (`S65`)
   - `"The worldtube is the ticket. The engine asks the server for the chunks under each bot's footprint, ahead of time, at the level the footprint needs: read means loaded; a write, a fluid or a dropped item means ticking for as long as the obligation lasts."` — `map/detail/scales.md:68`
   - `"At level 33 (loaded, nothing ticks) collisions see the real blocks; scheduled ticks, other entities and pickup delays are frozen. At 32 blocks tick, at 31 entities do. In an unloaded chunk the server treats the terrain as air and the player ghosts through it"` — `map/detail/scales.md:69` (`GLM unloaded-chunk-move`)
   - `"Default: a traveler is a body, with its contact set only. The swarm switches presence on where it pays (a farm, a site), and it is always on around humans, where everything stays vanilla."` — `map/detail/scales.md:72`
   - `"Killed if: anything differs inside a human's bubble, or a bot reaches a chunk before it loads."` — `map/detail/scales.md:74` & `map/detail/server.md:108`
3. **Situations / conditions where it arises**
   - `200+` dispersed bots walking across separate regions simultaneously (`would take 200 × 441 = 88,200 ticking chunks on vanilla PLAYER_SIMULATION -> impossible`, takes `~3` chunks/bot at level `33` -> `~0` ticking chunks); a traveler sprinting past an unwatered wheat field or dark cave (`zero crop growth / creeper spawn scar`) vs a farmer bot toggling presence ON inside its farm site; any bot entering a human player's simulation bubble (`human bubble stays 100% vanilla`).
4. **Controlled-environment test harness design**
   - **Minimal arena:**
     - *Dispersed 200-bot Scalability + Chunk Guarantee:* Spread `200` independent bots `>= 500` blocks apart running steady traversal over natural chunks under `20 TPS`, logging server `ms/tick`, loaded/ticking chunk counts, and ghosting/fall-through checks.
     - *Presence Toggle + Human Bubble Parity:* Run Bot 1 past a dark cave + unripe wheat crop with `presence = OFF` (`500` ticks) vs at a farm with `presence = ON`; separately place Human Dummy `H` inside `10`-chunk bubble while Bot 2 executes routes nearby, comparing mob spawns, crop growth, locator bar, and tracking against vanilla client baseline.
   - **Measurable pass signal:** (1) `server ms/tick p99 < 50 ms` across `200` lone dispersed bots (breaking the `25–50` vanilla wall cleanly), with `0` unloaded-chunk ghosting (`player feet always in level <= 33 chunk`); (2) Traveling with `presence = OFF` produces `0` hostile mob spawns and `0` uncommanded crop growth ticks along route, whereas activating `presence = ON` at a farm advances growth/spawner ticks normally; (3) Differential check inside `H`'s bubble yields `0` behavioral discrepancies vs vanilla player.
   - **Anti-cheat falsifier / mutant:** Leave vanilla `PLAYER_LOADING` / `PLAYER_SIMULATION` (`25×25` disc) enabled per dispersed bot (`no worldtube tickets` -> `server ms/tick` blows `50 ms` above `~40` lone bots), OR drop level below `33` ahead of arrival (`bot ghosts through terrain as air`), OR suppress presence inside human radius (`breaks vanilla human bubble gate`).
5. **Current readiness**
   - `blocked on GLM server-player-readers + server mod ticket driver`: `unloaded-chunk-move` bytecode contract is pinned (`GLM reproduced`), `wt/server-mind` tested `1,000` dispersed bots using pre-loaded/ender-pearl-style tickets (`0.871 ms/tick`, `.mca` cached), **while** general dynamic `33/32/31` worldtube ticket emission, selective `presence` gating, and auditing every Mojang call site in `server-player-readers` remain `Open before build` (`server.md:107`).

---

### 5.7 Seed-Backed Lazy World Regeneration, Causal "No-Ticks" & Sensor/Social Limits

1. **Property name & mechanism**
   Four architectural horizon properties define the ultimate outer envelope of the project:
   - **Lazy `worldgen(seed) + sparse StateId scars` (`S54`/`S57`):** Instead of holding `10k × 10k` chunks (`~76 GB` dense or `12–39 GB` JVM), known terrain can be rederived from the seed plus sparse delta scars (`< 6 KB/chunk` -> `0.4–2 GB`), and on unknown servers **seed cracking** (`S189`) allows planning beyond seen view distance (`always overridden by seen chunks`).
   - **Causal "No-Ticks" Engine (`inbox.md:1137`, `principles.md:43`):** Behavior is driven purely by causal events (`input`, `contact`, `cell cross`, `clock threshold`, `edit`), emitting exact Mojang ticks only at the server verification/observation seam (`"ticks are not really needed at all"`).
   - **Robotics Incomplete-Knowledge End Goal (`S189`/`X12`) & Human Input Limits (`S32`):** Gradually stripping omniscience toward pure first-person player sensors (`step after step`), bounded by realistic human mouse/input velocity ceilings (`"so that it doesn't look superhuman too much"`).
   - **Social Idle Attention / Behavioral Easter Eggs (`S66`):** Low-priority idle swarm cues emerging naturally when free (`looking + sneaking toward typing human`, turning/teasing near clumsy bot deaths).
2. **Verbatim quote(s) + exact source `file:line` & session tag**
   - `"you really don't need to store that much especially given how cheap it is to rederive anything, you only care about the blocks you see and the causal chains of things hapenning."` — `archive/turns/julien-2026-11-09.md:156` (`S54`)
   - `"also we have access to the seed btw"` — `archive/turns/julien-2026-11-09.md:168` (`S57`)
   - `"Minecraft shall be seen as a causal chain engine which simplifies a lot, ticks are not really needed at all they are a by-product of a way of thinking not of behvavior, behavior is king."` — `archive/turns/inbox.md:1137` (`2026-11-23 18:06`)
   - `"Seed cracking ... from an omniscient bot toward robotics-like constraints, with less information, sensing only what a player senses, 'step after step'"` — `map/detail/plan.md:54-55` (`S189`)
   - `"so that it doesn't look superhuman too much"` (`archive/turns/julien-2026-11-04.md:181`, `S32`); `"when you type nearby bots doing nothing they'll look at while sneaking as if attentive... if a bot falls to his death all the other bots will make fun of him"` (`archive/turns/julien-2026-11-10.md:22`, `S66`).
3. **Situations / conditions where it arises**
   - Planet-scale `10k × 10k+` maps without copying region archives; unknown public-style worlds where visible bedrock/structure motifs leak the seed before chunks beyond `render=12` are visited (`seed cracking`); blind/fog-of-war exploration (`robotics sensor constraint`); casual co-play beside Julien chatting near idle swarm groups.
4. **Controlled-environment test harness design**
   - **Minimal arena:**
     - *Seed+Scars parity:* Generate `10k × 10k` test map with `1,000` localized block scars; verify RAM usage stays within class-space bounds (`<= 2 GB`) and chunk reads outside scars reconstruct exact same `StateId` as saved `.mca`.
     - *Blind / Fog-of-War Frontier (`F-frontier`):* Strip unseen-chunk access beyond view distance `R = 8` on an unvisited rugged seed (`peaks & chasms`); measure discovery replanning latency (`no stalls on newly loaded chunks`), and compare cracked-seed prediction accuracy before chunk arrival.
     - *Causal face (`law::face::advance`):* Run `31` track recordings + `62` extreme motions comparing jumped causal steps vs per-tick stepping (`0` anomalies, `>= 3×` tick jump speedup).
   - **Measurable pass signal:** `law::face::advance` achieves `0` anomalies across all recordings/extremes (`already 0 on port-fold`!); scar-backed chunk loader matches disk `.mca` bit-for-bit; fog-of-war sensor mode reaches `B` on `R-far` trips with `0` unloaded-chunk crashes and strictly lower regret when predicted seed matches terrain.
   - **Anti-cheat falsifier / mutant:** Peek at disk `.mca` beyond server-sent chunks during `robotics-sensor` mode (silent oracle leak mutant -> detected immediately by gating `World` trait to player packet stream only); OR run tick-by-tick `core.rs` ignoring causal event boundaries (`event face disabled` -> loses `~6×` per jumped tick speedup).
5. **Current readiness**
   - **Partial:** `law::face::advance` causal jumping is `offline-proven` on `wt/port-fold` (`22.2%` of recorded ticks jumped, `~0.19 µs` vs `1.2 µs`, `0` anomalies on 31 recordings + 62 extremes, `inbox.md:1442`), and `wt/port-fold` keeps pregenerated fixtures; **seed-backed lazy regeneration, worldgen seed cracking, robotics partial-sensor gate, human-rate cap, and idle easter eggs remain `Later / blocked`** on Steps 1–8 (`plan.md:54-55`, `scales.md:86`).

---

## Section 6: Cross-Cutting Gaps, Architectural Tensions & Completeness Matrix

Thorough cross-checking against live code and reports uncovers **nine critical gaps/tensions** where current code or specs do not yet guarantee the desired emergent properties:

1. **Fragmentation Across Unmerged Branches vs. "One Mechanism"**
   - *Verified state:* `main` (`b0d9ea3e`) flies flat ground in `~13 µs` (`resident.rs` + `solve.rs`) cleanly, while:
     - `wt/flight-ladder` (`c74c4a5c`) handles steps/fences/stairs offline, unmerged/unjudged;
     - `wt/port-fold` (`43f43730`) holds 99.65% (`32,252 / 32,366`) of 26.2 law, redstone, items, pearls, boats, elytra, pistons, `push_pair`, and `law::face::advance` (`155` lines), completely unwired to `plan`/`bot`;
     - `wt/build-4d` (`087b18ba`) compiles offline 4D build DAGs disconnected from `law` physics/travel/hotbars;
     - `wt/server-mind` (`36aa5f99`) runs `10,000` real-player bots using flat `StepCurve` parade formations disconnected from `es-1b-probe` `Form-P` and `port-fold`;
     - `jobs/es-1b-probe` validates `10^9` bots offline in a standalone crate.
   - *Impact:* None of Domains 2–4 and only pieces of Domains 1 and 5 can emerge end-to-end until these threads unify cleanly onto **One Mechanism** in `main` (`map/now.md:24-30`).

2. **Concrete Forward DFS (`wt/flight-ladder`) vs. Approved Monotone Backward Sets (`representation.md:179`)**
   - *Verified tension:* Approved architecture specifies backward `(min, +)` composition over law-threshold monotone intervals (`families / pieces / points / holds`) verified by `u2_pins_point_and_hold` (`learning.md:114`). However, `wt/flight-ladder` (`joints/cut.rs`) uses concrete full-`bits()` `JointStore::point` forward DFS with a `12,000`-tick budget (`off-spec` temporary expedient to unblock 4 tracks). If left as-is, `Kill tests 11, 14` (resolution-invariant few pieces, local `slide_want`) and continuous-velocity transfer (`R-retarget`, `J_rare` generalization) are not guaranteed.

3. **`solve.rs` `events.rs` vs. `law::face::advance` & Missing `O(1)` Uniform-Box Spine**
   - *Verified gap (`inbox.md:1428-1434`):* `law::face::advance` skips steady ticks cleanly (`~0.19 µs`), **but** remains `O(cells)` in corridor reads. Switching `solve.rs` directly from `events::advance_cycles` (`O(binades)`) to `face::advance` right now would **regress** the 1-million-block line from microseconds to `~1 s` (`inbox.md:1428`) until an **`O(1)` uniform-box spatial spine** (`16³` section + `4×4×4` subcell mask) lands first.

4. **Lossy `ModelId` Storage & Triple-World Duplication (`kit-edit-ledger-redteam` C1 & C2)**
   - *Verified bug:* `WorldSnapshot` (`engine/world/src/snapshot.rs:107`) packs `Box<[u16]>` `ModelId`s (`519/591`), **erasing exact block identity** (`stone == dirt == planks`, destroying break rates, silk touch, placement legality, and exact block-state restoration). Meanwhile `wt/port-fold` `SimWorld` (`sim.rs:561-562`) duplicates `WorldOverlay` (`model_id()` returns `None` on edited cells + sticky `|=` bools at `sim.rs:95-98` that never clear on undo) alongside `HashMap<BlockPos, SimCell>`. This blocks **all of Domain 2 (`R-lego`, ledger, slow media)** until unified onto canonical `StateId(u16)` + `WorldDelta` (`C1`, `C2`).

5. **Hardcoded `Kit::BareLegs` & `Goal::C` Violating Admissibility & Unused-Item Invariance (`C3`, `C4`, `C5`)**
   - *Verified bug:* `engine/plan/src/solve.rs:7-8` hardcodes `pub enum Kit { BareLegs }` and `engine/plan/src/goal.rs:7` hardcodes `pub const C: f64 = 19.902886 / 20.0;` (Kill test 16 violation whenever pearls/boats/elytra/Eff5 tools are carried). Furthermore, factored kit tables (`4.7 KB` body profile + `4.8 KB` tool tier, `kit-edit-ledger-redteam:40`) and per-word `Footprint::reads` mask hygiene (`C4`) are not implemented yet.

6. **Domain 3 Learning Loop & Domain 4 Swarm Economics Remain `Doc-Only`**
   - *Verified gap:* Despite detailed designs in `learning.md §2` and `scales.md §4.1–4.9`, the code on `main` does not yet build: (a) the single learning queue & background pointer-swap updater; (b) `Footprint::writes` & backward `writes ∩ failed_reads` retrosynthesis; (c) Pareto effect archive & `Smaller circles` envelope collapse; (d) Nerode byte-equal pattern table merging; (e) `Play` frontier puzzle mutator; (f) live `push_pair` planner producer, Ski-rental infrastructure rules, Pigou congestion pricing, Bertsekas task auction, `PriceField`s, `(min, +)` travel wiring into `build-4d`, or **any model for cascading edit-replan-edit instability** (`learning.md:85`: `"Edits break certificates, bots replan, bots edit | cascades | open: no model"`).

7. **PvP Contact Singularity Boundary & Premature Combat Schema Risk (`pvp-strategy:2046,2067`)**
   - *Verified tension:* Neutral 6-block melee with `12`-tick cooldowns breaks crystal `(word, count)` compression — forcing compute to follow contact duration rather than static terrain (`representation.md:188`). Attempting to absorb finite death prices or learned heuristics (`NNUE`) directly inside `Word` certificates corrupts exact `found/forbidden/searched/refused` invariants and `goal.md` ruin barriers (`pvp-strategy:2061,2070`). Meanwhile, designing Rust combat structs before Step 3 (`NeedsJoints`) and stick latency (`1.8–2.3s` `plan-track` CLI vs resident stick) clear risks pre-freezing unvalidated abstractions (`pvp-strategy:2062,2071`). Note also the intentional strategic tension: **Golden Bridge vs "No Golden Bridge / No Wussy Warrior"**, which reconciles only when keeping a taxed attrition corridor emerges conditionally from protecting the leader's survival asymmetry rather than hardcoding passivity.

8. **ES Integration Hazards & Vanilla Dispersed Chunk Wall (`es-1b-probe` §5 & `server.md`)**
   - *Verified hazards:* Upstream ES `shipmem` has 4 concrete traps (`es-1b-probe/report.md:112-125`): (1) `Enc::default()` panics for `n >= 4`; (2) `Program::hot` keys truncate bot ordinals at `u32` (`> 4.29B` in 1 stream); (3) `Program::split()` ignores unsealed `hot` edits and linearly scans `O(k)` over `ef`; (4) `belt.rs` workspace coupling. On the Minecraft server boundary (`server.md:97-107`), `10,000` dense/clustered bots already run live (`server-mind`), **but** `200+` lone dispersed bots hit Mojang's `PLAYER_SIMULATION` (`441` chunks/bot) CPU wall until worldtube tickets (`33/32/31`), `presence = tool`, and `GLM server-player-readers` are wired.

9. **Asymmetry in Sine Lookup, Missing Fixtures & Open Bugs**
   - *Verified physical/test blockers:* (a) Mojang's 65,536-entry `Mth.sin` breaks exact `90°`/mirror symmetry (`footprints.md:122-139`: `7` of `8` horizontal frames fail replay at `1e-17..3e-5` blocks, temporarily dropping rotation symmetry folding — `footprints.md:139`); (b) `51` of `91` recorded human tracks still lack chunk fixtures and `22` predate fixture (`now.md:10,56`); (c) `basis/contact` is still unwritten (`now.md:10`); (d) S38 sulfur-spike inventory deletion anomaly remains untraced (`now.md:64`); (e) live game jump-running visually shows slight local client-tick lag anomaly (`now.md:8`).

---

### Summary Readiness Matrix

| Domain | Count | Flown Live | Offline-Proven | Blocked / Spec-Only |
|---|---:|---:|---:|---:|
| **1. Movement & Representation** | 7 entries | 1 (`straight100/300/800`, partial `R-retarget` & shock re-anchor) | 3 (`u2` pins/holds, `flowers` offline, `P4` & `26.2` speed constants/elytra law) | 3 (+ merges of `wt/flight-ladder`, `O(1)` world spine, `R-brach` wiring, `R-far` / `thin-bridge`) |
| **2. Creativity, Tools & 4D Edits** | 7 entries | 0 | 2 (`WorldGait` 0-wakeup; `wt/port-fold` kernels & `place-reach-legality` pinned) | 5 (`C1/C2/C7` missing `StateId`/`WorldDelta`, `Place/Break` verbs, backward `writes ∩ reads`, ledger, wormholes) |
| **3. Learning & Statistical Invariants** | 7 entries | 0 | 0 (`walked`/surprise counters & anomaly bin exist, unused) | 7 (one queue, Pareto archive, Nerode merge, `Smaller circles`, Play, Kill tests 12/18–24, `width` rank) |
| **4. Swarm, Logistics & PvP** | 7 entries | 1 partial (`10k` flat `server-mind` Stage 3 w/o prices) | 2 partial (`build-4d` abstract DAGs; `push_pair` `500`-bot dense `Battle`) | 4+ (Bertsekas/Pigou/Ski-rental/PIC fields, mission ledger cleanup, PvP contact solver & two-body flight) |
| **5. ES `10⁹` Scale & Server Interface** | 7 entries | 1 partial (`10k` dense / `1k` `.mca` observer tiers + GPU instancer) | 4 (`10^9` `Form-P`/`seal()`, `172 µs` split, `0`-wakeup `WorldGait`, `<130 µs` pure observer, `Battle` dissolve) | 2 (+ unmerged `Form-P` <-> `server-mind`, dynamic `33/32/31` tickets, `presence=tool`, seed/sensor end goals) |
