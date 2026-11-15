# Docs Truth Audit (15 November 2026)

Scope audited: all 11 core docs (`AGENTS.md`, `docs\HANDOFF.md`, `docs\clan.md`, `docs\engine.md`, `docs\representation.md`, `docs\scales.md`, `docs\vocabulary.md`, `docs\learning.md`, `docs\footprints.md`, `docs\ladder.md`, `docs\server.md`) checked against `C:\Users\PASCAL\Desktop\path tracing` (`main`), `D:\path-tracing-wt\3.2d-3` (Slices A and B, unmerged to `main`), `D:\path-tracing-evidence\flight-20261114\`, and today's records (`docs\PRINCIPLES.md`, `docs\reference\julien-2026-11-{04,09,10}.md`, `docs\reference\postmortem-2026-11-14.md`, `jobs\3.2d-3\spec.md`, `jobs\3.2d-3-{A,B}-judge\last.md`, `jobs\zones-design\{design,redteam-opus}.md`, `jobs\inventory-map\report.md`, `jobs\port-backlog\report.md`, `jobs\spec-premises\report.md`).

Note on status separation: `wt/3.2d-3` contains Slices A and B (`R0` resident JNI DLL + `R1` canonical codec/interner) and is NOT merged to `main`. Claims true only on `wt/3.2d-3` are marked explicitly.

---

## Part 1: Per-Doc Audit (False, Stale, or Contradicted Statements)

### 1. `AGENTS.md` (60 / 60 lines)

1. **Antigravity Claude model slot (Opus claimed available)**
   - **(a) Doc (`AGENTS.md:28`):** `| The spare meter | Opus 5.5 and Sonnet 5.5 **through Antigravity** | the same minds on a meter that is not Julien's Anthropic plan: prefer it for judging and red-teaming, and keep the native plan as the reserve |`
   - **(b) Evidence:** `C:\Users\PASCAL\Desktop\path tracing\scripts\launch.ps1:23` refuses Opus on Antigravity (`if ($Who -eq 'opus') { throw 'Opus does not run on Antigravity (S115)...' }`); `C:\Users\PASCAL\Desktop\path tracing\scripts\gemini.ps1:30` matches; `C:\Users\PASCAL\Desktop\path tracing\docs\reference\julien-2026-11-10.md:325` (`S115: "antigravity is sonnet only you can use opus"`).
   - **(c) Replacement:** `| The spare meter | Sonnet 5.5 **through Antigravity** | a non-Anthropic-plan meter for Sonnet judging and red-teaming; native Opus/Fable stay on the native reserve |`

2. **Concurrency limits (`How many at once`)**
   - **(a) Doc (`AGENTS.md:37`):** `- **How many at once.** One **native** Claude sub-agent, plus up to four GPT Terra jobs, four at once only as fresh first prompts (S99; a resumed job runs with fewer), and only one GPT job while Astra runs, in dependency order, never on the same files. GLM has no slot limit (S47). Antigravity, per meter (S105): one Argon or two Hydrogen; one Opus or two Sonnet; adjust on a measured drain. `scripts/launch.ps1` enforces all of these. **The native Anthropic plan is the reserve and is exhausted last** (S102): Claude-shaped work goes to Antigravity's Claude meter first, and "fill the Anthropic context" means load the integrator with context, not drain its quota. Check what is already running before launching ([docs/clan.md](docs/clan.md) section 5).`
   - **(b) Evidence:** `C:\Users\PASCAL\Desktop\path tracing\scripts\launch.ps1:33` enforces `($astra -and $running.Count -ge 2) -or ($Who -eq 'astra' -and $running.Count -ge 2)` (two GPT jobs total when Astra runs, i.e., one Terra beside Astra; `launch.ps1:27` notes `"one GPT job beside Astra"`, consistent with `HANDOFF.md:14` and `clan.md:118`). `scripts\launch.ps1:23,57-78` sets Antigravity limits to `claude = 2` (Sonnet only, no Opus), `argon = 2` (one Argon on its weekly meter), and `gemini = 3` (up to three Hydrogen on the Gemini meter beside Argon). In addition, `julien-2026-11-10.md:257` (S102) reads `"give my antrhopic context filled up more than the others"`, so `"fill the Anthropic context"` is a paraphrase rather than a verbatim quote; and `HANDOFF.md:26` (S143) records the native Anthropic weekly plan at 97% (no native sub-agents while capped).
   - **(c) Replacement:** `- **How many at once.** Use `scripts/launch.ps1`; [docs/clan.md](docs/clan.md) sections 4–5 hold the measured limits (currently: ≤ 4 GPT, ≤ 2 total when Astra runs, no native sub-agents while native weekly is capped, ≤ 2 Antigravity Sonnet (no Opus), ≤ 1 Argon, ≤ 3 Hydrogen on the Gemini meter, no hard GLM slot limit). Native Anthropic quota is the reserve; put heavy context in the integrator rather than draining sub-agent quota (S102).`

---

### 2. `docs\HANDOFF.md` (120 / 120 lines)

1. **Scope of `julien-2026-11-10.md` and missing register rows**
   - **(a) Doc (`docs\HANDOFF.md:8`):** `[reference/julien-2026-11-10.md](reference/julien-2026-11-10.md) (S64–S105, X7–X16; S103 was never recorded verbatim). New turns go there verbatim, with a register row in PRINCIPLES.`
   - **(a) Doc (`docs\HANDOFF.md:27`):** `the uncited turns got register rows`
   - **(b) Evidence:** `C:\Users\PASCAL\Desktop\path tracing\docs\reference\julien-2026-11-10.md:1-495` covers **S64–S146** and X7–X16, while `C:\Users\PASCAL\Desktop\path tracing\docs\PRINCIPLES.md:320` stops at **S139**, leaving `S140–S146` (`julien-2026-11-10.md:461-495`) without register rows.
   - **(c) Replacement:** For line 8: `[reference/julien-2026-11-10.md](reference/julien-2026-11-10.md) (S64–S146, X7–X16; S103 was never recorded verbatim). New turns go there verbatim, with register rows in PRINCIPLES (S140–S146 still owed).` For line 27: replace `the uncited turns got register rows` with `register rows added through S139 (S140–S146 still owed)`.

2. **Antigravity slot rule in `The clan in three commands`**
   - **(a) Doc (`docs\HANDOFF.md:14`):** `on Antigravity (S105) one Argon or two Hydrogen, and one Opus or two Sonnet`
   - **(b) Evidence:** `scripts\launch.ps1:23,57-78` (`claude` limit 2, Sonnet only / no Opus; `argon` limit 2 = 1 Argon; `gemini` limit 3 = up to 3 Hydrogen beside Argon).
   - **(c) Replacement:** `on Antigravity (S115, S125) two Sonnet (no Opus), one Argon, and up to three Hydrogen on the Gemini meter`

3. **`Start here` notes `Hydrogen spec-premises (rerun)` as still open/active without noting `report.md` exists**
   - **(a) Doc (`docs\HANDOFF.md:23`):** `Hydrogen spec-premises (rerun)`
   - **(b) Evidence:** `C:\Users\PASCAL\Desktop\path tracing\jobs\spec-premises\report.md` already exists on disk (as does `jobs\inventory-map\report.md` and `jobs\port-backlog\report.md`). Also line 22 (`Next: release read_cost on B against main (judge B, finding 1), then Terra P1 (T1) on wt/3.2d-3`) is consistent with `jobs\3.2d-3-B-judge\last.md` (`read_cost` ignored in `wt/3.2d-3`).
   - **(c) Replacement:** State `Hydrogen spec-premises done (jobs/spec-premises/report.md)` or remove from the active in-flight list if adjudicated.

4. **Entire `## Now (14 November, S104–S106)` section (`lines 30–54`) is stale and self-contradictory**
   - **(a) Doc (`docs\HANDOFF.md:32`):** `Nothing has been played in the game.`
   - **(b) Evidence:** Contradicted by `docs\HANDOFF.md:45` (`THE FIRST FLIGHT FLEW`, 18 tracks + 3 straight tracks) and `D:\path-tracing-evidence\flight-20261114\live.txt` and `live-straight.txt`.
   - **(a) Doc (`docs\HANDOFF.md:38`):** `anomaly-1 step-up | Hydrogen (its first job: the bake-off row)`
   - **(b) Evidence:** Hydrogen has since completed `anomaly-1`, `inventory-map`, `port-backlog`, and `spec-premises` (`docs\HANDOFF.md:20,23,27`).
   - **(a) Doc (`docs\HANDOFF.md:39`):** `GLM | queue (6 port batches) + port-peaceful-regen, port-boat-turn-trace`
   - **(b) Evidence:** Superseded by `C:\Users\PASCAL\Desktop\path tracing\lab\queue.txt` (11 lenses on 15 November plus `*-zai-limit`) and `jobs\port-backlog\report.md`.
   - **(a) Doc (`docs\HANDOFF.md:41`):** `Main tests: 149 passed, 11 ignored.`
   - **(b) Evidence:** Reflects pre-`3.2d-3` `main`; on `D:\path-tracing-wt\3.2d-3`, `jobs\3.2d-3-A-judge\last.md` reports `191 passed / 18 ignored` and `jobs\3.2d-3-B-judge\last.md` reports `200 passed / 18 ignored` (unmerged to `main`).
   - **(a) Doc (`docs\HANDOFF.md:45`):** `Job 3.3h ... launches itself at 15:17` and mentions the `butoonpressureplates` finding.
   - **(b) Evidence:** `15:17` on 14 November is past (`docs\reference\postmortem-2026-11-14.md:22` notes 3.3h blocked/wasted). On `wt/3.2d-3` Slice B, `codec.rs` fixes button/pressure-plate equivalence with air per `jobs\3.2d-3\spec.md:109` (still open on `main`). Note also `tracks.txt:19-21` has bounds `292 -64 -496 308 -58 {-300,-80,476}`, so `x=300` is the centerline inside `[292, 308]`.
   - **(a) Doc (`docs\HANDOFF.md:51`):** `"Vanilla in behaviour, not pure vanilla."` (line 51) and **line 53** (`Where to test ... (S97, not yet in ladder.md)`).
   - **(b) Evidence:** Line 51 slightly alters S55 (`julien-2026-11-09.md:160`, see Part 4). Line 53's S97 test axes should live in `docs\ladder.md`.
   - **(c) Replacement for `lines 30–54`:** Replace `## Now (14 November, S104–S106)` with current 15 November state: `First flight (14 Nov, wt/3.3g-play): 8/18 tracks completed in game, straight100/300/800 hit 2.2–30.5s cold build; wt/3.2d-3 has Slices A–B built (200 passed, 18 ignored per Judge B; unmerged to main). Keep active items: 1.4d boat turns unhooked, S107 Idle CPU priority, and S97 test axes (moved to ladder.md).`

5. **Anomaly 2 impact note (`bears on 3.3c's start state`)**
   - **(a) Doc (`docs\HANDOFF.md:58`):** `server_regeneration never runs inside tick_movement (bears on 3.3c's start state).`
   - **(b) Evidence:** `3.2d-3` Slice A (`D:\path-tracing-wt\3.2d-3\engine\native\src\resident.rs`, `PlayerSnapshot.java`) initializes state via `from_observed_bits`, and `docs\HANDOFF.md:20` / `jobs\3.2d-3\spec.md:63-64` pin Peaceful scope where assuming regen phase is forbidden (`movement.rs:1253` port gap itself remains open on `main`).
   - **(c) Replacement:** `server_regeneration is not called inside tick_movement (main engine/law/src/movement.rs:1253; open port gap, though Peaceful 3.2d-3 initializes from observed bits without assuming regen phase).`

6. **Roadmap Phase 2 (`2. The first flight (slice A)`)**
   - **(a) Doc (`docs\HANDOFF.md:74`):** `| 2. The first flight (slice A) | One bot in the game on his 18 tracks, plus straight100/300/800 and the hard track (A gold, B diamond; L1a-c, L2a-c, R-far, R-retarget, R-brach). The planner (3.2d: sets backward, the point forward, one replay), the playback (3.3c), the executor, the proof-bearing bridge. |`
   - **(b) Evidence:** (1) The 18 tracks and `straight100/300/800` already flew on 14 November (`live.txt`, `live-straight.txt`); (2) `jobs\zones-design\design.md` and `jobs\3.2d-3\spec.md` replaced backward-sets planning for plain paths with resident `R0–R4` (`D:\path-tracing-wt\3.2d-3` Slices A–B already built), while `3.2d-3/spec.md:106-117` uses `"Slices A–K"` for its build sequence, colliding with `"slice A/B/C"` in `HANDOFF.md:74-77`.
   - **(c) Replacement:** `| 2. First flight & resident 3.2d-3 | 18 tracks & straight100/300/800 flown 14 Nov; finishing 3.2d-3 (JNI resident R0–R4, Slices A–K) toward hardtrack (A gold -107 -60 -107, B diamond -84 -25 -88) and L1a-c, L2a-c, R-far, R-retarget, R-brach. |`

---

### 3. `docs\clan.md` (176 / 200 lines)

1. **Hydrogen status claimed untested / zero survival (`lines 85, 173, 175`)**
   - **(a) Doc (`docs\clan.md:85`):** `| wide cheap lanes | Hydrogen | unknown | no run has survived its permission denials yet |`
   - **(a) Doc (`docs\clan.md:173`):** `2. **Hydrogen has never run a real job here.**`
   - **(a) Doc (`docs\clan.md:175`):** `Hydrogen's first job is anomaly-1-stepup (14 November); its result settles whether the lane earns real jobs or stays backup.`
   - **(b) Evidence:** `docs\HANDOFF.md:18,20,23,27`, `scripts\test-antigravity.ps1`, and completed artifacts `jobs\inventory-map\report.md`, `jobs\port-backlog\report.md`, `jobs\spec-premises\report.md` prove Hydrogen passed smoke tests and finished multiple real jobs.
   - **(c) Replacement:** Update line 85 status to `working (absolute Windows paths, incremental report.md writes)` and delete/rewrite lines 173–175 to reflect that Hydrogen passed `test-antigravity.ps1` and delivered `anomaly-1-stepup`, `inventory-map`, `port-backlog`, and `spec-premises`.

2. **Conflicting Opus & Antigravity slot rules (`lines 82, 119–120, 124, 142`)**
   - **(a) Doc (`docs\clan.md:119`):** `| Claude via Antigravity | sonnet / opus | **≤ 2 Sonnet or 1 Opus** (never both) | ... |`
   - **(a) Doc (`docs\clan.md:120`):** `| Google via Antigravity | argon / hydrogen | **≤ 1 Argon or 2 Hydrogen** | ... |`
   - **(a) Doc (`docs\clan.md:124`):** `Uses claude-{opus,sonnet}-5-5-thinking and gemini-3.1-pro-{high,low};`
   - **(b) Evidence:** Directly contradicts line 82 in the same file (`S115: Antigravity is Sonnet only`) and `scripts\launch.ps1:23,57-78` (Opus rejected on Antigravity; `claude = 2`, `argon = 2` -> 1 Argon, `gemini = 3` -> up to 3 Hydrogen on the separate Gemini meter).
   - **(c) Replacement:** Line 119: `| Claude via Antigravity | sonnet | **≤ 2 Sonnet** (no Opus, S115) | ... |`. Line 120: `| Google via Antigravity | argon / hydrogen | **≤ 1 Argon + ≤ 3 Hydrogen** (separate Gemini meter, S125) | ... |`. Line 124: `Uses claude-sonnet-5-5-thinking, argon, and gemini-3.1-pro-...`.

3. **Native Opus / Sonnet routing in section 6 (`line 142`)**
   - **(a) Doc (`docs\clan.md:142`):** `design Astra, build Terra, red-team native Opus, judge native Sonnet`
   - **(b) Evidence:** `docs\HANDOFF.md:26` (`S143`, `julien-2026-11-10.md:475`): native Anthropic weekly plan hit 97% — no native sub-agents; judging and red-teaming route to Antigravity Sonnet or cross-family Terra.
   - **(c) Replacement:** `design Astra, build Terra, red-team / judge via Antigravity Sonnet (or Terra for Claude work) while native Anthropic weekly reserve is capped (S143).`

4. **Antigravity traps & pre-proof checklist (`lines 124, 164–169`)**
   - **(a) Doc (`docs\clan.md:124`):** `an empty report on one denied read_file is common, so inline what a job must read.`
   - **(a) Doc (`docs\clan.md:164–169`):** `Three things must be proved by an actual run before anything depends on this` (lists ArgOn/Antigravity smoke checks as unverified).
   - **(b) Evidence:** `scripts\gemini.ps1:50-53,74,86-97` now pre-grants `Root`, `D:\path-tracing-wt`, `D:\path-tracing-evidence`, and `D:\path-tracing-lab` in all path forms and mandates incremental writes to `report.md` every 3–4 files (still requiring absolute drive-letter paths without `file:///`). `scripts\launch.ps1:63-67` enforces `scripts\test-antigravity.ps1` before every Antigravity launch (`postmortem-2026-11-14.md:58-60`).
   - **(c) Replacement:** Update line 124 to note `gemini.ps1 pre-allows repo and D:\ workspaces via absolute drive paths (no relative or file:/// paths) and enforces incremental report writes every 3–4 reads`. Update lines 164–169 to mark item 1 (`test-antigravity.ps1`) as an active pre-launch gate and move to verified operational checks.

---

### 4. `docs\engine.md` (183 / 200 lines)

1. **`server-driven` fields claimed "never simulated"**
   - **(a) Doc (`docs\engine.md:37`):** `| server-driven | health, food, saturation, experience, inventory counts, server ticks, effects | recorded per tick and read by the law as input, never simulated |`
   - **(b) Evidence:** `C:\Users\PASCAL\Desktop\path tracing\engine\law\src\movement.rs:202` simulates `ticks_frozen = (ticks_frozen + 1).min(140)` inside the law, and `attributes.rs:51-54` + `movement.rs:217-233` simulate `sprinting` and `sprint_speed_modifier`. `jobs\3.2d-3\spec.md:80-81` clarifies that server-driven fields are excluded from `predicted == observed` replay failure while still read as guards.
   - **(c) Replacement:** `| server-driven | health, food, saturation, experience, inventory counts, server ticks, effects | read per tick as guard inputs; excluded from predicted-state equality checks (only law-owned transitions like freeze ticks and sprint state advance in law) |`

2. **Retired 12-phase numbers (`lines 39, 70, 71`)**
   - **(a) Doc (`docs\engine.md:39,70,71`):** `Phase 11` (line 39), `Phase 7` (line 70), `Phase 8` (line 71).
   - **(b) Evidence:** `docs\HANDOFF.md:72-78` replaced the 12-phase roadmap with Phases `0..6` (e.g., Phase 6 is Elytra / R10 / full game).
   - **(c) Replacement:** Update to current phase names/numbers (`Phase 5` server / `Phase 6` late-phase capabilities per `HANDOFF.md:72-78`, or `defer to later phase`).

3. **Unachieved latency guarantee written as present contract (`line 108`)**
   - **(a) Doc (`docs\engine.md:108`):** `- **anytime:** a first plan within one tick of the goal arriving, refined while the bot moves;`
   - **(b) Evidence:** `D:\path-tracing-evidence\flight-20261114\live.txt` took `690–1293 ms`; `live-straight.txt` took `2.2–30.5 s` cold; on `D:\path-tracing-wt\3.2d-3`, `engine\native\src\resident.rs:181` returns `improve: false`.
   - **(c) Replacement:** `- **anytime (target guarantee):** first plan within one tick once resident R1/R2 and background refine are built (current main flight cold time is 0.69–30.5s; wt/3.2d-3 Slice A returns improve: false);`

4. **GPU Vulkan split written in present tense as running (`line 118`)**
   - **(a) Doc (`docs\engine.md:118`):** `The GPU runs through Vulkan. It takes integer min-plus over ticks, distance-and-clearance fields, and bitboard light cones. Everything else runs on CPU.`
   - **(b) Evidence:** `jobs\inventory-map\report.md` and `docs\HANDOFF.md:27,98` confirm zero GPU code and zero threads in `engine/`; `docs\representation.md:142` and `jobs\3.2d-3\spec.md:138` state GPU work waits for a measured CPU crossover.
   - **(c) Replacement:** `The planned GPU path uses Vulkan (not yet built; zero GPU code in engine today; waits on measured CPU crossover per representation.md) for integer min-plus over ticks, distance/clearance fields, and bitboard light cones.`

5. **`aiStep` vertical velocity zeroing condition (`line 140`)**
   - **(a) Doc (`docs\engine.md:140`):** `Each vertical component is zeroed when below 0.003.`
   - **(b) Evidence:** `C:\Users\PASCAL\Desktop\path tracing\engine\law\src\movement.rs:171` checks `velocity.y.abs() < 0.003` (only `vy`, and on absolute magnitude `< 0.003`, not signed `vy < 0.003`).
   - **(c) Replacement:** `Vertical velocity is zeroed when |vy| < 0.003 (movement.rs:171).`

6. **`FoodConstants` sprinting rule (`line 150`)**
   - **(a) Doc (`docs\engine.md:150`):** `Sprinting needs food above 6.`
   - **(b) Evidence:** `jobs\3.2d-3\spec.md:63` clarifies that on Peaceful difficulty the `food_level > 6` sprint restriction is void. Meanwhile `main\engine\law\src\movement.rs:219,225` and `main\engine\words\src\class.rs:35` check `food_level > 6` unconditionally without checking `difficulty != 0` (`jobs\spec-premises\report.md`).
   - **(c) Replacement:** `Sprinting needs food above 6 on Easy/Normal/Hard; on Peaceful that restriction is void (spec-premises open port gap: movement.rs:219,225 and class.rs:35 currently omit the difficulty check).`

---

### 5. `docs\representation.md` (167 / 200 lines)

1. **Shared per-cycle word cache claimed live (`line 44`)**
   - **(a) Doc (`docs\representation.md:44`):** `the word's per-cycle increments live once in the shared cache, as in ES's cycle memo.`
   - **(b) Evidence:** On `main`, `engine\plan\src\flight.rs:134` allocates a fresh `ProposalTable::default()` per click and `arith::Cycle` lives only per `search.rs` call (`jobs\zones-design\design.md` D3, D5, D6). On `wt/3.2d-3`, `engine\native\src\resident.rs:126` keeps one persistent `ProposalTable`, awaiting Slice C `GeneratorStore`.
   - **(c) Replacement:** `the word's per-cycle increments belong in a persistent shared cache (ES cycle memo; on main today ProposalTable is allocated fresh per click and arith::Cycle per search call; wt/3.2d-3 keeps ProposalTable resident pending Slice C GeneratorStore).`

2. **Retired phase numbers (`lines 75, 164`)**
   - **(a) Doc (`docs\representation.md:75,164`):** `(Phase 3.1)` (line 75), `(Phase 2.1)` (line 164).
   - **(b) Evidence:** `docs\HANDOFF.md:72-78` uses Phases `0..6`.
   - **(c) Replacement:** Reference the current roadmap phase (`HANDOFF.md` Phase 3/5) or remove retired sub-phase labels.

3. **Coarse tier claimed "Found, not given; filled after measuring" (`line 81`)**
   - **(a) Doc (`docs\representation.md:81`):** `A coarse tier is reserved: model classes merged where replay shows they behave the same ... Found, not given; filled after measuring.`
   - **(b) Evidence:** Superseded by `jobs\zones-design\design.md` §3.1 and `jobs\3.2d-3\spec.md` §2 (implemented on `wt/3.2d-3` Slice B `codec.rs`): role/block classes merge deterministically from the law's `(verb/query mask, canonical answer bytes)` projection rather than post-hoc empirical replay merging.
   - **(c) Replacement:** `Role classes collapse block states deterministically from the law's (query mask, canonical answer bytes) equivalence (built on wt/3.2d-3 Slice B codec.rs), rather than empirically merging after replay.`

4. **"Health and food regenerate: they are constraints, not prices." (`line 149`)**
   - **(a) Doc (`docs\representation.md:149`):** `Health and food regenerate: they are constraints, not prices.`
   - **(b) Evidence:** Contradicted in full generality by `docs\PRINCIPLES.md:20` (S6), `docs\scales.md:100`, and `jobs\zones-design\redteam-opus.md:13-22` (on non-Peaceful 1M-block runs bare-kit hunger exhausts after ~1.4k blocks and gates sprint; on Peaceful food does not drain, and in `main` `movement.rs:1253` `server_regeneration` is not called in `tick_movement`, Anomaly 2).
   - **(c) Replacement:** `On Peaceful, food does not drain sprint; on survival difficulties with finite food, hunger and health act as regenerable constraints or resource limits (and on main Anomaly 2 leaves server_regeneration unhooked inside tick_movement).`

5. **Unbuilt design sections 6–9 (`lines 123–157`)**
   - **(a) Doc (`docs\representation.md:123–157`):** Sections 6–9 describe obligations/repair, two evaluation orders, R&D trials, building/tools, and server boundary in mixed present tense.
   - **(b) Evidence:** `jobs\inventory-map\report.md` verifies zero obligation fields in `engine/words/src/segment.rs`, zero local-repair engine code, zero GPU code, and zero multi-bot server implementation on `main`.
   - **(c) Replacement:** Add one brief banner under `§6`–`§9`: `(sections 6–9 describe target representation; unbuilt on main)`.

---

### 6. `docs\scales.md` (145 / 200 lines)

1. **Pace-layer change order (`line 15`)**
   - **(a) Doc (`docs\scales.md:15`):** `Ordered by how often they change: law < vocabulary < world and medium < fields < plans < controls.`
   - **(b) Evidence:** Directly contradicted by `docs\learning.md:56` (`The law is today the fastest-changing layer (a port delta in every GLM batch), which breaks the order the draft assumed.`).
   - **(c) Replacement:** `Intended change order once the law settles: law < vocabulary < world and medium < fields < plans < controls (today law changes fastest as port deltas land, per learning.md).`

2. **Stale `§3 Ladder of scales` Status column (`lines 25–27`) & Mojang `region 512` framing (`line 28`)**
   - **(a) Doc (`docs\scales.md:25-28`):** Scale `1` status `built, L0 in flight`; Scale `2` status `3.2a`; Scale `3` status `3.2b; 4.2`; line 28 cites `(view 192, a region 512)`.
   - **(b) Evidence:** `L0` already froze and ran to tick 111 (`wt/l0-veto` powder snow, `HANDOFF.md:47`, `postmortem-2026-11-14.md:30`); `3.2a` (`words`) and `3.2b` (`plan`) already exist on `main`; `wt/3.3g-play` flew first flight; and `wt/3.2d-3` Slices A–B are built (`3.2d-3/spec.md`). Also `S132` (`julien-2026-11-10.md:423`) and `jobs\3.2d-3\spec.md:137` clarify that planner zones must emerge from octree/class uniformity — never fixed 512 (which is only Mojang's `.mca` disk region size).
   - **(c) Replacement:** Update status cells to `1 | built; L0 ran to tick 111 (powder-snow port fix)`, `2 | 3.2a built on main; R1 codec on wt/3.2d-3`, `3 | 3.2b flew (FAIL on claims); 3.2d-3 in progress`. Clarify line 28: `(view 192; .mca file region 512, not a planner zone size — planner zones emerge from uniformity, S132)`.

3. **`writes` field claimed present in the word record (`line 41`)**
   - **(a) Doc (`docs\scales.md:41`):** `record what each item use writes (writes exists in the word record)`
   - **(b) Evidence:** `C:\Users\PASCAL\Desktop\path tracing\engine\words\src\segment.rs:41-49` defines `Segment { ticks, exit, state, footprint, uses, guards }` — there are NO `writes`, `needs`, or `obligations` fields in code (they exist only in `docs\vocabulary.md:82-86`).
   - **(c) Replacement:** `record what each item use writes (writes is specified in docs/vocabulary.md §8; unbuilt in engine/words/src/segment.rs today)`

4. **Retired `Phase 7` label (`line 47`)**
   - **(a) Doc (`docs\scales.md:47`):** `It pulls "LLM-proposed words" from Phase 7 to the lab`
   - **(b) Evidence:** `docs\HANDOFF.md:78` places Elytra / full game / learning R&D in current `Phase 6` (`3.2a` already built `words`).
   - **(c) Replacement:** `It pulls "LLM-proposed words" from late-phase backlog (Phase 6 / R&D) to the lab`

5. **Tables claimed to exist "only at constrictions" (`line 67`)**
   - **(a) Doc (`docs\scales.md:67`):** `Tables exist only at constrictions (passes, doors, portals).`
   - **(b) Evidence:** Superseded by `jobs\zones-design\design.md` §3.3 and `jobs\3.2d-3\spec.md` §3 (`GeneratorStore[LabelId].word(WordId)` compiles kernels for homogeneous zones too, with portals at emerging zone boundaries).
   - **(c) Replacement:** `Tables/generators hold one entry per uniform zone label (GeneratorStore[LabelId]), plus portals at emerging zone boundaries, passes, and doors.`

6. **Always-on invariant `ellipse never changes a plan` (`line 114`) & slice naming (`line 117`)**
   - **(a) Doc (`docs\scales.md:114`):** `a block outside the bracket's ellipse never changes a plan`
   - **(a) Doc (`docs\scales.md:117`):** `The three slices in HANDOFF's roadmap are the fastest answers`
   - **(b) Evidence:** `jobs\zones-design\redteam-opus.md:43-45` showed that before a valid upper bound exists the ellipse is unbounded, and at 1M blocks even `r = 1.1` spans ~`10^11 blocks^2`; `jobs\3.2d-3\spec.md:137` explicitly bans unbounded light-cone ellipses in favor of opening angle `θ`. Meanwhile `HANDOFF.md:74-77` uses `slice A/B/C` across Phases 2/4/5 while `3.2d-3/spec.md:104-117` uses `Slices A–K` for `3.2d-3`.
   - **(c) Replacement:** Line 114: replace the ellipse invariant with `a block outside the bounded opening angle θ (or after UB, the closed bracket) never changes a plan`. Line 117: clarify `the three roadmap phases in HANDOFF.md (now broken into Slices A–K inside 3.2d-3)`.

---

### 7. `docs\vocabulary.md` (117 / 200 lines)

1. **`BodyBlock` rule (`line 41`) & unremoved `BodyBlock` cuts/choices in code**
   - **(a) Doc (`docs\vocabulary.md:1,41`):** Header says `(version 2, 11 November)` while line 41 already states `(a segment ends here; the planner does not choose here, section 12.4, S101)` and line 117 says `It lands with job 3.2d.`
   - **(b) Evidence:** On `main`, `engine\words\src\segment.rs:81` still terminates segments on `BodyBlock`, and `engine\plan\src\search.rs` chooses per-segment at `BodyBlock` (neither `main` nor `wt/3.2d-3` Slices A–B has landed section 12.4; `jobs\3.2d-3\spec.md` schedules `F` work in Slice H). Also the job is `3.2d-3`, not `3.2d`.
   - **(c) Replacement:** Clarify on line 41/117: `(design rule per S101, scheduled for 3.2d-3 Slice H; main segment.rs:81 and search.rs still cut and choose at BodyBlock today)`.

2. **Lookup timing & `by_model` baseline (`line 19`)**
   - **(a) Doc (`docs\vocabulary.md:19`):** `and a lookup about 5 (job 1.6, measured)`
   - **(b) Evidence:** `docs\footprints.md:184-185` (`key_cost.rs` on `main`) measures lookup at `2.2` / `4.4 µs`, `docs\learning.md:60` calls ProposalTable lookup `49× its floor`, and `wt/3.2d-3` Slice B (`jobs\3.2d-3-B-judge\last.md` finding 1) removed `PatternTable::cell_id`'s `by_model` dense array cache with `read_cost` tests ignored.
   - **(c) Replacement:** `(job 1.6 / key_cost.rs measured lookup 2.2–4.4 µs on main; wt/3.2d-3 Slice B changed cell_id to byte-verified interning with read_cost still awaiting release)`.

3. **64-yaw single-aim grid (`line 55`)**
   - **(a) Doc (`docs\vocabulary.md:55`):** `Aims are sine-table indices on a grid of 64 yaws`
   - **(b) Evidence:** `jobs\zones-design\redteam-opus.md:25-29` and `jobs\3.2d-3\spec.md:55-60` show a 64-yaw grid misses distant targets past ~17k blocks; plain path uses two adjacent law-table indices (`step`). Furthermore, `main\engine\plan\src\reach.rs:42-45` does not use the sine table yet (it computes `f64::to_radians().cos()/sin()`).
   - **(c) Replacement:** `Aims on F (legacy) use a 64-yaw grid (note main reach.rs:42-45 still calls cos/sin directly); long-range plain-path steering per 3.2d-3 §2 uses two adjacent 65,536-entry law-table indices rather than a coarse 64-yaw grid.`

4. **Unbuilt `Width::clearance` (`line 68`) and `needs / writes / obligations` (`lines 83–85`)**
   - **(a) Doc (`docs\vocabulary.md:68,83-85`):** Line 68 lists `clearance` as read from collision code during replay; lines 83–85 list `needs`, `writes`, and `obligations` in `§8 The record`.
   - **(b) Evidence:** `C:\Users\PASCAL\Desktop\path tracing\engine\words\src\segment.rs:41-49,117-118` explicitly comments `"Clearance is not built: it needs a definition first"` (`Width` has only `{ early: bool, late: bool }`), and `Segment` has no `needs`, `writes`, or `obligations` fields.
   - **(c) Replacement:** Tag `clearance`, `needs`, `writes`, and `obligations` as `[design — not yet in engine/words/src/segment.rs]`.

5. **Legacy `§10 Search` and `§11 First flight` (`lines 97–99, 109`)**
   - **(a) Doc (`docs\vocabulary.md:97-99,109`):** `§10` describes `3.2b` per-block-crossing A* (`54 × 64`); line 109 states `The first flight: the bot races Julien's 18 tracks on certified chains, with his T as the bar.`
   - **(b) Evidence:** The first flight ran on 14 November (`D:\path-tracing-evidence\flight-20261114\live.txt`: 8/18 finished, 10 `NO SENTENCE`); `docs\learning.md:15` reports `3.2b` A* alone finds `0/18`; `jobs\3.2d-3\spec.md` restricts legacy `F` (`search.rs`) to exceptional/non-plain spans in Slice H.
   - **(c) Replacement:** Mark `§10` as legacy `F` search (to be scoped to non-plain spans in `3.2d-3` Slice H) and update `§11` to past tense noting the 14 November flight (8/18 completed).

---

### 8. `docs\learning.md` (76 / 200 lines)

1. **Claim that nothing here is built before first flight (`line 3`)**
   - **(a) Doc (`docs\learning.md:3`):** `Nothing here is built before the first flight says where the bots fail.`
   - **(b) Evidence:** The first flight ran on 14 November (`D:\path-tracing-evidence\flight-20261114\live.txt` and `live-straight.txt`, 8/18 passed, 10 failed, `straight*` hit 2.2–30.5s), triggering `zones-design` and `3.2d-3` (Slices A–B built on `wt/3.2d-3`).
   - **(c) Replacement:** `First flight (14 Nov) showed the gaps (8/18 passed, 10 NO SENTENCE, straight* O(L²) cold time); 3.2d-3 (Slices A–B on wt/3.2d-3) addresses the cold-start and straight-path failures.`

2. **"sets backward, the point forward, the slide, one replay" (`line 15`)**
   - **(a) Doc (`docs\learning.md:15`):** `The intent is that acting is: sets backward, the point forward, the slide, one replay.`
   - **(b) Evidence:** `jobs\zones-design\design.md` and `jobs\3.2d-3\spec.md:135,140` supersede this for `3.2d-3`: plain/uniform zones use closed-form generator lookup from `LabelId -> GeneratorStore` (`R1/R2`) without building backward cell sets; `slide` (nudging/rejoin) is deferred out of `3.2d-3` to its own design door; and full backward solver `I1/I2` also moved to a separate future door (`3.2d-3/spec.md:135`).
   - **(c) Replacement:** `In 3.2d-3, plain/homogeneous spans act by LabelId -> GeneratorStore lookup and closed-form stepping without backward sets; full backward solver (I1/I2) and nudging/slide wait for separate design doors (3.2d-3/spec.md).`

3. **Stale future tense for `3.2b` (`lines 30, 64`)**
   - **(a) Doc (`docs\learning.md:64`):** `It is job 3.2b, already on the critical path.`
   - **(b) Evidence:** `3.2b` is built on `main` and flew on 14 November (`live.txt`: 8/18; `HANDOFF.md:32` judged FAIL on `3.2b` claims).
   - **(c) Replacement:** `Job 3.2b already landed and flew on 14 November (8/18 on live.txt; judged FAIL on its own latency claims and exposed the O(L²) cold-plan bug on straight*).`

---

### 9. `docs\footprints.md` (200 / 200 lines)

1. **`Footprint` struct omits fourth field `state_reads` (`line 19` and `lines 29–35`)**
   - **(a) Doc (`docs\footprints.md:19`):** `- Footprint { reads, swept, environment }, with contains(p) and extend(&other).`
   - **(b) Evidence:** `C:\Users\PASCAL\Desktop\path tracing\engine\law\src\footprint.rs:37-46` defines `pub struct Footprint` with **four** fields: `reads`, `swept`, `environment`, AND **`state_reads: StateReads`**, plus `read_state` / `write_state` hooks (`footprint.rs:245-251`) omitted from lines 29–35.
   - **(c) Replacement:** `- Footprint { reads, swept, environment, state_reads }, with contains(p) and extend(&other), plus read_state / write_state hooks on footprint.rs:245-251.`

2. **`§3 cell_digest` & `PatternTable` (`lines 60–97`) and `§6 Open` (`line 198`) vs `wt/3.2d-3` Slice B**
   - **(a) Doc (`docs\footprints.md:60-97,198`):** Describes unkeyed 128-bit FNV digesting and `PatternTable::cell_id` fast `by_model` dense array cache (`lines 94–97`); line 198 lists as Open: `Unkeyed 128-bit FNV: a collision gives one content a wrong id; storing contents would rule it out.` Also line 103 references retired `(Phase 3)`.
   - **(b) Evidence:** On `main` (`engine\law\src\pattern.rs`), lines 60–97 and 198 are still true. On `wt/3.2d-3` (Slice B, unmerged to `main`), `codec.rs` + `pattern.rs` replaced unkeyed-only matching with `Canonical` (`digest + bytes`) and `codec::Interner` (digest match compares canonical bytes, closing the line 198 collision risk with forced-collision tests and 13/13 mutants caught per `jobs\3.2d-3-B-judge\last.md`), while removing `PatternTable`'s `by_model` dense array cache (`Judge B finding 1`, `read_cost` tests ignored).
   - **(c) Replacement:** Mark `§3` (`lines 60–97`) and `§5` timings (`lines 184–185`) as `main (v2)`; note `wt/3.2d-3 Slice B (unmerged) adds byte-verified Canonical interning in codec.rs (closing the FNV collision item in §6) and drops the by_model dense cache pending read_cost verification`. Replace `(Phase 3)` on line 103 with `3.2d-3 Slice C/H`.

---

### 10. `docs\ladder.md` (117 / 200 lines)

1. **`Scoreboard` end-of-`T` definition contradicts `Goal::touched` (`line 7`)**
   - **(a) Doc (`docs\ladder.md:7`):** `T ends when the bot stands in the goal cell with everything restored.`
   - **(b) Evidence:** Contradicts line 16 of the same file (`to the tick it touches B, measured by the same detector`) and `C:\Users\PASCAL\Desktop\path tracing\engine\plan\src\goal.rs:9-21` (`Goal::touched` tests whether the body's before/after-tick bounding-box hull intersects `B`'s cell grown by `1e-3`, not standing still in `B`).
   - **(c) Replacement:** `T ends on the tick the bot touches goal cell B (Goal::touched bounding-box hull within 1e-3) with everything restored.`

2. **Wrong turn citation on Swarm (`line 25`)**
   - **(a) Doc (`docs\ladder.md:25`):** `hundreds of bots (S17)`
   - **(b) Evidence:** `docs\reference\julien-2026-11-04.md:73-79`: `S17` is the brachistochrone (`lines 73–75`), while hundreds of bots is **S18** (`lines 77–79`).
   - **(c) Replacement:** `hundreds of bots (S18)`

3. **Retired 12-phase roadmap numbers (`lines 25, 99–105`)**
   - **(a) Doc (`docs\ladder.md:25,99-105`):** References `Phase 10`, `Phase 3`, `Phase 5`, `Phase 6`, `Phase 7`, `Phase 8`, and `Phase 11`.
   - **(b) Evidence:** `docs\HANDOFF.md:72-78` now uses Phases `0..6` (so labeling `R4/R5` as `Phase 3` and `R6` as `Phase 5` disagrees with `HANDOFF.md:75-77`, where Phase 3 is learning on hard maps, Phase 5 is one mind in server, and Phase 6 is Elytra / full game / `R10`).
   - **(c) Replacement:** Map `R-brach` / `R3` to `Phase 2`, `R4–R6` to `Phase 3–4`, `R8–R9` to `Phase 5`, and `R10` / `Phase 10` to `Phase 6` per `HANDOFF.md:72-78`.

4. **`No superflat` rule stated without scope (`line 36`)**
   - **(a) Doc (`docs\ladder.md:36`):** `- **No superflat:** every superflat chunk is the same, so the memo, repair and scale tests could not fail there.`
   - **(b) Evidence:** Julien's 18 tracks (line 16), `R-lego` (line 97, literally `"on the superflat"`), `straight100/300/800`, `hardtrack`, and `3.2d-3`'s 1M fixture (`jobs\3.2d-3\spec.md:100`, `"structure/feature-free superflat generator plus complete scars journal"`) all use superflat intentionally.
   - **(c) Replacement:** `- **No superflat for memo/repair/scale rungs:** every superflat chunk is identical, so R-scale, R-far, and varied-terrain repair tests must not use superflat (point/track/1M-scar fixtures may use superflat where terrain variation is not under test).`

5. **Stale `to freeze before ...` status values (`lines 87–95`)**
   - **(a) Doc (`docs\ladder.md:87-95`):** `L0: to freeze before 1.5 runs`; `L0-x: to freeze before L0's first run`; `Legs ladder, R1, R2, R3, R-retarget, R-scale: to freeze before Phase 2 runs`; `R-far: frozen with Phase 2's rungs`.
   - **(b) Evidence:** `L0` already ran against `wt/l0-veto` (90/90 before powder snow tick 111, `HANDOFF.md:47`, `postmortem-2026-11-14.md:30`) before `L0-x` existed, and Phase 2's first flight (`flight-20261114`) already ran on Julien's 18 tracks + `straight100/300/800` before `R1–R3`, `R-retarget`, `R-scale`, and `R-far` were built/frozen.
   - **(c) Replacement:** Update `L0` to `frozen; ran to tick 111 (powder-snow port gap)`; `L0-x` to `open / queued`; and `Legs ladder, R1–R3, R-retarget, R-scale, R-far` to `open / unbuilt (must freeze before gating the 3.2d-3 / hardtrack completion runs)`. Note also S97's axis-by-axis map list (`HANDOFF.md:53`) is still absent from `ladder.md`.

---

### 11. `docs\server.md` (115 / 200 lines)

1. **Stale phase header (`line 1`)**
   - **(a) Doc (`docs\server.md:1`):** `# One mind in the server (Phase 2 spec, draft)`
   - **(b) Evidence:** `docs\HANDOFF.md:77` (S95 reorder) moved `One mind in the server (slice C)` to **Phase 5**, and `HANDOFF.md:51` records Swarm 0 and scale parked.
   - **(c) Replacement:** `# One mind in the server (Phase 5 spec, draft — parked until Phases 2–4 land)`

2. **In-process DLL rule (`lines 61–68`) vs `wt/3.2d-3` client JNI DLL**
   - **(a) Doc (`docs\server.md:61-68`):** Rejects an in-process Rust DLL (`"And Windows locks a loaded DLL, so every engine rebuild would mean a server restart"`) in favor of a separate process + shared memory.
   - **(b) Evidence:** On `D:\path-tracing-wt\3.2d-3` (`jobs\3.2d-3\spec.md:10-11`, `engine\native\src\lib.rs`, `bot\src\pathtracing\ResidentRunner.java`), the single-bot **client** architecture explicitly moved **to** an in-process JNI DLL (`pathtracing_native`).
   - **(c) Replacement:** Clarify on lines 61–68: `Applies to the Phase 5 multi-bot server host (to protect the server JVM and world saves and avoid locking the server on engine rebuilds); the client-side 3.2d-3 R0 bot already runs in-process via JNI DLL (pathtracing_native) on wt/3.2d-3.`

---

## Part 2: Budgets and Proposed Cuts / Moves

Budgets from `AGENTS.md:58`: `HANDOFF.md ≤ 120`, each spec in `docs/ ≤ 200`, `AGENTS.md ≤ 60`.

| File | Current Lines | Budget | Headroom | Status |
|---|---:|---:|---:|---|
| `AGENTS.md` | **60** | 60 | 0 | **AT LIMIT** (cannot grow) |
| `docs\HANDOFF.md` | **120** | 120 | 0 | **AT LIMIT** (plus bloated line 45) |
| `docs\footprints.md` | **200** | 200 | 0 | **AT LIMIT** (must cut to update) |
| `docs\engine.md` | 183 | 200 | 17 | Comfortable |
| `docs\clan.md` | 176 | 200 | 24 | Comfortable |
| `docs\representation.md` | 167 | 200 | 33 | Comfortable |
| `docs\scales.md` | 145 | 200 | 55 | Plenty of room |
| `docs\vocabulary.md` | 117 | 200 | 83 | Plenty of room |
| `docs\ladder.md` | 117 | 200 | 83 | Plenty of room |
| `docs\server.md` | 115 | 200 | 85 | Plenty of room |
| `docs\learning.md` | 76 | 200 | 124 | Plenty of room |

### Concrete Cuts / Moves for Files at `0` Headroom

1. **`AGENTS.md` (60 / 60 lines):**
   - **Cut/move `How many at once` slot numbers (line 37):** Keep only `"Use scripts/launch.ps1; measured slot limits and meters live in docs/clan.md sections 4–5. Native Anthropic quota is the reserve; put heavy context in the integrator rather than sub-agents (S102)."` Saves ~30–40% of line 37 cleanly and prevents repeating stale numbers.
   - **Trim line 30 (`Fable orchestrates from 10 November (S63); this lifts the 4 November "no Fable"...`):** Line 19 in the Team table already states `"Fable 5.1 from 10 November (S63)"`. Dropping line 30 + blank line saves **2 lines** (`60 -> 58`).

2. **`docs\HANDOFF.md` (120 / 120 lines):**
   - **Delete stale `## Now (14 November, S104–S106)` (lines 30–54 = 25 lines, including the huge ~1,500-word line 45):** Replace with a crisp 4–5 line `## Now (15 November)` summarizing only post-flight state, freeing **~20 lines**. Move the two durable nuggets out:
     - S97's axis-by-axis map list (line 53) -> move to `docs\ladder.md`.
     - Historical flight log commentary (`live.txt`, `straight*`, powder-snow 111) -> point to `D:\path-tracing-evidence\flight-20261114\` and `docs\reference\postmortem-2026-11-14.md`.
   - **Trim `The clan in three commands` (lines 12–16):** Replace the repeated slot numbers on line 14 with a pointer to `docs\clan.md` section 4–5 (saving 1–2 lines).

3. **`docs\footprints.md` (200 / 200 lines):**
   - **Compress the 8-row `frames.rs` symmetry table (`lines 128–137`, 10 lines) into 2 prose lines:** State `Identity passes 62/62; every rotated or mirrored frame fails bit-exact replay (1/30 to 9/30 ticks diverge at 90° multiples; 1e-17..3e-5 gaps in math.rs), fixing canonical orientation.` Saves **8 lines**.
   - **Condense `§5 Measured cost` (`lines 179–195`) history of `v1 vs v2`:** Keep the current cost row + 1 sentence summarizing `key_cost.rs` vs `wt/3.2d-3` `read_cost` pending, cutting **~4–6 lines**. Total **12–14 lines freed**, enough to add `state_reads` / `read_state` (`§1`) and `wt/3.2d-3` Slice B `Canonical` interning (`§3`, `§6`).

---

## Part 3: Copies Instead of Pointers

1. **Companion slot limits, meter rules & launch syntax**
   - **Copies:** `AGENTS.md:37`, `docs\HANDOFF.md:14`, `docs\clan.md:118-120` (and `142`).
   - **Canonical owner:** `docs\clan.md` (§4–5, which already declares it owns assignment and capacity).
   - **Pointers:** `AGENTS.md` and `docs\HANDOFF.md` keep a 1-sentence pointer to `docs\clan.md` §4–5 and `scripts\launch.ps1`.

2. **Antigravity sandbox path traps, BOM rule, soft-deny warning & pre-launch check**
   - **Copies:** `docs\clan.md:124-126,165-170`, `docs\HANDOFF.md:18,119`, `docs\reference\postmortem-2026-11-14.md:16,35-42,57-60`.
   - **Canonical owner:** `docs\clan.md` §5/5c (for live operator rules) and `postmortem-2026-11-14.md` (for incident history).
   - **Pointers:** `HANDOFF.md` keeps only the 1-line trap note pointing to `docs\clan.md`.

3. **Worldtube chunk ticket levels (33 / 32 / 31), Fermi chunk MB/CPU math, presence-as-tool & kill line**
   - **Copies:** `docs\scales.md:52-61` AND `docs\server.md:83-110` repeat ticket levels 33/32/31, chunk memory/CPU figures, and the worldtube-owns-chunks boundary nearly verbatim.
   - **Canonical owner:** `docs\server.md` (§ Worldtube and § Presence; `scales.md:3` already says `server.md` owns the server boundary).
   - **Pointers:** `docs\scales.md` §4.2 keeps only the Fermi ratio + 1-line pointer to `docs\server.md`.

4. **`Speeds.java` law figures (`9×10⁻⁶`, `0.003`, `19.903`, `9.734`, `19.345`)**
   - **Copies:** `docs\engine.md:11,92,139-140,170-183`, `docs\representation.md:48,51`, `docs\scales.md:11`, `docs\PRINCIPLES.md:119`.
   - **Canonical owner:** `docs\engine.md` (owns the law and Mojang constants).
   - **Pointers:** `representation.md` and `scales.md` cite only the invariant summary and point to `docs\engine.md`.

5. **Always-on invariants (6 invariants list)**
   - **Copies:** `docs\scales.md:114` AND `docs\ladder.md:26` duplicate the same 6 invariants almost word-for-word.
   - **Canonical owner:** `docs\ladder.md:26` (governs rungs and gates).
   - **Pointers:** `docs\scales.md` §4.8 points to `docs\ladder.md` instead of re-listing all 6.

6. **Scoreboard rules & human baseline rules**
   - **Copies:** `docs\PRINCIPLES.md:11-14`, `docs\ladder.md:7-29`, `docs\HANDOFF.md:82`, `docs\representation.md:147`.
   - **Canonical owner:** `docs\ladder.md` (defines `T`, brackets, distributions, baselines).
   - **Pointers:** `HANDOFF.md:82` holds only the current numeric scoreboard values (or empty/live counts) and points to `docs\ladder.md` for definitions.

7. **Three grains, 5 model roles, `0.9ⁿ` column math & segment/word record**
   - **Copies:** `docs\representation.md:25-83` AND `docs\vocabulary.md:3-21,72-86` (and timing comparison vs `docs\footprints.md:182-186`).
   - **Canonical owner:** `docs\vocabulary.md` owns the word/segment record (`§3–§8`); `docs\footprints.md` owns `key_cost` timings and `Footprint` internals; `docs\representation.md` keeps architectural principles and points.

8. **`R-lego` & Rediscovery rung pass/kill criteria**
   - **Copies:** `docs\scales.md:48-50` AND `docs\ladder.md:96-98`.
   - **Canonical owner:** `docs\ladder.md`.
   - **Pointers:** `docs\scales.md` §4.1 names `R-lego` and points to `docs\ladder.md`.

9. **Hardware (`7800X3D`), `D:` storage rule, Windows toolchain, and Lithium/C2ME mod exclusion**
   - **Copies:** `docs\engine.md:16,114-121`, `docs\HANDOFF.md:114-119`, `docs\ladder.md:40-78`.
   - **Canonical owner:** `docs\engine.md` (compute/runtime/rig contract) and `docs\ladder.md` (gate benchmark rules); `HANDOFF.md:114-120` trims redundant hardware prose to operational traps only.

10. **Sparse vs dense Knudsen evaluation regimes & lazy bot (`push_pair`)**
    - **Copies:** `docs\representation.md:134-143`, `docs\scales.md:86`, `docs\server.md:51-59`.
    - **Canonical owner:** `docs\server.md` for multi-bot lazy protocol (`push_pair`); `docs\representation.md` for engine dense-vs-sparse compute split; `docs\scales.md` keeps a 1-line pointer.

---

## Part 4: Julien's Words (Misquotes / Attribution Audit)

Rule checked: Julien's turns go verbatim into `docs\reference\`; models never reword them (`AGENTS.md:44`). Checked every quoted string attributed to Julien across the 11 docs against `docs\PRINCIPLES.md` and `docs\reference\julien-2026-11-{04,09,10}.md`.

### Discrepancies Found (4)

1. **`AGENTS.md:37` and `docs\clan.md:89` — `"fill the Anthropic context"` / `"Fill the Anthropic context"`**
   - **What docs print:** Quoted as if verbatim from `S102` (`"fill the Anthropic context"` in `AGENTS.md:37`; `"Fill the Anthropic context"` in `docs\clan.md:89`).
   - **Verbatim original (`docs\reference\julien-2026-11-10.md:257`, S102):** `"you need to give my antrhopic context filled up more than the others"`.
   - **Recommendation:** Remove the quotation marks around the paraphrase (e.g. `give Anthropic more context than the others (S102)`), or quote the exact words.

2. **`docs\HANDOFF.md:51` — `"Vanilla in behaviour, not pure vanilla."`**
   - **What doc prints:** `"Vanilla in behaviour, not pure vanilla."` attributed to `S55`.
   - **Verbatim original (`docs\reference\julien-2026-11-09.md:160`, S55):** `"i think the goal is vanilla in behavior not pure vanilla."` (differs in British spelling `"behaviour"`, capitalization, and added period).
   - **Recommendation:** Either quote verbatim (`"vanilla in behavior not pure vanilla"`) or unquote as summary.

3. **`docs\engine.md:78` — `"an old route can remain valid while becoming poor" is the case this covers (X2).`**
   - **What doc prints:** Puts `"an old route can remain valid while becoming poor"` in quotation marks as `X2`.
   - **Verbatim original (`docs\reference\julien-2026-11-04.md:127`, X2):** In `X2`, those words sit inside double quotation marks where Julien was quoting an **other AI's** wording before replying `"this is the type of causal chain i'm thinking about"`, not Julien's own coined sentence.
   - **Recommendation:** Attribute cleanly (unquote as X2's scenario, or quote Julien's own words `"this is the type of causal chain i'm thinking about"`).

4. **`docs\scales.md:81` — `so "prices as fields" is exact`**
   - **What doc prints:** `"prices as fields"` in quotes alongside `S44`/`S46`.
   - **Verbatim check:** That exact three-word phrase does not appear verbatim in `docs\reference\julien-2026-11-{04,09,10}.md`; it is a coinage from `docs\representation.md:148`.
   - **Recommendation:** Drop the quotation marks or explicitly write `(representation.md §8)` so it is not read as a verbatim Julien quote.

### Verified Exact / Intentional Ellipses

- All remaining verbatim quotes checked in `docs\clan.md:5`, `docs\engine.md:7,75,82`, `docs\scales.md:12`, `docs\vocabulary.md:113`, `docs\learning.md:13-21,36`, `docs\ladder.md:14`, `docs\server.md:78`, `docs\HANDOFF.md:45,64`, and `docs\PRINCIPLES.md:7,16,20-47` match `docs\reference\julien-2026-11-{04,09,10}.md` (using standard `…` ellipses and preserving Julien's spelling/lowercase).
- Note on `docs\ladder.md:42`: quotes ES `docs/test-ladder.md` (external ES repo doctrine), not a Julien turn.
- Note on `docs\PRINCIPLES.md:320` + `docs\HANDOFF.md:8,27`: `docs\reference\julien-2026-11-10.md:461-495` already holds verbatim turns `S140–S146`, which have not yet received register rows in `docs\PRINCIPLES.md`.
