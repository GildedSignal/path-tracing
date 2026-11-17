use std::{collections::HashMap, sync::atomic::{AtomicUsize, Ordering}, time::Instant};
use law::{Block, BlockPos, PlayerState, World, codec::{self, Interner, Label, LegsRole, StateQuestion}, footprint::RecordingWorld, state_reads::{StateField as F, StateReads}};
use law::codec::proof::{Binding, ReadCertificates, SnapshotSource};
use plan::{flight::track, settle, Still};
use words::{Arguments, tiny_programs};

struct Counted<'a, W> { world: &'a W, reads: AtomicUsize, ticks: AtomicUsize }
impl<W: World> World for Counted<'_, W> {
    fn observe_tick(&self) {self.ticks.fetch_add(1,Ordering::Relaxed);self.world.observe_tick();}
    fn block(&self, p: BlockPos) -> &Block { self.reads.fetch_add(1, Ordering::Relaxed); self.world.block(p) }
    fn read_state(&self, f: F) { self.reads.fetch_add(1, Ordering::Relaxed); self.world.read_state(f); }
    fn write_state(&self, f: F) { self.world.write_state(f); }
    fn observe_region(&self, q: law::footprint::Query, lo: BlockPos, hi: BlockPos) { self.world.observe_region(q, lo, hi); }
    fn observe_sweep(&self, from: law::Aabb, to: law::Aabb) { self.world.observe_sweep(from, to); }
    fn loaded(&self, p: BlockPos) -> bool { self.world.loaded(p) }
    fn difficulty(&self) -> u8 { self.world.difficulty() }
    fn natural_regeneration(&self) -> bool { self.world.natural_regeneration() }
    fn min_y(&self) -> i32 { self.world.min_y() }
    fn fast_lava(&self) -> bool { self.world.fast_lava() }
    fn has_fluid(&self) -> bool { self.world.has_fluid() }
    fn has_inside_effects(&self) -> bool { self.world.has_inside_effects() }
}
fn parse(line: &str) -> Vec<u64> { line.split_whitespace().map(|s| u64::from_str_radix(s, 16).unwrap()).collect() }
fn motor(s: &PlayerState, reads: StateReads) -> Vec<u64> {
    let mut out = Vec::new();
    for f in reads.fields() {
        match f {
            F::t | F::position | F::main_supporting_block | F::saturation | F::exhaustion | F::food_tick_timer => (),
            F::food_level => out.push((s.food_level > 6) as u64),
            F::health => out.push((s.health > 0.0) as u64),
            _ => s.push_field_bits(f, &mut out),
        }
    }
    // Include guards even when the first tick writes a resource before reading it.
    out.extend([(s.food_level > 6) as u64, (s.health > 0.0) as u64]);
    out
}
fn cycle_key(s: &PlayerState) -> Vec<u64> {
    let mut s = s.clone();
    s.t = 0; s.position = law::Vec3::default(); s.main_supporting_block = None;
    let mut b = s.bits();
    b[8] = (s.food_level > 6) as u64; b[9] = 0;
    // health, saturation and exhaustion are at the beginning of the float group.
    b[47] = (s.health > 0.0) as u64; b[48] = 0; b[49] = 0;
    b
}
fn label(codec: &mut Interner, w: &impl World, s: &PlayerState) -> codec::LabelId {
    let p = s.main_supporting_block.unwrap();
    let feet = [p[0], p[1]+1, p[2]];
    let roles = [LegsRole::Support02, LegsRole::Support05, LegsRole::BodyFeet, LegsRole::Inside];
    let classes: Vec<_> = roles.into_iter().enumerate().map(|(i,r)| { let q = if i < 2 {p} else {feet}; codec.legs(w.block(q), w.loaded(q), r).unwrap() }).collect();
    codec.label(Label { law_schema: codec::SCHEMA, body_kit: b"bare", support: &classes[..2], medium: &classes[2..] }).unwrap()
}
#[test]
fn t1_observed_transient() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let native = std::fs::read_to_string(root.join("bot/tests/fixtures/native-state.hex")).unwrap();
    let bits = parse(native.lines().next().unwrap());
    PlayerState::from_observed_bits(&bits, &vec![true;bits.len()]).unwrap();
    let recorded = std::fs::read_to_string(root.join("bot/tests/fixtures/recorded-state.hex")).unwrap();
    let mut lines = recorded.lines(); let bits = parse(lines.next().unwrap()); let mask: Vec<bool> = lines.next().unwrap().split_whitespace().map(|v| v == "1").collect();
    assert_eq!(PlayerState::from_observed_bits(&bits, &mask).unwrap_err().reason, "unknown word");
    println!("A observed schema: PASS");
    let track_lines = include_str!("../../../world/tests/fixtures/tracks.txt");
    let tracks: Vec<_> = ["straight100", "straight300", "straight800"].iter().map(|n| track(track_lines.lines().find(|l| l.split_whitespace().next() == Some(*n)).unwrap()).unwrap()).collect();
    let world = world::WorldSnapshot::load(root.join("engine/world/tests/fixtures/world"), [285,-64,-490], [322,-48,-360]).unwrap();
    assert_eq!(world.difficulty(), 0, "fixture difficulty");
    let counted = Counted { world: &world, reads: AtomicUsize::new(0), ticks: AtomicUsize::new(0) };
    let mut starts = Vec::new();
    for (i, track) in tracks.iter().enumerate() {
        let a = track.a; let mut s = settle(&Still(&track.world), [a[0] as f64+0.5, a[1] as f64+1.0, a[2] as f64+0.5], 0).unwrap();
        for (j, x) in [300.5,304.5,315.25].iter().enumerate() {
            s.position.x = *x; s.t = [0,77,1543][j];
            s.main_supporting_block = Some([x.floor() as i32, a[1], a[2]]);
            if j == 1 { s.food_level=17; s.saturation=5.0; s.exhaustion=0.25; } else { s.food_level=20; s.saturation=20.0; s.exhaustion=0.0; }
            let bits = s.bits(); let observed = PlayerState::from_observed_bits(&bits, &vec![true;bits.len()]).unwrap();
            starts.push((i,j,observed));
        }
    }
    let program_id = 30usize; // F S j a s n in tiny_programs' product order.
    let programs = tiny_programs(); let program = &programs[program_id];
    let yaw = ((0.5 * std::f64::consts::TAU / 65536.0) / 0.017453292f32 as f64) as f32;
    let args = Arguments { refined_yaw: Some(yaw), ..Arguments::default() };
    let start = &starts[0].2; let mut state = start.clone();
    let mut seen = HashMap::new(); seen.insert(cycle_key(&state), 0usize);
    let mut footprints = Vec::new(); let mut states = vec![state.clone()];
    let (first, period) = loop {
        let recording = RecordingWorld::new(&world);
        let metered = Counted { world: &recording, reads: AtomicUsize::new(0), ticks: AtomicUsize::new(0) };
        let input = program.select(&metered, &state, (states.len()-1) as u32, args).unwrap().1;
        law::try_tick(&mut state, &metered, &input).unwrap();
        assert!(metered.reads.load(Ordering::Relaxed) > 0);
        footprints.push(recording.take()); states.push(state.clone());
        if let Some(&f) = seen.get(&cycle_key(&state)) { break (f, states.len()-1-f); }
        assert!(states.len() < 4096, "no recurrence in 4096 ticks");
        seen.insert(cycle_key(&state), states.len()-1);
    };
    println!("transient_ticks={first} cycle_period={period}");
    let reads = StateReads(footprints.iter().fold(0, |v,f| v | f.state_reads.0));
    let mut codec = Interner::default();
    let first_label = label(&mut codec, &world, start);
    let id = codec.intern(&codec::state_key(StateQuestion::Recurrence, &motor(start, reads))).unwrap();
    let key = (first_label, program_id, 0u16, args.pitch.to_bits(), id);
    let mut memo = HashMap::new(); memo.insert(key, states[first].clone());
    let source = SnapshotSource { world: &world, ticks: (0, 4096), min: [285,-64,-490], max: [322,-48,-360] };
    let refs: Vec<_> = footprints.iter().enumerate().map(|(i,f)| (i as u64,f)).collect();
    let mut certs = ReadCertificates::default();
    let cert = certs.capture(&mut codec, Binding::new(start, b"F S j a s n", law::LAW_HASH), &refs, &source).unwrap();
    println!("prepared_entries={} certificate_cells={} cycle_increment=({:?},{:?})", memo.len(), certs.cells(cert).unwrap().len(), states[first+period].position.x-states[first].position.x, states[first+period].position.z-states[first].position.z);
    let mut times = Vec::new(); let mut failures = Vec::new();
    for (i,j,s) in &starts {
        let this_label = label(&mut codec, &world, s);
        let motor_id = codec.intern(&codec::state_key(StateQuestion::Recurrence, &motor(s, reads))).unwrap();
        let k = (this_label,program_id,0u16,args.pitch.to_bits(),motor_id);
        let before = counted.reads.load(Ordering::Relaxed); let tick_before = counted.ticks.load(Ordering::Relaxed); let clock = Instant::now();
        let entry = memo.get(&k);
        let mut applied = entry.cloned();
        if let Some(ref mut a) = applied {
            a.t = s.t + first as u64;
            a.position.x = s.position.x + (states[first].position.x - start.position.x);
            a.position.z = s.position.z + (states[first].position.z - start.position.z);
            let dx = s.position.x.floor() as i32 - start.position.x.floor() as i32;
            let dz = s.position.z.floor() as i32 - start.position.z.floor() as i32;
            a.main_supporting_block = a.main_supporting_block.map(|p| [p[0]+dx,p[1],p[2]+dz]);
        }
        times.push(clock.elapsed().as_nanos());
        let hot = counted.reads.load(Ordering::Relaxed)-before; let hot_ticks = counted.ticks.load(Ordering::Relaxed)-tick_before;
        let mut reference = s.clone();
        for n in 0..first { let input = program.select(&counted, &reference, n as u32, args).unwrap().1; law::try_tick(&mut reference, &counted, &input).unwrap(); }
        if k != key { failures.push(format!("track={i} start={j} memo key mismatch {k:?} != {key:?}")); continue; }
        if hot != 0 || hot_ticks != 0 { failures.push(format!("track={i} start={j} hot world reads={hot} hot_tick_calls={hot_ticks}")); }
        let a = applied.unwrap();
        macro_rules! check { ($f:expr,$g:expr,$name:expr) => { if $f != $g { failures.push(format!("track={i} start={j} {} actual={:?} expected={:?}",$name,$f,$g)); } }; }
        check!(a.t,reference.t,"t"); check!(a.position.x.to_bits(),reference.position.x.to_bits(),"x"); check!(a.position.y.to_bits(),reference.position.y.to_bits(),"y"); check!(a.position.z.to_bits(),reference.position.z.to_bits(),"z");
        check!(a.velocity.x.to_bits(),reference.velocity.x.to_bits(),"vx"); check!(a.velocity.y.to_bits(),reference.velocity.y.to_bits(),"vy"); check!(a.velocity.z.to_bits(),reference.velocity.z.to_bits(),"vz");
        check!(a.on_ground,reference.on_ground,"on_ground"); check!(a.sprinting,reference.sprinting,"sprinting"); check!(a.no_jump_delay,reference.no_jump_delay,"no_jump_delay"); check!(a.fall_distance.to_bits(),reference.fall_distance.to_bits(),"fall_distance"); check!(a.main_supporting_block,reference.main_supporting_block,"support");
    }
    let mut negative = start.clone(); negative.on_ground = !negative.on_ground;
    let neg_id = codec.intern(&codec::state_key(StateQuestion::Recurrence, &motor(&negative, reads))).unwrap();
    assert!(!memo.contains_key(&(first_label, program_id, 0u16, args.pitch.to_bits(), neg_id)), "negative control");
    times.sort(); println!("hot lookup+apply ns min={} median={} max={} n={}",times[0],times[times.len()/2],times[times.len()-1],times.len());
    for failure in &failures { println!("FAIL {failure}"); }
    assert!(failures.is_empty(), "{} failed assertions", failures.len());
}


