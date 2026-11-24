//! First-exception bisection along a counted corridor extend.
use law::{Aabb, BlockPos, Vec3, codec::{Interner, proof::Refusal}};
use crate::{
    events,
    inline::List,
    lattice,
    solve::{Crossing, Motion, Span, WordStep, direction},
    source::{CorridorResult, CorridorSource},
    state_view::StateAt,
    store::Phase,
};

pub(super) type Cells = (BlockPos, BlockPos);

pub(super) struct Window {
    pub first: BlockPos,
    pub exceptions: usize,
    pub bounds: Cells,
    pub prefixes: Vec<List<WordStep, 12>>,
}

pub(super) enum Cut {
    Clear(List<WordStep, 12>),
    Defect(Window),
    Unknown(Refusal),
}

pub(crate) fn cells(h: Aabb, floor: i32, radius: f64, height: f32) -> Cells {
    (
        [(h.min.x - radius).floor() as i32, floor.min(h.min.y.floor() as i32), (h.min.z - radius).floor() as i32],
        [(h.max.x + radius).floor() as i32, (h.max.y + f64::from(height)).floor() as i32, (h.max.z + radius).floor() as i32],
    )
}

impl Span {
    fn rows(&self) -> &[Phase] {
        if self.held { std::slice::from_ref(&self.gait.grounded) } else { &self.gait.phases }
    }
    fn periods(&self) -> u32 {
        (self.start.phase as u32 + self.ticks) / self.rows().len() as u32
    }
    fn sample(&self, ticks: u32, hull: &mut Aabb) -> Motion {
        let mut m = self.start;
        let rows = self.rows();
        let dir = direction(self.yaw);
        let mut include = |p: Vec3| *hull = crate::goal::hull(*hull, Aabb { min: p, max: p });
        let mut remain = ticks;
        if self.count > 1 && ticks >= rows.len() as u32 {
            let mut dx = List::<f64, 16>::new();
            let mut dz = List::<f64, 16>::new();
            for row in rows {
                let next = (m.phase + 1) % rows.len();
                let d = m.tick(row, dir, next, self.top);
                include(m.p);
                dx.push(d[0]);
                dz.push(d[1]);
            }
            let skip = ticks / rows.len() as u32 - 1;
            m = self.start;
            m.p.x = events::advance_cycles(0, m.p.x, &dx, skip.into());
            m.p.z = events::advance_cycles(1, m.p.z, &dz, skip.into());
            m.t += u64::from(skip) * rows.len() as u64;
            remain -= skip * rows.len() as u32;
            include(m.p);
        }
        for _ in 0..remain {
            let phase = m.phase;
            m.tick(&rows[phase], dir, (phase + 1) % rows.len(), self.top);
            include(m.p);
        }
        m
    }
}

impl Crossing<'_> {
    fn checkpoint(&self, mut c: u32, keep: bool) -> (Motion, Cells, Cells, f64, List<WordStep, 12>) {
        let mut words = List::new();
        let floor = self.proof.min[1];
        let mut hull = Aabb { min: self.from.position, max: self.from.position };
        let mut m = Motion {
            p: self.from.position,
            v: self.from.velocity,
            phase: self.gait.entry(self.from, self.top).unwrap_or(0),
            t: self.from.t,
            wanted: self.from.movement_this_tick.unwrap_or_default(),
        };
        let mut rows = &self.gait.phases[..];
        let mut yaw = lattice::bracket(self.goal.center().x - m.p.x, self.goal.center().z - m.p.z)[0];
        for w in &self.words {
            let span = w.events.span.as_ref().unwrap();
            rows = span.rows();
            yaw = span.yaw;
            if c == 0 { break; }
            let n = span.periods();
            let complete = c >= n && (span.start.phase as u32 + span.ticks).is_multiple_of(rows.len() as u32);
            let take = if complete { span.ticks } else { c.min(n) * rows.len() as u32 - span.start.phase as u32 };
            m = span.sample(take, &mut hull);
            if keep && take > 0 {
                let mut step = w.clone();
                if take != w.ticks {
                    let mut end = StateAt::shared(rows.last().unwrap().after.clone());
                    end.t = m.t;
                    end.position = m.p;
                    end.velocity = m.v;
                    end.yaw = lattice::yaw(yaw);
                    end.movement = Some(m.wanted);
                    end.main_supporting_block = Some([m.p.x.floor() as i32, floor, m.p.z.floor() as i32]);
                    end.health = self.from.health;
                    end.food_level = self.from.food_level;
                    end.exhaustion = self.from.exhaustion;
                    end.saturation = self.from.saturation;
                    end.food_tick_timer = self.from.food_tick_timer;
                    step.end = end;
                    step.expected_end_t = m.t;
                    step.ticks = take;
                    let partial = step.events.span.as_mut().unwrap();
                    partial.ticks = take;
                    if span.count > 1 {
                        step.args.count = c.min(n);
                        partial.count = step.args.count;
                    }
                }
                words.push(step);
            }
            c = c.saturating_sub(n);
            if !complete { break; }
        }
        let prefix = cells(hull, floor, self.radius, self.from.pose.height());
        let mut next = m;
        for _ in 0..rows.len() {
            let phase = next.phase % rows.len();
            next.tick(&rows[phase], direction(yaw), (phase + 1) % rows.len(), self.top);
            hull = crate::goal::hull(hull, Aabb { min: next.p, max: next.p });
        }
        (m, prefix, cells(hull, floor, self.radius, self.from.pose.height()), next.p.minus(m.p).horizontal_sqr().sqrt(), words)
    }
}

pub(super) fn cut<S: CorridorSource>(mut crossing: Crossing<'_>, arrival: Result<(), String>, source: &S, codec: &mut Interner) -> Result<Cut, String> {
    let floor = crossing.proof.min[1];
    let label = crossing.proof.label;
    let bounds = cells(Aabb { min: crossing.min, max: crossing.max }, floor, crossing.radius, crossing.from.pose.height());
    let (mut first, exceptions) = match source.corridor_query(codec, label, bounds.0, bounds.1) {
        CorridorResult::Clear(proof) => {
            arrival?;
            for w in &mut crossing.words { w.certificate = proof.clone(); }
            return Ok(Cut::Clear(crossing.words));
        }
        CorridorResult::Unknown(reason) => return Ok(Cut::Unknown(reason)),
        CorridorResult::Exception { first_pos, exceptions } => (first_pos, exceptions),
    };
    let end: u32 = crossing.words.iter().map(|w| w.events.span.as_ref().unwrap().periods()).sum();
    let (mut lo, mut hi) = (0u32, end);
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        let b = crossing.checkpoint(mid, false).2;
        if matches!(source.corridor_query(codec, label, b.0, b.1), CorridorResult::Clear(_)) {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    let (hit, _, b, _, _) = crossing.checkpoint(lo, false);
    if let CorridorResult::Exception { first_pos, .. } = source.corridor_query(codec, label, b.0, b.1) {
        first = first_pos;
    }
    let mut prefixes = Vec::new();
    for c in (0..=lo).rev() {
        let (m, body, _, r, mut words) = crossing.checkpoint(c, true);
        if m.p.minus(hit.p).horizontal_sqr() > (r * 1.5).powi(2) && !prefixes.is_empty() {
            break;
        }
        if c > 0 {
            let CorridorResult::Clear(proof) = source.corridor_query(codec, label, body.0, body.1) else { continue; };
            for w in &mut words { w.certificate = proof.clone(); }
        }
        prefixes.push(words);
        if prefixes.len() >= 2 {
            break;
        }
    }
    if prefixes.is_empty() {
        prefixes.push(List::new());
    }
    Ok(Cut::Defect(Window { first, exceptions, bounds: b, prefixes }))
}
