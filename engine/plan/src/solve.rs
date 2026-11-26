//! One command extends a word, then certifies its swept corridor.
use std::sync::Arc;
use law::{Aabb, BlockPos, PlayerState, Vec3, codec::{Interner, LabelId, proof::Refusal}};
use crate::{
    Goal, certify, cut, defect,
    events::{self, Event, EventKind},
    inline::List,
    lattice,
    source::{CorridorProof, CorridorResult, CorridorSource},
    state_view::StateAt,
    store::{Gait, GeneratorStore, Phase, SPRINT, SPRINT_JUMP, WALK, sprint_allowed},
};
use words::Arguments;

pub use crate::kit::{Kit, KitCone, KitCounts, KitOp, KitSpec, KitView, Obligation, ObligationLedger};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bracket { pub lb: u64, pub ub: Option<u64> }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Stage {
    Bracket,
    ClosedCheck,
    Extend,
    Cut,
    Defect { proposals: u32, learn_ticks: u64, certify_ticks: u64, nanos: u128 },
    Compose,
    Allocate,
    Learn { program: u16, ticks: u64, nanos: u128 },
}
#[derive(Clone, Debug)]
pub struct WordStep {
    pub program: u16,
    pub args: Arguments,
    pub ticks: u32,
    pub expected_end_t: u64,
    pub events: WordEvents,
    pub certificate: CorridorProof,
    pub end: StateAt,
    pub edit: Option<KitOp>,
}
#[derive(Clone, Debug, Default)]
pub struct WordEvents {
    pub(crate) marks: List<Event, 3>,
    pub(crate) span: Option<Span>,
}
impl WordEvents {
    pub(crate) fn push_mark_unique(&mut self, ev: Event) {
        if !self.marks.iter().any(|e| e == &ev) {
            self.marks.push(ev);
        }
    }
}
#[derive(Clone, Debug)]
pub(crate) struct Span {
    pub(crate) gait: Arc<Gait>,
    pub(crate) start: Motion,
    pub(crate) top: f64,
    pub(crate) held: bool,
    pub(crate) yaw: u16,
    pub(crate) ticks: u32,
    pub(crate) count: u32,
}
#[derive(Clone, Debug)]
pub struct Plan {
    pub ticks: u64,
    pub words: List<WordStep, 12>,
    pub end: StateAt,
    pub bracket: Bracket,
    pub label: LabelId,
    pub deadline: u64,
    pub lattice_cost: lattice::LatticeCost,
    pub thinking: ThinkingCosts,
}
#[derive(Clone, Copy, Default, Debug)]
pub struct ThinkingCosts { pub entry_ns: u128, pub bearing_ns: u128, pub inverse_ns: u128, pub output_ns: u128 }
#[derive(Clone, Debug)]
pub struct PrefixPlan {
    pub plan: Plan,
    pub target: Goal,
    pub final_goal: Goal,
    pub remaining_est_ticks: u64,
}
#[derive(Clone, Debug)]
pub enum PlanOutcome {
    Ready(Plan),
    Prefix(Box<PrefixPlan>),
    NeedsJoints { exception: BlockPos, exceptions: usize, bracket: Bracket },
    Unknown(Refusal),
}
#[derive(Clone, Debug)]
pub struct SolveResult { pub outcome: PlanOutcome, pub stages: List<Stage, 16>, pub explanation: Option<String> }

#[derive(Clone, Copy, Debug)]
pub(crate) struct Motion { pub(crate) p: Vec3, pub(crate) v: Vec3, pub(crate) phase: usize, pub(crate) t: u64, pub(crate) wanted: Vec3 }
impl Motion {
    pub(crate) fn tick(&mut self, row: &Phase, dir: [f64; 2], next: usize, top: f64) -> [f64; 2] {
        #[cfg(feature = "solve-counters")] crate::solve_meter::phase(row);
        if self.v.horizontal_sqr() < 9.0e-6 { self.v.x = 0.0; self.v.z = 0.0; }
        let axis = |v: f64, d: f64| (v + d * row.impulse) + d * row.forward;
        let x = axis(self.v.x, dir[0]);
        let z = axis(self.v.z, dir[1]);
        self.wanted = Vec3::new(x, row.wanted_y, z);
        let moved = |d: f64| if row.contact && d.abs() < 1.0e-7 { 0.0 } else { d };
        let ds = [moved(x), moved(z)];
        self.p.x += ds[0];
        self.p.z += ds[1];
        self.p.y = if row.after.on_ground { top } else { self.p.y + row.dy };
        self.v = Vec3::new((x * row.factor) * row.drag, row.after.velocity.y, (z * row.factor) * row.drag);
        self.t += 1;
        self.phase = next;
        ds
    }
}
pub(crate) fn direction(yaw: u16) -> [f64; 2] { let (x, z) = lattice::direction(yaw); [x, z] }
fn dot(a: Vec3, b: [f64; 2]) -> f64 { a.x * b[0] + a.z * b[1] }
fn cross(a: Vec3, b: [f64; 2]) -> f64 { a.x * b[1] - a.z * b[0] }

pub(crate) struct Crossing<'a> {
    pub(crate) gait: &'a Arc<Gait>,
    pub(crate) from: &'a PlayerState,
    pub(crate) goal: Goal,
    pub(crate) top: f64,
    pub(crate) radius: f64,
    pub(crate) min: Vec3,
    pub(crate) max: Vec3,
    pub(crate) words: List<WordStep, 12>,
    pub(crate) proof: CorridorProof,
    pub(crate) bearing_ns: u128,
}
impl Crossing<'_> {
    fn include(&mut self, p: Vec3) {
        self.min = Vec3::new(self.min.x.min(p.x), self.min.y.min(p.y), self.min.z.min(p.z));
        self.max = Vec3::new(self.max.x.max(p.x), self.max.y.max(p.y), self.max.z.max(p.z));
    }
    fn emit(&mut self, start: Motion, m: Motion, row: &Phase, program: u16, yaw: u16, count: u64) -> Result<(), String> {
        if start.t == m.t { return Ok(()); }
        let mut end = StateAt::shared(row.after.clone());
        end.position = m.p;
        end.velocity = m.v;
        end.movement = Some(m.wanted);
        end.t = m.t;
        end.yaw = lattice::yaw(yaw);
        end.main_supporting_block = row.after.main_supporting_block.map(|_| [m.p.x.floor() as i32, self.proof.min[1], m.p.z.floor() as i32]);
        end.health = self.from.health;
        end.food_level = self.from.food_level;
        end.food_tick_timer = self.from.food_tick_timer;
        end.saturation = self.from.saturation;
        end.exhaustion = self.from.exhaustion;
        let args = Arguments { count: count.try_into().map_err(|_| "count width")?, refined_yaw: Some(end.yaw), ..Default::default() };
        let ticks = (m.t - start.t).try_into().map_err(|_| "tick width")?;
        let mut events = WordEvents {
            span: Some(Span { gait: self.gait.clone(), start, top: self.top, held: program != SPRINT_JUMP, yaw, ticks, count: args.count }),
            ..Default::default()
        };
        if self.words.last().is_some_and(|w| w.args.refined_yaw != args.refined_yaw) {
            events.marks.push(Event { tick: start.t, kind: EventKind::YawSwitch });
        }
        self.words.push(WordStep { program, args, ticks, expected_end_t: m.t, events, certificate: self.proof.clone(), end, edit: None });
        Ok(())
    }
    fn bearing(&self, m: Motion, rows: &[Phase]) -> [u16; 2] {
        let mut drag = 1.0;
        let mut carry = 0.0;
        for i in 0..rows.len() {
            let row = &rows[(m.phase + i) % rows.len()];
            carry += drag;
            drag *= row.factor * row.drag;
        }
        let d = self.goal.center().minus(m.p);
        let memory = carry / (1.0 - drag);
        lattice::bracket(d.x - memory * m.v.x, d.z - memory * m.v.z)
    }
    fn run(&mut self, mut m: Motion) -> Result<(), String> {
        let gait = self.gait;
        let mut held = gait.program == WALK;
        loop {
            let rows = if held { std::slice::from_ref(&gait.grounded) } else { &gait.phases };
            let program = if held && gait.program == SPRINT_JUMP { SPRINT } else { gait.program };
            let clock = std::time::Instant::now();
            let pair = self.bearing(m, rows);
            let yaw = pair[0];
            let dir = direction(yaw);
            self.bearing_ns += clock.elapsed().as_nanos();
            let mut start = m;
            loop {
                let before = m;
                let mut lo = before.p;
                let mut hi = before.p;
                for i in m.phase..rows.len() {
                    let next = if i + 1 == rows.len() { 0 } else { i + 1 };
                    m.tick(&rows[i], dir, next, self.top);
                    lo = Vec3::new(lo.x.min(m.p.x), lo.y.min(m.p.y), lo.z.min(m.p.z));
                    hi = Vec3::new(hi.x.max(m.p.x), hi.y.max(m.p.y), hi.z.max(m.p.z));
                }
                self.include(lo);
                self.include(hi);
                let height = self.from.pose.height();
                let body = |p| Aabb::player(p, height);
                if self.goal.touched_boxes(body(lo), body(hi)) {
                    let mut probe = before;
                    for i in before.phase..rows.len() {
                        let prior = probe.p;
                        probe.tick(&rows[i], dir, if i + 1 == rows.len() { 0 } else { i + 1 }, self.top);
                        if self.goal.touched_boxes(body(prior), body(probe.p)) {
                            self.emit(start, probe, &rows[i], program, yaw, 1)?;
                            return Ok(());
                        }
                    }
                }
                let remaining = self.goal.center().minus(m.p);
                let projection = dot(remaining, dir);
                let edge = (self.radius + 0.5) * (dir[0].abs() + dir[1].abs());
                if projection <= edge && !held && before.phase == 0 {
                    m = before;
                    self.emit(start, m, gait.phases.last().unwrap(), program, yaw, 1)?;
                    // Walking lands on its floor; a mode without one holds its depth.
                    let ground_body = body(Vec3::new(m.p.x, if gait.floor { self.top } else { m.p.y }, m.p.z));
                    let (above, below) = self.goal.axis_sides(ground_body, ground_body, 1);
                    if !(above && below) { return Err("goal lies outside the landing's vertical interval".into()); }
                    held = true;
                    break;
                }
                if projection <= 0.0 && held {
                    self.emit(start, m, &gait.grounded, program, yaw, 1)?;
                    if remaining.horizontal_sqr() >= self.goal.center().minus(start.p).horizontal_sqr() {
                        return Err("bearing cannot bring the body to the goal".into());
                    }
                    break;
                }
                if before.phase != 0 || m.v.x != before.v.x || m.v.z != before.v.z { continue; }

                self.emit(start, m, rows.last().unwrap(), program, yaw, 1)?;
                let mut dx = List::<f64, 16>::new();
                let mut dz = List::<f64, 16>::new();
                let mut probe = m;
                for (i, row) in rows.iter().enumerate() {
                    let d = probe.tick(row, dir, if i + 1 == rows.len() { 0 } else { i + 1 }, self.top);
                    dx.push(d[0]);
                    dz.push(d[1]);
                }
                let displacement = Vec3::new(dx.iter().sum(), 0.0, dz.iter().sum());
                let speed = dot(displacement, dir);
                if speed <= 0.0 { return Err("word has zero advance".into()); }
                let count = ((projection - edge) / speed).floor().max(0.0);
                let other = direction(pair[1]);
                let det = cross(displacement, other);
                let switch = cross(remaining, other) / det;
                let count = if switch >= 1.0 && switch < count { switch.floor() } else { count };
                if count > f64::from(u32::MAX) / rows.len() as f64 { return Err("word exceeds script width".into()); }
                let q = count as u64;
                start = m;
                let at = |cycles| Vec3::new(
                    events::advance_cycles(0, start.p.x, &dx, cycles),
                    start.p.y,
                    events::advance_cycles(1, start.p.z, &dz, cycles),
                );
                let enters = |end| [0, 2].into_iter().all(|axis| {
                    let (low, high) = self.goal.axis_sides(body(start.p), body(end), axis);
                    low && high
                });
                let mut safe = q;
                m.p = at(safe);
                if safe > 0 && enters(m.p) {
                    let mut first = 0;
                    while first < safe {
                        let mid = first + (safe - first) / 2;
                        if enters(at(mid)) { safe = mid; } else { first = mid + 1; }
                    }
                    safe = first.saturating_sub(1);
                    m.p = at(safe);
                }
                m.t += safe * rows.len() as u64;
                self.include(m.p);
                self.emit(start, m, rows.last().unwrap(), program, yaw, safe)?;
                if safe > 0 { break; }
                start = m;
            }
            if m.t - self.from.t > Arguments::MAX_TICKS { return Err("sentence exceeds script width".into()); }
            if held { m.phase = 0; }
        }
    }
}

/// The fastest level gait of the mode the body is in, among the programs it may run: walking's is
/// the sprint-jump, swimming's the sprint stroke. Level: its cycle lands on its floor or returns to
/// its height (a stroke that climbs or sinks is a vertical move, not a zone). Learned once per label.
fn mode_gait<S: CorridorSource>(store: &GeneratorStore, label: LabelId, source: &S, from: &PlayerState) -> Result<Arc<Gait>, String> {
    let programs: &[u16] = if sprint_allowed(from) { &[SPRINT_JUMP, SPRINT, WALK] } else { &[WALK] };
    let speed = |g: &Gait| crate::defect::cycle_advance(&g.phases) / g.phases.len() as f64;
    let level = |g: &Gait| g.floor || g.phases.iter().map(|p| p.dy).sum::<f64>() == 0.0;
    let mut last = String::from("no level gait in this mode");
    let mut best: Option<Arc<Gait>> = None;
    for &p in programs {
        match store.gait(label, source, from, p) {
            Ok(g) if level(&g) => if best.as_ref().is_none_or(|b| speed(&g) > speed(b)) { best = Some(g); },
            Ok(_) => {}
            Err(e) => last = e,
        }
    }
    best.ok_or(last)
}

/// Shared corridor extend helper for `solve()` and defect tail handoffs.
pub(crate) fn extend_corridor<S: CorridorSource>(
    from: &PlayerState,
    to: Goal,
    label: LabelId,
    source: &S,
    store: &GeneratorStore,
    codec: &mut Interner,
) -> Result<List<WordStep, 12>, String> {
    let g = mode_gait(store, label, source, from)?;
    let support = source.support_cell(from);
    let top = crate::store::support_top(source.block(support), support[1]);
    let phase = g.entry(from, top).ok_or("vertical phase outside this word")?;
    let m = Motion { p: from.position, v: from.velocity, phase, t: from.t, wanted: from.movement_this_tick.unwrap_or_default() };
    let radius = Aabb::player(Vec3::default(), from.pose.height()).max.x;
    let mut crossing = Crossing {
        gait: &g,
        from,
        goal: to,
        top,
        radius,
        min: m.p,
        max: m.p,
        words: List::new(),
        proof: CorridorProof { label, min: support, max: support, version: law::LAW_HASH },
        bearing_ns: 0,
    };
    crossing.run(m)?;
    let bounds = cut::cells(Aabb { min: crossing.min, max: crossing.max }, support[1], radius, from.pose.height());
    match source.corridor_query(codec, label, bounds.0, bounds.1) {
        CorridorResult::Clear(proof) => {
            for w in &mut crossing.words {
                w.certificate = proof.clone();
            }
            Ok(crossing.words)
        }
        CorridorResult::Exception { .. } | CorridorResult::Unknown(_) => {
            Err("tail corridor is not clear".into())
        }
    }
}

pub fn solve<S: CorridorSource>(
    from: &PlayerState,
    to: Goal,
    kit: Kit,
    source: &S,
    store: &GeneratorStore,
    codec: &mut Interner,
    deadline_ticks: u64,
) -> SolveResult {
    let first = solve_step(from, to, kit, source, store, codec, deadline_ticks);
    let PlanOutcome::Prefix(pref) = first.outcome else {
        return first;
    };
    let lb = pref.plan.bracket.lb;
    let label = pref.plan.label;
    let mut thinking = pref.plan.thinking;
    let mut all_words = pref.plan.words;
    let mut total_ticks = pref.plan.ticks;
    let mut cur_state = pref.plan.end.materialize();
    let mut all_stages = first.stages;

    for _ in 0..128 {
        let step = solve_step(&cur_state, to, kit, source, store, codec, deadline_ticks);
        for st in step.stages {
            if all_stages.len() < 16 {
                all_stages.push(st);
            }
        }
        let last_note = step.explanation;
        match step.outcome {
            PlanOutcome::Ready(p) => {
                all_words.extend(p.words);
                total_ticks += p.ticks;
                thinking.entry_ns += p.thinking.entry_ns;
                thinking.bearing_ns += p.thinking.bearing_ns;
                thinking.inverse_ns += p.thinking.inverse_ns;
                thinking.output_ns += p.thinking.output_ns;
                return SolveResult {
                    outcome: PlanOutcome::Ready(Plan {
                        ticks: total_ticks,
                        words: all_words,
                        end: p.end,
                        bracket: Bracket { lb, ub: Some(total_ticks) },
                        label,
                        deadline: deadline_ticks,
                        lattice_cost: Default::default(),
                        thinking,
                    }),
                    stages: all_stages,
                    explanation: last_note,
                };
            }
            PlanOutcome::Prefix(np) => {
                all_words.extend(np.plan.words);
                total_ticks += np.plan.ticks;
                cur_state = np.plan.end.materialize();
                thinking.entry_ns += np.plan.thinking.entry_ns;
                thinking.bearing_ns += np.plan.thinking.bearing_ns;
                thinking.inverse_ns += np.plan.thinking.inverse_ns;
                thinking.output_ns += np.plan.thinking.output_ns;
            }
            other => {
                return SolveResult { outcome: other, stages: all_stages, explanation: last_note };
            }
        }
    }
    SolveResult {
        outcome: PlanOutcome::NeedsJoints { exception: to.cell, exceptions: 0, bracket: Bracket { lb, ub: None } },
        stages: all_stages,
        explanation: Some("horizon window step limit exceeded".into()),
    }
}

pub fn solve_step<S: CorridorSource>(
    from: &PlayerState,
    to: Goal,
    kit: Kit,
    source: &S,
    store: &GeneratorStore,
    codec: &mut Interner,
    deadline_ticks: u64,
) -> SolveResult {
    let kit_view = match kit {
        Kit::BareLegs => None,
        Kit::Carried(spec) => Some(KitView::for_spec(spec)),
    };
    let kit_key = match kit {
        Kit::BareLegs => 0,
        Kit::Carried(spec) => spec.capability_key(),
    };
    let restore_required = match kit {
        Kit::BareLegs => true,
        Kit::Carried(spec) => spec.restore_required,
    };
    let mut stages = crate::list![Stage::Bracket, Stage::ClosedCheck];
    let lb_u32 = match &kit_view {
        None => to.lower_bound(from),
        Some(kv) => kv.cone.lower_bound(from, to),
    };
    let bracket = Bracket { lb: u64::from(lb_u32), ub: None };
    let clock = std::time::Instant::now();
    let mut note: Option<String> = None;
    let work = (|| -> Result<PlanOutcome, String> {
        if !store.bind_codec(codec) { return Ok(PlanOutcome::Unknown(Refusal::Schema)); }
        let at_goal = to.reached(from, from);
        let known = if at_goal { None } else { store.words.borrow().route_for_kit_in_world(from, to, kit_key, source) };
        let label = match known.as_ref().and_then(|route| route.words.first()) {
            Some(word) => word.certificate.label,
            None => source.entry_label(codec, from).map_err(|e| format!("entry label: {e:?}"))?,
        };
        let mut thinking = ThinkingCosts::default();
        let mut horizon_target: Option<(Goal, u64)> = None;
        let route = if at_goal {
            crate::word_store::Route { words: List::new(), ticks: 0, end: StateAt::point(from.clone()) }
        } else if let Some(r) = known {
            stages.extend([Stage::Extend, Stage::Cut, Stage::Compose]);
            r
        } else {
            let blocked = [
                ("difficulty", kit_view.is_none() && source.difficulty() != 0),
                ("effects", from.effects),
                ("boat", from.profile.boat),
                ("riding", from.profile.riding),
                ("fall_flying", from.fall_flying),
                ("flying", from.flying),
                ("using_item", from.profile.using_item),
            ];
            if let Some((name, _)) = blocked.iter().find(|(_, b)| *b) {
                return Err(format!("word needs bare legs and invariant resource guards: {name}"));
            }
            stages.push(Stage::Extend);
            let defect_window = || {
                let p = [from.position.x.floor() as i32, from.position.y.floor() as i32, from.position.z.floor() as i32];
                let lo = [p[0].min(to.cell[0]) - 2, p[1].min(to.cell[1]) - 4, p[2].min(to.cell[2]) - 2];
                let hi = [p[0].max(to.cell[0]) + 2, p[1].max(to.cell[1]) + 4, p[2].max(to.cell[2]) + 2];
                cut::Cut::Defect(cut::Window { first: p, exceptions: 1, bounds: (lo, hi), prefixes: Vec::new() })
            };
            let cut_outcome = if from.crouching || to.settle {
                Ok(defect_window())
            } else {
                (|| -> Result<cut::Cut, String> {
                    let ticks = store.preparation_ticks();
                    let g = mode_gait(store, label, source, from)?;
                    thinking.entry_ns = clock.elapsed().as_nanos();
                    if store.preparation_ticks() != ticks {
                        stages.push(Stage::Learn { program: g.program, ticks: store.preparation_ticks() - ticks, nanos: thinking.entry_ns });
                    }
                    let support = source.support_cell(from);
                    let top = crate::store::support_top(source.block(support), support[1]);
                    let phase = g.entry(from, top).ok_or("vertical phase outside this word")?;
                    let m = Motion { p: from.position, v: from.velocity, phase, t: from.t, wanted: from.movement_this_tick.unwrap_or_default() };
                    let radius = Aabb::player(Vec3::default(), from.pose.height()).max.x;
                    let mut crossing = Crossing {
                        gait: &g,
                        from,
                        goal: to,
                        top,
                        radius,
                        min: m.p,
                        max: m.p,
                        words: List::new(),
                        proof: CorridorProof { label, min: support, max: support, version: law::LAW_HASH },
                        bearing_ns: 0,
                    };
                    let time = std::time::Instant::now();
                    let arrival = crossing.run(m);
                    thinking.bearing_ns = crossing.bearing_ns;
                    thinking.inverse_ns = time.elapsed().as_nanos() - thinking.bearing_ns;
                    cut::cut(crossing, arrival, source, codec)
                })()
            };
            let cut_outcome = match cut_outcome {
                Ok(c) => c,
                Err(_) => defect_window(),
            };
            let time = std::time::Instant::now();
            match cut_outcome {
                cut::Cut::Clear(mut words) => {
                    if let Some(w) = words.first_mut() {
                        w.events.marks.push(Event { tick: from.t, kind: EventKind::FoodGuard { sprint_allowed: sprint_allowed(from) } });
                    }
                    if let Some(w) = words.last_mut() {
                        w.events.marks.push(Event { tick: w.end.t, kind: EventKind::GoalBoundary });
                    }
                    stages.extend([Stage::Cut, Stage::Compose]);
                    thinking.output_ns = time.elapsed().as_nanos();
                    crate::word_store::compose_chain(from, to, words, &mut store.words.borrow_mut().work)
                }
                cut::Cut::Unknown(r) => return Ok(PlanOutcome::Unknown(r)),
                cut::Cut::Defect(win) => {
                    stages.push(Stage::Cut);
                    let first_pos = win.first;
                    let exceptions = win.exceptions;
                    // Fast-path: if a long clear sprint-jump prefix lands >= 36 ticks before the defect and far from `to`, emit immediately.
                    let pref_ticks: u32 = win.prefixes.first().map_or(0, |p| p.iter().map(|w| w.ticks).sum());
                    if pref_ticks >= 36 {
                        if let Some(first_pref) = win.prefixes.first() {
                            if let Some(last_w) = first_pref.last() {
                                if last_w.program == SPRINT_JUMP && last_w.ticks >= 12 {
                                    let end_state = last_w.end.materialize();
                                    let rem_dist = (end_state.position.x - to.center().x).hypot(end_state.position.z - to.center().z);
                                    if end_state.on_ground && rem_dist > 20.0 {
                                        let sup = end_state.main_supporting_block.unwrap_or([
                                            end_state.position.x.floor() as i32,
                                            end_state.position.y.floor() as i32 - 1,
                                            end_state.position.z.floor() as i32,
                                        ]);
                                        let mid = Goal { cell: sup, face: Some(crate::Face::Up), settle: false };
                                        let est_rem = to.lower_bound(&end_state) as u64;
                                        stages.push(Stage::Compose);
                                        let mut route = crate::word_store::compose_chain(from, mid, first_pref.clone(), &mut store.words.borrow_mut().work);
                                        route.ticks = u64::from(pref_ticks);
                                        return Ok(PlanOutcome::Prefix(Box::new(PrefixPlan {
                                            plan: Plan {
                                                ticks: route.ticks,
                                                words: route.words,
                                                end: route.end,
                                                bracket: Bracket { lb: bracket.lb, ub: None },
                                                label,
                                                deadline: deadline_ticks,
                                                lattice_cost: Default::default(),
                                                thinking,
                                            },
                                            target: mid,
                                            final_goal: to,
                                            remaining_est_ticks: est_rem,
                                        })));
                                    }
                                }
                            }
                        }
                    }

                    let horiz_dist = (from.position.x - to.center().x).hypot(from.position.z - to.center().z);
                    let est_vol = {
                        let mut lo = [
                            from.position.x.floor().min(to.cell[0] as f64) as i32 - 20,
                            from.position.y.floor().min(to.cell[1] as f64) as i32 - 24,
                            from.position.z.floor().min(to.cell[2] as f64) as i32 - 20,
                        ];
                        let mut hi = [
                            from.position.x.floor().max(to.cell[0] as f64) as i32 + 20,
                            from.position.y.floor().max(to.cell[1] as f64) as i32 + 8,
                            from.position.z.floor().max(to.cell[2] as f64) as i32 + 20,
                        ];
                        if let Some((bmin, bmax)) = source.bounds_box() {
                            for a in 0..3 {
                                lo[a] = lo[a].max(bmin[a]);
                                hi[a] = hi[a].min(bmax[a]);
                            }
                        }
                        let dx = (hi[0] - lo[0] + 1).max(1) as usize;
                        let dy = (hi[1] - lo[1] + 1).max(1) as usize;
                        let dz = (hi[2] - lo[2] + 1).max(1) as usize;
                        dx.saturating_mul(dy).saturating_mul(dz)
                    };
                    let trim_win = |anchor: &PlayerState, mid: Goal, w: cut::Window| -> cut::Window {
                        let fx = from.position.x.floor() as i32;
                        let fy = from.position.y.floor() as i32;
                        let fz = from.position.z.floor() as i32;
                        let ap = [anchor.position.x.floor() as i32, anchor.position.y.floor() as i32, anchor.position.z.floor() as i32];
                        cut::Window {
                            first: w.first,
                            exceptions: w.exceptions,
                            bounds: (
                                [fx.min(ap[0]).min(mid.cell[0]) - 2, fy.min(ap[1]).min(mid.cell[1]) - 4, fz.min(ap[2]).min(mid.cell[2]) - 2],
                                [fx.max(ap[0]).max(mid.cell[0]) + 2, fy.max(ap[1]).max(mid.cell[1]) + 4, fz.max(ap[2]).max(mid.cell[2]) + 2],
                            ),
                            prefixes: w.prefixes,
                        }
                    };
                    let mut active_goal = to;
                    let mut active_win = win.clone();
                    let mut rem_est = 0u64;
                    let mut far_cache = None;
                    let anchor = win.prefixes.first().and_then(|p| p.last()).map_or_else(|| from.clone(), |w| w.end.materialize());
                    if horiz_dist > 26.0 || est_vol > 96_000 {
                        let far = crate::far_field::FarField::get_or_build(store, source, label, from, to, kit_view.as_ref(), kit_key);
                        if let Some((mid, est)) = far.next_horizon(&anchor, to, 20.0) {
                            active_goal = mid;
                            rem_est = est;
                            active_win = trim_win(&anchor, mid, win.clone());
                        }
                        far_cache = Some(far);
                    }

                    let defect_res = match defect::solve_defect(from, active_goal, label, active_win, source, store, codec, kit_view.as_ref(), kit_key, &mut stages, &mut thinking) {
                        Ok(res) => Ok(res),
                        Err((ex_count, why)) => {
                            if active_goal != to {
                                if let Some((mid2, est2)) = far_cache.as_ref().and_then(|f| f.next_horizon(&anchor, to, 12.0)).filter(|&(m, _)| m != active_goal) {
                                    active_goal = mid2;
                                    rem_est = est2;
                                    defect::solve_defect(from, active_goal, label, trim_win(&anchor, mid2, win), source, store, codec, kit_view.as_ref(), kit_key, &mut stages, &mut thinking)
                                } else {
                                    Err((ex_count, why))
                                }
                            } else {
                                Err((ex_count, why))
                            }
                        }
                    };
                    let (raw_steps, obs) = match defect_res {
                        Ok(res) => res,
                        Err((ex_count, why)) => {
                            note = Some(why);
                            return Ok(PlanOutcome::NeedsJoints { exception: first_pos, exceptions: ex_count, bracket });
                        }
                    };
                    let mut cert_ticks = 0u64;
                    let programs = words::tiny_programs();
                    let (route, cached) = match certify::certify_route(from, active_goal, label, source.provenance(), source, &programs, &raw_steps, kit_key, restore_required, &mut cert_ticks) {
                        Ok(c) => c,
                        Err((Refusal::Coverage, why)) => {
                            note = Some(format!("certify:{why}"));
                            return Ok(PlanOutcome::NeedsJoints { exception: first_pos, exceptions, bracket });
                        }
                        Err((r, why)) => {
                            note = Some(format!("certify:{why}"));
                            return Ok(PlanOutcome::Unknown(r));
                        }
                    };
                    if let Some(Stage::Defect { certify_ticks, .. }) = stages.iter_mut().rev().find(|s| matches!(s, Stage::Defect { .. })) {
                        *certify_ticks = cert_ticks;
                    }
                    {
                        let mut ws = store.words.borrow_mut();
                        ws.work.certify_ticks += cert_ticks;
                        ws.work.law_ticks += cert_ticks;
                        ws.promote_route(&obs);
                        if active_goal == to {
                            ws.file_route_for_kit(from, to, kit_key, cached);
                        }
                    }
                    if active_goal != to {
                        horizon_target = Some((active_goal, rem_est));
                    }
                    stages.push(Stage::Compose);
                    route
                }
            }
        };
        stages.extend([Stage::ClosedCheck, Stage::Allocate]);
        if let Some((mid, rem_est)) = horizon_target {
            return Ok(PlanOutcome::Prefix(Box::new(PrefixPlan {
                plan: Plan {
                    ticks: route.ticks,
                    words: route.words,
                    end: route.end,
                    bracket: Bracket { lb: bracket.lb, ub: None },
                    label,
                    deadline: deadline_ticks,
                    lattice_cost: Default::default(),
                    thinking,
                },
                target: mid,
                final_goal: to,
                remaining_est_ticks: rem_est,
            })));
        }
        Ok(PlanOutcome::Ready(Plan {
            ticks: route.ticks,
            words: route.words,
            end: route.end,
            bracket: Bracket { lb: bracket.lb, ub: Some(route.ticks) },
            label,
            deadline: deadline_ticks,
            lattice_cost: Default::default(),
            thinking,
        }))
    })();
    match work {
        Ok(outcome) => SolveResult { outcome, stages, explanation: note },
        Err(e) => SolveResult { outcome: PlanOutcome::Unknown(Refusal::StateOrContext), stages, explanation: Some(e) },
    }
}
