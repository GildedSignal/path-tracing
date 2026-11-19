//! 4D Forward Birth Wave `E(p)`, Backward Reverse-Demolition Wave `D(p)`,
//! emergent temporary scaffolding obligations, and Bernstein footprint DAG extraction.

use crate::blueprint::{
    body_intersects_block, ray_intermediate_cells, within_reach_4_5, Blueprint, Direction,
    Registry, StateId, SurvivalRule,
};
use law::{BlockPos, Vec3};
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, VecDeque};
use std::cmp::Reverse;

/// Capabilities carried by the building swarm for 4D Reverse-Demolition & DAG compilation.
#[derive(Clone, Debug)]
pub struct BuildKit {
    pub has_scaffolding: bool,
    pub has_ender_pearl: bool,
    pub has_out_of_vault_teleport: bool,
}

impl Default for BuildKit {
    fn default() -> Self {
        Self {
            has_scaffolding: true,
            has_ender_pearl: true,
            has_out_of_vault_teleport: false,
        }
    }
}

/// A compiled placement or temporary-edit node in the 4D causal DAG.
#[derive(Clone, Debug)]
pub struct BuildNode {
    pub idx: usize,
    pub pos: BlockPos,
    pub state: StateId,
    pub is_temporary: bool,
    /// Standing pocket `(x, y, z)` (feet block cell) from which the bot executes this placement.
    pub stand_pos: BlockPos,
    pub crouch_crawl: bool,
    /// Existing neighbor cell clicked against, and the face of `clicked_pos` that is clicked.
    pub clicked_pos: BlockPos,
    pub clicked_face: Direction,
    pub yaw: f32,
    /// Intermediate cells crossed by the sightline ray (must be air at placement tick).
    pub ray_cells: Vec<BlockPos>,
    /// Body cells occupied by the placing bot at `stand_pos` (must be air at placement tick).
    pub body_cells: Vec<BlockPos>,
    /// Exit corridor cells from `stand_pos` to exterior (must remain passable at placement tick).
    pub exit_cells: Vec<BlockPos>,
    /// Causal predecessors (indices into `CompiledBuild::nodes`) that MUST complete before this node.
    pub preds: Vec<usize>,
    /// Causal successors that depend on this node.
    pub succs: Vec<usize>,
    /// Forward earliest birth tick `E(p)` (1-tick per causal layer + travel).
    pub earliest_tick: u32,
    /// Backward critical-path tail `D(p)` (ticks before completion `T`).
    pub critical_tail: u32,
}

#[derive(Clone, Debug)]
pub struct CompiledBuild {
    pub nodes: Vec<BuildNode>,
    /// Temporary scaffolding root cells that must be broken during the cleanup wave once their dependents finish.
    pub scaffolding_roots: Vec<BlockPos>,
    /// All temporary scaffolding cells erected.
    pub scaffolding_cells: Vec<BlockPos>,
    /// Causal critical path floor `T_crit = max_p (E(p) + D(p))`.
    pub critical_path_ticks: u32,
    /// Total edits (permanent blocks + temporary scaffolding placements + root breaks).
    pub total_edits: usize,
}

/// Compile a `Blueprint` under a `BuildKit` into the 4D Bernstein causal DAG via coupled
/// forward reach/support (with emergent scaffolding) and backward reverse demolition.
pub fn compile_4d(
    bp: &Blueprint,
    reg: &Registry,
    kit: &BuildKit,
) -> Result<CompiledBuild, String> {
    if bp.target.is_empty() {
        return Ok(CompiledBuild {
            nodes: Vec::new(),
            scaffolding_roots: Vec::new(),
            scaffolding_cells: Vec::new(),
            critical_path_ticks: 0,
            total_edits: 0,
        });
    }

    // 1. Check whether any connected component of `bp.target` is disconnected from ground `y = bp.ground_y`,
    //    or whether a tall vertical column gains from an emergent temporary scaffolding spine.
    let (scaf_cells, scaf_roots) = synthesize_emergent_scaffolding(bp, reg, kit);

    // Working set of all blocks present at the peak of construction (blueprint + temporary scaffolding).
    let mut full_world: BTreeMap<BlockPos, (StateId, bool)> = BTreeMap::new();
    for (&pos, &st) in &bp.target {
        full_world.insert(pos, (st, false));
    }
    for &pos in &scaf_cells {
        full_world.entry(pos).or_insert((reg.scaffolding, true));
    }

    // 2. Build a rooted forward support tree from ground (`y == bp.ground_y - 1`) through `full_world`
    //    respecting `SurvivalRule` (`NeedBelow`, `NeedAbove`, `NeedSide`, `WaterBasin`, `Gravity`, `AnyNeighbor`).
    //    `support_depth[p]` is the minimum causal support depth from ground, and `support_parent[p]` is the
    //    chosen click-face neighbor `(clicked_pos, clicked_face)`.
    let (support_depth, support_parent, must_precede) =
        compute_forward_support_tree(bp, reg, &full_world)?;

    // 3. Backward 4D Reverse-Demolition Wave from `full_world` down to ground:
    //    Peel blocks in reverse time `tau` from the kit's exterior light cone inward,
    //    never demolishing a block `p` before any block `q` that depends on `p` in `must_precede`
    //    (its support children), and recording for each demolished block `p` its earliest reachable
    //    placement pose `(stand_pos, crouch_crawl, ray_cells, exit_cells)` in the remaining world.
    let demo_records = run_reverse_demolition(
        bp,
        reg,
        kit,
        &full_world,
        &support_depth,
        &support_parent,
        &must_precede,
    )?;

    // 4. Reverse the demolition sequence into forward construction order `0..M`,
    //    and extract the minimal 4D Bernstein causal DAG (`Present` support edges + `Absent` ray/body/exit edges).
    let mut nodes: Vec<BuildNode> = Vec::with_capacity(demo_records.len());
    let mut pos_to_idx: BTreeMap<BlockPos, usize> = BTreeMap::new();

    for (idx, rec) in demo_records.into_iter().rev().enumerate() {
        pos_to_idx.insert(rec.pos, idx);
        let state_entry = reg.entry(rec.state);
        let yaw = state_entry
            .required_yaw
            .unwrap_or_else(|| rec.clicked_face.opposite().canonical_yaw());
        nodes.push(BuildNode {
            idx,
            pos: rec.pos,
            state: rec.state,
            is_temporary: rec.is_temporary,
            stand_pos: rec.stand_pos,
            crouch_crawl: rec.crouch_crawl,
            clicked_pos: rec.clicked_pos,
            clicked_face: rec.clicked_face,
            yaw,
            ray_cells: rec.ray_cells,
            body_cells: rec.body_cells,
            exit_cells: rec.exit_cells,
            preds: Vec::new(),
            succs: Vec::new(),
            earliest_tick: 1,
            critical_tail: 1,
        });
    }

    // Add Bernstein footprint edges (guaranteed acyclic because every edge respects `u < v` in forward order).
    let n = nodes.len();
    let mut preds_set: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); n];

    for v in 0..n {
        let p_v = nodes[v].pos;
        // (A) Positive Premises (`Present`): `clicked_pos`, explicit `must_precede` (e.g. water basin walls, survival anchor),
        //     and the floor block under `stand_pos` (if it is a constructed block).
        let clicked = nodes[v].clicked_pos;
        if let Some(&u) = pos_to_idx.get(&clicked) {
            if u < v {
                preds_set[v].insert(u);
            }
        }
        if let Some(parents) = must_precede.get(&p_v) {
            for &par in parents {
                if let Some(&u) = pos_to_idx.get(&par) {
                    if u < v {
                        preds_set[v].insert(u);
                    }
                }
            }
        }
        let stand_floor = [
            nodes[v].stand_pos[0],
            nodes[v].stand_pos[1] - 1,
            nodes[v].stand_pos[2],
        ];
        if let Some(&u) = pos_to_idx.get(&stand_floor) {
            if u < v {
                preds_set[v].insert(u);
            }
        }
        // If standing inside a scaffolding cell itself, that scaffolding cell must exist first.
        if let Some(&u) = pos_to_idx.get(&nodes[v].stand_pos) {
            if u < v && nodes[u].is_temporary {
                preds_set[v].insert(u);
            }
        }
    }

    // (B) Negative Premises (`Absent`): if node `u < v` requires cell `c` to be AIR when `u` is placed
    //     (because `c` is in `u`'s `body_cells`, `ray_cells`, or `exit_cells`), and node `v > u` places a
    //     solid/occluding block at `c` (`!is_temporary`), then `u` MUST be placed before `v` (`u -> v`).
    for u in 0..n {
        let mut absent_cells = Vec::with_capacity(
            nodes[u].body_cells.len() + nodes[u].ray_cells.len() + nodes[u].exit_cells.len(),
        );
        absent_cells.extend_from_slice(&nodes[u].body_cells);
        absent_cells.extend_from_slice(&nodes[u].ray_cells);
        absent_cells.extend_from_slice(&nodes[u].exit_cells);
        for c in absent_cells {
            if let Some(&v) = pos_to_idx.get(&c) {
                if u < v && !nodes[v].is_temporary {
                    let st_v = reg.entry(nodes[v].state);
                    if !st_v.block.shape.is_empty() {
                        preds_set[v].insert(u);
                    }
                }
            }
        }
    }

    for v in 0..n {
        let pv: Vec<usize> = preds_set[v].iter().copied().collect();
        for &u in &pv {
            nodes[u].succs.push(v);
        }
        nodes[v].preds = pv;
    }

    // 5. Compute pure 4D Causal Layer Depth `E(p)` (forward birth wave) and Critical Tail `D(p)`
    //    (backward dependency tail) over the Bernstein DAG (1 causal layer per dependency step, zero made-up costs).
    for v in 0..n {
        let mut e = 1u32;
        for &u in &nodes[v].preds {
            e = e.max(nodes[u].earliest_tick + 1);
        }
        nodes[v].earliest_tick = e;
    }

    let mut max_span = 0u32;
    for u in (0..n).rev() {
        let mut tail = 1u32;
        for &v in &nodes[u].succs {
            tail = tail.max(nodes[v].critical_tail + 1);
        }
        nodes[u].critical_tail = tail;
        max_span = max_span.max(nodes[u].earliest_tick + tail - 1);
    }

    let total_edits = nodes.len() + scaf_roots.len();
    if !scaf_roots.is_empty() {
        // 1 extra causal layer at the end to break the scaffolding root(s).
        max_span += 1;
    }

    Ok(CompiledBuild {
        nodes,
        scaffolding_roots: scaf_roots,
        scaffolding_cells: scaf_cells,
        critical_path_ticks: max_span,
        total_edits,
    })
}

/// Synthesize emergent temporary `scaffolding` columns wherever:
/// 1. A component of `bp.target` is floating in mid-air above `bp.ground_y` (Extreme 4: Floating Sky Island), OR
/// 2. High blocks (such as the Pagoda's Sorin spire `y = -25..-10` or a `1x1xH` Sky Needle) sit above any nearby
///    walkable floor by more than 4 blocks, so a 1-tick/block scaffolding elevator beside them lowers `T`
///    and collapses in 1 root-break tick at the end.
fn synthesize_emergent_scaffolding(
    bp: &Blueprint,
    reg: &Registry,
    kit: &BuildKit,
) -> (Vec<BlockPos>, Vec<BlockPos>) {
    if !kit.has_scaffolding {
        return (Vec::new(), Vec::new());
    }

    let mut scaf_set: BTreeSet<BlockPos> = BTreeSet::new();
    let mut roots: BTreeSet<BlockPos> = BTreeSet::new();

    // Check connected components of `bp.target`: any component whose minimum `y > bp.ground_y` needs a
    // vertical scaffolding spine from `bp.ground_y` (or the block below) up to `anchor_cell[1] - 1`.
    // Note: never pick a `NeedAbove` block (like a hanging lantern) as the bottom support anchor of a component!
    let mut visited: BTreeSet<BlockPos> = BTreeSet::new();
    for &start in bp.target.keys() {
        if visited.contains(&start) {
            continue;
        }
        let mut comp = Vec::new();
        let mut q = VecDeque::new();
        q.push_back(start);
        visited.insert(start);
        let mut min_cell = start;
        let mut min_supportable: Option<BlockPos> = None;
        let mut max_y_cell = start;
        while let Some(cur) = q.pop_front() {
            comp.push(cur);
            if cur[1] < min_cell[1] {
                min_cell = cur;
            }
            let st = bp.target[&cur];
            if reg.entry(st).survival != SurvivalRule::NeedAbove
                && min_supportable.is_none_or(|m| cur[1] < m[1])
            {
                min_supportable = Some(cur);
            }
            if cur[1] > max_y_cell[1] {
                max_y_cell = cur;
            }
            for d in Direction::VALUES {
                let nb = d.relative(cur);
                if bp.target.contains_key(&nb) && visited.insert(nb) {
                    q.push_back(nb);
                }
            }
        }
        let anchor = min_supportable.unwrap_or(min_cell);
        if min_cell[1] > bp.ground_y {
            let x = anchor[0];
            let z = anchor[2];
            let base_y = (bp.ground_y..anchor[1])
                .rev()
                .find(|&y| bp.target.contains_key(&[x, y, z]) || scaf_set.contains(&[x, y, z]))
                .map(|y| y + 1)
                .unwrap_or(bp.ground_y);
            if base_y < anchor[1] {
                roots.insert([x, base_y, z]);
                for y in base_y..anchor[1] {
                    scaf_set.insert([x, y, z]);
                }
            }
        }

        // Also check if the component itself has a tall vertical span (`max_y - min_y >= 8`) where an adjacent
        // vertical scaffolding column provides an unobstructed 1-tick/block climb.
        let span_y = max_y_cell[1] - min_cell[1];
        if span_y >= 8 {
            // Find the nearest empty `(x, z)` column within 2 blocks of `(max_y_cell[0], max_y_cell[2])`
            // that is clear of `bp.target` from `bp.ground_y` to `max_y_cell[1] - 1`, or clear in its upper span.
            let cx = max_y_cell[0];
            let cz = max_y_cell[2];
            let mut candidates = Vec::new();
            for dx in -3i32..=3 {
                for dz in -3i32..=3 {
                    if dx == 0 && dz == 0 {
                        continue;
                    }
                    if dx * dx + dz * dz <= 9 {
                        candidates.push((dx * dx + dz * dz, [cx + dx, cz + dz]));
                    }
                }
            }
            candidates.sort_by_key(|&(d2, _)| d2);
            for (_, [sx, sz]) in candidates {
                // Find the highest blueprint block in column `(sx, sz)` below `max_y_cell[1]`.
                let base_y = (bp.ground_y..max_y_cell[1])
                    .rev()
                    .find(|&y| bp.target.contains_key(&[sx, y, sz]))
                    .map(|y| y + 1)
                    .unwrap_or(bp.ground_y);
                if max_y_cell[1] - base_y >= 6 {
                    let all_clear = (base_y..max_y_cell[1])
                        .all(|y| !bp.target.contains_key(&[sx, y, sz]));
                    if all_clear {
                        roots.insert([sx, base_y, sz]);
                        for y in base_y..max_y_cell[1] {
                            scaf_set.insert([sx, y, sz]);
                        }
                        break;
                    }
                }
            }
        }
    }

    (
        scaf_set.into_iter().collect(),
        roots.into_iter().collect(),
    )
}

fn compute_forward_support_tree(
    bp: &Blueprint,
    reg: &Registry,
    full_world: &BTreeMap<BlockPos, (StateId, bool)>,
) -> Result<
    (
        BTreeMap<BlockPos, u32>,
        BTreeMap<BlockPos, (BlockPos, Direction)>,
        BTreeMap<BlockPos, Vec<BlockPos>>,
    ),
    String,
> {
    let mut must_precede: BTreeMap<BlockPos, Vec<BlockPos>> = BTreeMap::new();
    for (&p, &(st, is_temp)) in full_world {
        if is_temp {
            if p[1] > bp.ground_y {
                let below = [p[0], p[1] - 1, p[2]];
                if full_world.contains_key(&below) {
                    must_precede.entry(p).or_default().push(below);
                }
            }
            continue;
        }
        match reg.entry(st).survival {
            SurvivalRule::NeedBelow | SurvivalRule::Gravity => {
                let below = [p[0], p[1] - 1, p[2]];
                if below[1] >= bp.ground_y && full_world.contains_key(&below) {
                    must_precede.entry(p).or_default().push(below);
                }
            }
            SurvivalRule::NeedAbove => {
                let above = [p[0], p[1] + 1, p[2]];
                if full_world.contains_key(&above) {
                    must_precede.entry(p).or_default().push(above);
                }
            }
            SurvivalRule::NeedSide(dir) => {
                let side = dir.relative(p);
                if full_world.contains_key(&side) {
                    must_precede.entry(p).or_default().push(side);
                }
            }
            SurvivalRule::WaterBasin => {
                let below = [p[0], p[1] - 1, p[2]];
                if below[1] >= bp.ground_y && full_world.contains_key(&below) {
                    must_precede.entry(p).or_default().push(below);
                }
                for d in Direction::HORIZONTAL {
                    let nb = d.relative(p);
                    if let Some(&(nb_st, _)) = full_world.get(&nb) {
                        if reg.entry(nb_st).survival != SurvivalRule::WaterBasin {
                            must_precede.entry(p).or_default().push(nb);
                        }
                    }
                }
            }
            SurvivalRule::AnyNeighbor => {}
        }
    }

    // Dijkstra-like wave from ground (`y == bp.ground_y`) to assign every block its support depth and click parent.
    let mut depth: BTreeMap<BlockPos, u32> = BTreeMap::new();
    let mut parent: BTreeMap<BlockPos, (BlockPos, Direction)> = BTreeMap::new();
    let mut pq: BinaryHeap<Reverse<(u32, BlockPos)>> = BinaryHeap::new();

    // Seed blocks at `y == bp.ground_y` whose mandatory `must_precede` are empty (supported directly by `y = ground_y - 1`).
    for (&p, _) in full_world {
        if p[1] == bp.ground_y && must_precede.get(&p).is_none_or(|v| v.is_empty()) {
            depth.insert(p, 1);
            parent.insert(p, ([p[0], bp.ground_y - 1, p[2]], Direction::Up));
            pq.push(Reverse((1, p)));
        }
    }

    while let Some(Reverse((d_u, u))) = pq.pop() {
        if depth.get(&u).copied().unwrap_or(u32::MAX) < d_u {
            continue;
        }
        for dir in Direction::VALUES {
            let v = dir.relative(u);
            if !full_world.contains_key(&v) {
                continue;
            }
            // Check if all mandatory `must_precede[v]` already have a depth assigned.
            let mut max_req = d_u;
            let mut req_parent = (u, dir);
            let mut ready = true;
            if let Some(reqs) = must_precede.get(&v) {
                for &r in reqs {
                    if let Some(&dr) = depth.get(&r) {
                        if dr >= max_req {
                            max_req = dr;
                            // Compute direction from `r` to `v`.
                            for d2 in Direction::VALUES {
                                if d2.relative(r) == v {
                                    req_parent = (r, d2);
                                }
                            }
                        }
                    } else {
                        ready = false;
                        break;
                    }
                }
            }
            if !ready {
                continue;
            }
            let cand = max_req + 1;
            if cand < depth.get(&v).copied().unwrap_or(u32::MAX) {
                depth.insert(v, cand);
                parent.insert(v, req_parent);
                pq.push(Reverse((cand, v)));
            }
        }
    }

    if depth.len() != full_world.len() {
        // Any remaining unreached blocks (e.g. a water basin cell whose border and water depended on each other)
        // are seeded once their mandatory survival anchor (`NeedBelow`, `NeedAbove`, `NeedSide`) already has `depth`.
        let mut changed = true;
        while changed && depth.len() < full_world.len() {
            changed = false;
            let unreached: Vec<BlockPos> = full_world
                .keys()
                .copied()
                .filter(|k| !depth.contains_key(k))
                .collect();
            for v in unreached {
                let (st_v, is_temp_v) = full_world[&v];
                if !is_temp_v {
                    let surv_ok = match reg.entry(st_v).survival {
                        SurvivalRule::NeedBelow | SurvivalRule::Gravity | SurvivalRule::WaterBasin => {
                            let below = [v[0], v[1] - 1, v[2]];
                            below[1] < bp.ground_y || depth.contains_key(&below)
                        }
                        SurvivalRule::NeedAbove => depth.contains_key(&[v[0], v[1] + 1, v[2]]),
                        SurvivalRule::NeedSide(d) => depth.contains_key(&d.relative(v)),
                        SurvivalRule::AnyNeighbor => true,
                    };
                    if !surv_ok {
                        continue;
                    }
                }
                let mut best_u: Option<(u32, BlockPos, Direction)> = None;
                for dir in Direction::VALUES {
                    let u = dir.opposite().relative(v);
                    if let Some(&du) = depth.get(&u) {
                        if best_u.is_none_or(|b| du < b.0) {
                            best_u = Some((du, u, dir));
                        }
                    }
                }
                if let Some((du, u, dir)) = best_u {
                    depth.insert(v, du + 1);
                    parent.insert(v, (u, dir));
                    let list = must_precede.entry(v).or_default();
                    list.retain(|r| depth.contains_key(r));
                    if !list.contains(&u) {
                        list.push(u);
                    }
                    changed = true;
                }
            }
        }
    }

    if depth.len() != full_world.len() {
        let missing: Vec<BlockPos> = full_world
            .keys()
            .filter(|&k| !depth.contains_key(k))
            .take(5)
            .copied()
            .collect();
        return Err(format!(
            "blueprint has {} unsupported blocks (first: {:?})",
            full_world.len() - depth.len(),
            missing
        ));
    }

    // Also ensure every block's chosen `parent` is added to `must_precede` so reverse demolition never removes
    // a block's chosen click-face parent before the block itself!
    for (&v, &(p_pos, _)) in &parent {
        if p_pos[1] >= bp.ground_y && full_world.contains_key(&p_pos) {
            let list = must_precede.entry(v).or_default();
            if !list.contains(&p_pos) {
                list.push(p_pos);
            }
        }
    }

    Ok((depth, parent, must_precede))
}

struct DemoRecord {
    pos: BlockPos,
    state: StateId,
    is_temporary: bool,
    stand_pos: BlockPos,
    crouch_crawl: bool,
    clicked_pos: BlockPos,
    clicked_face: Direction,
    ray_cells: Vec<BlockPos>,
    body_cells: Vec<BlockPos>,
    exit_cells: Vec<BlockPos>,
}

fn run_reverse_demolition(
    bp: &Blueprint,
    reg: &Registry,
    kit: &BuildKit,
    full_world: &BTreeMap<BlockPos, (StateId, bool)>,
    support_depth: &BTreeMap<BlockPos, u32>,
    support_parent: &BTreeMap<BlockPos, (BlockPos, Direction)>,
    must_precede: &BTreeMap<BlockPos, Vec<BlockPos>>,
) -> Result<Vec<DemoRecord>, String> {
    // Build reverse dependency counts: if `u` is in `must_precede[v]`, then `v` must be demolished BEFORE `u`.
    let mut support_children: BTreeMap<BlockPos, Vec<BlockPos>> = BTreeMap::new();
    let mut rem_children: BTreeMap<BlockPos, usize> = BTreeMap::new();
    for &p in full_world.keys() {
        rem_children.insert(p, 0);
    }
    for (&v, parents) in must_precede {
        for &u in parents {
            if full_world.contains_key(&u) {
                support_children.entry(u).or_default().push(v);
                *rem_children.get_mut(&u).unwrap() += 1;
            }
        }
    }

    // Bounding box padded by 3 blocks so bots can walk/stand outside the blueprint freely.
    let lo = [bp.min[0] - 3, bp.ground_y, bp.min[2] - 3];
    let hi = [bp.max[0] + 3, bp.max[1] + 3, bp.max[2] + 3];

    let mut remaining: BTreeMap<BlockPos, (StateId, bool)> = full_world.clone();

    // Maintain exterior reachability tree over currently passable cells in `remaining`!
    // A cell `c` is passable for the bot body if `c` contains no solid block (or contains climbable scaffolding!).
    let is_passable = |p: BlockPos, rem: &BTreeMap<BlockPos, (StateId, bool)>| -> bool {
        if p[1] < bp.ground_y {
            return false;
        }
        match rem.get(&p) {
            None => true,
            Some(&(st, is_temp)) => is_temp || reg.entry(st).block.shape.is_empty(),
        }
    };

    // Track which cells are currently connected to the exterior boundary (`x == lo[0]` or `hi[0]`, `z == lo[2]` or `hi[2]`),
    // along with a parent pointer to reconstruct a short exit corridor `exit_cells`.
    let mut ext_parent: BTreeMap<BlockPos, Option<BlockPos>> = BTreeMap::new();
    let mut bfs: VecDeque<BlockPos> = VecDeque::new();

    if kit.has_out_of_vault_teleport {
        // Out-of-vault teleport (stasis chamber / chorus): any passable cell is directly escapable in 1 tick.
        for x in lo[0]..=hi[0] {
            for y in lo[1]..=hi[1] {
                for z in lo[2]..=hi[2] {
                    let c = [x, y, z];
                    if is_passable(c, &remaining) {
                        ext_parent.insert(c, None);
                    }
                }
            }
        }
    } else {
        for x in lo[0]..=hi[0] {
            for y in lo[1]..=hi[1] {
                for z in lo[2]..=hi[2] {
                    if x == lo[0] || x == hi[0] || z == lo[2] || z == hi[2] || y == hi[1] {
                        let c = [x, y, z];
                        if is_passable(c, &remaining) && !ext_parent.contains_key(&c) {
                            ext_parent.insert(c, None);
                            bfs.push_back(c);
                        }
                    }
                }
            }
        }
        expand_exterior(&mut bfs, &mut ext_parent, lo, hi, &remaining, &is_passable);
    }

    // Priority queue of blocks ready to be demolished (`rem_children[p] == 0`):
    // Prefer demolishing permanent blocks before temporary scaffolding (`is_temp == false` first),
    // and higher `support_depth` first so leaf blocks peel cleanly before trunks.
    let mut ready_q: BinaryHeap<(bool, u32, BlockPos)> = BinaryHeap::new();
    for (&p, &(_, is_temp)) in &remaining {
        if rem_children[&p] == 0 {
            let d = support_depth[&p];
            ready_q.push((!is_temp, d, p));
        }
    }

    let mut deferred: Vec<(bool, u32, BlockPos)> = Vec::new();
    let mut records: Vec<DemoRecord> = Vec::with_capacity(full_world.len());

    while records.len() < full_world.len() {
        let mut progressed = false;

        while let Some((not_temp, d, p)) = ready_q.pop() {
            if !remaining.contains_key(&p) {
                continue;
            }
            let (st, is_temp) = remaining[&p];
            let (clicked_pos, clicked_face) = support_parent[&p];

            if let Some((stand_pos, crouch_crawl, ray_cells, body_cells, exit_cells)) =
                find_demolition_pose(
                    p,
                    st,
                    clicked_face,
                    bp,
                    reg,
                    kit,
                    &remaining,
                    &ext_parent,
                    &is_passable,
                )
            {
                // Demolish `p`!
                remaining.remove(&p);
                records.push(DemoRecord {
                    pos: p,
                    state: st,
                    is_temporary: is_temp,
                    stand_pos,
                    crouch_crawl,
                    clicked_pos,
                    clicked_face,
                    ray_cells,
                    body_cells,
                    exit_cells,
                });
                progressed = true;

                // Cell `p` is now air: expand exterior reachability through `p`!
                if kit.has_out_of_vault_teleport {
                    ext_parent.insert(p, None);
                } else {
                    for dir in Direction::VALUES {
                        let nb = dir.relative(p);
                        if ext_parent.contains_key(&nb) {
                            ext_parent.insert(p, Some(nb));
                            bfs.push_back(p);
                            expand_exterior(
                                &mut bfs,
                                &mut ext_parent,
                                lo,
                                hi,
                                &remaining,
                                &is_passable,
                            );
                            break;
                        }
                    }
                }

                // Wake any support parent `u` whose `rem_children` drops to 0.
                if let Some(parents) = must_precede.get(&p) {
                    for &u in parents {
                        if let Some(cnt) = rem_children.get_mut(&u) {
                            *cnt = cnt.saturating_sub(1);
                            if *cnt == 0 && remaining.contains_key(&u) {
                                let (_, u_temp) = remaining[&u];
                                ready_q.push((!u_temp, support_depth[&u], u));
                            }
                        }
                    }
                }

                // Re-insert any deferred candidates now that `ext_parent` expanded.
                for item in deferred.drain(..) {
                    ready_q.push(item);
                }
            } else {
                deferred.push((not_temp, d, p));
            }
        }

        if !progressed {
            // None of the currently childless blocks had an unoccluded exterior pose (e.g., a hermetically sealed vault
            // whose outer wall is currently supporting an inner ceiling block while the inner ceiling supports a chandelier!).
            // In a sealed vault, a wall block `w` on the exterior shell CAN be removed first (as a breach tunnel) as long as
            // the ceiling above it has alternative neighbor support in `remaining`!
            if let Some(breach_pos) = rewire_support_for_breach(
                bp,
                reg,
                kit,
                &remaining,
                &ext_parent,
                &is_passable,
                &mut support_children,
                &mut rem_children,
            ) {
                let (_, b_temp) = remaining[&breach_pos];
                for item in deferred.drain(..) {
                    ready_q.push(item);
                }
                ready_q.push((!b_temp, support_depth[&breach_pos], breach_pos));
                continue;
            }

            // Fallback: pick the deferred block closest to the exterior boundary and relax sightline occlusion,
            // while still choosing a passable `stand_pos` within 4.5m reach that is not an undemolished solid block.
            if let Some((_not_temp, _d, p)) = deferred.pop() {
                for item in deferred.drain(..) {
                    ready_q.push(item);
                }
                let (st, is_temp) = remaining.remove(&p).unwrap();
                let (clicked_pos, clicked_face) = support_parent[&p];
                let stand_pos = find_fallback_stand_pos(p, st, bp.ground_y, reg, &remaining, &ext_parent, &is_passable);
                records.push(DemoRecord {
                    pos: p,
                    state: st,
                    is_temporary: is_temp,
                    stand_pos,
                    crouch_crawl: false,
                    clicked_pos,
                    clicked_face,
                    ray_cells: Vec::new(),
                    body_cells: vec![stand_pos, [stand_pos[0], stand_pos[1] + 1, stand_pos[2]]],
                    exit_cells: Vec::new(),
                });
                for dir in Direction::VALUES {
                    let nb = dir.relative(p);
                    if ext_parent.contains_key(&nb) {
                        ext_parent.insert(p, Some(nb));
                        bfs.push_back(p);
                        expand_exterior(&mut bfs, &mut ext_parent, lo, hi, &remaining, &is_passable);
                        break;
                    }
                }
                if let Some(parents) = must_precede.get(&p) {
                    for &u in parents {
                        if let Some(cnt) = rem_children.get_mut(&u) {
                            *cnt = cnt.saturating_sub(1);
                            if *cnt == 0 && remaining.contains_key(&u) {
                                let (_, u_temp) = remaining[&u];
                                ready_q.push((!u_temp, support_depth[&u], u));
                            }
                        }
                    }
                }
            } else {
                return Err("reverse demolition stalled with empty queue".into());
            }
        }
    }

    Ok(records)
}

fn expand_exterior(
    bfs: &mut VecDeque<BlockPos>,
    ext_parent: &mut BTreeMap<BlockPos, Option<BlockPos>>,
    lo: BlockPos,
    hi: BlockPos,
    remaining: &BTreeMap<BlockPos, (StateId, bool)>,
    is_passable: &impl Fn(BlockPos, &BTreeMap<BlockPos, (StateId, bool)>) -> bool,
) {
    while let Some(cur) = bfs.pop_front() {
        for dir in Direction::VALUES {
            let nb = dir.relative(cur);
            if nb[0] < lo[0]
                || nb[0] > hi[0]
                || nb[1] < lo[1]
                || nb[1] > hi[1]
                || nb[2] < lo[2]
                || nb[2] > hi[2]
            {
                continue;
            }
            if !ext_parent.contains_key(&nb) && is_passable(nb, remaining) {
                ext_parent.insert(nb, Some(cur));
                bfs.push_back(nb);
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn find_demolition_pose(
    target: BlockPos,
    state: StateId,
    clicked_face: Direction,
    bp: &Blueprint,
    reg: &Registry,
    kit: &BuildKit,
    remaining: &BTreeMap<BlockPos, (StateId, bool)>,
    ext_parent: &BTreeMap<BlockPos, Option<BlockPos>>,
    is_passable: &impl Fn(BlockPos, &BTreeMap<BlockPos, (StateId, bool)>) -> bool,
) -> Option<(BlockPos, bool, Vec<BlockPos>, Vec<BlockPos>, Vec<BlockPos>)> {
    let ground_y = bp.ground_y;
    let target_shape = &reg.entry(state).block.shape;

    // Search candidate standing cells `s` within 4 blocks of `target`.
    let mut best: Option<(i32, BlockPos, bool, Vec<BlockPos>, Vec<BlockPos>)> = None;

    for dx in -4i32..=4 {
        for dy in -4i32..=2 {
            for dz in -4i32..=4 {
                if dx * dx + dy * dy + dz * dz > 20 {
                    continue;
                }
                let s = [target[0] + dx, (target[1] + dy).max(ground_y), target[2] + dz];
                if s == target || !ext_parent.contains_key(&s) || !is_passable(s, remaining) {
                    continue;
                }
                let head = [s[0], s[1] + 1, s[2]];
                let head_ok = head != target && is_passable(head, remaining);
                let crouch_crawl = !head_ok;
                if crouch_crawl && !kit.has_ender_pearl {
                    // Without a 1x1 crawl/pearl capability, standing requires 2 vertical cells `[s, head]`.
                    continue;
                }
                if body_intersects_block(s, crouch_crawl, target, target_shape) {
                    continue;
                }
                let eye = Vec3::new(
                    s[0] as f64 + 0.5,
                    s[1] as f64 + if crouch_crawl { 0.4 } else { 1.62 },
                    s[2] as f64 + 0.5,
                );
                if !within_reach_4_5(eye, target) {
                    continue;
                }

                // Check if the ray from `eye` to `target` hits any solid non-temporary block in `remaining`.
                let ray = ray_intermediate_cells(eye, target, clicked_face);
                let occluded = ray.iter().any(|&r| {
                    r != target
                        && remaining
                            .get(&r)
                            .is_some_and(|&(st, is_temp)| !is_temp && !reg.entry(st).block.shape.is_empty())
                });
                if occluded {
                    continue;
                }

                // Score pose: strongly prefer cells that are never part of the blueprint (so placements don't
                // unnecessarily block each other's standing cells), prefer solid support, and prefer close distance.
                let below = [s[0], s[1] - 1, s[2]];
                let supported = s[1] == ground_y
                    || remaining.contains_key(&below)
                    || remaining.get(&s).is_some_and(|&(_, is_temp)| is_temp);
                let in_blueprint = bp.target.contains_key(&s) || bp.target.contains_key(&head);
                let score = (if in_blueprint { 40 } else { 0 })
                    + (if supported { 0 } else { 100 })
                    + (if crouch_crawl { 20 } else { 0 })
                    + (dx * dx + dy * dy + dz * dz);

                let body_cells = if crouch_crawl {
                    vec![s]
                } else {
                    vec![s, head]
                };

                if best.as_ref().is_none_or(|b| score < b.0) {
                    best = Some((score, s, crouch_crawl, ray, body_cells));
                }
            }
        }
    }

    let (_, stand_pos, crouch_crawl, ray_cells, body_cells) = best?;

    // Trace up to 6 steps of the exit path from `stand_pos` through `ext_parent` so the immediate exit aperture stays clear.
    let mut exit_cells = Vec::new();
    let mut cur = ext_parent.get(&stand_pos).copied().flatten();
    let mut steps = 0;
    while let Some(c) = cur {
        if c != target {
            exit_cells.push(c);
        }
        steps += 1;
        if steps >= 6 {
            break;
        }
        cur = ext_parent.get(&c).copied().flatten();
    }

    Some((stand_pos, crouch_crawl, ray_cells, body_cells, exit_cells))
}

/// When an outer wall block `w` is reachable from the exterior, but `rem_children[w] > 0` because
/// the initial forward BFS routed a ceiling/neighbor block `v`'s support through `w`, check if `v`
/// has an alternative adjacent block `alt != w` in `remaining` that can support `v` instead.
/// If so, rewire `v` to `alt`, freeing `w` to be demolished as an entry breach tunnel!
#[allow(clippy::too_many_arguments)]
fn rewire_support_for_breach(
    bp: &Blueprint,
    reg: &Registry,
    kit: &BuildKit,
    remaining: &BTreeMap<BlockPos, (StateId, bool)>,
    ext_parent: &BTreeMap<BlockPos, Option<BlockPos>>,
    is_passable: &impl Fn(BlockPos, &BTreeMap<BlockPos, (StateId, bool)>) -> bool,
    support_children: &mut BTreeMap<BlockPos, Vec<BlockPos>>,
    rem_children: &mut BTreeMap<BlockPos, usize>,
) -> Option<BlockPos> {
    for (&w, &(st_w, _)) in remaining {
        if rem_children.get(&w).copied().unwrap_or(0) == 0 {
            continue;
        }
        // Check if `w` can be reached and demolished from the current exterior right now.
        if find_demolition_pose(
            w,
            st_w,
            Direction::Up,
            bp,
            reg,
            kit,
            remaining,
            ext_parent,
            is_passable,
        )
        .is_none()
        {
            continue;
        }
        // Check if every child `v` of `w` has `AnyNeighbor` survival and at least one OTHER neighbor `alt != w` in `remaining`.
        let children = support_children.get(&w).cloned().unwrap_or_default();
        let can_rewire = children.iter().all(|&v| {
            if !remaining.contains_key(&v) {
                return true;
            }
            let (st_v, _) = remaining[&v];
            if reg.entry(st_v).survival != SurvivalRule::AnyNeighbor {
                return false;
            }
            Direction::VALUES.iter().any(|&d| {
                let alt = d.relative(v);
                alt != w && (alt[1] < bp.ground_y || remaining.contains_key(&alt))
            })
        });
        if can_rewire {
            rem_children.insert(w, 0);
            return Some(w);
        }
    }
    None
}

fn find_fallback_stand_pos(
    target: BlockPos,
    state: StateId,
    ground_y: i32,
    reg: &Registry,
    remaining: &BTreeMap<BlockPos, (StateId, bool)>,
    ext_parent: &BTreeMap<BlockPos, Option<BlockPos>>,
    is_passable: &impl Fn(BlockPos, &BTreeMap<BlockPos, (StateId, bool)>) -> bool,
) -> BlockPos {
    let target_shape = &reg.entry(state).block.shape;
    for dx in -4i32..=4 {
        for dy in -4i32..=2 {
            for dz in -4i32..=4 {
                let s = [target[0] + dx, (target[1] + dy).max(ground_y), target[2] + dz];
                let head = [s[0], s[1] + 1, s[2]];
                if s == target || head == target {
                    continue;
                }
                if !is_passable(s, remaining) || !is_passable(head, remaining) {
                    continue;
                }
                if body_intersects_block(s, false, target, target_shape) {
                    continue;
                }
                let eye = Vec3::new(s[0] as f64 + 0.5, s[1] as f64 + 1.62, s[2] as f64 + 0.5);
                if within_reach_4_5(eye, target) && ext_parent.contains_key(&s) {
                    return s;
                }
            }
        }
    }
    for dx in -4i32..=4 {
        for dy in -4i32..=2 {
            for dz in -4i32..=4 {
                let s = [target[0] + dx, (target[1] + dy).max(ground_y), target[2] + dz];
                let head = [s[0], s[1] + 1, s[2]];
                if s == target || head == target {
                    continue;
                }
                if !is_passable(s, remaining) || !is_passable(head, remaining) {
                    continue;
                }
                if body_intersects_block(s, false, target, target_shape) {
                    continue;
                }
                let eye = Vec3::new(s[0] as f64 + 0.5, s[1] as f64 + 1.62, s[2] as f64 + 0.5);
                if within_reach_4_5(eye, target) {
                    return s;
                }
            }
        }
    }
    [target[0] + 2, target[1].max(ground_y), target[2]]
}
