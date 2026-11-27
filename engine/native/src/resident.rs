//! R0 worker. Existing F search only; no zone/store/search implementation here.
use std::{collections::BTreeMap, sync::{Arc, Mutex, OnceLock, atomic::{AtomicU64, Ordering}, mpsc}};
use law::{Block, BlockPos, PlayerState, World, basis::cell::StateId, sim::WorldDelta};
use plan::Goal;
use crate::kit_wire;

pub const READY: u64 = 0;
pub const WORD_PLAN: u64 = 1;
pub const REFUSAL: u64 = 3;
/// A plan with world edits: `WORD_PLAN`'s layout with `[a, a × kit_wire::ACTION_WORDS]` between the
/// inputs and the hull. A plan without edits stays `WORD_PLAN`, word for word.
pub const KIT_PLAN: u64 = 4;
/// Streaming continuation packets appended to an in-flight `WORD_PLAN` / `KIT_PLAN`.
/// Layout: `[opcode, version, tick_offset, input_count, inputs (count*3)..., (if 6: k, k*10 actions...), hx0, hz0, hx1, hz1]`.
pub const PLAN_APPEND: u64 = 5;
pub const KIT_PLAN_APPEND: u64 = 6;
pub fn refusal(version: u64, reason: impl AsRef<str>) -> Vec<u64> {
    let at = crate::qpc().0 as u64;
    let mut out = vec![REFUSAL, version, at, at];
    out.extend(reason.as_ref().bytes().map(u64::from)); out
}

#[derive(Default)]
struct Source {
    session: u64,
    revision: u64,
    law_revision: u64,
    difficulty: u8,
    regeneration: Option<bool>,
    // One versioned palette per received section; fine state bytes are retained.
    sections: BTreeMap<BlockPos, Section>,
    error: Option<String>,
}
struct Section {
    palette: Vec<(String, Result<StateId, String>)>,
    indices: Vec<usize>,
    collision_bits: [[u16; 256]; 2],
    uniform_state: Option<StateId>,
    flags: u8, // bit 0: fluid, bit 1: inside, bit 2: unknown
}
#[inline]
fn cell_reach(entry: &Result<StateId, String>, p: BlockPos) -> u8 {
    let Ok(sid) = entry else { return 3; };
    let b = sid.block_at(p);
    let mut result = u8::from(b.inside == law::Inside::PowderSnow);
    for shape in b.collision_shapes().filter(|s| !s.is_empty()) {
        if !shape.is_block() && shape.coords.iter().any(|axis| axis[0] < 0.0 || axis[axis.len() - 1] > 1.0) {
            return 3;
        }
        result = 1;
    }
    result
}
#[inline]
fn entry_flags(entry: &Result<StateId, String>) -> u8 {
    let Ok(sid) = entry else { return 7; };
    let b = sid.model_block();
    u8::from(b.fluid.kind != law::FluidKind::Empty) | (u8::from(b.has_inside_effects()) << 1)
}
impl Section {
    fn new(palette: Vec<(String, Result<StateId, String>)>, indices: Vec<usize>) -> Self {
        let mut s = Self { palette, indices, collision_bits: [[0; 256]; 2], uniform_state: None, flags: 0 };
        s.recompute();
        s
    }
    fn recompute(&mut self) {
        self.collision_bits = [[0; 256]; 2];
        self.flags = 0;
        for (_, entry) in &self.palette {
            self.flags |= entry_flags(entry);
        }
        let first_idx = self.indices[0];
        let mut all_same = true;
        let reach_pal: Vec<u8> = self.palette.iter().map(|(_, e)| cell_reach(e, [0; 3])).collect();
        for (i, &idx) in self.indices.iter().enumerate() {
            if idx != first_idx { all_same = false; }
            let r = reach_pal[idx];
            if r != 0 {
                let row = i >> 4;
                let bit = 1u16 << (i & 15);
                if (r & 1) != 0 { self.collision_bits[0][row] |= bit; }
                if (r & 2) != 0 { self.collision_bits[1][row] |= bit; }
            }
        }
        self.uniform_state = if all_same {
            self.palette[first_idx].1.as_ref().ok().copied().filter(|sid| sid.model().is_some_and(|m| m.fixed()))
        } else {
            None
        };
    }
    fn set_cell(&mut self, i: usize, id: usize) {
        let old_id = std::mem::replace(&mut self.indices[i], id);
        let r = cell_reach(&self.palette[id].1, [0; 3]);
        let row = i >> 4;
        let bit = 1u16 << (i & 15);
        self.collision_bits[0][row] = (self.collision_bits[0][row] & !bit) | if (r & 1) != 0 { bit } else { 0 };
        self.collision_bits[1][row] = (self.collision_bits[1][row] & !bit) | if (r & 2) != 0 { bit } else { 0 };
        let new_f = entry_flags(&self.palette[id].1);
        let old_f = entry_flags(&self.palette[old_id].1);
        if (old_f & !new_f) != 0 {
            self.flags = self.indices.iter().fold(0u8, |acc, &idx| acc | entry_flags(&self.palette[idx].1));
        } else {
            self.flags |= new_f;
        }
        if self.uniform_state.is_some() && self.palette[id].1.as_ref().ok().copied() != self.uniform_state {
            self.uniform_state = None;
        }
    }
}
#[inline]
fn x_mask(lo: i32, hi: i32) -> u16 { (u16::MAX << lo) & (u16::MAX >> (15 - hi)) }
impl Source {
    fn cell(&self, p: BlockPos) -> Option<&Result<StateId, String>> {
        let s = self.sections.get(&p.map(|v| v.div_euclid(16)))?;
        let i = ((p[1].rem_euclid(16) * 16 + p[2].rem_euclid(16)) * 16 + p[0].rem_euclid(16)) as usize;
        Some(&s.palette[s.indices[i]].1)
    }
    fn bounds(&self) -> Option<(BlockPos, BlockPos)> {
        let mut lo = [i32::MAX;3]; let mut hi = [i32::MIN;3];
        for p in self.sections.keys() { for a in 0..3 { lo[a] = lo[a].min(p[a]*16); hi[a] = hi[a].max(p[a]*16+15); } }
        (!self.sections.is_empty()).then_some((lo, hi))
    }
    fn flags_in(&self, lo: BlockPos, hi: BlockPos, mask: u8) -> bool {
        for sz in lo[2].div_euclid(16)..=hi[2].div_euclid(16) {
            for sy in lo[1].div_euclid(16)..=hi[1].div_euclid(16) {
                for sx in lo[0].div_euclid(16)..=hi[0].div_euclid(16) {
                    if self.sections.get(&[sx, sy, sz]).is_none_or(|s| (s.flags & mask) != 0) {
                        return true;
                    }
                }
            }
        }
        false
    }
}
impl World for Source {
    fn known_block(&self, p: BlockPos) -> Result<&Block, &str> {
        match self.cell(p).ok_or("section not received")? { Ok(sid) => Ok(sid.block_at(p)), Err(why) => Err(why) }
    }
    fn block(&self, p: BlockPos) -> &Block {
        match self.cell(p) {
            Some(Ok(sid)) => sid.block_at(p),
            Some(Err(why)) => panic!("source refuses {p:?}: {why}"),
            None => panic!("unknown source cell {p:?}"),
        }
    }
    fn state_id(&self, p: BlockPos) -> Option<StateId> { self.cell(p)?.as_ref().ok().copied() }
    fn uniform(&self, lo: BlockPos, hi: BlockPos) -> Option<&Block> {
        if (0..3).any(|i| lo[i] > hi[i]) { return None; }
        let mut found: Option<StateId> = None;
        for sz in lo[2].div_euclid(16)..=hi[2].div_euclid(16) {
            for sy in lo[1].div_euclid(16)..=hi[1].div_euclid(16) {
                for sx in lo[0].div_euclid(16)..=hi[0].div_euclid(16) {
                    let s = self.sections.get(&[sx, sy, sz])?;
                    if let Some(u) = s.uniform_state {
                        if *found.get_or_insert(u) != u { return None; }
                        continue;
                    }
                    let base = [sx * 16, sy * 16, sz * 16];
                    for z in lo[2].max(base[2])..=hi[2].min(base[2] + 15) {
                        for y in lo[1].max(base[1])..=hi[1].min(base[1] + 15) {
                            for x in lo[0].max(base[0])..=hi[0].min(base[0] + 15) {
                                let i = ((y.rem_euclid(16) * 16 + z.rem_euclid(16)) * 16 + x.rem_euclid(16)) as usize;
                                let sid = s.palette[s.indices[i]].1.as_ref().ok().copied()?;
                                if !sid.model().is_some_and(|m| m.fixed()) { return None; }
                                if *found.get_or_insert(sid) != sid { return None; }
                            }
                        }
                    }
                }
            }
        }
        found.map(|sid| sid.block_at(lo))
    }
    fn loaded(&self, p: BlockPos) -> bool { self.sections.contains_key(&p.map(|v| v.div_euclid(16))) }
    fn difficulty(&self) -> u8 { self.difficulty }
    // The rule is not sent to a remote client: projected as vanilla's default; it changes health, never a walk, and the server corrects it.
    fn natural_regeneration(&self) -> bool { self.regeneration.unwrap_or(true) }
    fn has_fluid(&self) -> bool { self.sections.values().any(|s| (s.flags & 1) != 0) }
    fn has_inside_effects(&self) -> bool { self.sections.values().any(|s| (s.flags & 2) != 0) }
    fn has_fluid_in(&self, lo: BlockPos, hi: BlockPos) -> bool { self.flags_in(lo, hi, 1) }
    fn has_inside_effects_in(&self, lo: BlockPos, hi: BlockPos) -> bool { self.flags_in(lo, hi, 2) }
    fn collision_cell_bounds(&self, bounds: law::Aabb, lo: BlockPos, hi: BlockPos) -> Option<(BlockPos, BlockPos)> {
        let mut result = None;
        self.visit_collision_cells(bounds, lo, hi, |p| {
            let (a, b) = result.get_or_insert((p, p));
            for i in 0..3 { a[i] = a[i].min(p[i]); b[i] = b[i].max(p[i]); }
            true
        });
        result
    }
    fn visit_collision_cells(&self, bounds: law::Aabb, lo: BlockPos, hi: BlockPos, mut visit: impl FnMut(BlockPos) -> bool) -> bool {
        let near_lo = [bounds.min.x, bounds.min.y, bounds.min.z].map(law::math::floor);
        let near_hi = [bounds.max.x, bounds.max.y, bounds.max.z].map(law::math::floor);
        for z in lo[2]..=hi[2] {
            for y in lo[1]..=hi[1] {
                let row = ((y.rem_euclid(16) * 16 + z.rem_euclid(16)) * 16) as usize >> 4;
                for sx in lo[0].div_euclid(16)..=hi[0].div_euclid(16) {
                    let bx = sx * 16;
                    let (a, b) = (bx.max(lo[0]) & 15, (bx + 15).min(hi[0]) & 15);
                    let Some(section) = self.sections.get(&[sx, y.div_euclid(16), z.div_euclid(16)]) else {
                        for x in bx.max(lo[0])..=(bx + 15).min(hi[0]) {
                            if !visit([x, y, z]) { return false; }
                        }
                        continue;
                    };
                    let mut candidates = section.collision_bits[1][row];
                    let (start, end) = (near_lo[0].max(bx), near_hi[0].min(bx + 15));
                    if start <= end && (near_lo[1]..=near_hi[1]).contains(&y) && (near_lo[2]..=near_hi[2]).contains(&z) {
                        candidates |= section.collision_bits[0][row] & x_mask(start & 15, end & 15);
                    }
                    candidates &= x_mask(a, b);
                    while candidates != 0 {
                        if !visit([bx | candidates.trailing_zeros() as i32, y, z]) { return false; }
                        candidates &= candidates - 1;
                    }
                }
            }
        }
        true
    }
}
struct Cursor<'a> { data: &'a [u64], at: usize }
impl Cursor<'_> {
    fn word(&mut self) -> Result<u64,String> { let v = *self.data.get(self.at).ok_or("truncated delta")?; self.at += 1; Ok(v) }
    fn pos(&mut self) -> Result<BlockPos,String> {
        let mut p = [0;3]; for v in &mut p { let n = self.word()?; *v = n as i32; if *v as u64 != n { return Err("coordinate range".into()); } } Ok(p)
    }
    fn text(&mut self) -> Result<String,String> {
        let len = usize::try_from(self.word()?).map_err(|_| "string length")?;
        if len > self.data.len()-self.at { return Err("truncated string".into()); }
        let mut bytes = Vec::with_capacity(len);
        for _ in 0..len { bytes.push(u8::try_from(self.word()?).map_err(|_| "string byte")?); }
        String::from_utf8(bytes).map_err(|_| "string utf8".into())
    }
}
fn model(text: &str) -> Result<StateId,String> {
    let (name, tail) = text.split_once('[').unwrap_or((text,""));
    let mut properties = Vec::new();
    for pair in tail.trim_end_matches(']').split(',').filter(|p| !p.is_empty()) {
        let (k,v) = pair.split_once('=').ok_or("block property")?;
        properties.push((k.to_owned(),v.to_owned()));
    }
    world::state_id(&world::BlockState { name: name.to_owned(), properties })
}
fn law_same(a: &str, b: &str) -> bool {
    if a == b { return true; }
    let (Ok(sa), Ok(sb)) = (model(a), model(b)) else { return false; };
    sa.block() == sb.block() && sa.model().is_some() && sa.model() == sb.model() && sa.rules() == sb.rules()
}
fn block_delta_changes_law(data: &[u64]) -> bool {
    let mut c = Cursor { data, at: 3 };
    let Ok(_) = c.pos() else { return true; };
    let Ok(state) = c.text() else { return true; };
    if c.at >= data.len() { return true; }
    let Ok(before) = c.text() else { return true; };
    if c.at != data.len() { return true; }
    !law_same(&state, &before)
}
fn delta(source: &mut Source, data: &[u64]) -> Result<(),String> {
    let mut c = Cursor { data, at: 0 };
    let kind = c.word()?; let session = c.word()?; let revision = c.word()?;
    if kind == 0 {
        let difficulty = u8::try_from(c.word()?).map_err(|_| "difficulty")?;
        let regeneration = match c.word()? { 0 => Some(false), 1 => Some(true), 2 => None, _ => return Err("regeneration flag".into()) };
        if difficulty > 3 { return Err("difficulty range".into()); }
        if c.at != data.len() { return Err("trailing delta words".into()); }
        *source = Source { session, revision, difficulty, regeneration, ..Source::default() };
        world::model_table(); // Parse once at world join, on this worker.
    } else {
        if source.session != session || revision <= source.revision { return Err("stale source session/revision".into()); }
        // The server's difficulty arrives after the world is joined: kind 4 updates it in place.
        if kind == 4 {
            let difficulty = u8::try_from(c.word()?).map_err(|_| "difficulty")?;
            if difficulty > 3 || c.at != data.len() { return Err("difficulty delta".into()); }
            source.difficulty = difficulty; source.revision = revision; return Ok(());
        }
        let p = c.pos()?;
        match kind {
            1 => {
                let count = usize::try_from(c.word()?).map_err(|_| "palette length")?;
                if count == 0 || count > data.len()-c.at { return Err("palette length".into()); }
                let mut palette = Vec::new();
                for _ in 0..count { let name = c.text()?; let answer = model(&name); palette.push((name,answer)); }
                let mut indices = Vec::with_capacity(4096);
                for _ in 0..4096 { let i = usize::try_from(c.word()?).map_err(|_| "palette index")?; if i >= count { return Err("palette index".into()); } indices.push(i); }
                if c.at != data.len() { return Err("trailing delta words".into()); }
                source.sections.insert(p, Section::new(palette, indices));
            }
            2 => {
                let name = c.text()?;
                if c.at < data.len() { let _ = c.text()?; }
                if c.at != data.len() { return Err("trailing delta words".into()); }
                if let Some(section) = source.sections.get_mut(&p.map(|v| v.div_euclid(16))) {
                    let id = match section.palette.iter().position(|(n,_)| n == &name) { Some(i) => i, None => { let answer = model(&name); section.palette.push((name,answer)); section.palette.len()-1 } };
                    let i = ((p[1].rem_euclid(16)*16+p[2].rem_euclid(16))*16+p[0].rem_euclid(16)) as usize;
                    section.set_cell(i, id);
                }
            }
            3 => { if c.at != data.len() { return Err("trailing delta words".into()); } source.sections.remove(&p); }
            _ => return Err("delta kind".into()),
        }
        source.revision = revision;
    }
    if c.at != data.len() { return Err("trailing delta words".into()); } Ok(())
}

pub struct Request { pub state: PlayerState, pub meta: Vec<u64>, pub goal: Goal, pub issue: u64, pub version: u64 }
#[derive(Clone)]
struct ActiveContinuation {
    requested: u64,
    support: BlockPos,
    final_goal: Goal,
    carried: kit_wire::Carried,
    state: PlayerState,
    tick_offset: u64,
    bounds: (BlockPos, BlockPos),
}
enum Work { Delta(Vec<u64>, u64, bool), Goal(Request) }
struct Resident { send: mpsc::Sender<Work>, results: Mutex<mpsc::Receiver<Vec<u64>>>, version: Arc<AtomicU64>, law_revision: Arc<AtomicU64>, observation: Mutex<Vec<u64>>, known: Mutex<std::collections::HashMap<[u64; 3], u64>> }
fn resident() -> &'static Resident {
    static RESIDENT: OnceLock<Resident> = OnceLock::new();
    RESIDENT.get_or_init(|| {
        let (send, recv) = mpsc::channel(); let (done, results) = mpsc::channel();
        let version = Arc::new(AtomicU64::new(0)); let law_revision = Arc::new(AtomicU64::new(0));
        let (v, r) = (version.clone(), law_revision.clone());
        std::thread::Builder::new().name("pathtracing-F".into()).spawn(move || {
            // One living store for the bot's whole session: a gait learned once is never relearned (S230).
            let mut source = Source::default();
            let mut store = plan::store::GeneratorStore::default();
            let mut codec = law::codec::Interner::default();
            let mut active_cont: Option<ActiveContinuation> = None;
            let mut idle_consolidate = false;
            let mut idle_sections: Vec<BlockPos> = Vec::new();
            let mut last_provenance: Option<[u8; 32]> = None;
            loop {
                let can_idle_work = active_cont.is_none()
                    && (idle_consolidate || (last_provenance.is_some() && !idle_sections.is_empty()));
                let msg = if active_cont.is_some() || can_idle_work {
                    match recv.try_recv() {
                        Ok(w) => Some(w),
                        Err(mpsc::TryRecvError::Empty) => None,
                        Err(mpsc::TryRecvError::Disconnected) => break,
                    }
                } else {
                    match recv.recv() {
                        Ok(w) => Some(w),
                        Err(_) => break,
                    }
                };
                if let Some(work) = msg {
                    match work {
                        Work::Delta(data, law_rev, law_changed) => {
                            let reset = data.first() == Some(&0);
                            let kind = data.first().copied().unwrap_or(0);
                            let p = if data.len() >= 6 && matches!(kind, 1 | 2 | 3) {
                                Some([data[3] as i32, data[4] as i32, data[5] as i32])
                            } else { None };
                            if let Err(why) = delta(&mut source, &data) {
                                if why != "stale source session/revision" {
                                    source.error = Some(why.clone());
                                    let _ = done.send(refusal(0,why));
                                }
                            }
                            source.law_revision = law_rev;
                            if reset {
                                store = plan::store::GeneratorStore::default();
                                codec = law::codec::Interner::default();
                                active_cont = None;
                                idle_consolidate = false;
                                idle_sections.clear();
                                last_provenance = None;
                            } else if kind == 4 || (kind == 0 && !reset) {
                                store.words.borrow_mut().environment_event();
                                store.clear_far();
                                active_cont = None;
                                idle_sections.clear();
                            } else {
                                if let Some(cell) = p {
                                    let sec = if kind == 2 { cell.map(|v| v.div_euclid(16)) } else { cell };
                                    if kind == 1 && idle_sections.len() < 256 && !idle_sections.contains(&sec) {
                                        idle_sections.push(sec);
                                    }
                                    if kind != 2 || law_changed {
                                        store.invalidate_far_section(sec);
                                    }
                                    if store.words.borrow().has_certificates() {
                                        if kind == 2 {
                                            if law_changed { store.words.borrow_mut().edit(&mut codec, cell, &source); }
                                        } else {
                                            store.words.borrow_mut().edit_section(&mut codec, cell, &source);
                                        }
                                    }
                                }
                            }
                        }
                        Work::Goal(request) => {
                            let req_version = request.version;
                            active_cont = None;
                            let latest = v.clone();
                            let stale = move || latest.load(Ordering::Acquire) != req_version;
                            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| plan::supersede::while_current(stale, || plan_step_with_prov(&source, &store, &mut codec, request, &v, &r)))) {
                                Ok((answer, cont, prov)) => {
                                    let _ = done.send(answer);
                                    active_cont = cont;
                                    if let Some(p) = prov {
                                        last_provenance = Some(p);
                                    }
                                    idle_consolidate = true;
                                }
                                Err(p) => {
                                    let _ = done.send(refusal(req_version, format!("planner panic: {}", p.downcast_ref::<String>().cloned().or_else(|| p.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default())));
                                }
                            }
                        }
                    }
                    continue;
                }
                if let Some(mut cont) = active_cont.take() {
                    // Only a newer command ends it. A world change outside the plan's corridor does not
                    // touch the plan; one inside stops the bot in the mod (`sourceChanged`).
                    if v.load(Ordering::Acquire) == cont.requested {
                        let (latest, want) = (v.clone(), cont.requested);
                        let stale = move || latest.load(Ordering::Acquire) != want;
                        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| plan::supersede::while_current(stale, || advance_continuation(&source, &store, &mut codec, &mut cont, &v, &r)))) {
                            Ok(Some((pkt, more))) => {
                                let _ = done.send(pkt);
                                if more {
                                    active_cont = Some(cont);
                                }
                            }
                            Ok(None) => {}
                            Err(_) => { let _ = done.send(refusal(want, "next part: planner panic")); }
                        }
                    }
                } else if idle_consolidate {
                    store.consolidate_idle();
                    idle_consolidate = false;
                } else if let Some(prov) = last_provenance {
                    let req_ver = v.load(Ordering::Acquire);
                    let req_rev = r.load(Ordering::Acquire);
                    for _ in 0..4 {
                        if v.load(Ordering::Acquire) != req_ver || r.load(Ordering::Acquire) != req_rev {
                            break;
                        }
                        let Some(sec) = idle_sections.pop() else { break; };
                        store.warm_far_section(prov, &source, sec);
                    }
                }
            }
        }).expect("resident worker");
        Resident { send, results: Mutex::new(results), version, law_revision, observation: Mutex::new(Vec::new()), known: Mutex::new(std::collections::HashMap::new()) }
    })
}
pub fn publish(data: Vec<u64>) -> Vec<u64> {
    if data.len() < 3 { return refusal(0,"delta header"); }
    let r = resident();
    // The world changes only by an edit. A section arriving is the bot learning what is already
    // there, and an unload is forgetting it: neither moves the law under a plan. A section sent
    // again with other content is an edit (Julien 2026-11-27: "the world doesn't change").
    let law_changed = match data[0] {
        2 => block_delta_changes_law(&data),
        1 | 3 if data.len() >= 6 => {
            use std::hash::{Hash, Hasher};
            let key = [data[3], data[4], data[5]];
            let mut known = r.known.lock().unwrap();
            if data[0] == 3 { known.remove(&key); false } else {
                let mut h = std::collections::hash_map::DefaultHasher::new();
                data[6..].hash(&mut h);
                known.insert(key, h.finish()).is_some_and(|before| before != h.finish())
            }
        }
        0 => { r.known.lock().unwrap().clear(); true }
        _ => true,
    };
    let law_rev = if law_changed {
        r.law_revision.fetch_add(1,Ordering::AcqRel) + 1
    } else {
        r.law_revision.load(Ordering::Acquire)
    };
    if data[0] == 0 { r.observation.lock().unwrap().clear(); }
    match r.send.send(Work::Delta(data, law_rev, law_changed)) { Ok(_) => vec![READY,u64::from(law_changed)], Err(_) => refusal(0,"worker stopped") }
}
pub fn request(request: Request) -> Vec<u64> {
    let r = resident(); let v = request.version;
    if r.version.fetch_max(v,Ordering::AcqRel) > v { return refusal(v,"stale request version"); }
    match r.send.send(Work::Goal(request)) { Ok(_) => vec![READY,v], Err(_) => refusal(v,"worker stopped") }
}
pub fn cancel(version: u64) -> Vec<u64> { resident().version.fetch_max(version,Ordering::AcqRel); vec![READY,version] }
pub fn poll() -> Vec<u64> { resident().results.lock().unwrap().try_recv().unwrap_or_else(|_| vec![READY,0]) }
/// Last native observation is retained for P1/P4 capture. Event comparison and
/// continuation repair belong to F; no predicted fields are substituted here.
pub fn observe(event: &[u64],bits: Vec<u64>,known: Vec<u64>,provenance: Vec<u64>) -> Vec<u64> {
    let version = event.get(1).copied().unwrap_or(0);
    let mut saved = vec![READY,version,event.len() as u64]; saved.extend(event); saved.push(bits.len() as u64);
    saved.extend(bits); saved.extend(known); saved.extend(provenance);
    *resident().observation.lock().unwrap() = saved;
    vec![READY,version,crate::qpc().0 as u64]
}
pub fn observation() -> Vec<u64> { resident().observation.lock().unwrap().clone() }

struct ResidentCorridor<'a> {
    source: &'a Source,
    support: BlockPos,
    floor_block: &'a Block,
    air_block: &'static Block,
    air_state: StateId,
    bounds: (BlockPos, BlockPos),
    first_uncovered: std::cell::Cell<Option<BlockPos>>,
    provenance_hash: [u8; 32],
}
impl<'a> ResidentCorridor<'a> {
    fn new(source: &'a Source, support: BlockPos, bounds: (BlockPos, BlockPos)) -> Result<Self, String> {
        let floor_block = source.known_block(support).map_err(|e| format!("support {support:?}: {e}"))?;
        law::codec::legs_presence(floor_block).map_err(|e| format!("support {support:?}: {e:?}"))?;
        let air_state = StateId::parse("minecraft:air", |_| None)?;
        let air_block = air_state.model_block();
        let mut bytes = b"resident-corridor/1".to_vec();
        bytes.extend(law::codec::SCHEMA);
        bytes.extend(support[1].to_le_bytes());
        for b in [floor_block, air_block] {
            bytes.extend(law::codec::legs_presence(b).map_err(|e| format!("legs presence: {e:?}"))?.bytes());
        }
        let mut provenance_hash = [0; 32];
        provenance_hash[..16].copy_from_slice(&law::codec::digest(&bytes).to_le_bytes());
        bytes.push(1);
        provenance_hash[16..].copy_from_slice(&law::codec::digest(&bytes).to_le_bytes());
        Ok(Self {
            source, support, floor_block, air_block, air_state, bounds,
            first_uncovered: std::cell::Cell::new(None), provenance_hash,
        })
    }
    #[inline]
    fn covered(&self, p: BlockPos) -> bool {
        (0..3).all(|a| p[a] >= self.bounds.0[a] && p[a] <= self.bounds.1[a])
    }
    #[inline]
    fn reaches(&self, p: BlockPos) -> bool {
        let inside = self.covered(p);
        if !inside && self.first_uncovered.get().is_none() { self.first_uncovered.set(Some(p)); }
        inside
    }
    #[inline]
    fn base(&self, p: BlockPos) -> &Block {
        if p[1] == self.support[1] { self.floor_block } else { self.air_block }
    }
}
impl World for ResidentCorridor<'_> {
    #[inline]
    fn known_block(&self, p: BlockPos) -> Result<&Block, &str> {
        if !self.reaches(p) { return Err("outside the source"); }
        self.source.known_block(p)
    }
    #[inline]
    fn block(&self, p: BlockPos) -> &Block {
        if !self.covered(p) { return self.base(p); }
        self.source.known_block(p).unwrap_or_else(|_| self.base(p))
    }
    #[inline]
    fn state_id(&self, p: BlockPos) -> Option<StateId> {
        self.covered(p).then(|| self.source.state_id(p)).flatten()
    }
    fn uniform(&self, lo: BlockPos, hi: BlockPos) -> Option<&Block> {
        if !self.covered(lo) || !self.covered(hi) { return None; }
        self.source.uniform(lo, hi)
    }
    #[inline]
    fn loaded(&self, p: BlockPos) -> bool {
        self.covered(p) && self.source.cell(p).is_some_and(Result::is_ok)
    }
    fn difficulty(&self) -> u8 { self.source.difficulty() }
    fn natural_regeneration(&self) -> bool { self.source.natural_regeneration() }
    fn min_y(&self) -> i32 { self.bounds.0[1] }
    fn fast_lava(&self) -> bool { self.source.fast_lava() }
    fn has_fluid(&self) -> bool { self.source.has_fluid() }
    fn has_inside_effects(&self) -> bool { self.source.has_inside_effects() }
    fn has_fluid_in(&self, lo: BlockPos, hi: BlockPos) -> bool { self.source.has_fluid_in(lo, hi) }
    fn has_inside_effects_in(&self, lo: BlockPos, hi: BlockPos) -> bool { self.source.has_inside_effects_in(lo, hi) }
    fn collision_cell_bounds(&self, bounds: law::Aabb, lo: BlockPos, hi: BlockPos) -> Option<(BlockPos, BlockPos)> {
        if !self.covered(lo) || !self.covered(hi) { return Some((lo, hi)); }
        self.source.collision_cell_bounds(bounds, lo, hi)
    }
    fn visit_collision_cells(&self, bounds: law::Aabb, lo: BlockPos, hi: BlockPos, visit: impl FnMut(BlockPos) -> bool) -> bool {
        if !self.covered(lo) || !self.covered(hi) {
            let mut visit = visit;
            for z in lo[2]..=hi[2] {
                for y in lo[1]..=hi[1] {
                    for x in lo[0]..=hi[0] {
                        let p = [x, y, z];
                        if (self.known_block(p).is_err() || !self.block(p).never_collides()) && !visit(p) {
                            return false;
                        }
                    }
                }
            }
            return true;
        }
        self.source.visit_collision_cells(bounds, lo, hi, visit)
    }
}
impl law::codec::proof::Source for ResidentCorridor<'_> {
    fn cell(&self, _tick: u64, p: BlockPos, mask: u8) -> Result<law::codec::Canonical, law::codec::proof::Refusal> {
        let block = self.known_block(p).map_err(|_| law::codec::proof::Refusal::Coverage)?;
        law::codec::cell(block, self.loaded(p), mask).map_err(law::codec::proof::Refusal::Law)
    }
    fn environment(&self, _tick: u64, q: &law::footprint::EnvironmentRead) -> Result<law::codec::Canonical, law::codec::proof::Refusal> {
        law::codec::environment(self, q).map_err(law::codec::proof::Refusal::Law)
    }
}
impl plan::source::CorridorSource for ResidentCorridor<'_> {
    fn indexed_collisions(&self) -> bool { true }
    fn support_cell(&self, s: &PlayerState) -> BlockPos {
        s.main_supporting_block.unwrap_or([s.position.x.floor() as i32, self.support[1], s.position.z.floor() as i32])
    }
    fn floor_y(&self, _: &PlayerState) -> i32 { self.support[1] }
    fn provenance(&self) -> [u8; 32] { self.provenance_hash }
    fn bounds_box(&self) -> Option<(BlockPos, BlockPos)> { Some(self.bounds) }
    fn entry_label(&self, codec: &mut law::codec::Interner, state: &PlayerState) -> Result<law::codec::LabelId, law::codec::proof::Refusal> {
        let feet = [state.position.x.floor() as i32, state.position.y.floor() as i32, state.position.z.floor() as i32];
        let support = state.main_supporting_block.unwrap_or([feet[0], self.support[1], feet[2]]);
        let mut classes = [None; 4];
        for (i, (p, role)) in [
            (support, law::codec::LegsRole::Support02),
            (support, law::codec::LegsRole::Support05),
            (feet, law::codec::LegsRole::BodyFeet),
            (feet, law::codec::LegsRole::Inside),
        ].into_iter().enumerate() {
            let loaded = self.loaded(p);
            if !loaded { return Err(law::codec::proof::Refusal::Loaded); }
            classes[i] = Some(codec.legs(self.block(p), loaded, role).map_err(law::codec::proof::Refusal::Law)?);
        }
        codec.bare_legs_label(classes.map(Option::unwrap)).ok_or(law::codec::proof::Refusal::Schema)
    }
    fn corridor_query(&self, codec: &mut law::codec::Interner, expected: law::codec::LabelId, min: BlockPos, max: BlockPos) -> plan::source::CorridorResult {
        use law::codec::{self, LegsRole, proof::Refusal};
        use plan::source::CorridorResult;
        if !self.reaches(min) || !self.reaches(max) { return CorridorResult::Unknown(Refusal::Loaded); }
        let check_pos = [min[0], self.support[1], min[2]];
        {
            let mut classes = [None; 4];
            for (i, (blk, role)) in [
                (self.floor_block, LegsRole::Support02),
                (self.floor_block, LegsRole::Support05),
                (self.air_block, LegsRole::BodyFeet),
                (self.air_block, LegsRole::Inside),
            ].into_iter().enumerate() {
                match codec.legs(blk, true, role) {
                    Ok(c) => classes[i] = Some(c),
                    Err(e) => return CorridorResult::Unknown(Refusal::Law(e)),
                }
            }
            match codec.bare_legs_label(classes.map(Option::unwrap)) {
                Some(id) if id == expected => (),
                Some(_) => return CorridorResult::Exception { first_pos: check_pos, exceptions: 1 },
                None => return CorridorResult::Unknown(Refusal::Schema),
            }
        }
        let halo_min = min.map(|v| v.saturating_sub(1));
        let halo_max = max.map(|v| v.saturating_add(1));
        if !self.reaches(halo_min) || !self.reaches(halo_max) { return CorridorResult::Unknown(Refusal::Coverage); }
        let inside = |p: BlockPos| (0..3).all(|i| min[i] <= p[i] && p[i] <= max[i]);
        let floor_bytes = match [LegsRole::Support02, LegsRole::Support05, LegsRole::BodyFeet, LegsRole::Inside].into_iter().map(|r| codec::legs_bytes(self.floor_block, true, r)).collect::<Result<Vec<_>, _>>() {
            Ok(v) => v, Err(e) => return CorridorResult::Unknown(Refusal::Law(e)),
        };
        let air_bytes = match [LegsRole::Support02, LegsRole::Support05, LegsRole::BodyFeet, LegsRole::Inside].into_iter().map(|r| codec::legs_bytes(self.air_block, true, r)).collect::<Result<Vec<_>, _>>() {
            Ok(v) => v, Err(e) => return CorridorResult::Unknown(Refusal::Law(e)),
        };
        let mut unknown: Option<BlockPos> = None;
        let mut first: Option<BlockPos> = None;
        let mut exceptions = 0usize;
        let mut refusal = None;
        let roles = [LegsRole::Support02, LegsRole::Support05, LegsRole::BodyFeet, LegsRole::Inside];
        for sz in halo_min[2].div_euclid(16)..=halo_max[2].div_euclid(16) {
            for sy in halo_min[1].div_euclid(16)..=halo_max[1].div_euclid(16) {
                for sx in halo_min[0].div_euclid(16)..=halo_max[0].div_euclid(16) {
                    let base = [sx * 16, sy * 16, sz * 16];
                    let s_lo = [0, 1, 2].map(|i| halo_min[i].max(base[i]));
                    let s_hi = [0, 1, 2].map(|i| halo_max[i].min(base[i] + 15));
                    let Some(section) = self.source.sections.get(&[sx, sy, sz]) else {
                        let count = (0..3).map(|i| (s_hi[i] - s_lo[i] + 1) as usize).product::<usize>();
                        unknown = Some(unknown.map_or(s_lo, |old| old.min(s_lo)));
                        exceptions += count;
                        continue;
                    };
                    if let Some(u) = section.uniform_state {
                        if u == self.air_state && !(s_lo[1]..=s_hi[1]).contains(&self.support[1]) && s_lo[1] >= min[1] {
                            continue;
                        }
                    }
                    // Preclassify fixed-model palette entries vs expected air_bytes & floor_bytes.
                    let air_match: Vec<Option<bool>> = section.palette.iter().map(|(_, entry)| {
                        let Ok(sid) = entry else { return None };
                        if !sid.model().is_some_and(|m| m.fixed()) { return Some(false); }
                        let b = sid.model_block();
                        let same = roles.iter().zip(&air_bytes).try_fold(true, |ok, (&r, ab)| {
                            Ok::<_, law::pattern::PatternError>(ok && codec::legs_bytes(b, true, r)? == *ab)
                        });
                        Some(same.unwrap_or(false))
                    }).collect();
                    let can_skip_air_band = !(s_lo[1]..=s_hi[1]).contains(&self.support[1]) && s_lo[1] >= min[1];
                    if can_skip_air_band && air_match.iter().all(|&m| m == Some(true)) {
                        continue;
                    }
                    for z in s_lo[2]..=s_hi[2] {
                        for y in s_lo[1]..=s_hi[1] {
                            let exp_bytes = if y == self.support[1] { &floor_bytes } else { &air_bytes };
                            let exp_blk = if y == self.support[1] { self.floor_block } else { self.air_block };
                            for x in s_lo[0]..=s_hi[0] {
                                let p = [x, y, z];
                                let idx = section.indices[((y.rem_euclid(16) * 16 + z.rem_euclid(16)) * 16 + x.rem_euclid(16)) as usize];
                                if y != self.support[1] && air_match[idx] == Some(true) {
                                    continue;
                                }
                                let b = match &section.palette[idx].1 {
                                    Ok(sid) => sid.block_at(p),
                                    Err(_) => {
                                        unknown = Some(unknown.map_or(p, |old| old.min(p)));
                                        exceptions += 1;
                                        continue;
                                    }
                                };
                                if !inside(p) {
                                    let bl = std::array::from_fn::<_, 3, _>(|i| f64::from(min[i]) - f64::from(p[i]));
                                    let bh = std::array::from_fn::<_, 3, _>(|i| f64::from(max[i]) + 1.0 - f64::from(p[i]));
                                    let bounds = law::Aabb::new(bl[0], bl[1], bl[2], bh[0], bh[1], bh[2]);
                                    if !b.collision_shapes().chain(exp_blk.collision_shapes()).any(|s| s.intersects(bounds)) {
                                        continue;
                                    }
                                }
                                match roles.iter().zip(exp_bytes).try_fold(false, |diff, (&r, eb)| {
                                    Ok::<_, law::pattern::PatternError>(diff || codec::legs_bytes(b, true, r)? != *eb)
                                }) {
                                    Ok(true) => {
                                        exceptions += 1;
                                        first = Some(first.map_or(p, |old| old.min(p)));
                                    }
                                    Ok(false) => (),
                                    Err(e) => refusal = Some(Refusal::Law(e)),
                                }
                            }
                        }
                    }
                }
            }
        }
        if let Some(r) = refusal { return CorridorResult::Unknown(r); }
        if let Some(first_pos) = unknown.or(first) {
            CorridorResult::Exception { first_pos, exceptions }
        } else {
            CorridorResult::Clear(plan::source::CorridorProof { label: expected, min, max, version: self.provenance_hash })
        }
    }
}

#[cfg_attr(not(test), allow(dead_code))]
fn plan(source: &Source, store: &plan::store::GeneratorStore, codec: &mut law::codec::Interner, request: Request, version: &AtomicU64, revision: &AtomicU64) -> Vec<u64> {
    plan_step(source, store, codec, request, version, revision).0
}

fn find_support_below(source: &Source, p: BlockPos, min_y: i32, current: Option<BlockPos>) -> Result<BlockPos, String> {
    if let Some(s) = current { return Ok(s); }
    for depth in 1..=32 {
        let cell = [p[0], p[1] - depth, p[2]];
        if cell[1] < min_y { break; }
        match source.known_block(cell) {
            Ok(block) if block.never_collides() => (),
            Ok(_) => return Ok(cell),
            Err(reason) => return Err(format!("support {cell:?}: {reason}")),
        }
    }
    Ok([p[0], (p[1] - 1).max(min_y), p[2]])
}

fn plan_hull(words: &[plan::WordStep], fallback_lo: [i32; 2], fallback_hi: [i32; 2]) -> [u64; 4] {
    let mut hx0 = i32::MAX; let mut hz0 = i32::MAX; let mut hx1 = i32::MIN; let mut hz1 = i32::MIN;
    for w in words {
        hx0 = hx0.min(w.certificate.min[0]); hz0 = hz0.min(w.certificate.min[2]);
        hx1 = hx1.max(w.certificate.max[0]); hz1 = hz1.max(w.certificate.max[2]);
    }
    if hx0 > hx1 {
        hx0 = fallback_lo[0]; hz0 = fallback_lo[1];
        hx1 = fallback_hi[0]; hz1 = fallback_hi[1];
    }
    [i64::from(hx0) as u64, i64::from(hz0) as u64, i64::from(hx1) as u64, i64::from(hz1) as u64]
}

#[cfg_attr(not(test), allow(dead_code))]
fn plan_step(source: &Source, store: &plan::store::GeneratorStore, codec: &mut law::codec::Interner, request: Request, version: &AtomicU64, revision: &AtomicU64) -> (Vec<u64>, Option<ActiveContinuation>) {
    let (pkt, cont, _) = plan_step_with_prov(source, store, codec, request, version, revision);
    (pkt, cont)
}

fn plan_step_with_prov(source: &Source, store: &plan::store::GeneratorStore, codec: &mut law::codec::Interner, request: Request, version: &AtomicU64, revision: &AtomicU64) -> (Vec<u64>, Option<ActiveContinuation>, Option<[u8; 32]>) {
    let Request { state, meta, goal: target, issue, version: requested } = request;
    let goal = target.cell;
    let entered = crate::qpc().0 as u64;
    if let Some(why) = &source.error { return (refusal(requested,format!("source publication failed: {why}")), None, None); }
    if meta.len() < 7 || meta[0] != source.session || meta[5] != source.revision { return (refusal(requested,"observation/source session or revision mismatch"), None, None); }
    if meta[6] as usize != meta.len()-7 || meta[6] == 0 || meta[7..].iter().any(|&v|v>255) { return (refusal(requested,"missing/invalid body-kit context"), None, None); }
    if meta[2] != 0 { return (refusal(requested,"unsupported input phase"), None, None); }
    let carried = kit_wire::read(&meta[7..]);
    let Some((lo,hi)) = source.bounds() else { return (refusal(requested,"no received sections"), None, None); };
    let p = [state.position.x.floor() as i32, state.position.y.floor() as i32, state.position.z.floor() as i32];
    let a = match find_support_below(source, p, lo[1], state.main_supporting_block) {
        Ok(s) => s,
        Err(why) => return (refusal(requested, why), None, None),
    };
    if (0..3).any(|i| goal[i] < lo[i] || goal[i] > hi[i]) { return (refusal(requested,"the goal is outside the chunks the bot has received"), None, None); }
    let corridor = match ResidentCorridor::new(source, a, (lo, hi)) { Ok(c) => c, Err(why) => return (refusal(requested, why), None, None) };
    let prov = corridor.provenance_hash;
    let result = plan::solve_step(&state, target, carried.kit, &corridor, store, codec, u64::MAX);
    if version.load(Ordering::Acquire) != requested || revision.load(Ordering::Acquire) != source.law_revision {
        return (refusal(requested,"cancelled or source changed while thinking"), None, Some(prov));
    }
    let (found, eval_goal, cont) = match result.outcome {
        plan::PlanOutcome::Ready(plan) => (plan, target, None),
        plan::PlanOutcome::Prefix(pref) => {
            let next_state = pref.plan.end.materialize();
            let tick_offset = pref.plan.ticks;
            let cont = ActiveContinuation {
                requested,
                support: a,
                final_goal: pref.final_goal,
                carried: carried.clone(),
                state: next_state,
                tick_offset,
                bounds: (lo, hi),
            };
            (pref.plan, pref.target, Some(cont))
        }
        other => return (refusal(requested, format!("{:?} {}", other, result.explanation.unwrap_or_default())), None, Some(prov)),
    };
    let programs = words::tiny_programs();
    match inputs(source, &programs, &state, &found.words, eval_goal, found.ticks, &carried) {
        Ok((inputs, actions)) => {
            let edits = !actions.is_empty();
            let mut out = vec![if edits { KIT_PLAN } else { WORD_PLAN },requested,issue,entered,crate::qpc().0 as u64,source.revision,state.t,inputs.len() as u64 / 3];
            let bits = state.bits(); out.push(bits.len() as u64); out.extend(bits); out.extend(inputs);
            if edits { out.push((actions.len() / kit_wire::ACTION_WORDS) as u64); out.extend(actions); }
            let fallback_lo = [p[0].min(goal[0]).saturating_sub(2).max(lo[0]), p[2].min(goal[2]).saturating_sub(2).max(lo[2])];
            let fallback_hi = [p[0].max(goal[0]).saturating_add(2).min(hi[0]), p[2].max(goal[2]).saturating_add(2).min(hi[2])];
            out.extend(plan_hull(&found.words, fallback_lo, fallback_hi));
            (out, cont, Some(prov))
        }
        Err(why) => (refusal(requested,why), None, Some(prov)),
    }
}

fn advance_continuation(
    source: &Source,
    store: &plan::store::GeneratorStore,
    codec: &mut law::codec::Interner,
    cont: &mut ActiveContinuation,
    version: &AtomicU64,
    revision: &AtomicU64,
) -> Option<(Vec<u64>, bool)> {
    let bounds = source.bounds().unwrap_or(cont.bounds);
    let p = [cont.state.position.x.floor() as i32, cont.state.position.y.floor() as i32, cont.state.position.z.floor() as i32];
    let support = find_support_below(source, p, bounds.0[1], cont.state.main_supporting_block).unwrap_or(cont.support);
    // The next part is planned on the world as it is now. A newer command drops it silently; a
    // part that cannot be planned says why, so the bot never stands still without a word.
    let _ = revision;
    let corridor = match ResidentCorridor::new(source, support, bounds) {
        Ok(c) => c,
        Err(why) => return Some((refusal(cont.requested, format!("next part: {why}")), false)),
    };
    let result = plan::solve_step(&cont.state, cont.final_goal, cont.carried.kit, &corridor, store, codec, u64::MAX);
    if version.load(Ordering::Acquire) != cont.requested {
        return None;
    }
    let (found, eval_goal, more) = match result.outcome {
        plan::PlanOutcome::Ready(plan) => (plan, cont.final_goal, false),
        plan::PlanOutcome::Prefix(pref) => (pref.plan, pref.target, true),
        other => return Some((refusal(cont.requested, format!("next part: {other:?} {}", result.explanation.unwrap_or_default())), false)),
    };
    let programs = words::tiny_programs();
    let (inputs, mut actions) = inputs(source, &programs, &cont.state, &found.words, eval_goal, found.ticks, &cont.carried).ok()?;
    for chunk in actions.chunks_exact_mut(kit_wire::ACTION_WORDS) {
        chunk[0] = chunk[0].saturating_add(cont.tick_offset);
    }
    let edits = !actions.is_empty();
    let mut out = vec![
        if edits { KIT_PLAN_APPEND } else { PLAN_APPEND },
        cont.requested,
        cont.tick_offset,
        inputs.len() as u64 / 3,
    ];
    out.extend(inputs);
    if edits {
        out.push((actions.len() / kit_wire::ACTION_WORDS) as u64);
        out.extend(actions);
    }
    let fallback_lo = [p[0].min(eval_goal.cell[0]).saturating_sub(2).max(bounds.0[0]), p[2].min(eval_goal.cell[2]).saturating_sub(2).max(bounds.0[2])];
    let fallback_hi = [p[0].max(eval_goal.cell[0]).saturating_add(2).min(bounds.1[0]), p[2].max(eval_goal.cell[2]).saturating_add(2).min(bounds.1[2])];
    out.extend(plan_hull(&found.words, fallback_lo, fallback_hi));
    if more {
        cont.state = found.end.materialize();
        cont.tick_offset = cont.tick_offset.saturating_add(found.ticks);
        cont.support = support;
        cont.bounds = bounds;
    }
    Some((out, more))
}
/// The plan's words × counts as tick inputs, each tick's input from the word's program, run through
/// the law with each edit landing where `KitOp::lands_at` puts it; and one action record per edit.
/// A break's ticks hold still and look at the block. With edits, only the last word may touch
/// the goal: the mission ends when its cleanup ends.
fn inputs(source: &Source, programs: &[words::Program], start: &PlayerState, words: &[plan::WordStep], goal: Goal, predicted_ticks: u64, carried: &kit_wire::Carried) -> Result<(Vec<u64>,Vec<u64>),String> {
    fn tick<W: World>(world: &W, program: &words::Program, state: &mut PlayerState, elapsed: u32, args: words::Arguments, still: Option<(f32,f32)>) -> Result<[u64;3],String> {
        let (input, look) = match still {
            Some(look) => (Default::default(), look),
            None => { let (_,i) = program.select(world,state,elapsed,args).ok_or("a word's program has no row here")?; (i, (i.yaw, i.pitch)) }
        };
        let keys = [input.forward,input.back,input.left,input.right,input.jump,input.sneak,input.sprint].iter().enumerate().fold(0,|n,(i,b)| n | ((*b as u64)<<i));
        law::try_tick(state,world,&input).map_err(|e| e.0)?;
        Ok([keys,look.0.to_bits() as u64,look.1.to_bits() as u64])
    }
    let edits = words.iter().any(|w| w.edit.is_some());
    let mut delta = WorldDelta::new(source);
    let mut carried = carried.clone();
    let (mut state, mut inputs, mut actions, mut emptied) = (start.clone(), Vec::new(), Vec::new(), None);
    let mut first_touch = (!edits && goal.reached(start, start)).then_some(0u64);
    for (i, w) in words.iter().enumerate() {
        let counts = !edits || i + 1 == words.len();
        let mut still = None;
        if let Some(op) = w.edit {
            let a = carried.action(&delta, &state, op, state.t - start.t, w.ticks, &mut emptied)?;
            if op.holds_still() { still = Some((f32::from_bits(a[7] as u32), f32::from_bits(a[8] as u32))); }
            actions.extend(a);
        }
        let mut touched = false;
        for elapsed in 0..w.ticks {
            if let Some(op) = w.edit.filter(|op| op.lands_at(w.ticks) == elapsed) { op.apply(&mut delta); }
            let before = state.bounding_box();
            let program = &programs[w.program as usize];
            inputs.extend(if edits { tick(&delta,program,&mut state,elapsed,w.args,still)? } else { tick(source,program,&mut state,elapsed,w.args,None)? });
            touched |= goal.reached_boxes(before, &state);
            if touched && counts && still.is_none() { first_touch.get_or_insert(state.t - start.t); }
        }
        if counts && still.is_some() && (touched || goal.reached(&state, &state)) { first_touch.get_or_insert(state.t - start.t); }
    }
    if predicted_ticks > 0 && first_touch != Some(predicted_ticks) {
        return Err(format!("replay deviates: first touch {:?} predicted {}", first_touch, predicted_ticks));
    }
    Ok((inputs, actions))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn text(out: &mut Vec<u64>,s: &str) { out.push(s.len() as u64); out.extend(s.bytes().map(u64::from)); }
    #[test]
    fn received_palettes_edits_and_unloads_are_the_only_loaded_evidence() {
        let mut source = Source::default(); delta(&mut source,&[0,7,1,0,2]).unwrap();
        assert!(!source.loaded([0,0,0]));
        let mut section = vec![1,7,2,0,0,0,1]; text(&mut section,"minecraft:stone"); section.extend([0;4096]);
        delta(&mut source,&section).unwrap(); assert!(source.loaded([0,0,0])); assert!(!source.block([0,0,0]).air);
        let mut edit = vec![2,7,3,0,0,0]; text(&mut edit,"minecraft:air");
        delta(&mut source,&edit).unwrap(); assert!(source.block([0,0,0]).air); assert!(!source.block([1,0,0]).air);
        assert_eq!(source.sections[&[0,0,0]].palette[1].0,"minecraft:air");
        assert!(delta(&mut source,&edit).is_err());
        delta(&mut source,&[3,7,4,0,0,0]).unwrap(); assert!(!source.loaded([0,0,0])); assert!(source.cell([0,0,0]).is_none());
    }
    #[test]
    fn existing_f_uses_exact_observed_start_and_returns_memory_adapter() {
        let mut source = Source { session:7,revision:1,law_revision:1,difficulty:0,regeneration:Some(false),..Source::default() };
        let air = model("minecraft:air"); let stone = model("minecraft:stone");
        for x in -1..=1 { for z in -1..=1 { for y in -1..=1 {
            let indices = (0..4096).map(|i|usize::from(y<0 || y==0 && i/256==0)).collect();
            source.sections.insert([x,y,z],Section::new(vec![("minecraft:air".into(),air.clone()),("minecraft:stone".into(),stone.clone())], indices));
        } } }
        let state = PlayerState { t:77,position:law::Vec3::new(8.5,1.0,8.5),on_ground:true,main_supporting_block:Some([8,0,8]),food_level:11,saturation:2.5,exhaustion:1.25,..PlayerState::default() };
        let bits = state.bits();
        let request = Request { state,meta:vec![7,1,0,0,0,1,2,123,125],goal:Goal::of([8,0,12]),issue:123,version:3 };
        let answer = plan(&source,&plan::store::GeneratorStore::default(),&mut law::codec::Interner::default(),request,&AtomicU64::new(3),&AtomicU64::new(1));
        assert_eq!(answer[0],WORD_PLAN,"{answer:?}"); assert_eq!(answer[2],123);
        let n = answer[8] as usize; assert_eq!(answer[9..9+n],bits); assert!(answer[7]>0); assert_eq!(answer.len(),9+n+answer[7] as usize*3+4);
    }
    /// A stone floor (top at y = 1) over nine 16³ sections, and a 2-high stone wall at z = `wall`
    /// across every received x when given.
    fn field(wall: Option<i32>) -> Source {
        let mut source = Source { session:7,revision:1,law_revision:1,difficulty:0,regeneration:Some(false),..Source::default() };
        let palette = vec![("minecraft:air".into(),model("minecraft:air")),("minecraft:stone".into(),model("minecraft:stone"))];
        for x in -1..=1 { for z in -1..=1 { for y in -1..=1 {
            let indices = (0..4096).map(|i| { let (ly, lz) = (i/256, i/16%16); usize::from(y<0 || y==0 && (ly==0 || z==0 && Some(lz as i32)==wall && ly<=2)) }).collect();
            source.sections.insert([x,y,z],Section::new(palette.clone(), indices));
        } } }
        source
    }
    fn ask(source: &Source, body: Vec<u64>, goal: BlockPos) -> (Vec<u64>, usize) {
        let state = PlayerState { t:77,position:law::Vec3::new(8.5,1.0,8.5),on_ground:true,main_supporting_block:Some([8,0,8]),food_level:20,saturation:5.0,health:20.0,..PlayerState::default() };
        let mut meta = vec![7,1,0,0,0,1,body.len() as u64]; meta.extend(body);
        let answer = plan(source,&plan::store::GeneratorStore::default(),&mut law::codec::Interner::default(),Request { state,meta,goal:Goal::of(goal),issue:5,version:3 },&AtomicU64::new(3),&AtomicU64::new(1));
        let n = answer.get(8).copied().unwrap_or(0) as usize;
        (answer, n)
    }
    #[test]
    fn source_keeps_the_canonical_state_of_palettes_and_edits() {
        let mut source = Source::default(); delta(&mut source,&[0,7,1,0,2]).unwrap();
        let mut section = vec![1,7,2,0,0,0,3]; text(&mut section,"minecraft:air"); text(&mut section,"minecraft:oak_door[facing=north,half=lower,hinge=left,open=false,powered=false]"); text(&mut section,"minecraft:stone");
        section.extend((0..4096).map(|i| [1,2].get(i).copied().unwrap_or(0)));
        delta(&mut source,&section).unwrap();
        let door = StateId::parse("minecraft:oak_door",|p| match p { "facing"=>Some("north"),"half"=>Some("lower"),"hinge"=>Some("left"),"open"=>Some("false"),"powered"=>Some("false"),_=>None }).unwrap();
        assert_eq!(source.state_id([0,0,0]),Some(door)); assert_eq!(source.state_id([2,0,0]),StateId::parse("minecraft:air",|_| None).ok());
        let mut edit = vec![2,7,3,0,0,0]; text(&mut edit,"minecraft:oak_door[facing=north,half=lower,hinge=left,open=true,powered=false]");
        delta(&mut source,&edit).unwrap();
        assert_eq!(source.state_id([0,0,0]),plan::kit::toggle_open_state(door)); assert!(source.state_id([0,0,16]).is_none());
        let gs = plan::source::GeneratorSuperflatSource::from_world(&source,([0,0,0],[15,15,15]),[1,0,0]);
        assert_eq!(gs.unwrap().state_id([0,0,0]),plan::kit::toggle_open_state(door),"the corridor sees the live state");
    }
    #[test]
    fn carried_kit_on_flat_ground_sends_the_bare_word_plan() {
        let source = field(None);
        let (bare, n) = ask(&source, vec![123,125], [8,0,24]);
        let hotbar = [crate::kit_wire::tests::PICK, "{count:64,id:\"minecraft:stone\"}", "{}", "{}", "{}", "{}", "{}", "{}", "{}"];
        let (kit, _) = ask(&source, crate::kit_wire::tests::body("minecraft:overworld","survival",hotbar,"{}",0), [8,0,24]);
        assert_eq!(bare[0],WORD_PLAN,"{bare:?}"); assert_eq!(kit[0],WORD_PLAN,"{kit:?}");
        assert_eq!(bare[5..],kit[5..],"identical words, inputs and hull"); assert_eq!(bare.len(),9+n+bare[7] as usize*3+4);
    }
    #[test]
    fn walled_field_sends_kit_plan_with_one_click_or_hold_per_edit() {
        let source = field(Some(12));
        let hotbar = [crate::kit_wire::tests::PICK, "{count:64,id:\"minecraft:stone\"}", "{}", "{}", "{}", "{}", "{}", "{}", "{}"];
        let (bare, _) = ask(&source, vec![123,125], [8,0,14]);
        assert_eq!(bare[0],REFUSAL,"bare legs cannot cross a 2-high wall");
        let (a, n) = ask(&source, crate::kit_wire::tests::body("minecraft:overworld","survival",hotbar,"{}",0), [8,0,14]);
        assert_eq!(a[0],KIT_PLAN,"{}",String::from_utf8_lossy(&a[4..].iter().map(|&b| b as u8).collect::<Vec<_>>()));
        let count = a[7] as usize; let at = 9+n+count*3; let k = a[at] as usize;
        assert_eq!(a.len(),at+1+k*crate::kit_wire::ACTION_WORDS+4,"layout: header, bits, inputs, actions, hull");
        assert!(k >= 2,"an edit and its restore");
        println!("NATIVE_KIT_PLAN wall=2-high T={count} actions={k} words={}",a.len());
        let mut world = std::collections::HashMap::new();
        for r in a[at+1..at+1+k*crate::kit_wire::ACTION_WORDS].chunks(crate::kit_wire::ACTION_WORDS) {
            let (tick, slot, op, face) = (r[0] as usize, r[1], r[2], r[6] as usize);
            let pos = [r[3] as i64 as i32, r[4] as i64 as i32, r[5] as i64 as i32];
            assert!(tick < count && face < 6, "{r:?}");
            let input = &a[9+n+tick*3..9+n+tick*3+3];
            match op {
                crate::kit_wire::OP_BREAK_HOLD => {
                    assert_eq!(slot,0,"stone breaks with the pickaxe"); assert!(r[9] >= 1);
                    for t in tick..tick+r[9] as usize { assert_eq!(a[9+n+t*3],0,"a break holds still"); assert_eq!(a[9+n+t*3+1],r[7],"and looks at its block"); }
                    *world.entry(pos).or_insert(0) -= 1;
                }
                crate::kit_wire::OP_PLACE => { assert_eq!((slot, r[9]),(1,1),"stone places from its slot in one click"); let _ = input; *world.entry(pos).or_insert(0) += 1; }
                other => panic!("unexpected opcode {other}"),
            }
        }
        assert!(world.values().all(|&v| v == 0),"every edited cell is restored: {world:?}");
    }
    #[test]
    fn pressure_plate_toggle_keeps_law_while_door_and_break_change_it() {
        assert!(law_same("minecraft:oak_pressure_plate[powered=false]", "minecraft:oak_pressure_plate[powered=true]"));
        assert!(law_same("minecraft:light_weighted_pressure_plate[power=0]", "minecraft:light_weighted_pressure_plate[power=3]"));
        assert!(law_same("minecraft:oak_button[face=floor,facing=north,powered=false]", "minecraft:oak_button[face=floor,facing=north,powered=true]"));
        assert!(!law_same("minecraft:oak_door[facing=north,half=lower,hinge=left,open=false,powered=false]", "minecraft:oak_door[facing=north,half=lower,hinge=left,open=true,powered=false]"));
        assert!(!law_same("minecraft:sticky_piston[extended=false,facing=up]", "minecraft:sticky_piston[extended=true,facing=up]"));
        assert!(!law_same("minecraft:oak_pressure_plate[powered=false]", "minecraft:air"));

        let mut source = Source::default();
        delta(&mut source, &[0, 7, 1, 0, 2]).unwrap();
        let mut section = vec![1, 7, 2, 0, 0, 0, 2];
        text(&mut section, "minecraft:air");
        text(&mut section, "minecraft:oak_pressure_plate[powered=false]");
        section.extend((0..4096).map(|i| u64::from(i == 0)));
        delta(&mut source, &section).unwrap();

        let mut flip = vec![2, 7, 3, 0, 0, 0];
        text(&mut flip, "minecraft:oak_pressure_plate[powered=true]");
        text(&mut flip, "minecraft:oak_pressure_plate[powered=false]");
        assert!(!block_delta_changes_law(&flip));
        delta(&mut source, &flip).unwrap();
        assert_eq!(source.state_id([0, 0, 0]), StateId::parse("minecraft:oak_pressure_plate", |p| (p == "powered").then_some("true")).ok());

        let mut without_before = vec![2, 7, 4, 0, 0, 0];
        text(&mut without_before, "minecraft:oak_pressure_plate[powered=false]");
        assert!(block_delta_changes_law(&without_before));

        let mut door_flip = vec![2, 7, 5, 0, 0, 0];
        text(&mut door_flip, "minecraft:oak_door[facing=north,half=lower,hinge=left,open=true,powered=false]");
        text(&mut door_flip, "minecraft:oak_door[facing=north,half=lower,hinge=left,open=false,powered=false]");
        assert!(block_delta_changes_law(&door_flip));
    }
    #[test]
    fn water_start_cherry_barrier_and_settle_arrival_all_solve() {
        let mut source = Source { session: 7, revision: 1, law_revision: 1, difficulty: 0, regeneration: Some(false), ..Source::default() };
        let palette = vec![
            ("minecraft:air".into(), model("minecraft:air")),
            ("minecraft:stone".into(), model("minecraft:stone")),
            ("minecraft:water[level=0]".into(), model("minecraft:water[level=0]")),
            ("minecraft:cherry_fence[east=false,north=false,south=false,waterlogged=false,west=false]".into(),
                model("minecraft:cherry_fence[east=false,north=false,south=false,waterlogged=false,west=false]")),
        ];
        for x in -1..=1 { for z in -1..=1 { for y in -1..=1 {
            let indices = (0..4096).map(|i| {
                let (lx, ly, lz) = (i % 16, i / 256, i / 16 % 16);
                let wx = x * 16 + lx as i32; let wz = z * 16 + lz as i32;
                if y < 0 || (y == 0 && ly == 0) { 1 }
                else if y == 0 && ly == 1 && (7..=9).contains(&wx) && (7..=9).contains(&wz) { 2 }
                else if y == 0 && ly == 1 && (6..=10).contains(&wx) && wz == 12 { 3 }
                else { 0 }
            }).collect();
            source.sections.insert([x, y, z], Section::new(palette.clone(), indices));
        }}}
        let hotbar = ["{}"; 9];
        let body = crate::kit_wire::tests::body("minecraft:overworld", "survival", hotbar, "{}", 0);
        let mut meta = vec![7, 1, 0, 0, 0, 1, body.len() as u64]; meta.extend(body);

        // (a) Starting inside water + (d) un-restorable cherry_fence barrier -> routes around fence.
        let water_start = PlayerState {
            t: 77, position: law::Vec3::new(8.5, 1.0, 8.5), in_water: true, water_height: 0.9,
            food_level: 20, saturation: 5.0, health: 20.0, ..PlayerState::default()
        };
        let mut water_req = plan(&source, &plan::store::GeneratorStore::default(), &mut law::codec::Interner::default(),
            Request { state: water_start, meta: meta.clone(), goal: Goal::of([8, 0, 14]), issue: 5, version: 3 },
            &AtomicU64::new(3), &AtomicU64::new(1));
        assert!(water_req[0] == WORD_PLAN || water_req[0] == KIT_PLAN, "{}", String::from_utf8_lossy(&water_req[4..].iter().map(|&b| b as u8).collect::<Vec<_>>()));

        // (c) settle = true slows down cleanly on goal.
        let land_start = PlayerState {
            t: 77, position: law::Vec3::new(8.5, 1.0, 4.5), on_ground: true, main_supporting_block: Some([8, 0, 4]),
            food_level: 20, saturation: 5.0, health: 20.0, ..PlayerState::default()
        };
        water_req = plan(&source, &plan::store::GeneratorStore::default(), &mut law::codec::Interner::default(),
            Request { state: land_start, meta, goal: Goal { cell: [8, 0, 6], face: Some(plan::Face::Up), settle: true }, issue: 6, version: 3 },
            &AtomicU64::new(3), &AtomicU64::new(1));
        assert_eq!(water_req[0], WORD_PLAN, "{}", String::from_utf8_lossy(&water_req[4..].iter().map(|&b| b as u8).collect::<Vec<_>>()));

        // (d) Out-of-section delta does not poison source.
        let mut oos = vec![2, 7, 2, 1000, 0, 1000];
        text(&mut oos, "minecraft:stone");
        delta(&mut source, &oos).unwrap();
        assert!(source.error.is_none());
        assert_eq!(source.revision, 2);
    }

    #[test]
    fn long_corridor_streams_prefix_and_append_without_from_world_copy() {
        let mut source = Source { session: 7, revision: 1, law_revision: 1, difficulty: 0, regeneration: Some(false), ..Source::default() };
        let palette = vec![
            ("minecraft:air".into(), model("minecraft:air")),
            ("minecraft:stone".into(), model("minecraft:stone")),
            ("minecraft:oak_fence[east=false,north=false,south=false,waterlogged=false,west=false]".into(),
                model("minecraft:oak_fence[east=false,north=false,south=false,waterlogged=false,west=false]")),
        ];
        // Build 96-block corridor along +x with a narrow 3-wide fence wall every 24 blocks so defects occur past 26 blocks.
        for sx in 0..=6 {
            for sy in -1..=1 {
                for sz in 0..=1 {
                    let indices = (0..4096).map(|i| {
                        let (lx, ly, lz) = (i % 16, i / 256, (i / 16) % 16);
                        let wx = sx * 16 + lx as i32;
                        let wz = sz * 16 + lz as i32;
                        if sy < 0 || (sy == 0 && ly == 0) { 1 }
                        else if sy == 0 && (ly == 1 || ly == 2) && (wx == 24 || wx == 56) && (14..=18).contains(&wz) { 2 }
                        else { 0 }
                    }).collect();
                    source.sections.insert([sx, sy, sz], Section::new(palette.clone(), indices));
                }
            }
        }
        let state = PlayerState {
            t: 100, position: law::Vec3::new(4.5, 1.0, 16.5), on_ground: true,
            main_supporting_block: Some([4, 0, 16]), food_level: 20, saturation: 5.0, health: 20.0, ..PlayerState::default()
        };
        let target = Goal { cell: [72, 0, 16], face: Some(plan::Face::Up), settle: false };
        let body = crate::kit_wire::tests::body("minecraft:overworld", "survival", ["{}"; 9], "{}", 0);
        let mut meta = vec![7, 1, 0, 0, 0, 1, body.len() as u64];
        meta.extend(body);
        let store = plan::store::GeneratorStore::default();
        let mut codec = law::codec::Interner::default();
        let ver = AtomicU64::new(3);
        let rev = AtomicU64::new(1);

        let (pkt1, mut cont_opt) = plan_step(&source, &store, &mut codec, Request { state: state.clone(), meta, goal: target, issue: 9, version: 3 }, &ver, &rev);
        assert_eq!(pkt1[0], WORD_PLAN, "{}", String::from_utf8_lossy(&pkt1[4..].iter().map(|&b| b as u8).collect::<Vec<_>>()));
        let n_bits = pkt1[8] as usize;
        let count1 = pkt1[7] as usize;
        let mut all_inputs: Vec<[u64; 3]> = pkt1[9 + n_bits..9 + n_bits + count1 * 3]
            .chunks_exact(3).map(|c| [c[0], c[1], c[2]]).collect();
        let mut cont = cont_opt.take().expect("long defect corridor should stream Prefix continuation");
        assert_eq!(cont.tick_offset as usize, all_inputs.len());

        for _ in 0..16 {
            let (app, more) = advance_continuation(&source, &store, &mut codec, &mut cont, &ver, &rev).expect("continuation packet");
            assert_eq!(app[0], PLAN_APPEND);
            assert_eq!(app[2] as usize, all_inputs.len());
            let mc = app[3] as usize;
            all_inputs.extend(app[4..4 + mc * 3].chunks_exact(3).map(|c| [c[0], c[1], c[2]]));
            if !more { break; }
        }
        assert!(all_inputs.len() > count1);

        let mut sim = state;
        let mut hit = None;
        for (idx, inp_words) in all_inputs.iter().enumerate() {
            let before = sim.bounding_box();
            let k = inp_words[0];
            let inp = law::Input {
                forward: (k & 1) != 0, back: (k & 2) != 0, left: (k & 4) != 0, right: (k & 8) != 0,
                jump: (k & 16) != 0, sneak: (k & 32) != 0, sprint: (k & 64) != 0,
                yaw: f32::from_bits(inp_words[1] as u32), pitch: f32::from_bits(inp_words[2] as u32),
            };
            law::try_tick(&mut sim, &source, &inp).unwrap();
            if target.reached_boxes(before, &sim) { hit.get_or_insert(idx + 1); }
        }
        assert_eq!(hit, Some(all_inputs.len()));
    }

    #[test]
    fn idle_consolidate_and_far_section_prewarm_respect_invalidation() {
        let source = field(None);
        let store = plan::store::GeneratorStore::default();
        let prov = ResidentCorridor::new(&source, [0, 0, 0], ([-16, -16, -16], [31, 31, 31])).unwrap().provenance_hash;
        assert_eq!(store.cached_far_sections(), 0);
        assert!(store.warm_far_section(prov, &source, [0, 0, 0]));
        assert!(store.warm_far_section(prov, &source, [1, 0, 0]));
        assert!(!store.warm_far_section(prov, &source, [0, 0, 0]), "duplicate prewarm must be a no-op");
        assert_eq!(store.cached_far_sections(), 2);

        store.invalidate_far_section([0, 0, 0]);
        assert_eq!(store.cached_far_sections(), 1);
        assert_eq!(store.consolidate_idle(), 0);
    }

    #[test]
    fn empty_hotbar_planks_wall_emits_break_pickup_delay_and_place() {
        let mut source = Source { session: 7, revision: 1, law_revision: 1, difficulty: 0, regeneration: Some(false), ..Source::default() };
        let palette = vec![
            ("minecraft:air".into(), model("minecraft:air")),
            ("minecraft:stone".into(), model("minecraft:stone")),
            ("minecraft:oak_planks".into(), model("minecraft:oak_planks")),
        ];
        for x in -1..=1 { for z in -1..=1 { for y in -1..=1 {
            let indices = (0..4096).map(|i| {
                let (ly, lz) = (i / 256, i / 16 % 16);
                if y < 0 || (y == 0 && ly == 0) { 1 }
                else if y == 0 && z == 0 && lz == 12 && (1..=2).contains(&ly) { 2 }
                else { 0 }
            }).collect();
            source.sections.insert([x, y, z], Section::new(palette.clone(), indices));
        }}}
        // 4 non-block items + 5 free hotbar slots (same shape as ametwoodbox.kit.json).
        let hotbar = [
            "{count:1,id:\"minecraft:turtle_helmet\"}",
            "{count:1,id:\"minecraft:golden_chestplate\"}",
            "{count:1,id:\"minecraft:diamond_leggings\"}",
            "{count:1,id:\"minecraft:iron_boots\"}",
            "{}", "{}", "{}", "{}", "{}",
        ];
        let (a, n) = ask(&source, crate::kit_wire::tests::body("minecraft:overworld", "survival", hotbar, "{}", 0), [8, 0, 14]);
        assert_eq!(a[0], KIT_PLAN, "{}", String::from_utf8_lossy(&a[4..].iter().map(|&b| b as u8).collect::<Vec<_>>()));
        let count = a[7] as usize;
        let at = 9 + n + count * 3;
        let k = a[at] as usize;
        assert!(k >= 2, "expected break + restore place, got {k}");
        let mut break_ends: std::collections::HashMap<BlockPos, u64> = std::collections::HashMap::new();
        let mut places = 0usize;
        for r in a[at + 1..at + 1 + k * crate::kit_wire::ACTION_WORDS].chunks(crate::kit_wire::ACTION_WORDS) {
            let (tick, slot, op, hold) = (r[0], r[1], r[2], r[9]);
            let pos = [r[3] as i64 as i32, r[4] as i64 as i32, r[5] as i64 as i32];
            match op {
                crate::kit_wire::OP_BREAK_HOLD => {
                    assert_eq!(slot, 4, "bare hand picks first free hotbar slot");
                    break_ends.insert(pos, tick + hold);
                }
                crate::kit_wire::OP_PLACE => {
                    assert_eq!(slot, 4, "restored drop landed in first free hotbar slot");
                    let bend = *break_ends.get(&pos).expect("break before restore");
                    assert!(tick >= bend + law::items::DEFAULT_PICKUP_DELAY as u64, "place tick {tick} < ready {}", bend + 10);
                    places += 1;
                }
                other => panic!("unexpected opcode {other}"),
            }
        }
        assert_eq!(places, break_ends.len(), "every broken planks cell is restored");
    }
}
