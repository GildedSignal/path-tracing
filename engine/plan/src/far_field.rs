//! Persistent multi-scale `16³` portal graph & backward `(min, +)` field (`scales.md` §4.3).
//! Geometry is partitioned into `4×4` section tiles while preserving thin bridges and doorways;
//! stored per `(provenance, goal, kit_key)` and repaired per `16³` section on world edits.
use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
    sync::Arc,
};
use law::{
    BlockPos, FluidKind, PlayerState, World,
    codec::LabelId,
};
use crate::{
    Face, Goal,
    defect::{Legs, below, blockers, body_fits, has_step_support, way_clear},
    kit::{FieldEdit, KitView},
    source::CorridorSource,
    store::GeneratorStore,
};

#[derive(Clone, Debug)]
pub(crate) struct FarNode {
    pub rep: BlockPos,
    pub min: BlockPos,
    pub max: BlockPos,
    pub solid_below: bool,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct SectionSummary {
    pub nodes: Vec<FarNode>,
}

impl SectionSummary {
    pub(crate) fn build(w: &impl World, sec: BlockPos) -> Option<Self> {
        let lo = [sec[0] * 16, sec[1] * 16, sec[2] * 16];
        let hi = [lo[0] + 15, lo[1] + 15, lo[2] + 15];
        if !w.loaded(lo) {
            return None;
        }
        // Uniform O(1) section fast-paths: solid underground or empty sky air emit 0 nodes.
        if let Some(b) = w.uniform(lo, hi) {
            if !b.never_collides() {
                return Some(Self::default());
            }
            if b.never_collides() && !b.climbable && b.fluid.kind == FluidKind::Empty {
                let below_lo = [lo[0], lo[1] - 1, lo[2]];
                let below_hi = [hi[0], lo[1] - 1, hi[2]];
                if let Some(bb) = w.uniform(below_lo, below_hi) {
                    if bb.never_collides() && !bb.climbable && bb.fluid.kind == FluidKind::Empty {
                        return Some(Self::default());
                    }
                    let above_clear = w.uniform([lo[0], lo[1] + 1, lo[2]], [hi[0], lo[1] + 1, hi[2]])
                        .is_some_and(|ab| ab.never_collides() && !ab.has_inside_effects());
                    if above_clear && (!bb.never_collides() || bb.climbable) {
                        let solid_below = bb.shape.is_block();
                        let mut nodes = Vec::with_capacity(16);
                        for tz in 0..4 {
                            for tx in 0..4 {
                                let x0 = lo[0] + tx * 4;
                                let z0 = lo[2] + tz * 4;
                                nodes.push(FarNode {
                                    rep: [x0 + 1, lo[1], z0 + 1],
                                    min: [x0, lo[1], z0],
                                    max: [x0 + 3, lo[1], z0 + 3],
                                    solid_below,
                                });
                            }
                        }
                        return Some(Self { nodes });
                    }
                }
            }
        }

        let mut nodes = Vec::new();
        for tz in 0..4 {
            for tx in 0..4 {
                let bx = lo[0] + tx * 4;
                let bz = lo[2] + tz * 4;
                for ly in 0..16 {
                    let y = lo[1] + ly;
                    let mut mask = 0u16;
                    let mut solid = 0u16;
                    for sz in 0..4 {
                        for sx in 0..4 {
                            let p = [bx + sx, y, bz + sz];
                            if w.known_block(p).is_err() {
                                continue;
                            }
                            if has_step_support(w, p) && body_fits(w, p) {
                                let bit = 1u16 << (sz * 4 + sx);
                                mask |= bit;
                                let sb = w.block(below(p));
                                let pb = w.block(p);
                                if sb.shape.is_block() && pb.never_collides() && pb.fluid.kind == FluidKind::Empty {
                                    solid |= bit;
                                }
                            }
                        }
                    }
                    while mask != 0 {
                        let seed = mask.trailing_zeros() as usize;
                        mask &= !(1u16 << seed);
                        let mut comp = vec![seed];
                        let mut head = 0usize;
                        let mut min_x = bx + (seed % 4) as i32;
                        let mut max_x = min_x;
                        let mut min_z = bz + (seed / 4) as i32;
                        let mut max_z = min_z;
                        while head < comp.len() {
                            let c = comp[head];
                            head += 1;
                            let cx = (c % 4) as i32;
                            let cz = (c / 4) as i32;
                            let cp = [bx + cx, y, bz + cz];
                            for [dx, dz] in [[1, 0], [-1, 0], [0, 1], [0, -1]] {
                                let (nx, nz) = (cx + dx, cz + dz);
                                if !(0..4).contains(&nx) || !(0..4).contains(&nz) {
                                    continue;
                                }
                                let ni = (nz * 4 + nx) as usize;
                                let nbit = 1u16 << ni;
                                if (mask & nbit) == 0 {
                                    continue;
                                }
                                let np = [bx + nx, y, bz + nz];
                                if way_clear(w, cp, np) {
                                    mask &= !nbit;
                                    comp.push(ni);
                                    min_x = min_x.min(np[0]);
                                    max_x = max_x.max(np[0]);
                                    min_z = min_z.min(np[2]);
                                    max_z = max_z.max(np[2]);
                                }
                            }
                        }
                        let cx = (min_x + max_x) / 2;
                        let cz = (min_z + max_z) / 2;
                        let mut best_cell = comp[0];
                        let mut best_score = i32::MAX;
                        for &c in &comp {
                            let p = [bx + (c % 4) as i32, y, bz + (c / 4) as i32];
                            let is_solid = (solid & (1u16 << c)) != 0;
                            let score = (p[0] - cx).abs() + (p[2] - cz).abs() - if is_solid { 100 } else { 0 };
                            if score < best_score {
                                best_score = score;
                                best_cell = c;
                            }
                        }
                        let rep = [bx + (best_cell % 4) as i32, y, bz + (best_cell / 4) as i32];
                        let solid_below = (solid & (1u16 << best_cell)) != 0;
                        nodes.push(FarNode {
                            rep,
                            min: [min_x, y, min_z],
                            max: [max_x, y, max_z],
                            solid_below,
                        });
                    }
                }
            }
        }
        Some(Self { nodes })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct FarKey {
    pub provenance: [u8; 32],
    pub goal_cell: BlockPos,
    pub goal_face: Option<Face>,
    pub kit_key: u64,
}

#[derive(Default)]
pub(crate) struct FarCache {
    sections: HashMap<([u8; 32], BlockPos), Arc<SectionSummary>>,
    fields: HashMap<FarKey, Arc<FarField>>,
}

impl FarCache {
    pub fn invalidate_section(&mut self, sec: BlockPos) {
        self.sections.retain(|(_, s), _| *s != sec);
        self.fields.clear();
    }
    pub fn clear(&mut self) {
        self.sections.clear();
        self.fields.clear();
    }
    pub(crate) fn warm_section(&mut self, provenance: [u8; 32], w: &impl World, sec: BlockPos) -> bool {
        if self.sections.contains_key(&(provenance, sec)) {
            return false;
        }
        if let Some(s) = SectionSummary::build(w, sec) {
            self.sections.insert((provenance, sec), Arc::new(s));
            true
        } else {
            false
        }
    }
    pub(crate) fn cached_section_count(&self) -> usize {
        self.sections.len()
    }
}

pub(crate) struct FarField {
    nodes: Vec<FarNode>,
    cols: HashMap<[i32; 2], Vec<usize>>,
    dist: Vec<f32>,
    next: Vec<u32>,
}

#[inline]
fn axis_gap(lo1: i32, hi1: i32, lo2: i32, hi2: i32) -> (i32, i32, i32) {
    if hi1 < lo2 {
        (lo2 - hi1, hi1, lo2)
    } else if hi2 < lo1 {
        (lo1 - hi2, lo1, hi2)
    } else {
        (0, lo1.max(lo2), lo1.max(lo2))
    }
}

#[inline]
fn node_center(n: &FarNode) -> (f32, f32) {
    ((n.min[0] + n.max[0]) as f32 * 0.5, (n.min[2] + n.max[2]) as f32 * 0.5)
}

impl FarField {
    pub(crate) fn get_or_build<S: CorridorSource>(
        store: &GeneratorStore,
        source: &S,
        label: LabelId,
        from: &PlayerState,
        goal: Goal,
        kit: Option<&KitView>,
        kit_key: u64,
    ) -> Arc<Self> {
        let provenance = source.provenance();
        let key = FarKey {
            provenance,
            goal_cell: goal.cell,
            goal_face: goal.face,
            kit_key,
        };
        if let Some(existing) = store.far.borrow().fields.get(&key) {
            return existing.clone();
        }

        let fx = from.position.x.floor() as i32;
        let fy = from.position.y.floor() as i32;
        let fz = from.position.z.floor() as i32;
        let gx = goal.cell[0];
        let gy = goal.cell[1];
        let gz = goal.cell[2];

        let (mut lo, mut hi) = source.bounds_box().unwrap_or((
            [fx.min(gx) - 96, fy.min(gy) - 48, fz.min(gz) - 96],
            [fx.max(gx) + 96, fy.max(gy) + 32, fz.max(gz) + 96],
        ));
        // Keep search focused along the corridor ±96 blocks perpendicularly to avoid scanning unused million-block slabs.
        lo[0] = lo[0].max(fx.min(gx) - 96);
        lo[2] = lo[2].max(fz.min(gz) - 96);
        hi[0] = hi[0].min(fx.max(gx) + 96);
        hi[2] = hi[2].min(fz.max(gz) + 96);

        let smin = lo.map(|v| v.div_euclid(16));
        let smax = hi.map(|v| v.div_euclid(16));

        let mut nodes = Vec::new();
        let mut cols: HashMap<[i32; 2], Vec<usize>> = HashMap::new();
        for sz in smin[2]..=smax[2] {
            for sx in smin[0]..=smax[0] {
                let mut col_ids = Vec::new();
                for sy in smin[1]..=smax[1] {
                    let sec = [sx, sy, sz];
                    let cached = store.far.borrow().sections.get(&(provenance, sec)).cloned();
                    let summary = match cached {
                        Some(s) => s,
                        None => {
                            let Some(built) = SectionSummary::build(source, sec) else { continue };
                            let arc = Arc::new(built);
                            store.far.borrow_mut().sections.insert((provenance, sec), arc.clone());
                            arc
                        }
                    };
                    if summary.nodes.is_empty() {
                        continue;
                    }
                    let base = nodes.len();
                    nodes.extend(summary.nodes.iter().cloned());
                    col_ids.extend(base..nodes.len());
                }
                if !col_ids.is_empty() {
                    cols.insert([sx, sz], col_ids);
                }
            }
        }

        let legs = Legs::of(store, label, source, from).unwrap_or(Legs { h: 2.81, up: 6.19, down: 0.0, reach: 4 });
        let cushioned = kit.is_some_and(|kv| kv.makes(FieldEdit::Cushion, kv.spec.counts()));
        let depth = if cushioned { i32::MAX } else { (from.profile.safe_fall_distance + f64::from(from.health)).ceil() as i32 };

        let n = nodes.len();
        let mut dist = vec![f32::INFINITY; n];
        let mut next = vec![u32::MAX; n];
        let mut edits = vec![u16::MAX; n];
        let mut heap: BinaryHeap<Reverse<(u32, u16, usize)>> = BinaryHeap::new();

        // Seed nodes covering/adjacent to goal.cell.
        for sx in (gx.div_euclid(16) - 1)..=(gx.div_euclid(16) + 1) {
            for sz in (gz.div_euclid(16) - 1)..=(gz.div_euclid(16) + 1) {
                let Some(ids) = cols.get(&[sx, sz]) else { continue };
                for &idx in ids {
                    let node = &nodes[idx];
                    let (dx, _, _) = axis_gap(node.min[0], node.max[0], gx, gx);
                    let (dz, _, _) = axis_gap(node.min[2], node.max[2], gz, gz);
                    let dy = (node.rep[1] - gy).abs();
                    if dx <= 1 && dz <= 1 && dy <= 2 {
                        let d0 = (dx as f32).hypot(dz as f32) * legs.h;
                        if d0 < dist[idx] {
                            dist[idx] = d0;
                            edits[idx] = 0;
                            heap.push(Reverse((d0.to_bits(), 0, idx)));
                        }
                    }
                }
            }
        }

        let max_gap = f32::from(legs.reach) + 1.5;
        while let Some(Reverse((bits, ev, vi))) = heap.pop() {
            let dv = f32::from_bits(bits);
            if dv != dist[vi] || ev != edits[vi] {
                continue;
            }
            let vn = &nodes[vi];
            let (vcx, vcz) = node_center(vn);
            let vsx = vn.rep[0].div_euclid(16);
            let vsz = vn.rep[2].div_euclid(16);

            for dsz in -1..=1 {
                for dsx in -1..=1 {
                    let Some(cands) = cols.get(&[vsx + dsx, vsz + dsz]) else { continue };
                    for &ui in cands {
                        if ui == vi {
                            continue;
                        }
                        let un = &nodes[ui];
                        // Transition u -> v: climb at most 1 block, fall at most `depth` blocks.
                        let dy = vn.rep[1] - un.rep[1];
                        if dy > 1 || (depth != i32::MAX && -dy > depth) {
                            continue;
                        }
                        let (dx, ux, vx) = axis_gap(un.min[0], un.max[0], vn.min[0], vn.max[0]);
                        let (dz, uz, vz) = axis_gap(un.min[2], un.max[2], vn.min[2], vn.max[2]);
                        let h_gap = (dx as f32).hypot(dz as f32);
                        if h_gap > max_gap {
                            continue;
                        }
                        let (ucx, ucz) = node_center(un);
                        let trav = (vcx - ucx).hypot(vcz - ucz) * legs.h
                            + if dy > 0 { legs.up } else { legs.down * ((-dy).max(0) as f32) };
                        let lower = dv + trav;
                        if lower > dist[ui] {
                            continue;
                        }

                        let uc = [ux, un.rep[1], uz];
                        let vc = [vx, vn.rep[1], vz];
                        let (edge_toll, add_edit) = if dx <= 1 && dz <= 1 {
                            if way_clear(source, uc, vc) {
                                (0.0f32, 0u16)
                            } else if let Some(kv) = kit {
                                let Some((et, _)) = kv.edge(source, &blockers(source, uc, vc), FieldEdit::Clear) else {
                                    continue;
                                };
                                (et, 1u16)
                            } else {
                                continue;
                            }
                        } else {
                            // Parkour jump across air gap: verify apex corridor clearance.
                            let mid = [(uc[0] + vc[0]) / 2, uc[1].max(vc[1]) + 1, (uc[2] + vc[2]) / 2];
                            if !way_clear(source, [uc[0], mid[1], uc[2]], mid)
                                || !way_clear(source, mid, [vc[0], mid[1], vc[2]])
                            {
                                continue;
                            }
                            (1.5f32, 0u16)
                        };

                        let nd = lower + edge_toll;
                        let ne = ev.saturating_add(add_edit);
                        if nd < dist[ui] || (nd == dist[ui] && ne < edits[ui]) {
                            dist[ui] = nd;
                            edits[ui] = ne;
                            next[ui] = vi as u32;
                            heap.push(Reverse((nd.to_bits(), ne, ui)));
                        }
                    }
                }
            }
        }

        let built = Arc::new(Self { nodes, cols, dist, next });
        store.far.borrow_mut().fields.insert(key, built.clone());
        built
    }

    pub(crate) fn next_horizon(&self, from: &PlayerState, goal: Goal, window_blocks: f64) -> Option<(Goal, u64)> {
        let fx = from.position.x.floor() as i32;
        let fy = from.position.y.floor() as i32;
        let fz = from.position.z.floor() as i32;
        let fsx = fx.div_euclid(16);
        let fsz = fz.div_euclid(16);

        let mut start_idx: Option<usize> = None;
        let mut best_score = f32::INFINITY;
        for sz in (fsz - 1)..=(fsz + 1) {
            for sx in (fsx - 1)..=(fsx + 1) {
                let Some(cands) = self.cols.get(&[sx, sz]) else { continue };
                for &idx in cands {
                    if !self.dist[idx].is_finite() {
                        continue;
                    }
                    let n = &self.nodes[idx];
                    let (dx, _, _) = axis_gap(n.min[0], n.max[0], fx, fx);
                    let (dz, _, _) = axis_gap(n.min[2], n.max[2], fz, fz);
                    let dy = (n.rep[1] - fy).abs();
                    if dx <= 4 && dz <= 4 && dy <= 4 {
                        let s = self.dist[idx] + (dx as f32).hypot(dz as f32) * 3.0 + dy as f32 * 4.0;
                        if s < best_score {
                            best_score = s;
                            start_idx = Some(idx);
                        }
                    }
                }
            }
        }

        let mut cur = start_idx?;
        let goal_c = goal.center();
        let horiz_to_goal = (from.position.x - goal_c.x).hypot(from.position.z - goal_c.z);
        if horiz_to_goal <= window_blocks + 6.0 {
            return None;
        }

        let mut chosen: Option<usize> = None;
        let mut steps = 0usize;
        while steps < 256 {
            let n = &self.nodes[cur];
            let dx = f64::from(n.rep[0]) + 0.5 - from.position.x;
            let dz = f64::from(n.rep[2]) + 0.5 - from.position.z;
            let hd = dx.hypot(dz);
            let rem_goal = (f64::from(n.rep[0]) + 0.5 - goal_c.x).hypot(f64::from(n.rep[2]) + 0.5 - goal_c.z);
            if hd >= window_blocks - 5.0 && hd <= window_blocks + 6.0 {
                if n.solid_below || chosen.is_none() {
                    chosen = Some(cur);
                    if n.solid_below && hd >= window_blocks {
                        break;
                    }
                }
            }
            if hd > window_blocks + 6.0 {
                break;
            }
            if rem_goal <= 14.0 && chosen.is_none() {
                return None;
            }
            let nx = self.next[cur];
            if nx == u32::MAX {
                break;
            }
            cur = nx as usize;
            steps += 1;
        }

        let ci = chosen?;
        let cn = &self.nodes[ci];
        let rem_to_goal = (f64::from(cn.rep[0]) + 0.5 - goal_c.x).hypot(f64::from(cn.rep[2]) + 0.5 - goal_c.z);
        if rem_to_goal <= 14.0 {
            return None;
        }
        let target = if cn.solid_below {
            Goal { cell: below(cn.rep), face: Some(Face::Up), settle: false }
        } else {
            Goal::of(cn.rep)
        };
        let rem_ticks = self.dist[ci].ceil().max(0.0) as u64;
        Some((target, rem_ticks))
    }
}
