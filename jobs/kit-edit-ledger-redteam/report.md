FREEZE AFTER FIXES

# Red-team & frozen contract: kits, 4D world edits, world gaits, and mission obligations

One-way-door design review ahead of editing the world (`R-lego`, the five edit tracks, and the cold-layer compression of `wt/port-fold`), designed natively for **1 to 10⁹ (1 billion) bots** on the ES runtime spine (`D:\path-tracing-deps\emergent-space-engine`) and on top of the commitments already in `map/detail/representation.md` (`worldgen(seed) + scars`, 4D `W(t)`, mission-owned transferable obligations, `(min, +)` / `(max, +)`, tools as carried law inputs, sparse ↔ dense Knudsen switch) and `map/detail/scales.md` (`1–6 KB/chunk`, law read backwards, Pareto archives, ski rental, Bertsekas prices). Read-only against `main` (`81e13038`), `wt/port-fold` (`c86b6d3a`), and `D:\path-tracing-deps\emergent-space-engine`. No code in `engine/`, `bot/`, or `map/` was touched.

## Verdict

The map's architecture holds under **one mechanism**, with **eight structural faults** in the current code and naive single-bot extensions that must be fixed before freezing:

- **C1 (Lossy cell storage in `WorldSnapshot`):** `WorldSnapshot` (`engine/world/src/snapshot.rs:29`, `wt/port-fold:29`) stores `ModelId(u16)` (591 movement models), erasing block identity (`stone == dirt == planks == obsidian == bedrock`). Breaking, placing, interacting, kit projection, and exact restoration cannot run on it.
- **C2 (Triple-world duplication, sticky flags, and cache destruction on `wt/port-fold`):** `SimWorld` (`wt/port-fold/engine/law/src/sim.rs:561-562`) holds both `WorldOverlay` (`HashMap<BlockPos, Block>`) and `cells: HashMap<BlockPos, SimCell>`, while `SimCell` (`basis/cell.rs:309-314`) embeds `Block` (6 heap-allocated `VoxelShape`s) and `Rules`. `WorldOverlay::model_id` (`sim.rs:175-181`) returns `None` on every edited cell (killing dense caching), `visit_collision_cells` (`sim.rs:273-284`) allocates and sorts a `Vec` per overlapping query, and `unbounded`/`extra_fluids`/`extra_inside` (`sim.rs:95-98`) are sticky `|=` bools that never clear on undo.
- **C3 (Hardcoded bare-legs default violates one mechanism and admissibility):** `Goal::C` (`engine/plan/src/goal.rs:7`) and `Kit::BareLegs` (`engine/plan/src/solve.rs:8`) treat bare legs as the default. Bare legs is merely the empty kit (`kit = ∅`); under any kit, the kit's own fastest words and `KitCone` lead from tick 0 (Kill test 16).
- **C4 (Per-word read-mask hygiene — no "legs lens vs edit lens" split):** A word's corridor check must read **only** the projected cell answers present in that word's own `Footprint::reads`. Otherwise carrying a pickaxe splits a mixed stone/dirt/planks runway into fake defects for a sprint-jump gait — violating *"an unused item never raises T"* and breaking the ~13 µs floor — while a dig-straight gait over stone/granite/diorite/andesite naturally sees one uniform zone under its own read mask.
- **C5 (Combinatorial kit-table explosion at swarm scale):** Keying a monolithic ~69 KB table by `(body_modifiers × tools × item_counts)` would blow out L3 cache into gigabytes under diverse enchants, potions, and gear across 10,000 bots, and rebuild on every block placed (`64 → 63`). Factoring into orthogonal capability tables (`StateStaticInfo`, `LegsProjection`, `BreakProjection`) + separate `KitCounts` keeps **every possible gear combination across 10,000 bots under ~1 MB total**.
- **C6 (No ad-hoc 6-neighbor check — non-local certificates):** Editing one cell at `(pos, t)` can break a flying arrow or ender pearl 40 blocks away, a 42-block tripwire line, a 7-block scaffolding column, a quasi-connected piston 2 blocks below, or another bot's certified tube 200 ticks ahead. All causal wake-ups must pass through **one space-time inverted footprint index**, never a 6-neighbor shortcut.
- **C7 (Deterministic RNG + exact `ItemEntity` physics over void and swarm):** Mojang's `ItemEntity` drop (`items.rs:400-484`) is deterministic from server `JavaRandom` (an explicit input to the law per `footprints.md §2` and `server.md:72`). With `vy_0 = +0.2`, gravity `-0.04`, drag `0.98`, and 10-tick pickup delay, an item drops only **0.24 blocks below block center** at tick 10 while the player's `(1.0, 0.5, 1.0)` expanded pickup box is `2.6 × 2.8 × 2.6`. Therefore a bot (or a teammate in the swarm) adjacent to a void bridge block *can* catch the drop mid-air. Never hardcode "void = irreversible"; let the law's own `ItemEntity` trajectory + RNG input (or bounded `±0.1` b/t scatter cone when RNG is unobserved) decide catchability vs ruin.
- **C8 (Hidden cascade debts & periodic world machines):** Every direct and cascaded cell mutation enters the mission-owned double-entry ledger, and looping redstone/piston/flying machines must fold into **periodic world gaits** (the biggest repeating cycle `(cycle_program, phase, period P, Δpos)`) rather than ticking per cycle forever — answering `W(t)` in O(1) and waking via the same space-time footprint index.

---

## Measured 26.2 inputs & Fermi floors

All table sizes are measured directly from `wt/port-fold/engine/world/data/` and `jobs/law-grouping-audit/table.md`:

| Quantity | Measured 26.2 value | Source |
|---|---:|---|
| Total block states (`census.tsv`) | 32,366 (32,252 modeled, 114 dynamic) | `wt/port-fold/engine/world/data/census.tsv` (1,196 blocks) |
| Distinct movement physics models (`models.tsv`) | 591 (519 on `main`) | `wt/port-fold/engine/world/data/models.tsv` |
| Property maps (`maps.tsv`) | 44 (35 on `main`) | `wt/port-fold/engine/world/data/maps.tsv` |
| Registered items (`use-effects.tsv`) | 1,537 | `wt/port-fold/engine/world/data/use-effects.tsv` |
| Current hand-written `rules.tsv` rows | 40 / 1,196 blocks | `wt/port-fold/engine/law/src/basis/rules.tsv` |
| Cold-layer world/rule mass to compress | 5,863 code lines (Groups 1 + 4 + 8) | `jobs/law-grouping-audit/table.md:20,23,27` |

| Structure / Operation | Fermi floor | Target / Budget | Notes |
|---|---:|---:|---|
| Long-lived world storage (`10k × 10k`) | 0.4–2 GB | **Paletted sections (1–6 KB/chunk) + `seed` + sparse `StateId` scars** | Already in `representation.md §2` & `scales.md §4.7`; never raw 76 GB dense array |
| Hot unpacked window cell | 2 B / cell | **2 B / cell (`StateId(u16)`)** | **0 B overhead** vs current `WorldSnapshot` (`Box<[u16]>`) |
| Process-wide static tables (`STATE_TO_MODEL`, `STATE_TO_BLOCK`, `STATE_STATIC`, `STATE_DYNAMIC`, `BLOCK_RULES`) | ~278 KB | **~278 KB total (shared by all 10⁴ bots)** | Fits in L2; `BLOCK_RULES` (19.1 KB) fits in L1d |
| Factored kit projections (all combinations across 10⁴ bots) | < 1 MB | **4.7 KB/body profile + 4.8 KB/tool tier + 0 B place/interact copy** | Eliminates combinatorial L3 blowup; active bot's tables fit in L1d |
| Kit / tool swap | ~1 ns | **1 `Arc` pointer swap** | 0 world cells touched; `64 → 63` stack count triggers 0 rebuilds |
| Flat-ground acting with any unused kit | ~13 µs | **0 ns overhead** (median ≤ 15 µs) | Every word checks only its own `Footprint::reads` mask |
| Thread-local speculative edit + rollback (`k ≤ 16`) | ~20–50 ns | **≤ 50 ns/edit, 0 heap allocs** | Inline sorted `SmallVec` + journal pop |
| Space-time footprint invalidation (quiet section) | ~2 ns | **1 section bitboard test** | Wakes only affected certificates (`O(k)`): bots, arrows, redstone, world gaits |
| Undisturbed looping redstone / flying machine | 0 ticks/tick | **0 simulation cost; O(1) `W(t)` lookup via `(t - t_0) % P`** | Exact materialization on footprint disturbance |
| `R-lego` first gap discovery | ~10–40 verifications | **≤ 10³ verifications** (~0.1–1 ms on 1 core) | Causal join `(failed_reads ∩ verb_writes)`; gap #2..N is 1 pattern hit |
| 10⁹ bots (1,000 Form-P streams + 10⁶ Elias–Fano scars) | ~15–20 MB total (~0 B/cruising bot) | **Fits inside 96 MB L3; ~1–3 ms/tick** | `shipmem` Form P (`Prog` + `Ef` scars + `seal` reabsorption); naive 24 B/bot = 24 GB (dead) |

---

## Pillar 1: One 4D World (`StateId` + `seed + scars` + `W(t)` + Space-Time Index + World Gaits)

### 1.1 Canonical 2-byte cell identity (fixes C1, C2)

Every cell in both paletted chunk storage (`1–6 KB/chunk` over `worldgen(seed)` + sparse scars) and unpacked hot windows (`WorldSnapshot`) carries **one canonical `StateId(u16)`**:

```rust
/// Canonical Mojang 26.2 block-state id in 0..32_366 (bit 15 = dynamic/block-entity sidecar).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct StateId(pub u16);

/// Registry block id in 0..1_196 (shrinks BlockId from u32 to u16).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BlockId(pub u16);

/// Physics model id in 0..591 (row in models.tsv).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ModelId(pub u16);
```

- **Five process-wide static tables** (generated once from `server-26.2.jar`, hashed into `LAW_HASH`):
  1. `STATE_TO_MODEL: [ModelId; 32_366]` (64.7 KB) → `&'static Block` + `offset_variants`.
  2. `STATE_TO_BLOCK: [BlockId; 32_366]` (64.7 KB) → `BlockId`.
  3. `BLOCK_RULES: [Rules; 1_196]` (19.1 KB, replaces partial 40-row `basis/rules.tsv`) → hardness, requires_correct_tool, tool_tag_mask, conductor, push_reaction, piston_kind, blocks_motion, replaceable_by_fluid, can_be_replaced, is_signal_source, has_block_entity, survive_rule, restore_class.
  4. `STATE_STATIC: [StateStaticInfo; 32_366]` (64.7 KB) → kit-independent placement/interaction facts (`sturdy_faces: u8`, `replace_mask: u8`, `interact_kind: u8`, `waterlogged: bool`, `partner_half_flags: u8`).
  5. `STATE_DYNAMIC: [u16; 32_366]` (64.7 KB) → packed state properties (`facing`, `half`, `open`, `powered`, `fluid_level`, `falling`, `slab_type`, `stair_shape`, `scaffolding_distance`, `extended`, `attached`).
- **Sidecars (bit 15 set):** Only the ~114 non-local/block-entity states (shulkers, moving pistons, containers, spawners, signs) attach to a sparse `Sidecar` table. Destroying unmodeled block-entity NBT is classified `Irreversible(BlockEntityNbt)` (ruin barrier).
- **`World::state_id(pos) -> Option<StateId>`** becomes the single source of truth; `World::model_id(pos)` is `state_id.map(|s| STATE_TO_MODEL[s.0 as usize].0)` — **never returns `None` on edited cells!**

### 1.2 Two views of the same `WorldMut` interface: thread-local `WorldDelta` & shared 4D `W(t)` (fixes C2, C6)

Delete `WorldOverlay` (`sim.rs:65-317`) and `SimWorld.cells: HashMap<BlockPos, SimCell>` (`sim.rs:562`). Both speculative single-word search and live multi-bot execution use one cell identity and one reversible edit journal:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditCause {
    BotPlace, BotBreak, BotInteract, FluidSpread, FallingBlock,
    RedstoneCascade, PartnerHalf, PistonMove, WorldGaitStep, RecoveryRestore,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CellEdit {
    pub tick: u64,
    pub pos: BlockPos,
    pub before: StateId,
    pub after: StateId,
    pub cause: EditCause,
    pub mission: MissionId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IrreversibleReason {
    Unbreakable,
    BlockEntityNbt,
    NonRestorableDrop,
    ItemLost,
}

pub trait WorldMut: World {
    fn get_state(&self, tick: u64, pos: BlockPos) -> StateId;
    /// Returns `Err(IrreversibleReason)` on ruin barriers so callers cannot ignore them.
    fn set_state(
        &mut self,
        tick: u64,
        pos: BlockPos,
        after: StateId,
        cause: EditCause,
        mission: MissionId,
    ) -> Result<(), IrreversibleReason>;
    fn has_scheduled_tick(&self, pos: BlockPos, block: BlockId) -> bool;
    fn checkpoint(&self) -> Checkpoint;
    fn rollback(&mut self, mark: Checkpoint);
    fn journal_since(&self, mark: Checkpoint) -> &[CellEdit];
}
```

- **Speculative search (`WorldDelta<'a, W>`, thread-local, lock-free):**
  - Stores overrides in an inline `zyx`-sorted `SmallVec<[(BlockPos, StateId); 16]>` (spilling to a map only past 16 edits) → **0 heap allocs** in `visit_collision_cells` and **≤ 50 ns/edit** checkpoint/rollback.
  - **Ref-counted capability flags** (`extra_unbounded: u32`, `extra_fluids: u32`, `extra_inside: u32`) fix `WorldOverlay`'s sticky-bool bug (`sim.rs:95-98`).
- **Live swarm forecast world `W(t)` (`base + committed scars + scheduled future edits + active WorldGaits`):**
  - Any query `get_state(tick, pos)` resolves: (1) active scheduled edits at `pos` with `edit_tick ≤ tick`, else (2) active `WorldGait` covering `pos` evaluated in O(1) at `(tick - t_0) % P`, else (3) committed scars, else (4) paletted base section / `worldgen(seed)`.
  - Observer-filterable (`engine.md:101`): every edit is tick-stamped, so a `k`-tick delayed observer view filters by `edit.tick + delay <= observer_tick`.

### 1.3 One Space-Time Inverted Footprint Index (fixes C6 — replaces ad-hoc 6-neighbor checks)

Extend `law::footprint::Footprint` (`engine/law/src/footprint.rs:38-47`) with:
```rust
pub writes: BTreeMap<BlockPos, WriteMask>, // Shape=1, Fluid=2, Hooks=4, State=8
```

Every certified object in the engine — **a Form-P bot stream (1 handle for 10⁶ bots, NEVER 1 per bot!), a scarred individual bot, a flying arrow, a thrown pearl, a falling block, a 42-block tripwire line, a 7-block scaffolding column, a quasi-connected piston, a fluid equilibrium, a cached joint set, AND a periodic world gait** — registers its space-time read footprint `(pos, [t_start, t_end], ReadMask, CertHandle)` in **one inverted index**:

1. **Coarse filter:** one 64-bit presence mask per 16³ chunk section (`O(1)`, ~2 ns when undisturbed).
2. **Exact hit:** cell map `BlockPos -> SmallVec<[CertRef; 4]>` storing `(t_start, t_end, mask, handle)`.
3. **One invalidation rule (O(1) stream split at 10⁹ scale):** When `set_state(t, pos, before -> after)` fires, compute `diff = write_mask(before, after)` and notify every certificate at `pos` with `t <= t_end && (cert.mask & diff) != 0`.
   - A stream of 10⁶ bots registers **1** worldtube; an edit at `(pos, t)` wakes **1** handle and splits the stream's ordinal range `[0..i*, i*..N)` in **O(1)** (bots `0..i*` already cleared `pos`; bots `i*..N` reroute as one sub-stream).
   - A flying arrow whose DDA ray crosses `pos` at `t + 12` wakes immediately.
   - A tripwire hook 40 blocks away wakes immediately.
   - A QC piston 2 blocks below wakes immediately.
   - **Zero** ad-hoc 6-neighbor heuristics and **zero** O(N_bots) scans; **one mechanism** for local and non-local causality.

### 1.4 Periodic World Gaits: Looping Redstone & Flying Machines (fixes C8, Factorio / ES cycle fold)

Just as a bot on uniform ground folds into `(word, phase, count)`, a closed reactive world component (repeater clock, comparator loop, cycling piston door, observer chain, hopper timer, or translating slime/honey flying machine) folds into a **WorldGait**:

```rust
pub struct WorldGait {
    pub id: WorldGaitId,
    pub t0: u64,
    pub period: u32,                     // P: exact repeating cycle (or biggest repeating cycle of coupled clocks)
    pub shift_per_cycle: BlockPos,       // [0, 0, 0] for stationary machines; != 0 for flying machines!
    pub cycle_edits: Box<[CycleFrame]>,  // exact StateId transitions per active tick inside 0..P
    pub read_footprint: Box<[(BlockPos, u8)]>, // own cells + external triggers (buttons, plates, sensors, chunk loaded>=32)
    pub write_footprint: Box<[BlockPos]>,
}
```

- **Detection (exact, never approximate):** When a connected reactive component ticks in `SimWorld`, hash its normalized local state `(relative_cell_states, relative_pending_ticks, relative_block_events)`. On the first exact hash + byte-equality repeat at `t0 + P` (with optional constant `shift_per_cycle` for translating machines), compile the component into a `WorldGait`, register its `read_footprint` in the space-time index, and **remove it from `LevelTicks`** (0 per-tick simulation cost!).
- **Coupled clocks ("biggest repeating cycle"):** Coupled sub-machines fold at their exact combined cycle (`lcm` / exact repeat of the coupled component's state hash). Aperiodic or huge-period machines (e.g., 32-bit binary counter) hit the bounded detector cap and stay normal scheduled-tick components — never forced, never approximated.
- **O(1) 4D query & exact materialization:**
  - Undisturbed `W(t)` evaluates any cell inside the gait at `phase = (t - t0) % P` (shifted by `((t - t0) / P) * shift_per_cycle`) in O(1), letting bots plan through cycling piston doors or ride flying machines using exact arrival phases `t mod P` without simulating intermediate ticks.
  - The instant any external write, entity trigger, or chunk freeze (`level > 32`) hits `read_footprint` at tick `t*`, the space-time index **materializes** the component's exact `StateId`s and pending `LevelTicks` at `(t* - t0) % P`, resumes local stepping until it settles or locks onto a new cycle `P'`, and folds again.

---

## Pillar 2: Factored Kit Projections (`< 1 MB` at 10⁴ Bots), `KitCone`, and One-Mechanism Corridors

### 2.1 Orthogonal factorization: capabilities vs. counts (fixes C3, C4, C5)

Never build one combined table per `(body × tools × verbs × counts)`. Factor into **three independent, tiny, immutable capability views** + **separate forward `KitCounts`**:

```rust
/// 1. Body movement profile (boots, depth strider, soul speed, movement effects).
/// Size: 591 * 4 * 2 B = 4.7 KB per profile (~30 distinct profiles across 10^4 bots = ~140 KB).
pub struct LegsProjection {
    pub body_key: BodyMovementKey,
    pub by_model: Box<[[ClassId; 4]]>, // [Support02, Support05, BodyFeet, Inside]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum RestoreMode {
    IdenticalDrop,
    WaterloggedLeavesWater,
    PartnerHalf,
    Irreversible,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BreakRow {
    pub break_ticks_ground: u16, // u16::MAX = unbreakable / refused
    pub break_ticks_air: u16,    // 5x slower per mining.rs:110
    pub restore_mode: RestoreMode,
}

/// 2. Mining/break profile (best tool rules per Mojang tag, efficiency, silk touch, shears, haste/fatigue).
/// Size: 1,196 * 6 B = 7.2 KB per tool tier (fits in L1d! ~100 tiers = ~720 KB) + sparse state overrides
/// for the ~15 block kinds where StateId alters break/restore (waterlogged, double slabs, snow layers, ice).
pub struct BreakProjection {
    pub tool_key: ToolProfileKey,
    pub by_block: Box<[BreakRow; 1_196]>,
    pub state_overrides: Box<[(StateId, BreakRow)]>, // sorted small slice, binary-searched only when flag set
}

/// 3. Place/interact/sturdy-face facts: KIT-INDEPENDENT -> lives once in global STATE_STATIC[32_366] (64.7 KB).

/// Lightweight handle held per bot (~32 B): swapping any tool/armor/effect is a 1 ns Arc pointer swap.
#[derive(Clone)]
pub struct KitView {
    pub legs: Arc<LegsProjection>,
    pub breaking: Arc<BreakProjection>,
    pub verb_mask: u64,          // bitset of carried verb families with count >= 1 (0 <-> >=1 transition only)
    pub dimension_flags: u8,     // water_evaporates, fast_lava
    pub cone: Arc<KitCone>,      // admissible lower-bound envelope for this kit
    pub verb_effects: Arc<[VerbEffect]>,
}

/// Carried token counts & tool durability guards (travel in forward state / premises / prices, NEVER table keys).
pub struct KitCounts {
    pub tokens: SmallVec<[(TokenKind, u32); 8]>,
    pub min_tool_durability: u32,
}
```

### 2.2 One-mechanism corridor query (fixes C3, C4)

- **No bare-legs default:** `solve` selects candidate words and computes `LB` from `KitView.cone` directly — elytra, boat on ice, pearls, sprint-jump, or tunnel-digging all compete on the same `T` bracket from tick 0. Bare legs is simply `verb_mask == 0`.
- **Per-word read masks:** Every word `W` carries the exact projection mask its `Footprint::reads` asks for:
  - A sprint-jump word reads `LegsProjection` only → stone, dirt, and oak planks map to the same class regardless of carried pickaxes → **0 ns flat-ground overhead** and *"an unused item never raises T"*.
  - A tunnel-dig gait reads `LegsProjection + BreakProjection` → stone, granite, diorite, and andesite map to the same class (`1 tick`, restorable) → a 500-block mixed-stone mountain is one uniform zone, while obsidian/bedrock is a defect.
- **Admissible `KitCone` (Kill test 16):** Derived from verified generator speeds and wormhole bounds on every `LAW_HASH` (`c_horiz`, `c_up`, `c_down`, `wormholes`), rounded down one ulp before `.ceil()`. Adding an item can only lower or preserve `LB`, never raise it.

---

## Pillar 3: Reading the Law Backwards (`writes × reads` → `R-lego` with Zero Tool Search Code)

### 3.1 Law-probed `VerbEffect` table (no names, no catalogues)

Once per distinct `(verb_mask, dimension_flags)`, probe each carried verb through `RecordingWorld` + `WorldDelta` on canonical probe cells to extract its `VerbEffect` directly from the law's output:

| Law verb | Probed `Footprint::writes` | Probed preconditions (from `place.rs`, `mining.rs`, `interact.rs`, `world_hooks.rs`, `items.rs`) |
|---|---|---|
| `Place(block_class)` | `Cell(p)`: `Shape(empty → non-empty)`, `Hooks(...)` | `p` replaceable (`STATE_STATIC`); `∃ sturdy neighbor face q`; `eye_dist(q_face) ≤ 4.5`; `!aabb_intersects(player, placed_shape)`; `count ≥ 1` |
| `Break(tool_class)` | `Cell(p)`: `Shape(non-empty → empty)` | `break_ticks < u16::MAX`; drop catchable / restorable if mission requires restoration; `eye_dist(p) ≤ 4.5` |
| `Interact` | `Cell(p)`: `Shape(toggle)` or `Hooks(signal)` | `STATE_STATIC[s].interact_kind != None`; `eye_dist(p) ≤ 4.5` |
| `BucketEmpty(Water)` | `Cell(p)`: `Fluid(empty → source)`, `Hooks(fall_reset, swim)` | `water_evaporates == false`; `p` replaceable/waterloggable; `eye_dist ≤ 4.5`; `water_bucket ≥ 1` |
| `BucketEmpty(PowderSnow)` | `Cell(p)`: `Hooks(fall_reset, Inside::PowderSnow)` | `p` replaceable; succeeds even when `water_evaporates == true`; `powder_snow_bucket ≥ 1` |
| `BucketPickup` | `Cell(p)`: `Fluid(source → empty)` | `p` source or waterlogged; `empty_bucket ≥ 1` |
| `PlaceBoat` | `ExtraCollider(boat_aabb)`, `State(riding, fall_reset, ice_speed)` | clearance free; `boat ≥ 1`; `eye_dist ≤ 4.5` |
| `ThrowPearl` | `State(pos → landing_cone, health -= 5, cooldown = 20)` | `pearl ≥ 1`; `cooldown == 0`; `health > 5.0` |
| `DropItem(aim)` | `Entity(ItemEntity(pos, drop_velocity_aimed))` | deterministic transfer into teammate's pickup box (`items.rs:493`) |

### 3.2 The Causal Join (`failed_reads ∩ verb_writes`)

When learning opens at a joint `J` (missing set or open bracket `UB > LB_kit`), the failing attempt's `Footprint` already names the cells and properties that caused the failure:
1. **Blocked / Clamped (`Shape` non-empty at `p`)** → match verbs writing `Shape(non-empty → empty | toggle)` → proposes `Break(p)` or `Interact(p)`.
2. **Shortfall / No-landing (`Shape` empty below arc)** → intersect empty cells along the arc's lower envelope with legal placement preconditions (`sturdy neighbor face q`, reach `≤ 4.5`, `!aabb_intersects`) → match verbs writing `Shape(empty → solid_top)` → proposes `Place(...)` (4–6 candidates on a 6-block gap → ~10–40 verifications, 1.5 decades under `10³`).
3. **Lethal fall (`fall_reset == false` at landing)** → match verbs writing `Hooks(fall_reset = true)` → proposes `BucketEmpty(Water)` (auto-excluded in Nether), `BucketEmpty(PowderSnow)`, `Place(hay/slime/cobweb)`, `PlaceBoat`.
4. **Regime gap (`LB_wormhole` or `LB_carried_gait < UB`)** → proposes `ThrowPearl` / `PlaceBoat` / `Elytra` entry words.

Once a first placement creates a new face (`FaceInventory::edit`), consecutive placements hit the same relative pattern key and fold into a repeating **`(word, count)` bridging gait** — crossing 10 or 10,000 blocks in O(1) unique patterns.

### 3.3 Exact `ItemEntity` & deterministic RNG rule (fixes C7)

In Mojang 26.2 (`items.rs:400-484`), breaking a block spawns an `ItemEntity` at `pos + 0.5` with `spawn_velocity(rng) = (r*0.2 - 0.1, +0.2, r*0.2 - 0.1)`, gravity `-0.04`, drag `0.98`, and `pickup_delay = 10`.
- At tick 10 (earliest pickup), vertical offset is only **`-0.244` blocks below center** (`y ≈ block_y + 0.26`), while the player pickup box is **`2.6 × 2.8 × 2.6`** (`±1.3` horizontally, `-0.5` below feet).
- **Contract:** Server `JavaRandom` state is a deterministic law input (`footprints.md §2`). When known, `ItemEntity::tick` gives the exact trajectory; when unobserved, the law bounds the `±0.1` b/t horizontal cone. A break is **restorable** iff *some bot in the mission* can intersect the item's pickup box before it hits lava/cactus/void — and **ruin (`Irreversible(ItemLost)`)** otherwise. No terrain-name rule ("void = impossible") exists anywhere.

---

## Pillar 4: Mission-Owned Double-Entry Obligations, Prices in T, and Swarm Scaling

### 4.1 Self-closing words vs. Mission-owned `ObligationLedger` (fixes C8)

- **Goal-free `JointPattern` (`Piece` in `engine/plan/src/joints.rs:76-83`):** adds only `item_delta: SmallVec<[(TokenKind, i16); 2]>` and `debt_neutral: bool`. Self-closing edits (open-pass-close door, place-ahead/break-behind cycle, water-clutch picked up at `t ≤ 4` before `WATER_TICK_DELAY = 5`) add **zero state dimensions** to search.
- **Mission-owned `ObligationLedger`** (matches `representation.md §8` — obligations belong to the **mission**, not a single bot):

```rust
pub struct Obligation {
    pub id: ObligationId,
    pub mission: MissionId,
    pub pos: BlockPos,
    pub original_state: StateId,
    pub opened_tick: u64,
    pub release_tick: u64,       // (max, +) latest tick any teammate stream's worldtube reads pos!
    pub cause: EditCause,
    pub item_owed: SmallVec<[(TokenKind, i16); 2]>,
    pub undo_ticks_lb: u32,
}

#[derive(Clone, Debug, Default)]
pub struct ObligationLedger {
    pub open: Vec<Obligation>,   // active unclosed debits per mission
    pub wal_seq: u64,            // monotonic write-ahead log sequence flushed before live packet send
}
```

**Frozen ledger invariants:**
1. **Write-ahead log (WAL):** Every live world mutation flushes its debit `(pos, original_state, wal_seq)` to disk before the packet is sent.
2. **Every direct + cascaded `CellEdit` is a debit:** Fluid spread, falling sand, partner halves, and waterlogged fluid releases all enter the ledger automatically via `WorldMut::journal_since`. Fast clutch pickup (`t ≤ 4`) produces 0 spread debits and wins naturally in `T`.
3. **Shared infrastructure & `(max, +)` cleanup:** If Bot 1 places a bridge block at `t = 10` and a stream of 50 (or 10⁶) teammates crosses it in `W(t)`, `release_tick` advances to the stream's tail exit tick. Cleanup executes **once** after `release_tick`, assigned by Bertsekas auction to the capable stream/bot with lowest marginal `T`.
4. **Admissible open-debt lower bound (O(1) stream/field query — NEVER O(N_bots) at 10⁹ scale):**
   $$\text{LB}(S, G, O) = \max\Big(\text{LB}_{\text{kit}}(S \to G),\; \max_{o \in O}\big(\min_{s \in \text{capable\_streams}(o)} \big[\max(t_{\text{arrive}}(s \to \text{reach}(o)),\; o.\text{release\_tick}) + o.\text{undo\_ticks\_lb} + \text{LB}_{\text{kit}}(\text{reach}(o) \to G_s)\big]\big)\Big)$$
   where `capable_streams(o)` comes from the spatial/portal field index in O(1), never by iterating individual bots.
5. **Mid-run retarget (`R-retarget`) & crash recovery:** Retargeting carries `ledger.open` directly into the new `solve`; crash restart reconciles the WAL against live world cells and spawns a Recovery Mission for any cell still `!= original_state`.

### 4.2 Prices in T & Swarm Tool Flow (`Field::prices`, S96)

- Shadow price of token `r` at joint `J`: $\pi_J(r) = T(J, \text{without } r) - T(J, \text{with } r) \ge 0$, propagated backward so spending a non-recovered last block pays its downstream opportunity cost (`200` ticks > `15` ticks → *"keep it for there"*), while a self-closing edit (`Δr == 0`) pays `0`.
- `SupplyPriceField { fn supply_t(&self, pos: BlockPos, r: TokenKind) -> Option<u64>; }` unifies single-bot command budgets (`Some(0)` while `count > 0`, `None` beyond) with multi-bot chests, farms, and deterministic mid-sprint tool handoffs (`DropItem` + `ItemEntity` pickup).

---

## ES Runtime Spine: What to Steal from `emergent-space-engine` & 1-Billion-Bot Scaling

Audited directly against `D:\path-tracing-deps\emergent-space-engine\crates\`:

### 1. Crate-by-crate ES reuse map

| Action | ES Source (`D:\path-tracing-deps\emergent-space-engine\`) | What it gives Minecraft & 10⁹ bots |
|---|---|---|
| **Steal nearly verbatim** (zero relativity deps!) | `crates/shipmem/src/{bits,codec,program,shelf}.rs` | **Form P (`Prog::Const`/`Affine` + Elias–Fano `Ef` scar sets + `seal()` reabsorption + `FOR`/`DFOR`/`CHORD`/`DICT`/`TIED`/`REF` bit-packed scar codecs):** stores 10⁹ individual bots in ~15–20 MB RAM (`H_live`, never O(N)). |
| **Steal nearly verbatim** | `crates/universe/src/calendar.rs` (`Wheel`) | Zero-alloc intrusive `2^shift`-tick ring wheel + open-window `front` heap (4 B per active stream/scar/world-gait slot). Replaces `SimWorld`'s HashMap `LevelTicks` and schedules all events canonically. |
| **Steal / extract trait** | `crates/allocator/src/{alloc,forecast,qos,geom}.rs` | Multi-core region islands, window certificates, **scoped rollback** on collision, light-cone finality (`hard_speed = KitCone::c`), AIMD strides, least-slack scheduling, SITA lanes, EEVDF eligibility — bit-identical across thread counts. |
| **Adapt the pattern** | `crates/engine/src/{anchor,frame,tube}.rs`, `crates/universe/src/{fleet,view,observe}.rs`, `crates/space/src/events/{dense,handoff}.rs` | Exact-remainder binding (mirrors binade event face); **local dense `Battle` frame promotion & dissolution** when `< 2` members remain (Knudsen switch); **pure read-only `View`** (`observation_never_changes_the_digest`) materializing only bots inside a human player's observer cone. |
| **Skip (2D space/GR only)** | `crates/space` (`kerr`, `orbit`, `plunge`), `crates/encounters`, `crates/lattice`, `crates/lattice-metal` | 2D relativistic orbital/bounce physics and Apple Metal (this machine is Windows + AMD RX 7900 XTX / Vulkan). |

### 2. The 4-Tier `shipmem` Residency Ladder (1 to 10⁹ Bots, Bit-Identical Across Tiers)

Naive `24 B/bot × 10⁹ = 24 GB` breaks RAM and L3; iterating `10⁹` bots takes 10 s/tick. ES `shipmem` keeps memory and CPU proportional to **distinctions + surprises (`H_live`)**, never `N`:

| Tier | Representation | Bytes / bot | Active population | Compute |
|---|---|---:|---:|---|
| **Tier 0 — Form-P Stream** | `shipmem::program::Prog` (`Const`/`Affine` departure lattice) + `KitView` + certified `Word`/`Field` | **~0 B** (`128 B` per stream of 10⁶) | up to **10⁹** | **0/tick** (evaluated lazily at `(ordinal, t)` in O(1); 1 worldtube handle per stream) |
| **Tier 1 — Elias–Fano Scar** | `Ef` scar ordinals (`~2 + log2(n/k)` bits) + `codec.rs` bit-packed deltas (`seal` reabsorbs on rejoin!) | **~12–24 B/scar** | **10⁵–10⁶** (~15–20 MB) | O(surprises) via `Wheel` + space-time footprint index |
| **Tier 2 — Dense Contact Frame (`Battle`)** | Compact ~96 B hot `PlayerState` in lockstep on `k = 1` generator kernel (Knudsen → 1; CPU SIMD / Vulkan GPU after crossover) | **~96 B** | **10²–10⁴** | Tight pits, PvP, active edits; **dissolves back to Tier 0/1** when members separate |
| **Tier 3 — Observer-Materialized** | Vanilla Mojang `ServerPlayer` + 1 GPU draw call (`wt/server-mind`) **strictly inside human view/interaction cone** | Mojang / GPU instance | **10¹–10⁴** visible | Pure projection of Tier 0–2 (`observation_never_changes_the_digest`) |

### 3. Cache, Compounding, and Smarts over Time

- **Cache & L3 scaling (96 MB L3 ceiling on 7800X3D):** World = paletted `1–6 KB/chunk` + `worldgen(seed)` + sparse `StateId` scars; factored kit tables `< 1 MB`; 10⁹ bots in Form P + 10⁶ scars = ~15–20 MB; Nerode content-hash merging + closed-form generator replacement + Pareto dominance keep vocabulary lookup flat forever.
- **Compounding & free-compute smarts:** Causal join → cached operator word → repeating `(word, count)` gait → compound word collapses depth-12 problems to depth-1 lookups. One server-audited discovery enters the shared archive for all 10⁹ bots at once; free compute on idle cores drains the single least-slack queue ordered by `(T_saved × traffic) / cost` and runs boundary play (`learning.md §2.9`).

---

## Dead code unlocked on `wt/port-fold` & `main` (Principle 21)

| Current code to delete / collapse | Location | Measured lines | Replaced by |
|---|---|---:|---|
| `WorldOverlay` | `wt/port-fold/engine/law/src/sim.rs:65-317` | ~255 | `WorldDelta<'a, W>` / 4D `W(t)` over `StateId(u16)` |
| `SimCell` (`Block` + `Rules` copies) & `SimWorld.cells` | `wt/port-fold/engine/law/src/basis/cell.rs:260-433`, `sim.rs:562` | ~220 | 2-byte `StateId(u16)` + static `[Rules; 1_196]` |
| Private world adapters (`BucketWorld`, `PistonWorld`, `PortalWorld`) + doubled cell silos | `wt/port-fold` Groups 1, 4, 8 (`law-grouping-audit/table.md`) | ~5,863 → ~1,670 | Unified `WorldMut` + table-driven `cell` rules (~0.55 decades shrink) |
| `ProjectionTable` HashMap + `UnportedVerb` stubs | `engine/law/src/codec.rs:110-169` | ~60 | Factored `LegsProjection` + `BreakProjection` + `STATE_STATIC` |
| Hardcoded `Goal::C` & `Kit::BareLegs` | `engine/plan/src/goal.rs:7`, `solve.rs:8` | ~15 | `KitCone` + `KitView` / `KitCounts` |

---

## Spartan Kill Tests, Extreme & Chaotic Rungs, and Red-Team Mutants (`M1–M16`)

Every gate requires **positive control (pass)**, **red-team mutant (must fail)**, and **raw distributions** (`n, min, median, p90, p99, max`).

### 1. Reversible `WorldMut`, 4D `W(t)`, and Space-Time Index Gates
- **`E1-undo-depth` (Extreme, 10× ladder):** $10^1, 10^2, 10^3, 10^4$ mixed static + reactive edits with nested checkpoints and full rollback → byte-identical `StateId`, `model_id()`, `has_fluid()`, `has_inside_effects()`, and `(z, y, x)` collision visit order vs untouched snapshot; ≤ 50 ns/edit rollback; **0 heap allocs** for $k \le 16$.
- **`E1-far-coords` (Extreme):** Place/break/reach/rollback at $x, z \in \{10^4, 10^6, 2.9 \times 10^7\}$.
- **`E1-sticky-flag` (Extreme):** Place water source (`has_fluid() == true`), pick up / roll back → `has_fluid() == false`.
- **`E1-nonlocal-cert` (Extreme):** Single block edit at `(pos, t)` simultaneously invalidates (a) a flying arrow 30 blocks away, (b) a 40-block tripwire line, (c) a QC piston 2 blocks below, and (d) another bot's certified tube 200 ticks ahead — while touching 0 unrelated certificates in quiet sections.
- **Mutants (must fail):**
  - `M1`: omit cascaded writes from `Footprint::writes`.
  - `M2`: sticky `|=` bool on `extra_fluids`.
  - `M3`: return `None` from `model_id()` on edited cells.
  - `M12`: replace space-time footprint index with a 6-neighbor check (must fail `E1-nonlocal-cert` on arrow/tripwire/QC piston!).

### 2. Factored Kit Projection & Admissibility Gates
- **`E2-zero-tax-flat` (Control + *"unused item never raises T"*):** `real_time.rs` on mixed stone/planks/dirt flat ground with full Survival toolkit → median ≤ 15 µs, 0 corridor exceptions, T identical to bare legs; AND tunnel-dig gait over mixed stone/granite/diorite/andesite produces 0 false exceptions under its own read mask.
- **`E2-kit-combo` (Extreme scaling):** 10,000 bots carrying combinatorial mixes of armor enchants, potions, tools, and stack counts → total projection table memory **< 1 MB**, swap **≤ 100 ns**, `64 → 63` cobble triggers **0 rebuilds**, `1 → 0` triggers exact `verb_mask` event.
- **`Kill test 16` (`E2-cone-admissible`):** 1,000 random `(from, to, kit)` triples across bare legs, blocks, water/powder-snow clutch, boat on blue ice, pearls, elytra → 0 sentences beat `KitCone::lower_bound`, and non-bare kits lead from tick 0 (no bare-legs-first detour).
- **`E2-material-wall` (Extreme, 10× ladder):** Wall thickness $1, 10, 100$ of dirt / stone / obsidian / bedrock with bare hands vs Eff-V silk-touch pickaxe → exact T crossover between dig-and-restore vs detour; bedrock refused.
- **`E2-last-block` (S96 Shadow Price):** 1 block in hand with optional early shortcut (−15 T) and mandatory downstream gap (−200 T) → keeps block for downstream gap unless early edit self-closes.
- **Mutants (must fail):**
  - `M4`: leak break properties into a sprint-jump word's corridor mask (`E2-zero-tax-flat` fails).
  - `M5`: include raw stack counts in capability keys (`E2-kit-combo` fails).
  - `M6`: hardcode bare-legs `Goal::C` when pearl/boat/elytra is carried (`Kill test 16` fails).
  - `M13`: merge body + tool + static tables into one unfactored per-signature table (fails `E2-kit-combo` memory budget).

### 3. Law-Read-Backwards, `R-lego`, and Deterministic `ItemEntity` Gates
- **`R-lego-1` (Arises):** 5- and 6-block gap over superflat floor, blocks in hand → crossing found with **zero tool-specific search code** in **≤ 10³ verifications**, block restored, server-audited.
- **`R-lego-2` (Stays):** Second gap of different shape/height → pattern-key reuse at ≤ 20% of `R-lego-1`'s verifications.
- **`R-lego-3` (Compounds):** Run-up + sprint-jump + mid-air place under feet + restore → beats stop-and-place by ≥ 5 ticks.
- **`R-lego-4` (Zero-catalogue cross-tool):** Same causal join solves: (a) 1-thick stone wall (silk-touch pick), (b) closed oak door (bare hands), (c) 25-block fall (water bucket picked up at `t ≤ 4`, 0 spread).
- **`E3-void-law` (Extreme — law decides, not hardcoded "void = impossible"):**
  - Adjacent break over void with deterministic `JavaRandom` (or teammate adjacent) → law proves drop is catchable at tick 10 (`y ≈ block_y + 0.26` inside `2.6 × 2.8 × 2.6` pickup box) and restores cleanly.
  - Far uncatchable break over void (4.5 blocks away, no bot near) with strict restoration → law proves drop leaves reach and refuses as `Irreversible(ItemLost)`.
  - Void gaps of $10, 100, 1000$ blocks with `MissionPolicy::KeepEdits` → solve in $O(1)$ unique patterns as `(word, count)`.
- **`E3-nether-bucket` (Extreme):** 25-block fall with `water_evaporates = true`, carrying water + powder-snow bucket → law probe rejects water and picks powder snow automatically.
- **Mutants (must fail):**
  - `M7`: disable `Footprint::writes` causal index → `R-lego-1` and `R-lego-4` drop to 0 solutions.
  - `M8`: disable sturdy-neighbor-face check in `place.rs` → server/jar audit catches illegal placement.

### 4. Periodic World Gaits, Swarm Cooperation, and Chaotic Stress Gates
- **`E4-redstone-gait` (Extreme, 10× ladder):** 4-tick repeater clock, coupled 4×10 = 20-tick piston gate, and a translating slime/honey flying machine run for $10^2, 10^3, 10^5$ ticks → **0 ticks simulated** after cycle lock; O(1) `W(t)` queries bit-identical to stepped `SimWorld`; disturbance by a bot/arrow at arbitrary `t*` materializes exact `StateId`s and `LevelTicks` bit-identically and re-locks after settling.
- **`E5-swarm-coop` (Swarm):** 50 bots cross a ravine in 4D `W(t)`: Bot 1 places 1 bridge block, Bots 2..50 cross at full speed (0 extra edits), cleanup executes **once** after `(max, +)` `release_tick` by the cheapest capable bot; PLUS mid-sprint deterministic tool throw/catch (`drop_velocity_aimed`) between two bots when shadow price exceeds handoff `T`.
- **`E5-dense-pit` (Extreme density / Knudsen switch):** 500 bots colliding and editing in a 10×10 pit → hot section promotes to a local `Battle` frame stepping lockstep on `k = 1` with bit-identical results and dissolves back to Tier 0/1 when members separate, while outside streams stay lazy.
- **`E6-1B-scale` (Extreme 10× ladder: `10³ → 10⁶ → 10⁹` bots in ES `shipmem` Form P):** 1,000 Form-P streams totaling 10⁹ individual bots + 10⁶ Elias–Fano scars:
  - Total bot residency memory **< 32 MB** (fits inside 96 MB L3);
  - 1 block edit inside a 10⁶-bot stream triggers **1** space-time index wake-up and **O(1)** ordinal split (not O(10⁶));
  - Tier 0 ↔ Tier 1 (`seal` reabsorption) ↔ Tier 2 (`Battle` promotion/dissolution) produce **bit-identical** state digests;
  - Moving a human observer light cone across a 10⁷-bot highway materializes only visible bots and passes **`observation_never_changes_the_digest`**.
- **`C-edit-chaos` (Chaotic):** Stone wall + waterlogged slab + overhead sand + looping piston gate + redstone torch beside a 25-block drop + flying arrow + mid-run `R-retarget` with 2 open obligations + simulated crash & WAL recovery → 0 restoration misses, 0 irreversible losses, 0 items left lying, `ledger.open == 0`, server audit matches.
- **Live flight gate (the judge):** Julien's edit tracks (the five edit-and-restore tracks + `doorcorridor`, `trapdoorparkour`, `carpetwater`, `grassjumpeasy`) flown in his game with `T ≤ his best T`, 0 server corrections, 0 restoration misses.
- **Mutants (must fail):**
  - `M9`: omit cascaded writes from `ObligationLedger`.
  - `M10`: omit open-obligation detour / `release_tick` from `LB`.
  - `M11`: skip WAL flush before live edit packet.
  - `M14`: skip `WorldGait` materialization when an arrow or block edit hits its `read_footprint` (fails `E4-redstone-gait`).
  - `M15`: allocate a flat per-bot slot or iterate `b in bots` in the space-time index / obligation LB (fails `E6-1B-scale` RAM and latency budgets).
  - `M16`: let observer materialization mutate simulation state/RNG (fails `observation_never_changes_the_digest` in `E6-1B-scale`).

---

## Recommended execution order (for the integrator, after tonight's runs land)

1. **`wt/port-fold` next compression pass:** Swap `WorldSnapshot` to `StateId(u16)`, generate full 1,196-row `BLOCK_RULES` + `STATE_STATIC` + `STATE_DYNAMIC` from the 26.2 jar, build `WorldDelta` / 4D `W(t)` + unified space-time inverted footprint index + `WorldGait` cycle folder (deleting `WorldOverlay`, `SimCell.block`, and private world adapters), and pass `E1-*`, `E4-redstone-gait`, `M1–M3`, `M12`, `M14`.
2. **Factored `KitView` & `KitCone` on `main`:** Replace `codec.rs::ProjectionTable` and hardcoded `Goal::C` / `Kit::BareLegs` with factored `< 1 MB` projections and per-word read masks, pass `E2-*`, `Kill test 16`, `M4–M6`, `M13` (0 regression on `real_time.rs`).
3. **Causal join + Mission `ObligationLedger` + `R-lego` + Swarm coop:** Wire `Footprint::writes`, law-probed `VerbEffect`, backward causal join, mission-owned `ObligationLedger` + WAL + `(max, +)` `release_tick`, deterministic `ItemEntity` catch check, pass `R-lego-1..4`, `E3-*`, `E5-*`, `C-edit-chaos`, `M7–M11`, then fly the five edit tracks in his game.
