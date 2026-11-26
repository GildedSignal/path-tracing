//! The law supplies one word's yaw-free forward input, drag and vertical phases.
use std::{cell::{Cell, OnceCell, RefCell}, sync::Arc};
use law::{Block, BlockPos, Input, PlayerState, Vec3, World, codec::{self, LabelId}};
use words::program::{tiny_program_id, HeldPredicate::{Always, Never}};
use crate::source::CorridorSource;

pub const SPRINT_JUMP: u16 = tiny_program_id(0, true, Always, Never);
pub const SPRINT: u16 = tiny_program_id(0, true, Never, Never);
pub const WALK: u16 = tiny_program_id(0, false, Never, Never);
pub fn sprint_allowed(s: &PlayerState) -> bool { codec::resource_guards(s.food_level, s.health)[3] != 0 }

struct Sample<'a> {
    floor: &'a Block, air: Block, y: i32, terms: Cell<(f64, f32)>, factor: Cell<f64>, ticks: Cell<u64>,
}
impl World for Sample<'_> {
    fn block(&self, p: BlockPos) -> &Block { if p[1] == self.y { self.floor } else { &self.air } }
    fn difficulty(&self) -> u8 { 0 }
    fn observe_tick(&self) { self.ticks.set(self.ticks.get() + 1); }
    fn observe_air_travel(&self, forward: f64, drag: f32) { self.terms.set((forward, drag)); }
    fn observe_speed_factor(&self, factor: f64) { self.factor.set(factor); }
}
#[derive(Debug)]
pub(crate) struct Phase {
    pub after: Arc<PlayerState>, pub forward: f64, pub impulse: f64, pub factor: f64, pub drag: f64,
    pub contact: bool, pub dy: f64, pub wanted_y: f64,
}
impl Phase {
    fn sample(w: &Sample<'_>, s: &mut PlayerState, input: Input) -> Result<Self, String> {
        let jump = input.jump && s.on_ground && s.no_jump_delay <= 1;
        let (wanted, actual) = law::try_tick_with_collision(s, w, &input).map_err(|e| e.0)?.ok_or("word has no collision readout")?;
        let (forward, drag) = w.terms.get();
        Ok(Self {
            after: Arc::new(s.clone()),
            forward,
            impulse: if jump && s.sprinting && wanted.y > 0.0 { 0.2 } else { 0.0 },
            factor: w.factor.get(),
            drag: f64::from(drag),
            contact: wanted.y != actual.y,
            dy: actual.y,
            wanted_y: wanted.y,
        })
    }
}
#[derive(Debug)]
pub(crate) struct Gait { pub phases: Vec<Phase>, pub grounded: Phase, pub top: f64, pub program: u16 }
impl Gait {
    fn learn(source: &impl CorridorSource, from: &PlayerState, program: u16, ticks: &Cell<u64>) -> Result<Self, String> {
        let p = source.support_cell(from);
        let floor = source.block(p);
        let w = Sample { floor, air: Block::default(), y: p[1], terms: Cell::default(), factor: Cell::new(1.0), ticks: Cell::default() };
        let height = floor.shape.coords[1].last().copied().ok_or("no supporting surface")?;
        let top = f64::from(p[1]) + height;
        let mut s = from.clone();
        s.position = Vec3::new(0.5, top, 0.5);
        s.velocity = Vec3::default(); s.main_supporting_block = Some([0, p[1], 0]); s.on_ground = true;
        s.no_jump_delay = 0;
        s.in_water = false; s.in_lava = false; s.water_height = 0.0; s.lava_height = 0.0;
        s.eye_in_water = false; s.was_eye_in_water = false; s.underwater = false; s.swimming = false;
        s.crouching = false; s.shift_key_down = false; s.pose = law::Pose::Standing;
        let keys = Input { forward: true, sprint: program != WALK, ..Default::default() };
        law::try_tick(&mut s, &w, &Input::default()).map_err(|e| e.0)?;
        law::try_tick(&mut s, &w, &Input::default()).map_err(|e| e.0)?;
        let grounded = Phase::sample(&w, &mut s, keys)?;
        let mut phases = Vec::new();
        let mut seen = std::collections::HashSet::new();
        loop {
            let phase = Phase::sample(&w, &mut s, Input { jump: program == SPRINT_JUMP, ..keys })?;
            phases.push(phase);
            if s.on_ground && s.no_jump_delay <= 1 { break; }
            if !seen.insert((s.position.y.to_bits(), s.velocity.y.to_bits(), s.no_jump_delay, s.on_ground)) {
                return Err("word does not return to its support".into());
            }
        }
        ticks.set(ticks.get() + w.ticks.get());
        Ok(Self { phases, grounded, top, program })
    }
    pub fn entry(&self, s: &PlayerState, top: f64) -> Option<usize> {
        if s.on_ground && s.no_jump_delay <= 1 { return Some(0); }
        self.phases.iter().position(|p| p.after.position.y - self.top == s.position.y - top &&
            p.after.velocity.y == s.velocity.y && p.after.on_ground == s.on_ground && p.after.no_jump_delay == s.no_jump_delay)
            .map(|i| (i + 1) % self.phases.len())
    }
}

#[derive(Clone, Default)]
pub struct SharedFieldSnapshot {
    sections: Vec<(([u8; 32], BlockPos), Arc<crate::far_field::SectionSummary>)>,
    far_fields: Vec<(crate::far_field::FarKey, Arc<crate::far_field::FarField>)>,
    topo_entries: Vec<crate::defect::TopoCacheEntry>,
}

impl SharedFieldSnapshot {
    pub fn is_empty(&self) -> bool {
        self.sections.is_empty() && self.far_fields.is_empty() && self.topo_entries.is_empty()
    }
}

#[derive(Default)]
pub struct GeneratorStore {
    pub words: RefCell<crate::word_store::WordStore>,
    pub(crate) far: RefCell<crate::far_field::FarCache>,
    pub(crate) topo: RefCell<crate::defect::TopoCache>,
    gaits: RefCell<law::solve_meter::HashMap<(LabelId, u16), Arc<Gait>>>,
    ticks: Cell<u64>, codec_binding: OnceCell<Arc<()>>,
}
impl GeneratorStore {
    pub fn preparation_ticks(&self) -> u64 { self.ticks.get() }
    pub fn invalidate_far_section(&self, sec: BlockPos) {
        self.far.borrow_mut().invalidate_section(sec);
        self.topo.borrow_mut().invalidate_section(sec);
    }
    pub fn clear_far(&self) {
        self.far.borrow_mut().clear();
        self.topo.borrow_mut().clear();
    }
    pub fn consolidate_idle(&self) -> usize {
        self.words.borrow_mut().consolidate()
    }
    pub fn warm_far_section(&self, provenance: [u8; 32], w: &impl World, sec: BlockPos) -> bool {
        self.far.borrow_mut().warm_section(provenance, w, sec)
    }
    pub fn warm_far_sections_in(
        &self,
        provenance: [u8; 32],
        w: &impl World,
        min: BlockPos,
        max: BlockPos,
        max_sections: usize,
    ) -> usize {
        let smin = min.map(|v| v.div_euclid(16));
        let smax = max.map(|v| v.div_euclid(16));
        let mut warmed = 0usize;
        let mut checked = 0usize;
        let mut cache = self.far.borrow_mut();
        for sz in smin[2]..=smax[2] {
            for sx in smin[0]..=smax[0] {
                for sy in smin[1]..=smax[1] {
                    if checked >= max_sections {
                        return warmed;
                    }
                    checked += 1;
                    if cache.warm_section(provenance, w, [sx, sy, sz]) {
                        warmed += 1;
                    }
                }
            }
        }
        warmed
    }
    pub fn prewarm_batch_fields<S: CorridorSource>(
        &self,
        source: &S,
        from: &PlayerState,
        goal: crate::Goal,
        swarm_min: BlockPos,
        swarm_max: BlockPos,
        codec: &mut codec::Interner,
    ) -> bool {
        let Ok(label) = source.entry_label(codec, from) else {
            return false;
        };
        let _ = crate::far_field::FarField::get_or_build_box(
            self,
            source,
            label,
            from,
            goal,
            None,
            0,
            swarm_min,
            swarm_max,
        );
        crate::defect::prewarm_batch_topo(
            self,
            source,
            label,
            from,
            goal,
            swarm_min,
            swarm_max,
        );
        true
    }
    pub fn export_shared_fields(&self) -> SharedFieldSnapshot {
        let far = self.far.borrow();
        let topo = self.topo.borrow();
        SharedFieldSnapshot {
            sections: far.export_sections(),
            far_fields: far.export_fields(),
            topo_entries: topo.export_entries(),
        }
    }
    pub fn import_shared_fields(&self, snap: &SharedFieldSnapshot) {
        {
            let mut far = self.far.borrow_mut();
            far.import_sections(&snap.sections);
            far.import_fields(&snap.far_fields);
        }
        self.topo.borrow_mut().import_entries(&snap.topo_entries);
    }
    pub fn topo_cache_builds(&self) -> u64 {
        self.topo.borrow().builds()
    }
    pub fn far_field_builds(&self) -> u64 {
        self.far.borrow().far_field_builds()
    }
    pub fn cached_far_sections(&self) -> usize {
        self.far.borrow().cached_section_count()
    }
    pub fn bind_codec(&self, codec: &mut codec::Interner) -> bool {
        Arc::ptr_eq(self.codec_binding.get_or_init(|| codec.identity().clone()), codec.identity())
    }
    pub(crate) fn gait(&self, label: LabelId, source: &impl CorridorSource, from: &PlayerState, program: u16) -> Result<Arc<Gait>, String> {
        if let Some(g) = self.gaits.borrow().get(&(label, program)) { return Ok(g.clone()); }
        let g = Arc::new(Gait::learn(source, from, program, &self.ticks)?);
        self.gaits.borrow_mut().insert((label, program), g.clone());
        Ok(g)
    }
}
