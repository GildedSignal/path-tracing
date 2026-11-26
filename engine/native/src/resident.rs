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
struct Section { palette: Vec<(String, Result<StateId, String>)>, indices: Vec<usize> }
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
    fn loaded(&self, p: BlockPos) -> bool { self.sections.contains_key(&p.map(|v| v.div_euclid(16))) }
    fn difficulty(&self) -> u8 { self.difficulty }
    // The rule is not sent to a remote client: projected as vanilla's default; it changes health, never a walk, and the server corrects it.
    fn natural_regeneration(&self) -> bool { self.regeneration.unwrap_or(true) }
    fn has_fluid(&self) -> bool { true }
    fn has_inside_effects(&self) -> bool { true }
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
                source.sections.insert(p, Section { palette, indices });
            }
            2 => {
                let name = c.text()?;
                if c.at < data.len() { let _ = c.text()?; }
                if c.at != data.len() { return Err("trailing delta words".into()); }
                if let Some(section) = source.sections.get_mut(&p.map(|v| v.div_euclid(16))) {
                    let id = match section.palette.iter().position(|(n,_)| n == &name) { Some(i) => i, None => { let answer = model(&name); section.palette.push((name,answer)); section.palette.len()-1 } };
                    let i = ((p[1].rem_euclid(16)*16+p[2].rem_euclid(16))*16+p[0].rem_euclid(16)) as usize;
                    section.indices[i] = id;
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
enum Work { Delta(Vec<u64>, u64, bool), Goal(Request) }
struct Resident { send: mpsc::Sender<Work>, results: Mutex<mpsc::Receiver<Vec<u64>>>, version: Arc<AtomicU64>, law_revision: Arc<AtomicU64>, observation: Mutex<Vec<u64>> }
fn resident() -> &'static Resident {
    static RESIDENT: OnceLock<Resident> = OnceLock::new();
    RESIDENT.get_or_init(|| {
        let (send, recv) = mpsc::channel(); let (done, results) = mpsc::channel();
        let version = Arc::new(AtomicU64::new(0)); let law_revision = Arc::new(AtomicU64::new(0));
        let (v, r) = (version.clone(), law_revision.clone());
        std::thread::Builder::new().name("pathtracing-F".into()).spawn(move || {
            // One living store for the bot's whole session: a gait learned once is never relearned (S230).
            let mut source = Source::default(); let mut store = plan::store::GeneratorStore::default(); let mut codec = law::codec::Interner::default();
            while let Ok(work) = recv.recv() {
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
                        } else if kind == 4 || (kind == 0 && !reset) {
                            store.words.borrow_mut().environment_event();
                        } else if store.words.borrow().has_certificates() {
                            if let Some(cell) = p {
                                if kind == 2 {
                                    if law_changed { store.words.borrow_mut().edit(&mut codec, cell, &source); }
                                } else {
                                    store.words.borrow_mut().edit_section(&mut codec, cell, &source);
                                }
                            }
                        }
                    }
                    Work::Goal(request) => {
                        let version = request.version;
                        let answer = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| plan(&source, &store, &mut codec, request, &v, &r)))
                            .unwrap_or_else(|p| refusal(version,format!("planner panic: {}",p.downcast_ref::<String>().cloned().or_else(|| p.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default())));
                        let _ = done.send(answer);
                    }
                }
            }
        }).expect("resident worker");
        Resident { send, results: Mutex::new(results), version, law_revision, observation: Mutex::new(Vec::new()) }
    })
}
pub fn publish(data: Vec<u64>) -> Vec<u64> {
    if data.len() < 3 { return refusal(0,"delta header"); }
    let r = resident();
    let law_changed = data[0] != 2 || block_delta_changes_law(&data);
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

fn plan(source: &Source, store: &plan::store::GeneratorStore, codec: &mut law::codec::Interner, request: Request, version: &AtomicU64, revision: &AtomicU64) -> Vec<u64> {
    let Request { state, meta, goal: target, issue, version: requested } = request;
    let goal = target.cell;
    let entered = crate::qpc().0 as u64;
    if let Some(why) = &source.error { return refusal(requested,format!("source publication failed: {why}")); }
    if meta.len() < 7 || meta[0] != source.session || meta[5] != source.revision { return refusal(requested,"observation/source session or revision mismatch"); }
    if meta[6] as usize != meta.len()-7 || meta[6] == 0 || meta[7..].iter().any(|&v|v>255) { return refusal(requested,"missing/invalid body-kit context"); }
    if meta[2] != 0 { return refusal(requested,"unsupported input phase"); }
    let carried = kit_wire::read(&meta[7..]);
    let Some((lo,hi)) = source.bounds() else { return refusal(requested,"no received sections"); };
    // The corridor between the bot and the goal, from the bottom of the world to a little above both.
    let p = [state.position.x.floor() as i32, state.position.y.floor() as i32, state.position.z.floor() as i32];
    // The ground: the block it stands on, or mid-jump/in-water the first solid block below it.
    let mut support = state.main_supporting_block;
    if support.is_none() { for depth in 1..=32 {
        let cell = [p[0], p[1] - depth, p[2]];
        if cell[1] < lo[1] { break; }
        match source.known_block(cell) {
            Ok(block) if block.never_collides() => (),
            Ok(_) => { support = Some(cell); break; }
            Err(reason) => return refusal(requested, format!("support {cell:?}: {reason}")),
        }
    }}
    let a = support.unwrap_or([p[0], (p[1] - 1).max(lo[1]), p[2]]);
    let near = [p[0].min(goal[0])-4, lo[1], p[2].min(goal[2])-4]; let far = [p[0].max(goal[0])+4, p[1].max(goal[1])+8, p[2].max(goal[2])+4];
    let (mut cl, mut ch) = ([near[0].max(lo[0]),lo[1],near[2].max(lo[2])], [far[0].min(hi[0]),far[1].min(hi[1]),far[2].min(hi[2])]);
    if (0..3).any(|i| goal[i] < cl[i] || goal[i] > ch[i]) { return refusal(requested,"the goal is outside the chunks the bot has received"); }
    let mut found = None;
    for _ in 0..8 {
        let corridor = match plan::source::GeneratorSuperflatSource::from_world(source,(cl,ch),a) { Ok(c) => c, Err(why) => return refusal(requested,why) };
        let result = plan::solve(&state, target, carried.kit, &corridor, store, codec, u64::MAX);
        if version.load(Ordering::Acquire) != requested || revision.load(Ordering::Acquire) != source.law_revision { return refusal(requested,"cancelled or source changed while thinking"); }
        match result.outcome {
            plan::PlanOutcome::Ready(plan) => { found = Some(plan); break; }
            other => {
                let prev = (cl, ch);
                if let Some(unc) = corridor.first_uncovered() {
                    for i in [0, 2] {
                        cl[i] = cl[i].min(unc[i] - 4).max(lo[i]);
                        ch[i] = ch[i].max(unc[i] + 4).min(hi[i]);
                    }
                    if (cl, ch) != prev { continue; }
                }
                if matches!(other, plan::PlanOutcome::NeedsJoints { .. }) {
                    cl[0] = cl[0].min(p[0].min(goal[0]) - 20).max(lo[0]);
                    cl[2] = cl[2].min(p[2].min(goal[2]) - 20).max(lo[2]);
                    ch[0] = ch[0].max(p[0].max(goal[0]) + 20).min(hi[0]);
                    ch[2] = ch[2].max(p[2].max(goal[2]) + 20).min(hi[2]);
                    if (cl, ch) != prev { continue; }
                }
                return refusal(requested,format!("{:?} {}",other,result.explanation.unwrap_or_default()));
            }
        }
    }
    let Some(found) = found else { return refusal(requested,"corridor growth exceeded pass cap"); };
    let programs = words::tiny_programs();
    match inputs(source, &programs, &state, &found.words, target, found.ticks, &carried) {
        Ok((inputs, actions)) => {
            let edits = !actions.is_empty();
            let mut out = vec![if edits { KIT_PLAN } else { WORD_PLAN },requested,issue,entered,crate::qpc().0 as u64,source.revision,state.t,inputs.len() as u64 / 3];
            let bits = state.bits(); out.push(bits.len() as u64); out.extend(bits); out.extend(inputs);
            if edits { out.push((actions.len() / kit_wire::ACTION_WORDS) as u64); out.extend(actions); }
            let mut hx0 = i32::MAX; let mut hz0 = i32::MAX; let mut hx1 = i32::MIN; let mut hz1 = i32::MIN;
            for w in &found.words {
                hx0 = hx0.min(w.certificate.min[0]); hz0 = hz0.min(w.certificate.min[2]);
                hx1 = hx1.max(w.certificate.max[0]); hz1 = hz1.max(w.certificate.max[2]);
            }
            if hx0 > hx1 { hx0 = cl[0]; hz0 = cl[2]; hx1 = ch[0]; hz1 = ch[2]; }
            out.extend([i64::from(hx0) as u64, i64::from(hz0) as u64, i64::from(hx1) as u64, i64::from(hz1) as u64]);
            out
        }
        Err(why) => refusal(requested,why),
    }
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
            source.sections.insert([x,y,z],Section { palette:vec![("minecraft:air".into(),air.clone()),("minecraft:stone".into(),stone.clone())],indices });
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
            source.sections.insert([x,y,z],Section { palette:palette.clone(),indices });
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
            source.sections.insert([x, y, z], Section { palette: palette.clone(), indices });
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
}
