//! R0 worker. Existing F search only; no zone/store/search implementation here.
use std::{collections::BTreeMap, sync::{Arc, Mutex, OnceLock, atomic::{AtomicU64, Ordering}, mpsc}};
use law::{Block, BlockPos, PlayerState, World};
use plan::Goal;

pub const READY: u64 = 0;
pub const WORD_PLAN: u64 = 1;
pub const HARD: u64 = 2;
pub const REFUSAL: u64 = 3;
pub fn refusal(version: u64, reason: impl AsRef<str>) -> Vec<u64> {
    let at = crate::qpc().0 as u64;
    let mut out = vec![REFUSAL, version, at, at];
    out.extend(reason.as_ref().bytes().map(u64::from)); out
}

#[derive(Default)]
struct Source {
    session: u64,
    revision: u64,
    difficulty: u8,
    regeneration: Option<bool>,
    // One versioned palette per received section; fine state bytes are retained.
    sections: BTreeMap<BlockPos, Section>,
    error: Option<String>,
}
struct Section { palette: Vec<(String, Result<&'static Block, String>)>, indices: Vec<usize> }
impl Source {
    fn cell(&self, p: BlockPos) -> Option<&Result<&'static Block, String>> {
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
        self.cell(p).ok_or("section not received")?.as_deref().map_err(String::as_str)
    }
    fn block(&self, p: BlockPos) -> &Block {
        match self.cell(p) {
            Some(Ok(block)) => block,
            Some(Err(why)) => panic!("source refuses {p:?}: {why}"),
            None => panic!("unknown source cell {p:?}"),
        }
    }
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
fn model(text: &str) -> Result<&'static Block,String> {
    let (name, tail) = text.split_once('[').unwrap_or((text,""));
    let mut properties = Vec::new();
    for pair in tail.trim_end_matches(']').split(',').filter(|p| !p.is_empty()) {
        let (k,v) = pair.split_once('=').ok_or("block property")?;
        properties.push((k.to_owned(),v.to_owned()));
    }
    world::model(&world::BlockState { name: name.to_owned(), properties })
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
                if c.at != data.len() { return Err("trailing delta words".into()); }
                let section = source.sections.get_mut(&p.map(|v| v.div_euclid(16))).ok_or("block delta outside received section")?;
                let id = match section.palette.iter().position(|(n,_)| n == &name) { Some(i) => i, None => { let answer = model(&name); section.palette.push((name,answer)); section.palette.len()-1 } };
                let i = ((p[1].rem_euclid(16)*16+p[2].rem_euclid(16))*16+p[0].rem_euclid(16)) as usize;
                section.indices[i] = id;
            }
            3 => { if c.at != data.len() { return Err("trailing delta words".into()); } source.sections.remove(&p); }
            _ => return Err("delta kind".into()),
        }
        source.revision = revision;
    }
    if c.at != data.len() { return Err("trailing delta words".into()); } Ok(())
}

pub struct Request { pub state: PlayerState, pub meta: Vec<u64>, pub goal: BlockPos, pub issue: u64, pub version: u64 }
enum Work { Delta(Vec<u64>), Goal(Request) }
struct Resident { send: mpsc::Sender<Work>, results: Mutex<mpsc::Receiver<Vec<u64>>>, version: Arc<AtomicU64>, source_revision: Arc<AtomicU64>, observation: Mutex<Vec<u64>> }
fn resident() -> &'static Resident {
    static RESIDENT: OnceLock<Resident> = OnceLock::new();
    RESIDENT.get_or_init(|| {
        let (send, recv) = mpsc::channel(); let (done, results) = mpsc::channel();
        let version = Arc::new(AtomicU64::new(0)); let source_revision = Arc::new(AtomicU64::new(0));
        let (v, r) = (version.clone(), source_revision.clone());
        std::thread::Builder::new().name("pathtracing-F".into()).spawn(move || {
            // One living store for the bot's whole session: a gait learned once is never relearned (S230).
            let mut source = Source::default(); let mut store = plan::store::GeneratorStore::default(); let mut codec = law::codec::Interner::default();
            while let Ok(work) = recv.recv() {
                match work {
                    Work::Delta(data) => {
                        let reset = data.first() == Some(&0);
                        if let Err(why) = delta(&mut source, &data) { source.error = Some(why.clone()); let _ = done.send(refusal(0,why)); }
                        if reset { store = plan::store::GeneratorStore::default(); codec = law::codec::Interner::default(); }
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
        Resident { send, results: Mutex::new(results), version, source_revision, observation: Mutex::new(Vec::new()) }
    })
}
pub fn publish(data: Vec<u64>) -> Vec<u64> {
    if data.len() < 3 { return refusal(0,"delta header"); }
    let r = resident(); r.source_revision.store(data[2],Ordering::Release);
    if data[0] == 0 { r.observation.lock().unwrap().clear(); }
    match r.send.send(Work::Delta(data)) { Ok(_) => vec![READY,0], Err(_) => refusal(0,"worker stopped") }
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
    let Request { state, meta, goal, issue, version: requested } = request;
    let entered = crate::qpc().0 as u64;
    if let Some(why) = &source.error { return refusal(requested,format!("source publication failed: {why}")); }
    if meta.len() < 7 || meta[0] != source.session || meta[5] != source.revision { return refusal(requested,"observation/source session or revision mismatch"); }
    if meta[6] as usize != meta.len()-7 || meta[6] == 0 || meta[7..].iter().any(|&v|v>255) { return refusal(requested,"missing/invalid body-kit context"); }
    if meta[2] != 0 { return refusal(requested,"unsupported input phase"); }
    let Some((lo,hi)) = source.bounds() else { return refusal(requested,"no received sections"); };
    // The corridor between the bot and the goal, from the bottom of the world to a little above both.
    let p = [state.position.x.floor() as i32, state.position.y.floor() as i32, state.position.z.floor() as i32];
    // The ground: the block it stands on, or mid-jump the first solid block below it.
    let mut support = state.main_supporting_block;
    if support.is_none() { for depth in 1..=8 {
        let cell=[p[0],p[1]-depth,p[2]];
        match source.known_block(cell) {Ok(block) if block.air=>(),Ok(_)=>{support=Some(cell);break;},Err(reason)=>return refusal(requested,format!("support {cell:?}: {reason}"))}
    }}
    let Some(a) = support else { return refusal(requested,"no ground under the bot within 8 blocks"); };
    let near = [p[0].min(goal[0])-4, lo[1], p[2].min(goal[2])-4]; let far = [p[0].max(goal[0])+4, p[1].max(goal[1])+8, p[2].max(goal[2])+4];
    let (cl, ch) = ([near[0].max(lo[0]),lo[1],near[2].max(lo[2])], [far[0].min(hi[0]),far[1].min(hi[1]),far[2].min(hi[2])]);
    if (0..3).any(|i| goal[i] < cl[i] || goal[i] > ch[i]) { return refusal(requested,"the goal is outside the chunks the bot has received"); }
    let corridor = match plan::source::GeneratorSuperflatSource::from_world(source,(cl,ch),a) { Ok(c) => c, Err(why) => return refusal(requested,why) };
    let result = plan::solve(&state, Goal { cell: goal }, plan::Kit::BareLegs, &corridor, store, codec, u64::MAX);
    if version.load(Ordering::Acquire) != requested || revision.load(Ordering::Acquire) != source.revision { return refusal(requested,"cancelled or source changed while thinking"); }
    let plan::PlanOutcome::Ready(found) = result.outcome else { return refusal(requested,format!("{:?} {}",result.outcome,result.explanation.unwrap_or_default())); };
    let programs = words::tiny_programs();
    match inputs(source, &programs, &state, found.words.iter().map(|w| (w.program,w.args,w.ticks))) {
        Ok(inputs) => {
            let mut out = vec![WORD_PLAN,requested,issue,entered,crate::qpc().0 as u64,source.revision,state.t,inputs.len() as u64 / 3];
            let bits = state.bits(); out.push(bits.len() as u64); out.extend(bits); out.extend(inputs); out
        }
        Err(why) => refusal(requested,why),
    }
}
/// The plan's words × counts as tick inputs, each tick's input from the word's program, run through the law.
fn inputs(source: &Source, programs: &[words::Program], start: &PlayerState, words: impl Iterator<Item=(u16,words::Arguments,u32)>) -> Result<Vec<u64>,String> {
    let mut state = start.clone(); let mut inputs = Vec::new();
    for (program,args,ticks) in words { for elapsed in 0..ticks {
        let (_,input) = programs[program as usize].select(source,&state,elapsed,args).ok_or("a word's program has no row here")?;
        let keys = [input.forward,input.back,input.left,input.right,input.jump,input.sneak,input.sprint].iter().enumerate().fold(0,|n,(i,b)| n | ((*b as u64)<<i));
        inputs.extend([keys,input.yaw.to_bits() as u64,input.pitch.to_bits() as u64]);
        law::try_tick(&mut state,source,&input).map_err(|e| e.0)?;
    } }
    Ok(inputs)
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
        let mut source = Source { session:7,revision:1,difficulty:0,regeneration:Some(false),..Source::default() };
        let air = model("minecraft:air"); let stone = model("minecraft:stone");
        for x in -1..=1 { for z in -1..=1 { for y in -1..=1 {
            let indices = (0..4096).map(|i|usize::from(y<0 || y==0 && i/256==0)).collect();
            source.sections.insert([x,y,z],Section { palette:vec![("minecraft:air".into(),air.clone()),("minecraft:stone".into(),stone.clone())],indices });
        } } }
        let state = PlayerState { t:77,position:law::Vec3::new(8.5,1.0,8.5),on_ground:true,main_supporting_block:Some([8,0,8]),food_level:11,saturation:2.5,exhaustion:1.25,..PlayerState::default() };
        let bits = state.bits();
        let request = Request { state,meta:vec![7,1,0,0,0,1,2,123,125],goal:[8,0,12],issue:123,version:3 };
        let answer = plan(&source,&plan::store::GeneratorStore::default(),&mut law::codec::Interner::default(),request,&AtomicU64::new(3),&AtomicU64::new(1));
        assert_eq!(answer[0],WORD_PLAN,"{answer:?}"); assert_eq!(answer[2],123);
        let n = answer[8] as usize; assert_eq!(answer[9..9+n],bits); assert!(answer[7]>0); assert_eq!(answer.len(),9+n+answer[7] as usize*3);
    }
}
