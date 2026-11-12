//! Job 1.6: what a certified replay costs, layer by layer (jobs/1.6-read-cost/prompt.md).
//! Counts are exact; times are p50 and p90 over interleaved samples, so two rows of one
//! table saw the same machine load. Printed, never gated: `jobs/1.6-read-cost/measure.ps1`.
use std::{alloc::{GlobalAlloc, Layout, System}, cell::{Cell, RefCell}, collections::{BTreeMap, HashMap, HashSet}, hint::black_box,
    path::PathBuf, sync::atomic::{AtomicI64, AtomicU64, Ordering::Relaxed}, time::Instant};
use law::{*, footprint::{self, Query, RecordingWorld}, shapes::ShapeView, state_reads::StateField};
use words::*;

struct Counting;
static ALLOCATIONS: AtomicU64 = AtomicU64::new(0);
static ALLOCATED: AtomicU64 = AtomicU64::new(0);
/// Bytes held now.
static LIVE: AtomicI64 = AtomicI64::new(0);
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Relaxed);
        ALLOCATED.fetch_add(layout.size() as u64, Relaxed);
        LIVE.fetch_add(layout.size() as i64, Relaxed);
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, p: *mut u8, layout: Layout) { LIVE.fetch_sub(layout.size() as i64, Relaxed); unsafe { System.dealloc(p, layout) } }
    unsafe fn realloc(&self, p: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Relaxed);
        ALLOCATED.fetch_add(size as u64, Relaxed);
        LIVE.fetch_add(size as i64 - layout.size() as i64, Relaxed);
        unsafe { System.realloc(p, layout, size) }
    }
}
#[global_allocator]
static GLOBAL: Counting = Counting;
/// Allocations (reallocations included) and bytes asked for while `work` ran.
fn allocations<T>(work: impl FnOnce() -> T) -> (u64, u64) {
    let before = (ALLOCATIONS.load(Relaxed), ALLOCATED.load(Relaxed));
    black_box(work());
    (ALLOCATIONS.load(Relaxed) - before.0, ALLOCATED.load(Relaxed) - before.1)
}

/// What the law asked its world, in order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Ask { Block, HookBlock, FluidBlock, RawFluidBlock, CollisionShape, Loaded, Region, Sweep, CellBounds, VisitCells,
    HasFluid, HasInsideEffects, MinY, FastLava, Difficulty, NaturalRegeneration, ExtraCollisions, ReadState, WriteState }
impl Ask {
    /// The asks that name one cell.
    fn cell(self) -> bool { matches!(self, Self::Block | Self::HookBlock | Self::FluidBlock | Self::RawFluidBlock | Self::CollisionShape | Self::Loaded) }
    /// The asks that hand the recorder a box of cells.
    fn region(self) -> bool { matches!(self, Self::Region | Self::CellBounds | Self::VisitCells) }
}
#[derive(Clone, Copy)]
struct Op { ask: Ask, query: Query, lo: BlockPos, hi: BlockPos, site: u32, tick: u32, boxes: u32 }

/// Logs every ask and forwards it, as the recorder does. `sites` names the law function
/// behind each ask from a backtrace: exact in a debug build, where nothing is inlined.
struct Probe<'a, W> { world: &'a W, ops: RefCell<Vec<Op>>, boxes: RefCell<Vec<[Aabb; 2]>>, sites: Option<RefCell<Sites>>, phase: Cell<&'static str>, tick: Cell<u32> }
#[derive(Default)]
struct Sites { names: Vec<String>, ids: HashMap<String, u32> }
impl<'a, W: World> Probe<'a, W> {
    fn new(world: &'a W, sites: bool) -> Self {
        Self { world, ops: RefCell::new(Vec::new()), boxes: RefCell::new(vec![[EMPTY; 2]]), sites: sites.then(RefCell::default), phase: Cell::new("segment"), tick: Cell::new(0) }
    }
    fn log(&self, ask: Ask, lo: BlockPos, hi: BlockPos, query: Query, boxes: [Aabb; 2]) {
        let site = self.sites.as_ref().map_or(0, |sites| {
            let trace = std::backtrace::Backtrace::force_capture().to_string();
            // Frames run from the innermost out; the law's own, outermost first, are the path.
            let mut path: Vec<&str> = trace.lines().filter_map(|line| {
                let name = line.trim_start().split_once(": ")?.1;
                let name = name.strip_prefix("law::movement::").or_else(|| name.strip_prefix("words::program::").map(|_| "program"))?;
                let name = name.split(['<', ':', '{']).next()?;
                (!matches!(name, "tick" | "tick_with_collision" | "tick_movement" | "bounding_box" | "collision_context")).then_some(name)
            }).collect();
            path.dedup();
            path.reverse();
            let name = if path.is_empty() { self.phase.get().to_owned() } else { path.join(">") };
            let mut sites = sites.borrow_mut();
            if let Some(&id) = sites.ids.get(&name) { return id; }
            let id = sites.names.len() as u32;
            sites.names.push(name.clone());
            sites.ids.insert(name, id);
            id
        });
        let boxes = if boxes[0].min == boxes[0].max { 0 } else { self.boxes.borrow_mut().push(boxes); self.boxes.borrow().len() as u32 - 1 };
        self.ops.borrow_mut().push(Op { ask, query, lo, hi, site, tick: self.tick.get(), boxes });
    }
    fn at(&self, ask: Ask, p: BlockPos, query: Query) { self.log(ask, p, p, query, [EMPTY; 2]); }
    fn flag(&self, ask: Ask) { self.log(ask, [0; 3], [0; 3], Query::Block, [EMPTY; 2]); }
}
const EMPTY: Aabb = Aabb { min: Vec3 { x: 0.0, y: 0.0, z: 0.0 }, max: Vec3 { x: 0.0, y: 0.0, z: 0.0 } };
impl<W: World> World for Probe<'_, W> {
    fn read_state(&self, _: StateField) { self.flag(Ask::ReadState); }
    fn write_state(&self, _: StateField) { self.flag(Ask::WriteState); }
    fn block(&self, p: BlockPos) -> &Block { self.at(Ask::Block, p, Query::Block); self.world.block(p) }
    fn hook_block(&self, p: BlockPos) -> &Block { self.at(Ask::HookBlock, p, Query::Hooks); self.world.hook_block(p) }
    fn fluid_block(&self, p: BlockPos) -> &Block { self.at(Ask::FluidBlock, p, Query::Fluid); self.world.fluid_block(p) }
    fn raw_fluid_block(&self, p: BlockPos) -> Option<&Block> { self.at(Ask::RawFluidBlock, p, Query::Fluid); self.world.raw_fluid_block(p) }
    fn collision_shape(&self, p: BlockPos, context: CollisionContext) -> &VoxelShape { self.at(Ask::CollisionShape, p, Query::Shape); self.world.collision_shape(p, context) }
    fn loaded(&self, p: BlockPos) -> bool { self.at(Ask::Loaded, p, Query::Loaded); self.world.loaded(p) }
    fn model_id(&self, p: BlockPos) -> Option<u16> { self.world.model_id(p) }
    fn observe_region(&self, query: Query, lo: BlockPos, hi: BlockPos) { self.log(Ask::Region, lo, hi, query, [EMPTY; 2]); }
    fn observe_sweep(&self, from: Aabb, to: Aabb) { self.log(Ask::Sweep, [0; 3], [0; 3], Query::Block, [from, to]); }
    fn collision_cell_bounds(&self, bounds: Aabb, lo: BlockPos, hi: BlockPos) -> Option<(BlockPos, BlockPos)> {
        self.log(Ask::CellBounds, lo, hi, Query::Shape, [bounds, EMPTY]);
        self.world.collision_cell_bounds(bounds, lo, hi)
    }
    fn visit_collision_cells(&self, bounds: Aabb, lo: BlockPos, hi: BlockPos, visit: impl FnMut(BlockPos) -> bool) -> bool {
        self.log(Ask::VisitCells, lo, hi, Query::Shape, [bounds, EMPTY]);
        self.world.visit_collision_cells(bounds, lo, hi, visit)
    }
    fn has_fluid(&self) -> bool { self.flag(Ask::HasFluid); self.world.has_fluid() }
    fn has_inside_effects(&self) -> bool { self.flag(Ask::HasInsideEffects); self.world.has_inside_effects() }
    fn min_y(&self) -> i32 { self.flag(Ask::MinY); self.world.min_y() }
    fn fast_lava(&self) -> bool { self.flag(Ask::FastLava); self.world.fast_lava() }
    fn difficulty(&self) -> u8 { self.flag(Ask::Difficulty); self.world.difficulty() }
    fn natural_regeneration(&self) -> bool { self.flag(Ask::NaturalRegeneration); self.world.natural_regeneration() }
    fn visit_extra_collisions(&self, bounds: Aabb, context: CollisionContext, visit: impl FnMut(ShapeView<'_>) -> bool) -> bool {
        self.log(Ask::ExtraCollisions, [0; 3], [0; 3], Query::Block, [bounds, EMPTY]);
        self.world.visit_extra_collisions(bounds, context, visit)
    }
}

/// Counts the cell fetches that reach the world itself, under every default method.
struct Fetches<'a, W> { world: &'a W, cells: RefCell<Vec<BlockPos>>, loaded: Cell<u64> }
impl<W: World> World for Fetches<'_, W> {
    fn block(&self, p: BlockPos) -> &Block { self.cells.borrow_mut().push(p); self.world.block(p) }
    fn loaded(&self, p: BlockPos) -> bool { self.loaded.set(self.loaded.get() + 1); self.world.loaded(p) }
    fn model_id(&self, p: BlockPos) -> Option<u16> { self.world.model_id(p) }
    fn collision_cell_bounds(&self, bounds: Aabb, lo: BlockPos, hi: BlockPos) -> Option<(BlockPos, BlockPos)> { self.world.collision_cell_bounds(bounds, lo, hi) }
    fn visit_collision_cells(&self, bounds: Aabb, lo: BlockPos, hi: BlockPos, visit: impl FnMut(BlockPos) -> bool) -> bool { self.world.visit_collision_cells(bounds, lo, hi, visit) }
    fn has_fluid(&self) -> bool { self.world.has_fluid() }
    fn has_inside_effects(&self) -> bool { self.world.has_inside_effects() }
    fn min_y(&self) -> i32 { self.world.min_y() }
    fn fast_lava(&self) -> bool { self.world.fast_lava() }
    fn difficulty(&self) -> u8 { self.world.difficulty() }
    fn natural_regeneration(&self) -> bool { self.world.natural_regeneration() }
    fn visit_extra_collisions(&self, bounds: Aabb, context: CollisionContext, visit: impl FnMut(ShapeView<'_>) -> bool) -> bool { self.world.visit_extra_collisions(bounds, context, visit) }
}

/// The same cells with both omission certificates forced false: what the fast paths are worth.
/// Bit-identical only where no fluid and no inside effect is in reach; the harness checks the bits.
struct Dry<'a, W>(&'a W);
impl<W: World> World for Dry<'_, W> {
    fn block(&self, p: BlockPos) -> &Block { self.0.block(p) }
    fn loaded(&self, p: BlockPos) -> bool { self.0.loaded(p) }
    fn model_id(&self, p: BlockPos) -> Option<u16> { self.0.model_id(p) }
    fn collision_cell_bounds(&self, bounds: Aabb, lo: BlockPos, hi: BlockPos) -> Option<(BlockPos, BlockPos)> { self.0.collision_cell_bounds(bounds, lo, hi) }
    fn visit_collision_cells(&self, bounds: Aabb, lo: BlockPos, hi: BlockPos, visit: impl FnMut(BlockPos) -> bool) -> bool { self.0.visit_collision_cells(bounds, lo, hi, visit) }
    fn has_fluid(&self) -> bool { false }
    fn has_inside_effects(&self) -> bool { false }
    fn min_y(&self) -> i32 { self.0.min_y() }
    fn difficulty(&self) -> u8 { self.0.difficulty() }
    fn natural_regeneration(&self) -> bool { self.0.natural_regeneration() }
}
/// Answers nothing: the recorder's own work, when the logged asks are played back onto it.
struct Null { air: Block }
impl World for Null {
    fn block(&self, _: BlockPos) -> &Block { &self.air }
    fn visit_collision_cells(&self, _: Aabb, _: BlockPos, _: BlockPos, _: impl FnMut(BlockPos) -> bool) -> bool { true }
}

struct Flat { ground: Block, air: Block }
impl World for Flat {
    fn block(&self, p: BlockPos) -> &Block { if p[1] == -1 { &self.ground } else { &self.air } }
    fn has_fluid(&self) -> bool { false }
    fn has_inside_effects(&self) -> bool { false }
}
fn flat_stone() -> Flat {
    let stone = world::BlockState { name: "minecraft:stone".into(), properties: Vec::new() };
    Flat { ground: world::model(&stone).unwrap().clone(), air: Block::default() }
}
fn point_of(json: &str, key: &str) -> [i32; 3] {
    let tail = &json[json.find(&format!("\"{key}\":[")).unwrap() + key.len() + 4..];
    let v: Vec<i32> = tail[..tail.find(']').unwrap()].split(',').map(|s| s.parse().unwrap()).collect();
    [v[0], v[1], v[2]]
}
/// One of Julien's tracks: its snapshot, and a start standing on A that faces B.
fn track(name: &str) -> Result<(world::WorldSnapshot, PlayerState, Arguments), world::LoadError> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let line = include_str!("../../world/tests/fixtures/tracks.txt").lines().find(|l| l.split_whitespace().next() == Some(name)).unwrap();
    let n: Vec<i32> = line.split_whitespace().skip(1).map(|v| v.parse().unwrap()).collect();
    let json = std::fs::read_to_string(root.join(format!("../../lab/tracks/{name}.json"))).unwrap();
    let (a, b) = (point_of(&json, "A"), point_of(&json, "B"));
    // The track's box, widened by the law's read halo: two cells to each side, eight above.
    let snapshot = world::WorldSnapshot::load(root.join("../world/tests/fixtures/world"), [n[0] - 2, n[1], n[2] - 2], [n[3] + 2, n[4] + 8, n[5] + 2])?;
    let yaw = ((a[0] - b[0]) as f32).atan2((b[2] - a[2]) as f32).to_degrees();
    let start = PlayerState { position: Vec3::new(a[0] as f64 + 0.5, (a[1] + 1) as f64, a[2] as f64 + 0.5), on_ground: true,
        velocity: Vec3::new(0.0, -0.0784000015258789, 0.0), ..PlayerState::default() };
    Ok((snapshot, start, Arguments { refined_yaw: Some(yaw), ..Arguments::default() }))
}
fn sprint() -> Input { Input { forward: true, sprint: true, ..Input::default() } }
fn jumping() -> Program {
    Program { rows: vec![Row { when: Predicate::OnGround, input: Input { jump: true, ..sprint() } }, Row { when: Predicate::Always, input: sprint() }] }
}
const TRACKS: [&str; 18] = ["stonesimple", "stonestaircase", "carpetwater", "fences", "amertystcoral", "ametwoodbox", "birchparkourmedium",
    "butoonpressureplates", "doorcorridor", "doorparkourblue", "fencemaze", "fencestraightbridge", "flowers", "grassjumpeasy",
    "mangrovejumpeasy", "mud", "redfakemaze", "trapdoorparkour"];
/// The tracks that are timed: flat stone beside water, stairs, water itself, fences.
const TIMED: [&str; 4] = ["stonesimple", "stonestaircase", "carpetwater", "fences"];

/// A chain of segments from `start`, each with its start state: stopped at `limit`, at a
/// refusal, or where the snapshot ends (a query outside it panics).
fn chain(world: &impl World, start: &PlayerState, program: &Program, args: Arguments, limit: usize) -> (Vec<(PlayerState, Segment)>, String) {
    let mut state = start.clone();
    let mut segments = Vec::new();
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let stop = loop {
        if segments.len() == limit { break "limit".to_owned(); }
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run_segment(world, &state, program, args))) {
            Ok(Ok(segment)) => { let next = segment.state.clone(); segments.push((state, segment)); state = next; }
            Ok(Err(error)) => break format!("{error:?}"),
            Err(_) => break "the snapshot ends".to_owned(),
        }
    };
    std::panic::set_hook(hook);
    (segments, stop)
}
/// The segment's loop again, on any world, so a probe sees every ask the recorder would.
fn rerun<W: World>(world: &W, start: &PlayerState, program: &Program, args: Arguments, ticks: u32, between: impl Fn(&'static str, u32)) -> PlayerState {
    let mut state = start.clone();
    between("select", 0);
    let mut input = program.select(world, &state, 0, args).unwrap().1;
    for elapsed in 1..=ticks {
        between("tick", elapsed - 1);
        let before = state.main_supporting_block;
        tick(&mut state, world, &input);
        if before != state.main_supporting_block {
            between("segment support", elapsed - 1);
            for p in [before, state.main_supporting_block].into_iter().flatten() { black_box((world.block(p), world.loaded(p))); }
        }
        between("select", elapsed - 1);
        input = program.select(world, &state, elapsed, args).unwrap().1;
    }
    state
}
fn volume(lo: BlockPos, hi: BlockPos) -> u64 { (0..3).map(|a| (hi[a] - lo[a] + 1).max(0) as u64).product() }
fn percentile(sorted: &[f64], p: f64) -> f64 { sorted[((sorted.len() - 1) as f64 * p).round() as usize] }
fn spread(mut values: Vec<f64>) -> String {
    values.sort_by(f64::total_cmp);
    format!("min={:.2} p50={:.2} p90={:.2} max={:.2}", values[0], percentile(&values, 0.5), percentile(&values, 0.9), values[values.len() - 1])
}

/// Every ask of a chain, with the footprints the real segments recorded.
struct Counts { ops: Vec<Op>, boxes: Vec<[Aabb; 2]>, sites: Vec<String>, fetches: Vec<BlockPos>, loaded: u64, segments: usize, ticks: u32, footprint_cells: Vec<usize>, swept_cells: Vec<usize> }
fn count(world: &impl World, segments: &[(PlayerState, Segment)], program: &Program, args: Arguments, sites: bool) -> Counts {
    let fetches = Fetches { world, cells: RefCell::default(), loaded: Cell::new(0) };
    let probe = Probe::new(&fetches, sites);
    let mut tick_base = 0;
    for (start, segment) in segments {
        let end = rerun(&probe, start, program, args, segment.ticks, |phase, tick| { probe.phase.set(phase); probe.tick.set(tick_base + tick); });
        assert_eq!(end.bits(), segment.state.bits(), "the probed run is the segment");
        tick_base += segment.ticks;
    }
    Counts { ops: probe.ops.take(), boxes: probe.boxes.take(), sites: probe.sites.map(|s| s.into_inner().names).unwrap_or_default(), fetches: fetches.cells.take(), loaded: fetches.loaded.get(),
        segments: segments.len(), ticks: tick_base, footprint_cells: segments.iter().map(|(_, s)| s.footprint.reads.len()).collect(),
        swept_cells: segments.iter().map(|(_, s)| s.footprint.swept.len()).collect() }
}
fn print_counts(name: &str, stop: &str, counts: &Counts) {
    let (segments, ticks) = (counts.segments as f64, counts.ticks as f64);
    println!("[{name}] {} segments, {} ticks, {:.2} ticks/segment, chain stopped at: {stop}", counts.segments, counts.ticks, ticks / segments);
    let mut by_ask = BTreeMap::<Ask, u64>::new();
    for op in &counts.ops { *by_ask.entry(op.ask).or_default() += 1; }
    print!("[{name}] asks per tick:");
    for (ask, n) in &by_ask { print!(" {ask:?}={:.1}", *n as f64 / ticks); }
    println!();
    // What the recorder pays: one map insert per cell ask (two for a raw fluid ask) and one per cell of each box.
    let cell_asks: u64 = counts.ops.iter().filter(|op| op.ask.cell()).count() as u64;
    let raw: u64 = by_ask.get(&Ask::RawFluidBlock).copied().unwrap_or(0);
    let box_cells: u64 = counts.ops.iter().filter(|op| op.ask.region()).map(|op| volume(op.lo, op.hi)).sum();
    let boxes = counts.ops.iter().filter(|op| op.ask.region()).count();
    // Per tick: the distinct cells asked about, and the boxes that repeat an earlier box of the same tick whole.
    let mut per_tick = BTreeMap::<u32, (HashSet<BlockPos>, HashSet<(BlockPos, BlockPos, u8)>, u64)>::new();
    for op in &counts.ops {
        let entry = per_tick.entry(op.tick).or_default();
        if op.ask.cell() { entry.0.insert(op.lo); }
        if op.ask.region() {
            if !entry.1.insert((op.lo, op.hi, op.query as u8)) { entry.2 += 1; }
            for z in op.lo[2]..=op.hi[2] { for y in op.lo[1]..=op.hi[1] { for x in op.lo[0]..=op.hi[0] { entry.0.insert([x, y, z]); } } }
        }
    }
    let distinct: usize = per_tick.values().map(|e| e.0.len()).sum();
    let repeated_boxes: u64 = per_tick.values().map(|e| e.2).sum();
    println!("[{name}] recorder map inserts per tick: {:.1} = {:.1} cell asks + {:.1} cells of {:.1} boxes ({:.1} boxes repeat one of the same tick); \
        distinct cells per tick {:.1}; so {:.1} inserts per distinct cell",
        (cell_asks + raw + box_cells) as f64 / ticks, (cell_asks + raw) as f64 / ticks, box_cells as f64 / ticks, boxes as f64 / ticks,
        repeated_boxes as f64 / ticks, distinct as f64 / ticks, (cell_asks + raw + box_cells) as f64 / distinct as f64);
    let mut cells = counts.footprint_cells.iter().map(|&c| c as f64).collect::<Vec<_>>();
    cells.sort_by(f64::total_cmp);
    let mut swept = counts.swept_cells.iter().map(|&c| c as f64).collect::<Vec<_>>();
    swept.sort_by(f64::total_cmp);
    println!("[{name}] per segment: footprint cells p50={} p90={} max={}; swept cells p50={} max={}; inserts {:.0}; cell fetches reaching the world {:.1}, loaded asks {:.1}",
        percentile(&cells, 0.5), percentile(&cells, 0.9), cells[cells.len() - 1], percentile(&swept, 0.5), swept[swept.len() - 1],
        (cell_asks + raw + box_cells) as f64 / segments, counts.fetches.len() as f64 / segments, counts.loaded as f64 / segments);
    println!("[{name}] cell fetches reaching the world per tick: {:.1}; state reports per tick: {:.1} reads, {:.1} writes",
        counts.fetches.len() as f64 / ticks, by_ask.get(&Ask::ReadState).copied().unwrap_or(0) as f64 / ticks, by_ask.get(&Ask::WriteState).copied().unwrap_or(0) as f64 / ticks);
    if !counts.sites.is_empty() {
        // Per law function: its asks, and the map inserts they cost the recorder.
        let mut by_site = BTreeMap::<u32, (u64, u64)>::new();
        for op in counts.ops.iter().filter(|op| op.ask.cell() || op.ask.region()) {
            let entry = by_site.entry(op.site).or_default();
            entry.0 += 1;
            entry.1 += if op.ask.region() { volume(op.lo, op.hi) } else if op.ask == Ask::RawFluidBlock { 2 } else { 1 };
        }
        let mut rows: Vec<_> = by_site.into_iter().collect();
        rows.sort_by_key(|&(_, (_, inserts))| std::cmp::Reverse(inserts));
        println!("[{name}] by law function (world asks per tick, recorder inserts per tick):");
        for (site, (asks, inserts)) in rows { println!("[{name}]   {:>7.1} {:>7.1}  {}", asks as f64 / ticks, inserts as f64 / ticks, counts.sites[site as usize]); }
    }
}

/// Counts on flat stone and on every track. In a debug build each ask also names its law function.
#[test]
fn what_the_law_asks_and_what_the_recorder_pays() {
    let sites = cfg!(debug_assertions);
    let program = jumping();
    let flat = flat_stone();
    let start = PlayerState { position: Vec3::new(8.5, 0.0, 8.5), on_ground: true, velocity: Vec3::new(0.0, -0.0784000015258789, 0.0), ..PlayerState::default() };
    let limit = if sites { 24 } else { 200 };
    let (segments, stop) = chain(&flat, &start, &program, Arguments::default(), limit);
    print_counts("flat stone", &stop, &count(&flat, &segments, &program, Arguments::default(), sites));
    for name in if sites { &TIMED[..] } else { &TRACKS[..] } {
        let (snapshot, start, args) = match track(name) { Ok(track) => track, Err(error) => { println!("[{name}] refused: {error:?}"); continue; } };
        let (segments, stop) = chain(&snapshot, &start, &program, args, limit);
        if segments.is_empty() { println!("[{name}] no segment: {stop}"); continue; }
        print_counts(name, &stop, &count(&snapshot, &segments, &program, args, sites));
    }
}

/// The time of each layer, interleaved so every layer of a table saw the same load.
#[test]
#[cfg_attr(debug_assertions, ignore = "a release measurement")]
fn where_the_time_of_a_certified_segment_goes() {
    let program = jumping();
    let flat = flat_stone();
    let start = PlayerState { position: Vec3::new(8.5, 0.0, 8.5), on_ground: true, velocity: Vec3::new(0.0, -0.0784000015258789, 0.0), ..PlayerState::default() };
    time_layers("flat stone", &flat, &start, &program, Arguments::default());
    for name in TIMED {
        let (snapshot, start, args) = match track(name) { Ok(track) => track, Err(error) => { println!("[{name}] refused: {error:?}"); continue; } };
        time_layers(name, &snapshot, &start, &program, args);
    }
}
fn time_layers(name: &str, world: &impl World, start: &PlayerState, program: &Program, args: Arguments) {
    let (segments, stop) = chain(world, start, program, args, 60);
    if segments.is_empty() { println!("[{name}] no segment: {stop}"); return; }
    let counts = count(world, &segments, program, args, false);
    let ticks: u32 = segments.iter().map(|(_, s)| s.ticks).sum();
    // The asks of each segment, to play back: onto the world (its answers alone) and onto a recorder (its work alone).
    let mut per_segment = Vec::new();
    let mut tick_base = 0;
    for (_, segment) in &segments {
        per_segment.push(counts.ops.iter().filter(|op| op.tick >= tick_base && op.tick < tick_base + segment.ticks).copied().collect::<Vec<_>>());
        tick_base += segment.ticks;
    }
    let mut fetch_ranges = Vec::new();
    {
        let fetches = Fetches { world, cells: RefCell::default(), loaded: Cell::new(0) };
        for (start, segment) in &segments {
            let from = fetches.cells.borrow().len();
            rerun(&fetches, start, program, args, segment.ticks, |_, _| {});
            fetch_ranges.push(from..fetches.cells.borrow().len());
        }
        assert_eq!(*fetches.cells.borrow(), counts.fetches);
    }
    let dry_same = segments.iter().all(|(start, segment)| rerun(&Dry(world), start, program, args, segment.ticks, |_, _| {}).bits() == segment.state.bits());
    let mut table = ProposalTable::default();
    for (start, segment) in &segments { table.learn(world, start, program, args, segment).unwrap(); }
    let proposals: Vec<Found> = segments.iter().map(|(start, _)| {
        *table.proposals(world, start).first().expect("a hit")
    }).collect();
    let table = RefCell::new(table);
    let certificates = RefCell::new(law::pattern::PatternTable::<()>::default());
    let null = Null { air: Block::default() };
    let context = CollisionContext::of(start);
    fn play<W: World>(recorder: &W, ops: &[Op], counts: &Counts, context: CollisionContext) {
        for op in ops {
            match op.ask {
                Ask::Block => { black_box(recorder.block(op.lo)); }
                Ask::HookBlock => { black_box(recorder.hook_block(op.lo)); }
                Ask::FluidBlock => { black_box(recorder.fluid_block(op.lo)); }
                Ask::RawFluidBlock => { black_box(recorder.raw_fluid_block(op.lo)); }
                Ask::CollisionShape => { black_box(recorder.collision_shape(op.lo, context)); }
                Ask::Loaded => { black_box(recorder.loaded(op.lo)); }
                Ask::Region => recorder.observe_region(op.query, op.lo, op.hi),
                Ask::Sweep => recorder.observe_sweep(counts.boxes[op.boxes as usize][0], counts.boxes[op.boxes as usize][1]),
                Ask::CellBounds => { black_box(recorder.collision_cell_bounds(counts.boxes[op.boxes as usize][0], op.lo, op.hi)); }
                Ask::VisitCells => { black_box(recorder.visit_collision_cells(counts.boxes[op.boxes as usize][0], op.lo, op.hi, |_| true)); }
                Ask::HasFluid => { black_box(recorder.has_fluid()); }
                Ask::HasInsideEffects => { black_box(recorder.has_inside_effects()); }
                Ask::MinY => { black_box(recorder.min_y()); }
                Ask::FastLava => { black_box(recorder.fast_lava()); }
                Ask::Difficulty => { black_box(recorder.difficulty()); }
                Ask::NaturalRegeneration => { black_box(recorder.natural_regeneration()); }
                Ask::ExtraCollisions => { black_box(recorder.visit_extra_collisions(counts.boxes[op.boxes as usize][0], context, |_| true)); }
                Ask::ReadState => recorder.read_state(StateField::position),
                Ask::WriteState => recorder.write_state(StateField::position),
            }
        }
    }
    type Layer<'a> = (&'static str, Box<dyn Fn(usize) + 'a>);
    let layers: Vec<Layer> = vec![
        ("lookup", Box::new(|i| { black_box(table.borrow_mut().proposals(world, &segments[i].0)); })),
        ("replay", Box::new(|i| { black_box(table.borrow_mut().replay(world, &segments[i].0, proposals[i]).unwrap()); })),
        ("recorded segment", Box::new(|i| { black_box(run_segment(world, &segments[i].0, program, args).unwrap()); })),
        ("law alone", Box::new(|i| { black_box(rerun(world, &segments[i].0, program, args, segments[i].1.ticks, |_, _| {})); })),
        ("law alone, certificates forced false", Box::new(|i| { black_box(rerun(&Dry(world), &segments[i].0, program, args, segments[i].1.ticks, |_, _| {})); })),
        ("the world's answers alone", Box::new(|i| { for &p in &counts.fetches[fetch_ranges[i].clone()] { black_box(world.block(black_box(p))); } })),
        ("the asks played back onto a recorder", Box::new(|i| { let recorder = RecordingWorld::new(&null); play(&recorder, &per_segment[i], &counts, context); black_box(recorder.take()); })),
        ("the asks played back onto nothing", Box::new(|i| play(black_box(&null), &per_segment[i], &counts, context))),
        ("the certificate alone", Box::new(|i| { black_box(certificates.borrow_mut().insert(world, &segments[i].1.footprint, &segments[i].0, ()).unwrap()); })),
        ("one state clone", Box::new(|i| { black_box(segments[i].0.clone()); })),
    ];
    let rounds = (2400 / segments.len()).max(40);
    let mut costs = vec![Vec::new(); layers.len()];
    for round in 0..rounds {
        for i in 0..segments.len() {
            // Each layer follows every other in turn, so none always runs on the caches another warmed.
            for layer in (0..layers.len()).map(|k| (k + round * 3 + i) % layers.len()) {
                let run = &layers[layer].1;
                let clock = Instant::now();
                run(i);
                costs[layer].push(clock.elapsed().as_secs_f64() * 1e6);
            }
        }
    }
    println!("[{name}] {} segments, {ticks} ticks, {:.2} ticks/segment, chain stopped at: {stop}; n={} per layer; forcing the certificates false keeps the bits: {dry_same}",
        segments.len(), ticks as f64 / segments.len() as f64, costs[0].len());
    for (layer, (label, run)) in layers.iter().enumerate() {
        let (count, bytes) = (0..segments.len()).map(|i| allocations(|| run(i))).fold((0, 0), |a, b| (a.0 + b.0, a.1 + b.1));
        println!("[{name}] us/segment {label}: {}; allocations {:.1}, {:.0} bytes", spread(costs[layer].clone()),
            count as f64 / segments.len() as f64, bytes as f64 / segments.len() as f64);
    }
    let p50 = |layer: usize| { let mut c = costs[layer].clone(); c.sort_by(f64::total_cmp); percentile(&c, 0.5) };
    println!("[{name}] p50 split: lookup {:.2} + certificate {:.2} + recorder's work {:.2} + law arithmetic {:.2} + world's answers {:.2} + rest of the segment (clones, events, support digests, take) {:.2} = {:.2}",
        p50(0), p50(1) - p50(2), p50(6) - p50(7), p50(3) - p50(5), p50(5), p50(2) - p50(3) - (p50(6) - p50(7)), p50(0) + p50(1));
    let table = table.borrow();
    println!("[{name}] has_fluid={} has_inside_effects={}; the table after the run: classes={} confirmed={} surprised={} refusals={}",
        world.has_fluid(), world.has_inside_effects(), table.classes().len(), table.counters.replayed - table.counters.surprises, table.counters.surprises, table.counters.refusals);
}

/// The same cells with the trait's own collision scan: `main`'s snapshot, which asks
/// `loaded` and `collision_shape` of every cell of every padded box.
struct Scan<'a, W>(&'a W);
impl<W: World> World for Scan<'_, W> {
    fn block(&self, p: BlockPos) -> &Block { self.0.block(p) }
    fn loaded(&self, p: BlockPos) -> bool { self.0.loaded(p) }
    fn model_id(&self, p: BlockPos) -> Option<u16> { self.0.model_id(p) }
    fn has_fluid(&self) -> bool { self.0.has_fluid() }
    fn has_inside_effects(&self) -> bool { self.0.has_inside_effects() }
    fn min_y(&self) -> i32 { self.0.min_y() }
    fn fast_lava(&self) -> bool { self.0.fast_lava() }
    fn difficulty(&self) -> u8 { self.0.difficulty() }
    fn natural_regeneration(&self) -> bool { self.0.natural_regeneration() }
}

/// A recorder's cell set: the one part the candidates differ in.
trait Store: Default {
    fn mark(&mut self, p: BlockPos, bits: u8);
    fn region(&mut self, lo: BlockPos, hi: BlockPos, bits: u8) {
        for z in lo[2]..=hi[2] { for y in lo[1]..=hi[1] { for x in lo[0]..=hi[0] { self.mark([x, y, z], bits); } } }
    }
    fn sorted(&mut self) -> BTreeMap<BlockPos, u8>;
    /// Whether the recorder keeps the environment reads.
    const ENVIRONMENT: bool = true;
}
/// No cells: what the rest of a recorder costs, with or without its environment list.
#[derive(Default)]
struct Nothing<const ENVIRONMENT: bool>;
impl<const E: bool> Store for Nothing<E> {
    fn mark(&mut self, _: BlockPos, _: u8) {}
    fn region(&mut self, _: BlockPos, _: BlockPos, _: u8) {}
    fn sorted(&mut self) -> BTreeMap<BlockPos, u8> { BTreeMap::new() }
    const ENVIRONMENT: bool = E;
}
/// `main`'s recorder: one map entry per ask.
#[derive(Default)]
struct Tree(BTreeMap<BlockPos, u8>);
impl Store for Tree {
    fn mark(&mut self, p: BlockPos, bits: u8) { *self.0.entry(p).or_default() |= bits; }
    fn sorted(&mut self) -> BTreeMap<BlockPos, u8> { std::mem::take(&mut self.0) }
}
/// GLM lane `recorder-cost`'s `ReadTable` (its candidate.patch): open addressing on the cell, sorted at the end.
struct Table { slots: Vec<(BlockPos, u8)>, len: usize }
impl Default for Table { fn default() -> Self { Self { slots: vec![([0; 3], 0); 256], len: 0 } } }
impl Table {
    fn hash(p: BlockPos) -> usize {
        let folded = (p[0] as u32 as u64) ^ ((p[1] as u32 as u64) << 21) ^ ((p[2] as u32 as u64) << 42);
        folded.wrapping_mul(0x9e3779b97f4a7c15).rotate_left(26) as usize
    }
    fn grow(&mut self) {
        let double = self.slots.len() * 2;
        let old = std::mem::replace(&mut self.slots, vec![([0; 3], 0); double]);
        self.len = 0;
        for (p, bits) in old { if bits != 0 { self.mark(p, bits); } }
    }
}
impl Store for Table {
    fn mark(&mut self, p: BlockPos, bits: u8) {
        let cap = self.slots.len();
        let mut index = Self::hash(p) & (cap - 1);
        loop {
            let slot = &mut self.slots[index];
            if slot.1 == 0 { *slot = (p, bits); self.len += 1; break; }
            if slot.0 == p { slot.1 |= bits; return; }
            index = (index + 1) & (cap - 1);
        }
        if self.len * 4 > cap * 3 { self.grow(); }
    }
    fn sorted(&mut self) -> BTreeMap<BlockPos, u8> {
        let mut cells: Vec<_> = self.slots.iter().filter(|s| s.1 != 0).copied().collect();
        cells.sort_unstable();
        self.slots.iter_mut().for_each(|s| *s = ([0; 3], 0));
        self.len = 0;
        cells.into_iter().collect()
    }
}
/// `RecordingWorld` with its cell set swapped: every other line as in `law::footprint`.
struct Rec<'a, W, S> { world: &'a W, store: RefCell<S>, swept: RefCell<std::collections::BTreeSet<BlockPos>>,
    environment: RefCell<Vec<footprint::EnvironmentRead>>, state_reads: Cell<u128>, written: Cell<u128> }
impl<'a, W: World, S: Store> Rec<'a, W, S> {
    fn new(world: &'a W) -> Self { Self { world, store: RefCell::default(), swept: RefCell::default(), environment: RefCell::default(), state_reads: Cell::new(0), written: Cell::new(0) } }
    fn take(&self) -> footprint::Footprint {
        self.written.set(0);
        footprint::Footprint { reads: self.store.borrow_mut().sorted(), swept: self.swept.take(), environment: self.environment.take(),
            state_reads: law::state_reads::StateReads(self.state_reads.replace(0)) }
    }
    fn read(&self, p: BlockPos, query: Query) { self.store.borrow_mut().mark(p, query as u8); }
    fn environment(&self, read: footprint::EnvironmentRead) { if S::ENVIRONMENT { self.environment.borrow_mut().push(read); } }
}
impl<W: World, S: Store> World for Rec<'_, W, S> {
    fn read_state(&self, field: StateField) { self.state_reads.set(self.state_reads.get() | (1 << field as u8) & !self.written.get()); }
    fn write_state(&self, field: StateField) { self.written.set(self.written.get() | 1 << field as u8); }
    fn block(&self, p: BlockPos) -> &Block { self.read(p, Query::Block); self.world.block(p) }
    fn hook_block(&self, p: BlockPos) -> &Block { self.read(p, Query::Hooks); self.world.hook_block(p) }
    fn fluid_block(&self, p: BlockPos) -> &Block { self.read(p, Query::Fluid); self.world.fluid_block(p) }
    fn raw_fluid_block(&self, p: BlockPos) -> Option<&Block> { self.read(p, Query::Fluid); self.read(p, Query::Loaded); self.world.raw_fluid_block(p) }
    fn collision_shape(&self, p: BlockPos, context: CollisionContext) -> &VoxelShape { self.read(p, Query::Shape); self.world.collision_shape(p, context) }
    fn loaded(&self, p: BlockPos) -> bool { self.read(p, Query::Loaded); self.world.loaded(p) }
    fn observe_region(&self, query: Query, lo: BlockPos, hi: BlockPos) { if (0..3).all(|a| lo[a] <= hi[a]) { self.store.borrow_mut().region(lo, hi, query as u8); } }
    fn observe_sweep(&self, from: Aabb, to: Aabb) {
        let lo = [from.min.x.min(to.min.x), from.min.y.min(to.min.y), from.min.z.min(to.min.z)].map(law::math::floor);
        let hi = [from.max.x.max(to.max.x), from.max.y.max(to.max.y), from.max.z.max(to.max.z)].map(law::math::floor);
        let mut swept = self.swept.borrow_mut();
        for z in lo[2]..=hi[2] { for y in lo[1]..=hi[1] { for x in lo[0]..=hi[0] { swept.insert([x, y, z]); } } }
    }
    fn collision_cell_bounds(&self, bounds: Aabb, lo: BlockPos, hi: BlockPos) -> Option<(BlockPos, BlockPos)> {
        self.observe_region(Query::Shape, lo, hi); self.world.collision_cell_bounds(bounds, lo, hi)
    }
    fn visit_collision_cells(&self, bounds: Aabb, lo: BlockPos, hi: BlockPos, visit: impl FnMut(BlockPos) -> bool) -> bool {
        self.observe_region(Query::Shape, lo, hi); self.world.visit_collision_cells(bounds, lo, hi, visit)
    }
    fn has_fluid(&self) -> bool { let v = self.world.has_fluid(); self.environment(footprint::EnvironmentRead::HasFluid(v)); v }
    fn has_inside_effects(&self) -> bool { let v = self.world.has_inside_effects(); self.environment(footprint::EnvironmentRead::HasInsideEffects(v)); v }
    fn min_y(&self) -> i32 { let v = self.world.min_y(); self.environment(footprint::EnvironmentRead::MinY(v)); v }
    fn fast_lava(&self) -> bool { let v = self.world.fast_lava(); self.environment(footprint::EnvironmentRead::FastLava(v)); v }
    fn difficulty(&self) -> u8 { let v = self.world.difficulty(); self.environment(footprint::EnvironmentRead::Difficulty(v)); v }
    fn natural_regeneration(&self) -> bool { let v = self.world.natural_regeneration(); self.environment(footprint::EnvironmentRead::NaturalRegeneration(v)); v }
    // The trait's own visit lends these: the same record as `RecordingWorld`'s.
    fn extra_collisions(&self, bounds: Aabb, context: CollisionContext) -> Vec<VoxelShape> {
        let shapes = self.world.extra_collisions(bounds, context);
        self.environment(footprint::EnvironmentRead::ExtraCollisions { bounds, context, shapes: shapes.clone() });
        shapes
    }
}
fn recorded<W: World, S: Store>(world: &W, start: &PlayerState, program: &Program, args: Arguments, ticks: u32) -> footprint::Footprint {
    let recorder = Rec::<W, S>::new(world);
    black_box(rerun(&recorder, start, program, args, ticks, |_, _| {}));
    recorder.take()
}
fn taped<W: World>(world: &W, start: &PlayerState, program: &Program, args: Arguments, ticks: u32) -> footprint::Footprint {
    let recorder = RecordingWorld::new(world);
    black_box(rerun(&recorder, start, program, args, ticks, |_, _| {}));
    recorder.take()
}
fn same_cells(a: &footprint::Footprint, b: &footprint::Footprint) -> bool { a.reads == b.reads && a.swept == b.swept && a.state_reads.0 == b.state_reads.0 }

/// The candidates side by side on one load: three cell sets (the map per ask, the GLM
/// table, the tape) on two worlds (`main`'s full scan, the snapshot that answers from its
/// models). Each candidate's footprint is checked against the others before it is timed.
#[test]
#[cfg_attr(debug_assertions, ignore = "a release measurement")]
fn the_candidates_side_by_side() {
    let program = jumping();
    let flat = flat_stone();
    let start = PlayerState { position: Vec3::new(8.5, 0.0, 8.5), on_ground: true, velocity: Vec3::new(0.0, -0.0784000015258789, 0.0), ..PlayerState::default() };
    side_by_side::<_, Flat>("flat stone", &flat, None, &start, &program, Arguments::default());
    for name in TIMED {
        let (snapshot, start, args) = match track(name) { Ok(track) => track, Err(error) => { println!("[{name}] refused: {error:?}"); continue; } };
        side_by_side(name, &Scan(&snapshot), Some(&snapshot), &start, &program, args);
    }
}
fn side_by_side<A: World, B: World>(name: &str, scan: &A, models: Option<&B>, start: &PlayerState, program: &Program, args: Arguments) {
    let (segments, stop) = chain(scan, start, program, args, 60);
    if segments.is_empty() { println!("[{name}] no segment: {stop}"); return; }
    let ticks: u32 = segments.iter().map(|(_, s)| s.ticks).sum();
    // Identity first: every cell set gives `main`'s footprint on `main`'s world; the models'
    // world gives the same bits, the same cells and the same Shape bits, and asks no more.
    let (mut narrowed, mut cells, mut kept) = (0, 0, 0);
    for (start, segment) in &segments {
        let reference = recorded::<_, Tree>(scan, start, program, args, segment.ticks);
        assert!(same_cells(&reference, &segment.footprint), "[{name}] the harness recorder is main's");
        assert!(same_cells(&recorded::<_, Table>(scan, start, program, args, segment.ticks), &reference), "[{name}] table");
        assert!(same_cells(&taped(scan, start, program, args, segment.ticks), &reference), "[{name}] tape");
        if let Some(models) = models {
            assert_eq!(rerun(models, start, program, args, segment.ticks, |_, _| {}).bits(), segment.state.bits(), "[{name}] the models' world keeps the bits");
            let fast = taped(models, start, program, args, segment.ticks);
            assert!(same_cells(&recorded::<_, Tree>(models, start, program, args, segment.ticks), &fast));
            assert_eq!(fast.reads.keys().collect::<Vec<_>>(), reference.reads.keys().collect::<Vec<_>>(), "[{name}] the same cells");
            assert_eq!(fast.swept, reference.swept);
            for (p, &bits) in &reference.reads {
                let now = fast.reads[p];
                assert!(now & !bits == 0 && now & Query::Shape as u8 == bits & Query::Shape as u8, "[{name}] {p:?}: {bits} -> {now}");
                narrowed += (now != bits) as u32;
                kept += (now == Query::Shape as u8) as u32;
            }
            cells += reference.reads.len();
        }
    }
    if models.is_some() {
        println!("[{name}] identity: three cell sets equal on both worlds; the models' world keeps every bit and cell, and narrows {narrowed} of {cells} cell masks \
            ({:.1}%) to fewer queries (Shape only now: {kept})", 100.0 * narrowed as f64 / cells as f64);
    } else { println!("[{name}] identity: three cell sets equal"); }
    type Layer<'a> = (&'static str, Box<dyn Fn(usize) + 'a>);
    let s = &segments;
    // The branch's whole replay, layer by layer, on the models' world.
    let filled: Vec<_> = models.iter().flat_map(|&models| s.iter().map(move |(start, segment)| {
        let recorder = RecordingWorld::new(models);
        rerun(&recorder, start, program, args, segment.ticks, |_, _| {});
        recorder
    })).collect();
    let mut table = ProposalTable::default();
    let mut proposals = Vec::new();
    if let Some(models) = models {
        for (start, segment) in s { table.learn(models, start, program, args, segment).unwrap(); }
        proposals = s.iter().map(|(start, _)| *table.proposals(models, start).first().expect("a hit")).collect();
    }
    let (table, certificates) = (RefCell::new(table), RefCell::new(law::pattern::PatternTable::<()>::default()));
    let (proposals, filled, footprints) = (&proposals, &filled, models.map(|m| s.iter().map(|(start, seg)| taped(m, start, program, args, seg.ticks)).collect::<Vec<_>>()));
    let (table, certificates, footprints) = (&table, &certificates, &footprints);
    let mut layers: Vec<Layer> = vec![
        ("main's world: law alone", Box::new(|i| { black_box(rerun(scan, &s[i].0, program, args, s[i].1.ticks, |_, _| {})); })),
        ("main's world: law + map per ask (main)", Box::new(|i| { black_box(recorded::<_, Tree>(scan, &s[i].0, program, args, s[i].1.ticks)); })),
        ("main's world: law + GLM table", Box::new(|i| { black_box(recorded::<_, Table>(scan, &s[i].0, program, args, s[i].1.ticks)); })),
        ("main's world: law + tape", Box::new(|i| { black_box(taped(scan, &s[i].0, program, args, s[i].1.ticks)); })),
    ];
    if let Some(models) = models {
        layers.extend::<Vec<Layer>>(vec![
            ("models' world: law alone", Box::new(move |i| { black_box(rerun(models, &s[i].0, program, args, s[i].1.ticks, |_, _| {})); })),
            ("models' world: law + map per ask", Box::new(move |i| { black_box(recorded::<_, Tree>(models, &s[i].0, program, args, s[i].1.ticks)); })),
            ("models' world: law + GLM table", Box::new(move |i| { black_box(recorded::<_, Table>(models, &s[i].0, program, args, s[i].1.ticks)); })),
            ("models' world: law + tape", Box::new(move |i| { black_box(taped(models, &s[i].0, program, args, s[i].1.ticks)); })),
            ("models' world: law + state hook, swept, environment (no cells)", Box::new(move |i| { black_box(recorded::<_, Nothing<true>>(models, &s[i].0, program, args, s[i].1.ticks)); })),
            ("models' world: law + state hook, swept (no cells, no environment)", Box::new(move |i| { black_box(recorded::<_, Nothing<false>>(models, &s[i].0, program, args, s[i].1.ticks)); })),
            ("tape: the sorted form of a filled tape", Box::new(move |i| { black_box(filled[i].footprint()); })),
            ("tape: made and taken, nothing read", Box::new(move |_| { black_box(RecordingWorld::new(models).take()); })),
            ("models' world: run_segment (the branch)", Box::new(move |i| { black_box(run_segment(models, &s[i].0, program, args).unwrap()); })),
            // The event rule's support comparison: one whole-model digest per side that has a supporting block.
            ("models' world: the support digests of run_segment's last tick", Box::new(move |i| {
                let (a, b) = (s[i].0.main_supporting_block, s[i].1.state.main_supporting_block);
                if a != b { for p in [a, b].into_iter().flatten() { black_box(law::pattern::cell_digest(models.block(p), models.loaded(p), 1 | 16).unwrap()); } }
            })),
            ("models' world: the certificate alone", Box::new(move |i| {
                black_box(certificates.borrow_mut().insert(models, &footprints.as_ref().unwrap()[i], &s[i].0, ()).unwrap()); })),
            ("models' world: lookup", Box::new(move |i| { black_box(table.borrow_mut().proposals(models, &s[i].0)); })),
            ("models' world: replay (the branch)", Box::new(move |i| { black_box(table.borrow_mut().replay(models, &s[i].0, proposals[i]).unwrap()); })),
        ]);
    }
    let rounds = (2400 / segments.len()).max(40);
    let mut costs = vec![Vec::new(); layers.len()];
    for round in 0..rounds {
        for i in 0..segments.len() {
            for layer in (0..layers.len()).map(|k| (k + round * 3 + i) % layers.len()) {
                let clock = Instant::now();
                (layers[layer].1)(i);
                costs[layer].push(clock.elapsed().as_secs_f64() * 1e6);
            }
        }
    }
    println!("[{name}] {} segments, {ticks} ticks, {:.2} ticks/segment, stopped at: {stop}; n={} per layer", segments.len(), ticks as f64 / segments.len() as f64, costs[0].len());
    let p50 = |layer: usize| { let mut c = costs[layer].clone(); c.sort_by(f64::total_cmp); percentile(&c, 0.5) };
    for (layer, (label, run)) in layers.iter().enumerate() {
        let (count, bytes) = (0..segments.len()).map(|i| allocations(|| run(i))).fold((0, 0), |a, b| (a.0 + b.0, a.1 + b.1));
        println!("[{name}] us/segment {label}: {}; p50 over main's recorded {:.2}; allocations {:.1}, {:.0} bytes", spread(costs[layer].clone()),
            p50(layer) / p50(1), count as f64 / segments.len() as f64, bytes as f64 / segments.len() as f64);
    }
}

/// What the certificate store keeps per replay: replays from the chain's own starts, which
/// never repeat bit for bit (food, exhaustion and position move every tick).
#[test]
fn what_a_replay_leaves_behind() {
    let program = jumping();
    for name in TIMED {
        let (snapshot, start, args) = match track(name) { Ok(track) => track, Err(error) => { println!("[{name}] refused: {error:?}"); continue; } };
        let (segments, _) = chain(&snapshot, &start, &program, args, 200);
        let mut certificates = law::pattern::PatternTable::<()>::default();
        // The first half warms the digests and the model slots; the second half is the marginal cost.
        let half = segments.len() / 2;
        let mut passes = Vec::new();
        for part in [&segments[..half], &segments[half..], &segments[..]] {
            let before = LIVE.load(Relaxed);
            for (start, segment) in part { certificates.insert(&snapshot, &segment.footprint, start, ()).unwrap(); }
            passes.push((LIVE.load(Relaxed) - before) as f64 / part.len() as f64);
        }
        let entries = certificates.entries().count();
        println!("[{name}] {} replays from distinct starts: {} start states, {entries} entries; bytes kept per replay: {:.0} in the first half (warming), \
            {:.0} in the second, {:.0} when the same starts come again", segments.len(), certificates.starts().len(), passes[0], passes[1], passes[2]);
    }
}

/// Question 3: a footprint bounded by the body's swept cells instead of recorded. Does the
/// bound hold, how much larger is it? (The reuse a key on it keeps was measured under version 1's class, job 1.6.)
#[test]
fn what_a_bound_footprint_would_cost() {
    let program = jumping();
    for name in TIMED {
        let (snapshot, start, args) = match track(name) { Ok(track) => track, Err(error) => { println!("[{name}] refused: {error:?}"); continue; } };
        let (segments, _) = chain(&snapshot, &start, &program, args, 200);
        let mut held = [0; 2];
        let (mut sizes, mut extra, mut shape_only) = (Vec::new(), Vec::new(), 0usize);
        let mut cells = 0usize;
        for (_, segment) in &segments {
            let f = &segment.footprint;
            // The swept cells grown by `r` in every axis.
            let grown = |r: i32| {
                let mut b = std::collections::BTreeSet::new();
                for p in &f.swept { for dz in -r..=r { for dy in -r..=r { for dx in -r..=r { b.insert([p[0] + dx, p[1] + dy, p[2] + dz]); } } } }
                b
            };
            let (one, two) = (grown(1), grown(2));
            held[0] += f.reads.keys().all(|p| one.contains(p)) as u32;
            held[1] += f.reads.keys().all(|p| two.contains(p)) as u32;
            sizes.push(two.len() as f64 / f.reads.len() as f64);
            extra.push(one.iter().filter(|p| !f.reads.contains_key(*p)).count() as f64);
            shape_only += f.reads.values().filter(|&&m| m == Query::Shape as u8).count();
            cells += f.reads.len();
        }
        println!("[{name}] {} segments: the recorded cells lie within the swept cells grown by 1 in {} segments, by 2 in {}; the bound that holds (grown by 2) is {} times the recorded set; \
            cells grown by 1 that were not read: {}; {:.0}% of recorded cells are read for their shape alone", segments.len(), held[0], held[1], spread(sizes), spread(extra), 100.0 * shape_only as f64 / cells as f64);
    }
}
