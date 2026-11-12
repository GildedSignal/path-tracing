//! Sets backward (job 3.2b step 5; learning.md, "Look backward without inverting").
//!
//! A tool is a program at an aim. Its reach table is its own trajectory, run once by the law:
//! on a floor from rest for the run-up, and in free space from each contact for the arc. A
//! free arc is the same wherever it starts, so the cells from which a tool lands on a cell
//! are its reach table mirrored about that cell, and the contacts before the takeoff say how
//! far behind to look. Carried back from the goal, this gives every standable cell a set
//! (the tools that land on the next set) and a time T to the goal.
//!
//! This layer is lossy: cells, not states; takeoffs from cell centres; turned tables. It
//! orders the forward pass and decides nothing.
use std::collections::{BinaryHeap, HashMap};
use std::cmp::Reverse;
use law::{Aabb, Block, BlockPos, BoxWorld, CollisionContext, PlayerState, Vec3, World};
use words::{Arguments, Program, body_block};
use crate::{Goal, Still, advance, settle};

/// One trajectory of a tool from a contact: positions relative to the contact, per tick.
struct Arc {
    path: Vec<[f64; 3]>,
    /// The contacts of the run-up before it, relative to the contact, nearest first.
    runup: Vec<[f64; 2]>,
    /// Walked on the floor (no arc): the body is carried by the ground in between.
    grounded: bool,
    tool: u16,
}
/// One landing of an arc turned to an aim: `ticks` after the contact, on a surface whose
/// height above the takeoff's lies in `lo..=hi`.
#[derive(Clone, Copy)]
struct Hop { ticks: u32, lo: f64, hi: f64, apex: f64, runup: u32, arc: u32, aim: u8 }

pub struct Reach {
    arcs: Vec<Arc>,
    /// Landing cell minus takeoff cell -> the hops that make it, fewest ticks first.
    hops: HashMap<(i32, i32), Vec<Hop>>,
    pub tools: usize,
    pub contacts: usize,
    pub landings: usize,
    pub kept: usize,
    pub compile_us: f64,
}
fn turn(aim: u8, x: f64, z: f64) -> (f64, f64) {
    let yaw = (Arguments { yaw_index: aim, ..Arguments::default() }.yaw() as f64).to_radians();
    (x * yaw.cos() - z * yaw.sin(), z * yaw.cos() + x * yaw.sin())
}
impl Reach {
    /// Runs every distinct tool on a floor of `floor` over `extent` blocks, and from each
    /// contact in free space down to `depth` blocks. `extent` and `depth` are the world's.
    pub fn compile(floor: &Block, programs: &[Program], extent: f64, depth: f64) -> Result<Reach, String> {
        let clock = std::time::Instant::now();
        let mut ground = BoxWorld::default();
        let wide = extent.ceil() as i32 + 8;
        ground.fill([-wide, -1, -wide], [wide, -1, wide], floor.clone());
        let void = BoxWorld::default();
        let rest = settle(&Still(&ground), [0.5, 0.0, 0.5], 0)?;
        let args = Arguments::default();
        let mut reach = Reach { arcs: Vec::new(), hops: HashMap::new(), tools: 0, contacts: 0, landings: 0, kept: 0, compile_us: 0.0 };
        let mut seen = std::collections::HashSet::new();
        for program in programs.iter().filter(|program| !program.timed()) {
            // The floor run: the tool from rest until it has crossed the world's extent.
            let mut states = vec![rest.clone()];
            let mut state = rest.clone();
            let run = advance(&Still(&ground), &mut state, program, args, 4096, &mut |_, after| {
                states.push(after.clone());
                after.position.minus(rest.position).horizontal_sqr() > extent * extent
            });
            if run.is_err() { continue; }
            if !seen.insert(states.iter().map(|s| [s.position.x.to_bits(), s.position.y.to_bits(), s.position.z.to_bits()]).collect::<Vec<_>>()) { continue; }
            let tool = reach.tools as u16;
            reach.tools += 1;
            // A contact: on the ground, at rest, just landed, or just entered a block.
            let contacts: Vec<usize> = (0..states.len()).filter(|&i| states[i].on_ground
                && (i == 0 || !states[i - 1].on_ground || body_block(&states[i]) != body_block(&states[i - 1]))).collect();
            for (rank, &at) in contacts.iter().enumerate() {
                let origin = states[at].position;
                let relative = |p: Vec3| [p.x - origin.x, p.y - origin.y, p.z - origin.z];
                let runup: Vec<[f64; 2]> = contacts[..rank].iter().rev().map(|&k| { let r = relative(states[k].position); [r[0], r[2]] }).collect();
                reach.contacts += 1;
                // The arc: what the tool does from here if the floor ends here.
                let mut path = Vec::new();
                let mut state = states[at].clone();
                let fell = advance(&Still(&void), &mut state, program, args, 4096, &mut |_, after| { path.push(relative(after.position)); after.position.y - origin.y < -depth });
                if fell.is_ok() {
                    let arc = reach.arcs.len() as u32;
                    let mut apex = 0f64;
                    for tick in 0..path.len() {
                        let (above, below) = (if tick == 0 { 0.0 } else { path[tick - 1][1] }, path[tick][1]);
                        apex = apex.max(below);
                        // Descending through every height in (below, above]: it lands there.
                        if below < above { reach.land(arc, tick as u32 + 1, f64::from_bits(if below < 0.0 { below.to_bits() - 1 } else { below.to_bits() + 1 }), above, apex, runup.len() as u32, path[tick]); }
                    }
                    reach.arcs.push(Arc { path, runup: runup.clone(), grounded: false, tool });
                }
                // The walk: carried by the floor into the next block, a step up included.
                let mut path = Vec::new();
                for next in at + 1..states.len() {
                    let (a, b) = (body_block(&states[at]), body_block(&states[next]));
                    if !states[next].on_ground || (a[0] - b[0]).abs() > 1 || (a[2] - b[2]).abs() > 1 { break; }
                    path.push(relative(states[next].position));
                }
                let arc = reach.arcs.len() as u32;
                for tick in 0..path.len() { reach.land(arc, tick as u32 + 1, 0.0, states[at].step_height as f64, 0.0, runup.len() as u32, path[tick]); }
                if !path.is_empty() { reach.arcs.push(Arc { path, runup, grounded: true, tool }); }
            }
        }
        // Keep the Pareto front per displacement: fewer ticks, a wider band, a shorter run-up,
        // a lower apex. A dominated hop is dropped by that proof, inside this lossy layer.
        for hops in reach.hops.values_mut() {
            hops.sort_by(|a, b| (a.ticks, a.runup).cmp(&(b.ticks, b.runup)));
            let mut front: Vec<Hop> = Vec::new();
            for hop in hops.iter() {
                if !front.iter().any(|kept| kept.lo <= hop.lo && kept.hi >= hop.hi && kept.runup <= hop.runup && kept.apex <= hop.apex) { front.push(*hop); }
            }
            reach.kept += front.len();
            *hops = front;
        }
        reach.compile_us = clock.elapsed().as_secs_f64() * 1e6;
        Ok(reach)
    }
    fn land(&mut self, arc: u32, ticks: u32, lo: f64, hi: f64, apex: f64, runup: u32, at: [f64; 3]) {
        for aim in 0..64 {
            let (x, z) = turn(aim, at[0], at[2]);
            self.landings += 1;
            self.hops.entry(((0.5 + x).floor() as i32, (0.5 + z).floor() as i32)).or_default().push(Hop { ticks, lo, hi, apex, runup, arc, aim });
        }
    }
}

#[derive(Clone, Debug)]
pub struct Cell {
    /// The cell stood on, and the height stood at.
    pub support: BlockPos,
    pub y: f64,
    /// Ticks to the goal by the backward pass; `u32::MAX`: no set reaches it.
    pub t: u32,
    /// The next set: the cell its best tool lands on, that hop's ticks, run-up and kind.
    pub via: Option<BlockPos>,
    pub hop: (u32, u32, bool, u16),
    /// The forward pass: expansions anchored here, and arrivals on a cell nearer the goal.
    pub tries: u32,
    pub arrivals: u32,
}
/// T over the standable cells, from the goal backward.
pub struct Field {
    pub cells: HashMap<BlockPos, Cell>,
    pub standable: usize,
    pub reached: usize,
    pub goals: usize,
    pub checks: u64,
    pub build_us: f64,
}
impl Field {
    /// `body`: the box the body's block may be in. `start`: the cell the start stands on;
    /// the pass stops once every cell at least as near the goal is settled.
    pub fn build<W: World>(world: &W, reach: &Reach, body: (BlockPos, BlockPos), goal: &Goal, start: Option<BlockPos>) -> Field {
        let clock = std::time::Instant::now();
        let context = |feet: f64| CollisionContext { feet_y: feet, descending: false, fall_distance: 0.0, leather_boots: false };
        let inside = |p: Vec3| (0..3).all(|a| { let v = law::math::floor([p.x, p.y, p.z][a]); v >= body.0[a] && v <= body.1[a] });
        let free = |p: Vec3| inside(p) && world.no_collision(Aabb::player(p, PlayerState::default().pose.height()), context(p.y));
        let mut cells: Vec<Cell> = Vec::new();
        let mut columns: HashMap<(i32, i32), Vec<u32>> = HashMap::new();
        for x in body.0[0]..=body.1[0] { for z in body.0[2]..=body.1[2] { for y in body.0[1] - 1..=body.1[1] {
            let shape = world.collision_shape([x, y, z], context(y as f64 + 2.0));
            let Some(&top) = shape.coords[1].last().filter(|_| !shape.is_empty()) else { continue };
            let stand = Vec3::new(x as f64 + 0.5, y as f64 + top, z as f64 + 0.5);
            if !free(stand) { continue; }
            columns.entry((x, z)).or_default().push(cells.len() as u32);
            cells.push(Cell { support: [x, y, z], y: stand.y, t: u32::MAX, via: None, hop: (0, 0, false, 0), tries: 0, arrivals: 0 });
        } } }
        let mut field = Field { cells: HashMap::new(), standable: cells.len(), reached: 0, goals: 0, checks: 0, build_us: 0.0 };
        // The goal's own set: the cells where a standing body's box meets the region.
        let zone = goal.zone();
        let mut open = BinaryHeap::new();
        for (id, cell) in cells.iter_mut().enumerate() {
            let [x, _, z] = cell.support;
            if (x - goal.cell[0]).abs() <= 1 && (z - goal.cell[2]).abs() <= 1 && cell.y < zone.max.y && cell.y + PlayerState::default().pose.height() as f64 > zone.min.y {
                cell.t = 0;
                field.goals += 1;
                open.push(Reverse((0u32, id as u32)));
            }
        }
        let start = start.and_then(|support| cells.iter().position(|cell| cell.support == support));
        let mut settled = vec![false; cells.len()];
        while let Some(Reverse((t, id))) = open.pop() {
            if settled[id as usize] || t > cells[id as usize].t { continue; }
            if start.is_some_and(|start| t > cells[start].t) { break; }
            settled[id as usize] = true;
            field.reached += 1;
            let (to, height) = (cells[id as usize].support, cells[id as usize].y);
            for (&(dx, dz), hops) in &reach.hops {
                let Some(column) = columns.get(&(to[0] - dx, to[2] - dz)) else { continue };
                for &from in column {
                    if settled[from as usize] { continue; }
                    let (support, y) = (cells[from as usize].support, cells[from as usize].y);
                    let rise = height - y;
                    let centre = (support[0] as f64 + 0.5, support[2] as f64 + 0.5);
                    for hop in hops {
                        if t + hop.ticks >= cells[from as usize].t { break; }
                        if !(hop.lo <= rise && rise <= hop.hi) { continue; }
                        field.checks += 1;
                        let arc = &reach.arcs[hop.arc as usize];
                        // The run-up: its contacts stand at the takeoff's height.
                        let level = arc.runup.iter().all(|r| {
                            let (rx, rz) = turn(hop.aim, r[0], r[1]);
                            let at = ((centre.0 + rx).floor() as i32, (centre.1 + rz).floor() as i32);
                            columns.get(&at).is_some_and(|column| column.iter().any(|&c| cells[c as usize].y == y))
                        });
                        if !level { continue; }
                        // The tube: the body's box at every tick meets nothing; it lands on the cell.
                        let clear = (0..hop.ticks as usize).all(|tick| {
                            let (px, pz) = turn(hop.aim, arc.path[tick][0], arc.path[tick][2]);
                            let landing = tick + 1 == hop.ticks as usize;
                            if arc.grounded && !landing { return true; }
                            free(Vec3::new(centre.0 + px, if landing { height } else { y + arc.path[tick][1] }, centre.1 + pz))
                        });
                        if !clear { continue; }
                        cells[from as usize].t = t + hop.ticks;
                        cells[from as usize].via = Some(to);
                        cells[from as usize].hop = (hop.ticks, hop.runup, arc.grounded, arc.tool);
                        open.push(Reverse((t + hop.ticks, from)));
                        break;
                    }
                }
            }
        }
        field.cells = cells.into_iter().filter(|cell| cell.t != u32::MAX).map(|cell| (cell.support, cell)).collect();
        field.build_us = clock.elapsed().as_secs_f64() * 1e6;
        field
    }
    /// Takes each cell's nearer time from `other`: a scheduled world's field is the best of
    /// its frames', which only orders.
    pub fn merge(&mut self, other: Field) {
        for (support, cell) in other.cells {
            if self.cells.get(&support).is_none_or(|known| cell.t < known.t) { self.cells.insert(support, cell); }
        }
        self.build_us += other.build_us;
        self.checks += other.checks;
    }
    /// The estimate of ticks to go from `position`, for a body whose last set was `anchor`,
    /// and the point it is sent toward. Along the chain of sets from the anchor to the goal,
    /// each set offers its own time plus the way to it at its hop's rate; the least offer is
    /// the estimate, so it falls continuously as the body moves down the chain.
    pub fn estimate(&self, anchor: BlockPos, position: Vec3, goal: &Goal) -> (f64, (f64, f64)) {
        let centre = |p: BlockPos| (p[0] as f64 + 0.5, p[2] as f64 + 0.5);
        let far = |a: (f64, f64), b: (f64, f64)| ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt();
        let here = (position.x, position.z);
        let Some(mut cell) = self.cells.get(&anchor) else { return (f64::INFINITY, centre(goal.cell)) };
        // The anchor's own offer is at its outgoing hop's rate.
        let rate = |from: &Cell| from.via.map(|next| (from.hop.0 as f64, far(centre(from.support), centre(next)))).filter(|hop| hop.1 > 0.0).map(|hop| hop.0 / hop.1);
        let mut pace = rate(cell).unwrap_or(1.0 / crate::goal::C);
        let mut best = (cell.t as f64 + pace * far(here, centre(cell.support)), centre(cell.via.unwrap_or(goal.cell)));
        while let Some(next) = cell.via {
            pace = rate(cell).unwrap_or(pace);
            cell = &self.cells[&next];
            let offer = cell.t as f64 + pace * far(here, centre(next));
            if offer < best.0 { best = (offer, centre(cell.via.unwrap_or(goal.cell))); }
        }
        best
    }
}
