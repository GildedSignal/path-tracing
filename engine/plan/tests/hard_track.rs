//! Offline construction/replay only. Never opens a game, server, or RCON connection.
use law::{BoxWorld, Input, PlayerState, World};
use plan::{settle, Still};
use world::{model, BlockState};

fn block(raw: &str) -> law::Block {
    let (name, tail) = raw.split_once('[').unwrap_or((raw, ""));
    let mut properties = tail.trim_end_matches(']').split(',').filter(|p| !p.is_empty())
        .map(|p| { let (k,v)=p.split_once('=').unwrap(); (k.into(),v.into()) }).collect::<Vec<_>>();
    properties.sort();
    model(&BlockState { name: format!("minecraft:{}", name.trim_start_matches("minecraft:")), properties }).unwrap().clone()
}
fn build() -> BoxWorld {
    build_text(include_str!("../../../lab/tracks/hardtrack.build.txt"))
}
fn build_text(commands:&str) -> BoxWorld {
    let mut world=BoxWorld::default();
    world.difficulty=0;
    world.fill([-120,-64,-120],[-50,-64,-50],block("bedrock"));
    world.fill([-120,-63,-120],[-50,-62,-50],block("dirt"));
    world.fill([-120,-61,-120],[-50,-61,-50],block("grass_block[snowy=false]"));
    for line in commands.lines() {
        let line=line.split('#').next().unwrap().trim();
        if line.is_empty() { continue; }
        let fields=line.split_whitespace().collect::<Vec<_>>();
        let at=|i:usize| -> [i32;3] { std::array::from_fn(|a|fields[i+a].parse().unwrap()) };
        let (lo,hi,raw)=match fields[0] {
            "/fill" => (at(1),at(4),fields[7]),
            "/setblock" => (at(1),at(1),fields[4]),
            other=>panic!("unsupported build command {other}"),
        };
        assert!((0..3).all(|a|lo[a]<=hi[a]));
        assert!(lo[0]>=-110 && hi[0]<=-60 && lo[2]>=-110 && hi[2]<=-60 && lo[1]>=-60 && hi[1]<=-20);
        let value=block(raw);
        for z in lo[2]..=hi[2] { for y in lo[1]..=hi[1] { for x in lo[0]..=hi[0] { world.set([x,y,z],value.clone()); } } }
    }
    world
}
fn start(world: &BoxWorld) -> PlayerState { settle(&Still(world),[-106.5,-59.0,-106.5],0).unwrap() }
#[derive(Clone)]
struct Part { ticks:u32, yaw:f32, keys:String }
fn parts() -> Vec<Part> {
    include_str!("../../../lab/tracks/hardtrack.witness.txt").lines().filter_map(|line| {
        let text=line.split('#').next().unwrap().trim();
        if text.is_empty() { return None; }
        let f=text.split_whitespace().collect::<Vec<_>>();
        assert_eq!(f.len(),3);
        Some(Part {ticks:f[0].parse().unwrap(),yaw:f[1].parse().unwrap(),keys:f[2].into()})
    }).collect()
}
fn sentence(world: &BoxWorld, parts: &[Part], trace: bool) -> Vec<PlayerState> {
    let mut state=start(world);
    let mut states=vec![state.clone()];
    for part in parts {
        let yaw=part.yaw; let keys=&part.keys;
        for _ in 0..part.ticks {
            let input=Input { forward:keys.contains('F'),back:keys.contains('B'),left:keys.contains('L'),right:keys.contains('R'),
                sprint:keys.contains('S'),sneak:keys.contains('C'),jump:keys.contains('J') || keys.contains('G') && state.on_ground,
                yaw,..Input::default() };
            law::try_tick(&mut state,world,&input).unwrap();
            states.push(state.clone());
            if trace && (state.t%5==0 || state.on_ground) { println!("tick {} p {:?} v {:?} g={} climb={} hp={}",state.t,state.position,state.velocity,state.on_ground,state.on_climbable,state.health); }
        }
        if trace { println!("END {} {yaw} {keys}: t={} p={:?} v={:?}",part.ticks,state.t,state.position,state.velocity); }
    }
    states
}
const A:[i32;3]=[-107,-60,-107];
const B:[i32;3]=[-84,-25,-88];
fn touch(states:&[PlayerState])->Option<usize> {
    states.windows(2).position(|s|plan::Goal {cell:B}.touched(&s[0],&s[1])).map(|i|i+1)
}
fn digest(states:&[PlayerState])->u64 {
    states.iter().flat_map(|s|s.bits()).flat_map(u64::to_le_bytes).fold(0xcbf29ce484222325,|h,b|(h^b as u64).wrapping_mul(0x100000001b3))
}
#[test]
fn hard_track_witness() {
    let world=build();
    let states=sentence(&world,&parts(),std::env::var_os("HARD_TRACE").is_some());
    let replay=sentence(&world,&parts(),false);
    assert_eq!(states.iter().map(|s|s.bits()).collect::<Vec<_>>(),replay.iter().map(|s|s.bits()).collect::<Vec<_>>());
    println!("ticks={} final={:?} touch={:?} digest={:016x}",states.len()-1,states.last().unwrap().position,touch(&states),digest(&states));
    assert_eq!(touch(&states),Some(943),"the sentence must reach B for the first time at its last tick");
    assert!(states.iter().all(|s|s.health==20.0),"no damage along the witness");
    assert_eq!(states[0].main_supporting_block,Some(A));
    assert_eq!(digest(&states),0xb9fce6d6b6563d41,"every tick's complete state bits are frozen");
    // This compact spelling is also a sentence of the EXISTING tiny programs,
    // not a new executor capability. Certify that sentence through the existing
    // planner replay interface and compare its exact endpoint with the raw replay.
    let steps=parts().iter().map(|part|plan::Step {
        program:match part.keys.as_str() {"F"=>18,"FJ"=>21,"FS"=>27,"FSJ"=>30,"FSG"=>33,keys=>panic!("no existing program for {keys}")},
        args:words::Arguments {refined_yaw:Some(part.yaw),..Default::default()},ticks:part.ticks,lane:plan::Lane::Replay,
    }).collect::<Vec<_>>();
    let certificate=plan::certify(&Still(&world),&words::tiny_programs(),&states[0],&steps,&plan::Goal {cell:B},Some(A),false).unwrap();
    assert_eq!((certificate.ticks,certificate.clock),(943,Some(939)));
    assert_eq!(certificate.end.bits(),states[943].bits());
    let left=states.iter().position(|s| !(s.on_ground && s.main_supporting_block==Some(A))).unwrap();
    let peak=states[536..=555].iter().map(|s|s.position.y).fold(f64::NEG_INFINITY,f64::max);
    let landing=states.iter().position(|s|s.t>316 && s.on_ground && s.position.y==-29.0).unwrap();
    let exit=states.iter().position(|s|s.t>536 && s.on_ground && s.position.y==-37.0 && s.position.x> -70.0).unwrap();
    let catch=states.iter().position(|s|s.t>808 && s.on_climbable && s.position.z> -93.0).unwrap();
    let mut record=format!("# Witness record\n\nGenerated by `hard_track_witness` with HARD_RECORD=1.\n\n- Total from settled A to first B touch: {} ticks.\n- First sample off A: {left}; detector clock: {} ticks (same subtraction as plan::certify).\n- Complete state FNV-1a digest, including the start and every tick: `{:016x}`. A second full replay compares every state bit.\n- Existing tiny-program mapping: F=18, FJ=21, FS=27, FSJ=30, FSG=33. The same sentence passes plan::certify and has the identical complete endpoint bits.\n- Minimum health: 20; damage: 0; refusals: none; world writes: none.\n- Main rebound apex: {peak:.15}; exit support top -37; vertical margin {:.15} blocks.\n\n| Event | Tick | Position | Velocity |\n|---|---:|---|---|\n",states.len()-1,states.len()-1-left,digest(&states),peak+37.0);
    for (name,tick) in [("settled A",0),("launch preparation",315),("ice launch",316),("landing",landing),("south approach",380),("diagonal entry",515),("slime contact",536),("rebound exit",exit),("ladder launch",808),("suspended ladder catch",catch),("B",943)] {
        let s=&states[tick];
        record.push_str(&format!("| {name} | {tick} | {:?} | {:?} |\n",s.position,s.velocity));
    }
    if std::env::var_os("HARD_RECORD").is_some() { std::fs::write(root().join("jobs/hard-track-2/witness.md"),record).unwrap(); }
}

/// Measure local timing slack by moving just the named switch/pulse, preserving
/// every later phase boundary and the mission horizon. This is replay, not search.
#[test]
fn timing_certificates() {
    let world=build();
    let mut record=String::from("# Timing replays\n\nGenerated by `cargo test --offline -p plan --test hard_track timing_certificates -- --nocapture` with HARD_RECORD=1.\n\nShift moves the named input boundary, keeping all later boundaries fixed. Success means B by tick 943 with full health. The table was frozen after measuring the rebuilt sentence. These are local perturbations of one sentence, not a population sample or an exhaustive maximal window.\n\n| Input boundary | Shift | B tick | Race T | Health minimum | Slime exit support | Final ladder caught |\n|---|---:|---:|---:|---:|---|---|\n");
    let mut arrivals=Vec::new(); let mut misses=0;
    for (name,before,after,expected) in [("climb to accelerator",1,2,[Some(943),Some(943),Some(943)]),("ice takeoff pulse",3,5,[None,Some(943),Some(943)]),("slime diagonal entry",6,7,[Some(943),Some(943),Some(939)]),("slime west turn",7,8,[Some(943),Some(943),Some(939)]),("north return turn",9,10,[Some(939),Some(943),None]),("ladder takeoff pulse",11,13,[None,Some(943),Some(940)]),("final climb release",13,14,[Some(943),Some(943),Some(943)])] {
        for (index,shift) in [-1i32,0,1].into_iter().enumerate() {
            let mut p=parts();
            p[before].ticks=(p[before].ticks as i32+shift) as u32;
            p[after].ticks=(p[after].ticks as i32-shift) as u32;
            let states=sentence(&world,&p,false);
            assert_eq!(touch(&states),expected[index],"frozen timing outcome for {name} {shift:+}");
            match touch(&states) {Some(t)=>arrivals.push(t),None=>misses+=1};
            let hp=states.iter().map(|s|s.health as u32).min().unwrap();
            if touch(&states).is_some() {assert_eq!(hp,20,"successful timing replay keeps full health");}
            let exit=states.iter().any(|s|s.on_ground && s.position.y==-37.0 && s.position.x > -70.0 && s.position.z > -70.0);
            let ladder=states.iter().any(|s|s.on_climbable && s.position.z > -93.0 && s.position.y > -36.0);
            let row=format!("| {name} | {shift:+} | {} | {} | {hp} | {exit} | {ladder} |\n",touch(&states).map_or("none".into(),|t|t.to_string()),touch(&states).map_or("none".into(),|t|(t-4).to_string()));
            print!("{row}"); record.push_str(&row);
        }
    }
    arrivals.sort();
    let percentile=|p:usize|arrivals[(p*arrivals.len()).div_ceil(100).saturating_sub(1)];
    record.push_str(&format!("\nSuccessful perturbations: n={}, min={}, median={}, p90={}, p99={}, max={} ticks; failed perturbations: {misses}. Duplicate zero shifts are intentional positive controls. No mean is used.\n",arrivals.len(),arrivals[0],percentile(50),percentile(90),percentile(99),arrivals.last().unwrap()));
    if std::env::var_os("HARD_RECORD").is_some() { std::fs::write(root().join("jobs/hard-track-2/timing.md"),record).unwrap(); }
}
fn root()->std::path::PathBuf { std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..") }

#[derive(Clone,Debug)]
struct Support { cell:[i32;3], climb:bool, slime:bool }
impl Support {
    fn low(&self)->f64 { (self.cell[1]+i32::from(!self.climb)) as f64 }
    fn high(&self)->f64 { (self.cell[1]+1) as f64 }
    fn element(&self)->u8 {
        let [x,y,z]=self.cell;
        if self.climb && x==-106 {1}
        else if y==-29 && (-104..=-84).contains(&x) && (-107..=-106).contains(&z) {2}
        else if self.slime && y==-48 {4}
        else if self.climb && (-84..=-83).contains(&x) && z==-92 {8}
        else {0}
    }
}

struct ReachLaw { jump:f64, gravity:f64, drag:f64, speed:f64, apex:f64, rotation:f64, step_height:f64 }
impl ReachLaw {
    fn derive()->Self {
        // Read the recurrence through the law. These are coefficient probes,
        // not a search for a successful control sequence.
        let air=BoxWorld::default();
        let mut p=PlayerState::default(); p.position=law::Vec3::new(0.0,10.0,0.0); p.velocity=law::Vec3::new(0.0,1.0,1.0);
        let gravity=p.gravity; let jump=p.jump_strength;
        let step_height=p.step_height as f64;
        assert!((0.0..1.0).contains(&step_height),"integral collision planes must have no positive step candidate");
        law::try_tick(&mut p,&air,&Input::default()).unwrap();
        let horizontal_drag=p.velocity.z; let drag=p.velocity.y/(1.0-gravity);
        // Bound every f32 yaw, including huge angles. At an internal f64 binade
        // crossing, adding the quarter turn can round the table index by half
        // an ULP. The largest non-saturating crossing is 2^62; saturated i64
        // conversion is handled separately below. This is machine arithmetic,
        // not a tolerance on an edge or a restriction to conventional headings.
        let table=include_bytes!("../../law/src/sin.bin");
        let at=|i:usize|f32::from_le_bytes(table[4*(i%65536)..4*(i%65536)+4].try_into().unwrap()) as f64;
        let radius=(1usize << (i64::BITS-2-f64::MANTISSA_DIGITS))+1;
        let mut queue=std::collections::VecDeque::<usize>::new();let mut next=16384-radius;
        let mut rotation_squared=0.0f64;let mut largest=0.0f64;
        for i in 0..65536 {
            let start=i+16384-radius;let end=i+16384+radius;
            while next<=end {
                let value=at(next).powi(2);
                while queue.back().is_some_and(|&j|at(j).powi(2)<=value) {queue.pop_back();}
                queue.push_back(next);next+=1;
            }
            while queue.front().is_some_and(|&j|j<start) {queue.pop_front();}
            rotation_squared=rotation_squared.max((at(i).powi(2)+at(*queue.front().unwrap()).powi(2)).next_up());
            largest=largest.max(at(i).abs());
        }
        let rotation=rotation_squared.sqrt().next_up().max(largest.hypot(at(65535)).next_up());
        let air_acceleration=(0.025999999f32 as f64*rotation).next_up();
        let air_fixed=((horizontal_drag*air_acceleration).next_up()/(1.0-horizontal_drag).next_down()).next_up();
        let mut before_jump=air_fixed; let mut after_jump=air_fixed;let mut increment=0.0f64;let mut ground_coefficients=Vec::new();
        for name in ["stone","blue_ice","slime_block"] {
            let mut w=BoxWorld::default(); w.fill([-10,-1,-10],[10,-1,10],block(name));
            let s=settle(&Still(&w),[0.5,0.0,0.5],0).unwrap();
            let mut run=s.clone(); law::try_tick(&mut run,&w,&Input {forward:true,sprint:true,yaw:0.0,..Input::default()}).unwrap();
            let f=law::math::modified_friction(w.block([0,-1,0]).friction,s.friction_modifier);
            let q=(f*horizontal_drag as f32) as f64;
            // movement::friction_speed, evaluated in its native f32 order.
            // input_vector normalizes every direction to norm <= 1, rather
            // than the .98 magnitude of the forward-only coefficient probe.
            let acceleration=if f as f64>0.6 {run.speed*(0.21600002f32/(f*f*f))} else {run.speed};
            let mut launch=s.clone(); law::try_tick(&mut launch,&w,&Input {forward:true,sprint:true,jump:true,yaw:0.0,..Input::default()}).unwrap();
            let addition=((acceleration as f64+0.2).next_up()*rotation).next_up();
            assert!(launch.position.z-s.position.z<=addition);
            assert!(!launch.on_ground,"a ground impulse is followed by an airborne tick");
            ground_coefficients.push((q,addition));
            // A positive jump request clears on_ground even on a ceiling hit:
            // move_player sets vertical_collision_below only for wanted.y < 0.
            // Therefore there is at least one AIR tick between ground impulses.
            // This two-tick envelope includes head-hitters of every phase. Idle
            // ground ticks and longer flights contract toward smaller fixed points.
            let numerator=(horizontal_drag*((q*addition).next_up()+air_acceleration).next_up()).next_up();
            let cycle=(numerator/(1.0-horizontal_drag*q).next_down()).next_up();
            let ground_fixed=((q*acceleration as f64*rotation).next_up()/(1.0-q).next_down()).next_up();
            before_jump=before_jump.max(cycle).max(ground_fixed);
            increment=increment.max(addition);
        }
        let mut flying=PlayerState::default(); flying.position=law::Vec3::new(0.0,10.0,0.0);
        flying.set_sprinting(true);
        law::try_tick(&mut flying,&air,&Input {forward:true,sprint:true,yaw:0.0,..Input::default()}).unwrap();
        assert!(flying.position.z<=air_acceleration);
        for (q,addition) in ground_coefficients {
            after_jump=after_jump.max((q*(before_jump+addition).next_up()).next_up());
        }
        let mut y=0.0f64; let mut v=jump; let mut apex=0.0f64;
        while v>0.0 {y+=v;apex=apex.max(y);v=(v-gravity)*drag;}
        let speed=(before_jump+increment).next_up().max((after_jump+air_acceleration).next_up());
        Self {jump,gravity,drag,speed,apex,rotation,step_height}
    }
    fn flight(&self,hi:f64,launch:f64,target:f64)->(f64,f64) {
        let mut y=hi; let mut v=launch; let mut peak=hi; let mut ticks=0;
        // Include the complete tick that crosses the target plane. Allowing
        // departure anywhere in a ladder cell uses its high endpoint.
        while ticks==0 || y>=target || v>0.0 {
            y+=v; peak=peak.max(y); v=(v-self.gravity)*self.drag; ticks+=1;
            assert!(ticks<1000,"finite unsupported flight");
        }
        (peak,self.speed*ticks as f64)
    }
    fn rebound(&self,top:f64,floor:f64)->f64 {
        let mut y=top; let mut v=0.0;
        while y>floor {y+=v;v=(v-self.gravity)*self.drag;}
        // One full gravity tick beyond the uncollided impact, as in the old
        // pocket bound. Partial collision correction is included outward.
        (-v+self.gravity).max(self.jump)
    }
}

struct SupportGraph { nodes:Vec<Support>, edges:Vec<Vec<usize>>, law:ReachLaw, tops:Vec<f64>, margins:[f64;3] }
/// A point body in empty/full voxels is an over-approximation of every pose.
/// During a flight with one vertical impulse the feet never go below BOTH
/// endpoints. A disconnected altitude slab therefore proves no transfer.
/// The outside ring retains paths that leave the box and re-enter it.
struct FreeSpace { free:Vec<bool>, attachments:Vec<Vec<[i32;3]>>, bands:std::collections::HashMap<(i32,i32),Vec<Vec<u32>>> }
impl FreeSpace {
    const N:usize=53;
    const H:usize=44;
    fn index(x:i32,y:i32,z:i32)->usize {(x+111) as usize+Self::N*((z+111) as usize+Self::N*(y+60) as usize)}
    fn new(world:&BoxWorld,nodes:&[Support])->Self {
        let mut free=vec![false;Self::N*Self::N*Self::H];
        for x in -111..=-59 {for y in -60..=-17 {for z in -111..=-59 {
            free[Self::index(x,y,z)]=!world.block([x,y,z]).shape.is_block();
        }}}
        let radius=(0.6f32/2.0) as f64; let height=law::Pose::Swimming.height();
        let samples=|low:f64,high:f64,r:f64| {
            let mut cuts=vec![low,high];
            for i in low.floor() as i32-1..=high.ceil() as i32+1 {
                for d in [-r,0.0,r] {let x=i as f64+d;if x>=low && x<=high {cuts.push(x);}}
            }
            cuts.sort_by(f64::total_cmp);cuts.dedup();
            let mid=cuts.windows(2).map(|w|(w[0]+w[1])/2.0).collect::<Vec<_>>();cuts.extend(mid);cuts
        };
        let attachments=nodes.iter().map(|p| {
            let overhang=radius;
            let xs=samples(p.cell[0] as f64-overhang,(p.cell[0]+1) as f64+overhang,radius);
            let zs=samples(p.cell[2] as f64-overhang,(p.cell[2]+1) as f64+overhang,radius);
            let ys=samples(p.low(),p.high(),height as f64);
            let mut cells=Vec::new();
            // Rectangle arrangements partition ALL possible endpoint centers.
            // Include boundary samples as exact-width fits are legal. Testing
            // the shortest pose only excludes an endpoint impossible for every
            // pose. This closes the false diagonal attachment outside a pocket.
            for &x in &xs {for &y in &ys {for &z in &zs {
                let center=law::Vec3::new(x,y,z); let body=law::Aabb::player(center,height);
                if p.climb {
                    let inside=[x.floor() as i32,y.floor() as i32,z.floor() as i32]==p.cell;
                    // The same node also covers the climbable shape's top
                    // face. Retaining the ENTIRE cell face is conservative
                    // for a thin ladder; actual top-face overhang is included.
                    let top=y==p.high() && body.max.x>p.cell[0] as f64 && body.min.x<(p.cell[0]+1) as f64 && body.max.z>p.cell[2] as f64 && body.min.z<(p.cell[2]+1) as f64;
                    if !inside && !top {continue;}
                } else if body.max.x<=p.cell[0] as f64 || body.min.x>=(p.cell[0]+1) as f64 || body.max.z<=p.cell[2] as f64 || body.min.z>=(p.cell[2]+1) as f64 {continue;}
                let mut clear=true;
                'blocks: for a in body.min.x.floor() as i32..=body.max.x.floor() as i32 {for b in body.min.y.floor() as i32..=body.max.y.floor() as i32 {for c in body.min.z.floor() as i32..=body.max.z.floor() as i32 {
                    if !(-111..=-59).contains(&a) || !(-60..=-17).contains(&b) || !(-111..=-59).contains(&c) {clear=false;break 'blocks;}
                    if !free[Self::index(a,b,c)] && body.intersects(law::Aabb::new(a as f64,b as f64,c as f64,(a+1) as f64,(b+1) as f64,(c+1) as f64)) {clear=false;break 'blocks;}
                }}}
                let cell=[x.floor() as i32,y.floor() as i32,z.floor() as i32];
                if clear && !cells.contains(&cell) {cells.push(cell);}
            }}}
            cells
        }).collect();
        Self {free,attachments,bands:Default::default()}
    }
    fn labels(&mut self,nodes:&[Support],lo:i32,hi:i32)->&Vec<Vec<u32>> {
        let lo=lo.max(-60); let hi=hi.min(-17);
        self.bands.entry((lo,hi)).or_insert_with(|| {
            let mut component=vec![0u32;self.free.len()]; let mut next=0;
            if lo<=hi {for y in lo..=hi {for z in -111..=-59 {for x in -111..=-59 {
                let i=Self::index(x,y,z);
                if !self.free[i] || component[i]!=0 {continue;}
                next+=1; component[i]=next; let mut queue=std::collections::VecDeque::from([[x,y,z]]);
                while let Some([a,b,c])=queue.pop_front() {
                    for [d,e,f] in [[a-1,b,c],[a+1,b,c],[a,b-1,c],[a,b+1,c],[a,b,c-1],[a,b,c+1]] {
                        if !(-111..=-59).contains(&d) || !(lo..=hi).contains(&e) || !(-111..=-59).contains(&f) {continue;}
                        let j=Self::index(d,e,f);
                        if self.free[j] && component[j]==0 {component[j]=next;queue.push_back([d,e,f]);}
                    }
                }
            }}}}
            nodes.iter().enumerate().map(|(i,_)| {
                let mut labels=Vec::new();
                // Include the body's entire overhang at both ends. Whole cells
                // are retained; neither center sampling nor line of sight prunes.
                for &[x,y,z] in &self.attachments[i] {
                    if (lo..=hi).contains(&y) {
                        let c=component[Self::index(x,y,z)]; if c!=0 && !labels.contains(&c) {labels.push(c);}
                    }
                }
                labels
            }).collect()
        })
    }
}
impl SupportGraph {
    fn build(world:&BoxWorld)->Self {
        fn same_shape(a:&law::VoxelShape,b:&law::VoxelShape)->bool {
            if a.coords!=b.coords {return false;}
            for x in 0..a.coords[0].len().saturating_sub(1) {for y in 0..a.coords[1].len().saturating_sub(1) {for z in 0..a.coords[2].len().saturating_sub(1) {
                if a.is_full([x,y,z])!=b.is_full([x,y,z]) {return false;}
            }}} true
        }
        let law=ReachLaw::derive(); let mut nodes=Vec::new();
        let frictions=[block("stone").friction,block("blue_ice").friction,block("slime_block").friction];
        for x in -110..=-60 {for y in -61..=-20 {for z in -110..=-60 {
            let cell=[x,y,z]; let b=world.block(cell);
            // collide() measures candidate heights from the grounded base, or
            // from the DOWNWARD-COLLISION result when falling. Starting at A,
            // every grounded base therefore has integral feet height. All shape
            // Y planes here are integral and step_height < 1: a positive step
            // candidate cannot exist. This proves that an unsupported transfer
            // cannot duck below both endpoints and then step back up.
            assert!(b.shape.coords[1].iter().all(|y|y.fract()==0.0),"fractional step plane at {cell:?}");
            assert!([&b.above_shape,&b.below_shape,&b.sneak_shape,&b.inside_cell_shape].iter().all(|shape|shape.as_ref().is_none_or(|shape|same_shape(shape,&b.shape))),"context-dependent collision plane at {cell:?}");
            assert!(b.fluid.kind==law::FluidKind::Empty && b.inside==law::Inside::None && b.speed_factor<=1.0 && b.jump_factor<=1.0,"reach envelope needs another law hook at {cell:?}");
            assert!(frictions.contains(&b.friction),"unbounded friction regime at {cell:?}");
            if b.climbable || b.slime || b.shape.is_block() && !world.block([x,y+1,z]).shape.is_block() {
                nodes.push(Support {cell,climb:b.climbable,slime:b.slime});
            } else {assert!(b.shape.is_empty() || b.shape.is_block(),"unhandled support shape at {cell:?}");}
        }}}
        let radius=0.6f32 as f64/2.0;
        let mut geometry=FreeSpace::new(world,&nodes);
        // A sealed roof separates its outside supports from the race. Bound
        // impact energy by every ordinary launch in the SAME free component;
        // this also includes unreachable internal supports, conservatively.
        let rooms=geometry.labels(&nodes,-60,-17);
        let tops=nodes.iter().enumerate().map(|(i,p)| {
            nodes.iter().enumerate().filter(|(j,q)|!q.slime && rooms[i].iter().any(|c|rooms[*j].contains(c)))
                .map(|(_,q)|q.high()+law.apex).fold(p.high()+law.apex,f64::max)
        }).collect::<Vec<_>>();
        let mut edges=vec![Vec::new();nodes.len()];
        let mut margins=[f64::INFINITY;3];
        for (i,p) in nodes.iter().enumerate() {
            let launch=if p.slime {law.rebound(tops[i],p.high())} else {law.jump};
            for (j,q) in nodes.iter().enumerate() {
                if i==j {continue;}
                let (peak,reach)=law.flight(p.high(),launch,q.low());
                if q.low()>peak {margins[0]=margins[0].min(q.low()-peak);continue;}
                let dx=((p.cell[0]-q.cell[0]).abs() as f64-1.0-2.0*radius).max(0.0);
                let dz=((p.cell[2]-q.cell[2]).abs() as f64-1.0-2.0*radius).max(0.0);
                if dx.hypot(dz)>reach {margins[1]=margins[1].min(dx.hypot(dz)-reach);continue;}
                let labels=geometry.labels(&nodes,p.low().min(q.low()).floor() as i32,peak.floor() as i32);
                if labels[i].iter().any(|label|labels[j].contains(label)) {edges[i].push(j);} else {margins[2]=1.0;}
            }
        }
        Self {nodes,edges,law,tops,margins}
    }
    fn index(&self,cell:[i32;3])->usize {self.nodes.iter().position(|p|p.cell==cell).unwrap_or_else(||panic!("missing support {cell:?}"))}
    fn path(&self,from:usize,to:usize,excluded:u8)->Option<Vec<usize>> {
        let mut prev=vec![usize::MAX;self.nodes.len()]; prev[from]=from;
        let mut queue=std::collections::VecDeque::from([from]);
        while let Some(p)=queue.pop_front() {
            if p==to {
                let mut path=vec![p]; let mut q=p;
                while q!=from {q=prev[q];path.push(q);} path.reverse();return Some(path);
            }
            for &q in &self.edges[p] {if prev[q]==usize::MAX && self.nodes[q].element()&excluded==0 {prev[q]=p;queue.push_back(q);}}
        }
        None
    }
    fn old_holes(&self,a:usize,b:usize)->String {
        fn ranges(values:impl Iterator<Item=usize>)->String {
            let mut runs=Vec::new();let mut run:Option<(usize,usize)>=None;
            for i in values {match run {Some((l,h)) if i==h+1=>run=Some((l,i)),Some((l,h))=>{runs.push(format!("{l}-{h}"));run=Some((i,i));},None=>run=Some((i,i))}}
            if let Some((l,h))=run {runs.push(format!("{l}-{h}"));} runs.join(",")
        }
        let mut encoded=String::from("# Complete old support graph. Inclusive successor-ID runs. B is terminal.\n# id x y z element successors\n");
        let mut reverse=vec![Vec::new();self.nodes.len()];
        for (i,p) in self.nodes.iter().enumerate() {
            let edges=if i==b {&[][..]} else {self.edges[i].as_slice()};
            for &j in edges {reverse[j].push(i);}
            encoded.push_str(&format!("{i}\t{}\t{}\t{}\t{}\t{}\n",p.cell[0],p.cell[1],p.cell[2],p.element(),ranges(edges.iter().copied())));
        }
        std::fs::write(root().join("jobs/hard-track-2/old-graph.tsv"),encoded).unwrap();
        let mut result=String::from("\n## Every skipping path (finite complete description)\n\nThe graph contains cycles, so its A-to-B walks cannot be listed one per line. `old-graph.tsv` lists every node and edge. Each line below lists the complete A/B core after removing an element; every A-to-B walk in that induced subgraph is a hole, and the union of these four languages is exactly every walk skipping an intended element. Successor runs and node runs are inclusive. B is terminal. These are conservative graph paths, not claims that each is physically executable.\n\n");
        let mut cores=String::from("# omitted-element-bit complete-A/B-core-node-ID-runs\n");
        for (name,mask) in [("initial ladder",1),("north ice",2),("main slime",4),("final ladder",8)] {
            let reach=|seed:usize,adj:&Vec<Vec<usize>>| {
                let mut seen=vec![false;self.nodes.len()]; seen[seed]=true;let mut queue=std::collections::VecDeque::from([seed]);
                while let Some(i)=queue.pop_front() {
                    if std::ptr::eq(adj,&self.edges) && i==b {continue;}
                    for &j in &adj[i] {if !seen[j] && self.nodes[j].element()&mask==0 {seen[j]=true;queue.push_back(j);}}
                } seen
            };
            let forward=reach(a,&self.edges);let backward=reach(b,&reverse);
            let core=(0..self.nodes.len()).map(|i|forward[i] && backward[i] && self.nodes[i].element()&mask==0).collect::<Vec<_>>();
            let count=core.iter().filter(|&&v|v).count();
            let edges=self.edges.iter().enumerate().filter(|(i,_)|core[*i] && *i!=b).map(|(_,edges)|edges.iter().filter(|&&j|core[j]).count()).sum::<usize>();
            result.push_str(&format!("- Omit {name}: {count} core nodes, {edges} core edges; core IDs are in `old-holes.tsv`, mask {mask}.\n"));
            cores.push_str(&format!("{mask}\t{}\n",ranges((0..core.len()).filter(|&i|core[i]))));
        }
        std::fs::write(root().join("jobs/hard-track-2/old-holes.tsv"),cores).unwrap();
        result
    }
    fn wrong_order(&self,a:usize,b:usize)->Option<Vec<[i32;3]>> {
        let mut prev=vec![usize::MAX;5*self.nodes.len()];let seed=5*a;prev[seed]=seed;
        let mut queue=std::collections::VecDeque::from([seed]);
        while let Some(state)=queue.pop_front() {
            let p=state/5;let stage=state%5;
            if p==b {
                if stage==4 {continue;}
                let mut path=vec![self.nodes[p].cell];let mut q=state;
                while q!=seed {q=prev[q];path.push(self.nodes[q/5].cell);}path.reverse();return Some(path);
            }
            for &q in &self.edges[p] {
                let next=if stage<4 && self.nodes[q].element()==1<<stage {stage+1} else {stage};
                let t=5*q+next;
                if prev[t]==usize::MAX {prev[t]=state;queue.push_back(t);}
            }
        } None
    }
    fn local_exclusions(&self,name:&str)->usize {
        let near=self.nodes.iter().enumerate().filter(|(_,p)|(-86..=-82).contains(&p.cell[0]) && (-107..=-87).contains(&p.cell[2])).map(|(i,_)|i).collect::<Vec<_>>();
        let mut csv=String::from("from_x,from_y,from_z,to_x,to_y,to_z,proof,bound,required,positive_margin\n");let mut count=0;
        let radius=(0.6f32/2.0) as f64;
        for &i in &near {for &j in &near {
            if i==j || self.edges[i].binary_search(&j).is_ok() {continue;}
            let p=&self.nodes[i];let q=&self.nodes[j];
            let launch=if p.slime {self.law.rebound(self.tops[i],p.high())} else {self.law.jump};
            let (peak,reach)=self.law.flight(p.high(),launch,q.low());
            let dx=((p.cell[0]-q.cell[0]).abs() as f64-1.0-2.0*radius).max(0.0);
            let dz=((p.cell[2]-q.cell[2]).abs() as f64-1.0-2.0*radius).max(0.0);
            let gap=dx.hypot(dz);
            let (proof,bound,required)=if q.low()>peak {("vertical_apex",peak,q.low())}
                else if gap>reach {("horizontal_airtime",reach,gap)}
                else {("common_free_components",0.0,1.0)};
            csv.push_str(&format!("{},{},{},{},{},{},{proof},{bound:.17},{required:.17},{:.17}\n",p.cell[0],p.cell[1],p.cell[2],q.cell[0],q.cell[1],q.cell[2],required-bound));count+=1;
        }}
        std::fs::write(root().join(format!("jobs/hard-track-2/{name}-shortcut-exclusions.csv")),csv).unwrap();count
    }
    fn critical_exclusions(&self) {
        let mut text=String::from("# Selected separating bounds\n\nGenerated by `whole_box_reach_certificate`. The CSV contains EVERY excluded ordered pair in x=-86..-82, z=-107..-87, at every inventoried height. Here are the additional course-wide separating bounds. Component deficit is integral: a common free component is required; zero is found, so the deficit is one. It is not a spatial epsilon.\n\n| Source | Target | Proof | Upper bound | Required | Positive deficit |\n|---|---|---|---:|---:|---:|\n");
        let radius=(0.6f32/2.0) as f64;
        for (from,to) in [([-101,-29,-104],[-76,-30,-106]),([-101,-29,-101],[-64,-48,-66]),([-100,-32,-102],[-101,-29,-102]),([-84,-32,-102],[-85,-20,-96]),([-84,-32,-102],[-84,-37,-92]),([-61,-30,-71],[-65,-38,-65]),([-76,-50,-109],[-76,-30,-108]),([-80,-45,-95],[-80,-38,-96]),([-85,-20,-96],[-84,-37,-92])] {
            let i=self.index(from);let j=self.index(to);assert!(!self.edges[i].contains(&j),"separating edge must be excluded: {from:?} -> {to:?}");
            let p=&self.nodes[i];let q=&self.nodes[j];let launch=if p.slime {self.law.rebound(self.tops[i],p.high())} else {self.law.jump};
            let (peak,reach)=self.law.flight(p.high(),launch,q.low());
            let dx=((p.cell[0]-q.cell[0]).abs() as f64-1.0-2.0*radius).max(0.0);let dz=((p.cell[2]-q.cell[2]).abs() as f64-1.0-2.0*radius).max(0.0);let gap=dx.hypot(dz);
            let (proof,bound,required)=if q.low()>peak {("apex height",peak,q.low())} else if gap>reach {("horizontal distance",reach,gap)} else {("common components",0.0,1.0)};
            text.push_str(&format!("| {from:?} | {to:?} | {proof} | {bound:.15} | {required:.15} | {:.15} |\n",required-bound));
        }
        text.push_str(&format!("\nTable rotation norm bound for every f32 yaw: {:.15}.\n",self.law.rotation));
        text.push_str(&format!("\nIntegral-step premise: step-height: {:.15}; smallest positive candidate: 1; deficit: {:.15}. Every collision Y plane in the fixture is asserted integral.\n",self.law.step_height,1.0-self.law.step_height));
        std::fs::write(root().join("jobs/hard-track-2/critical-exclusions.md"),text).unwrap();
    }
}

/// The old world is a required counterexample; the rebuilt world's conservative
/// graph must satisfy the ordered dominators, closed decoys and wall regression.
#[test]
fn whole_box_reach_certificate() {
    let world=build_text(include_str!("../../../jobs/hard-track-2/old.build.txt"));
    let graph=SupportGraph::build(&world);
    let lane=graph.index([-84,-29,-102]); let wall=graph.index([-85,-32,-96]);
    let ladder=graph.index([-84,-30,-92]);
    assert!(graph.edges[lane].contains(&wall),"must admit Julien's decoy-lane to wall transfer");
    assert!(graph.edges[wall].contains(&ladder),"must admit Julien's wall to final ladder transfer");
    let a=graph.index(A); let b=graph.index(B);
    assert!(graph.path(a,lane,2|4|8).is_some());assert!(graph.path(ladder,b,1|2|4).is_some());
    let mut record=format!("# Old whole-box graph\n\nNodes: {}; edges: {}; speed bound: {:.15}; jump apex: {:.15}.\n\nJulien regression: {:?} -> {:?} -> {:?}. Both edges retained.\n\n",graph.nodes.len(),graph.edges.iter().map(Vec::len).sum::<usize>(),graph.law.speed,graph.law.apex,graph.nodes[lane].cell,graph.nodes[wall].cell,graph.nodes[ladder].cell);
    for (name,element) in [("initial ladder",1),("north ice",2),("main slime",4),("final ladder",8)] {
        let path=graph.path(a,b,element).map(|path|path.iter().map(|&p|graph.nodes[p].cell).collect::<Vec<_>>());
        record.push_str(&format!("- Skipping {name}: {path:?}\n"));
    }
    record.push_str(&graph.old_holes(a,b));
    record.push_str(&format!("\nLocal excluded edges: {}; minimum exclusion margins (height, horizontal distance, component deficit): {:?}.\n",graph.local_exclusions("old"),graph.margins));
    print!("{record}");
    std::fs::write(root().join("jobs/hard-track-2/old-graph.md"),record).unwrap();
    let new=SupportGraph::build(&build());let a=new.index(A);let b=new.index(B);
    let mut record=format!("# Revised whole-box graph\n\nNodes: {}; edges: {}.\n\nSpeed bound: {:.15}; jump apex: {:.15}. Minimum exclusion margins (height, horizontal distance, component deficit): {:?}.\n\n",new.nodes.len(),new.edges.iter().map(Vec::len).sum::<usize>(),new.law.speed,new.law.apex,new.margins);
    assert!(new.path(a,b,0).is_some(),"the conservative graph must retain a possible race");
    for (name,mask) in [("initial ladder",1),("north ice",2),("main slime",4),("final ladder",8)] {
        record.push_str(&format!("- Skipping {name}: {:?}\n",new.path(a,b,mask).map(|path|path.iter().map(|&i|new.nodes[i].cell).collect::<Vec<_>>())));
    }
    let wrong=new.wrong_order(a,b);record.push_str(&format!("- Incomplete racing order: {wrong:?}\n"));
    let mut decoy_failures=Vec::new();
    for decoy in 0..3 {
        let members=new.nodes.iter().map(|p| {
            let [x,y,z]=p.cell;
            match decoy {
                0=>(-100..=-73).contains(&x) && (-103..=-101).contains(&z) && y<=-32,
                1=>(-76..=-71).contains(&x) && z==-109 && y<=-30,
                _=>(-80..=-77).contains(&x) && (-95..=-90).contains(&z) && y< -20,
            }
        }).collect::<Vec<_>>();
        assert!(members.iter().any(|&yes|yes),"nonempty committed decoy {decoy}");
        for (i,edges) in new.edges.iter().enumerate() {if members[i] {for &j in edges {if !members[j] {decoy_failures.push((decoy,new.nodes[i].cell,new.nodes[j].cell));}}}}
        record.push_str(&format!("- Decoy {}: {} supports.\n",decoy+1,members.iter().filter(|&&yes|yes).count()));
    }
    record.push_str(&format!("- Decoy outgoing edges: {decoy_failures:?}\n"));
    let lane=new.index([-84,-32,-102]);
    let wall=new.nodes.iter().enumerate().filter(|(_,p)|p.cell[0]==-85 && (-99..=-96).contains(&p.cell[2])).map(|(i,_)|i).collect::<Vec<_>>();
    assert!(!wall.is_empty());
    let old_route=wall.iter().any(|&i|new.path(lane,i,0).is_some() && new.path(i,b,0).is_some());
    record.push_str(&format!("- Decoy lane -> replacement wall -> B: {old_route}.\n"));
    record.push_str(&format!("- Local excluded edges: {}. See `new-shortcut-exclusions.csv`.\n",new.local_exclusions("new")));
    new.critical_exclusions();
    print!("{record}");std::fs::write(root().join("jobs/hard-track-2/new-graph.md"),record).unwrap();
    assert!(wrong.is_none(),"a conservative graph path still bypasses the ordered elements: {wrong:?}");
    assert!(decoy_failures.is_empty(),"committed decoy still has outgoing edges: {decoy_failures:?}");
    assert!(!old_route,"Julien's old wall transfer must not survive");
}

/// A closed vertical pocket up to its lowest return sill. Its interior has no
/// foothold or lift hook: the jump/rebound bound is therefore an exit bound.
fn pocket(world:&BoxWorld,lo:[i32;3],hi:[i32;3],sill:i32) {
    for x in lo[0]..=hi[0] { for z in lo[2]..=hi[2] {
        assert!(world.block([x,lo[1]-1,z]).shape.is_block());
        for y in lo[1]..sill {
            let b=world.block([x,y,z]);
            assert!(b.never_collides() && !b.climbable && b.inside==law::Inside::None && b.fluid.kind==law::FluidKind::Empty,"unexpected support/lift in pocket at {:?}",[x,y,z]);
        }
    } }
    for y in lo[1]..sill { for x in lo[0]..=hi[0] {
        for z in [lo[2]-1,hi[2]+1] { assert!(world.block([x,y,z]).shape.is_block(),"open pocket wall at {:?}",[x,y,z]); }
    } for z in lo[2]..=hi[2] {
        for x in [lo[0]-1,hi[0]+1] { assert!(world.block([x,y,z]).shape.is_block(),"open pocket wall at {:?}",[x,y,z]); }
    } }
}

#[test]
fn reach_and_decoy_certificates() {
    // Retain the old local claims as a baseline, beside the new whole-box proof.
    let world=build_text(include_str!("../../../jobs/hard-track-2/old.build.txt"));
    // The maximum jump height is the same recurrence at any yaw: horizontal keys
    // cannot add vertical impulse. No lift hooks exist in the stone pockets.
    let mut flat=BoxWorld::default(); flat.fill([-20,-1,-20],[20,-1,20],block("stone"));
    let mut p=settle(&Still(&flat),[0.5,0.0,0.5],0).unwrap();
    let mut apex=0.0f64;
    for t in 0..20 { law::try_tick(&mut p,&flat,&Input {jump:t==0,..Input::default()}).unwrap(); apex=apex.max(p.position.y); }
    assert!(apex < 1.3);
    // A generous ordinary-stone speed invariant, even allowing an impossible
    // ground jump every tick: ground (.4+.201+.131)*.5461 < .4;
    // air (.4+.027)*.9101 < .4. All bounds round OUTWARD from law coefficients.
    assert!((0.4+0.201+0.131)*0.5461 < 0.4);
    assert!((0.4+0.027)*0.9101 < 0.4);
    let mut velocity=0.4;
    let mut reach=0.4+0.201+0.131;
    velocity=(velocity+0.201+0.131)*0.5461;
    for _ in 1..15 { reach+=velocity+0.027; velocity=(velocity+0.027)*0.9101; }
    assert!(reach < 6.4,"ordinary stone cannot cross the seven-cell gap even with body overhang and a one-block drop");
    // Conservatively start the north-pocket fall ABOVE any entry jump. Bound
    // the impact by the uncollided fall; add a full gravity tick for partial
    // contact correction. Future rebounds dissipate; a .42 jump is smaller.
    let (mut y,mut vy)=(-26.0f64,0.0f64);
    let drag=0.98f32 as f64;
    while y > -49.0 { y+=vy; vy=(vy-0.08)*drag; }
    let mut rebound=-vy+0.08;
    let mut return_y=-49.0;
    while rebound>0.0 { return_y+=rebound; rebound=(rebound-0.08)*drag; }
    assert!(return_y < -29.0);
    pocket(&world,[-76,-35,-103],[-74,-35,-101],-31);
    pocket(&world,[-76,-49,-109],[-71,-49,-109],-29);
    pocket(&world,[-80,-44,-95],[-77,-44,-90],-37);
    // Verify the closed stone-pocket boundary surfaces, not only a sampled jump.
    for x in -76..=-73 { for z in -103..=-101 { assert!(world.block([x,-36,z]).shape.is_block()); } }
    for y in -35..=-32 { for z in -103..=-101 { assert!(world.block([-77,y,z]).shape.is_block()); } }
    for x in -80..=-77 { for z in -95..=-90 { assert!(world.block([x,-45,z]).shape.is_block()); } }
    // Return sills: pocket 1 needs four blocks; pocket 3 needs seven. The false
    // suspended ladder begins four blocks ABOVE its launch feet.
    assert!(apex < 4.0 && apex < 7.0);
    assert!(-37.0+apex < -33.0);
    // The upper baffle ends at x=-72 and the exit ledge begins at x=-75.
    // Subtract the body's radius at both planes: centers must go from -71.7
    // to -73.7. A fall from the entry shelf to the crouched head-clearance
    // plane has speed greater than .8; even granting three further airborne
    // moves at the generous .4+.027 horizontal bound cannot cover that span.
    let (mut falling_y,mut falling_v)=(-29.0f64,0.0f64);
    while falling_y > -35.5 { falling_y+=falling_v; falling_v=(falling_v-0.08)*drag; }
    assert!(falling_v < -0.8);
    assert!(3.0*(0.4+0.027) < 2.0);
    assert!(world.block([-73,-34,-100]).shape.is_block());
    assert!(world.block([-75,-38,-97]).shape.is_block());
    let text=format!("# Reach and decoy bounds\n\nGenerated by `reach_and_decoy_certificates` (HARD_RECORD=1).\n\n- Stone jump apex from rest: {apex:.15} blocks.\n- Generous stone/head-hitter horizontal bound over 15 airborne samples: {reach:.15} blocks; required center displacement at the ice gap: 6.4 blocks.\n- False ice pocket: floor feet -35; lowest return sill -31, four blocks up.\n- North slime pocket: entry feet at most -26 in the conservative drop; floor feet -49; conservative rebound apex {return_y:.15}; return sill -29.\n- False ladder pocket: floor feet -44; return sill -37, seven blocks up. Its ladder bottom is -33; launch feet -37 plus jump apex is {:.15}.\n- Direct falling transfer: crouched head-clearance plane -35.5; slow-entry fall velocity at its first crossing {falling_v:.15}; granting 3 further moves at .427 gives 1.281 blocks, less than the 2-block center separation. Jumping before the drop only increases downward speed at that plane.\n- Refusals: none.\n",-37.0+apex);
    print!("{text}");
    if std::env::var_os("HARD_RECORD").is_some() { std::fs::write(root().join("jobs/hard-track-2/old-local-bounds.md"),text).unwrap(); }
}

#[test]
fn the_required_mechanics_cannot_be_removed_from_the_witness() {
    let cases=[("initial ladder",[-106,-59,-107],[-106,-29,-106],"air"),
        ("blue ice",[-104,-29,-107],[-84,-29,-106],"stone"),
        ("low ceiling",[-104,-26,-107],[-86,-26,-106],"air"),
        ("slime rebound",[-64,-48,-66],[-61,-48,-62],"stone"),
        ("suspended ladder",[-84,-37,-92],[-83,-25,-92],"air")];
    let mut text=String::from("# Mechanic-removal controls\n\nGenerated by `the_required_mechanics_cannot_be_removed_from_the_witness` with HARD_RECORD=1. Each case changes only the named world cells, then replays the same sentence. These controls prove dependence of this witness, alongside the independent reach/pocket bounds; they do not prove global optimality.\n\n| Removed | First B touch | Minimum health |\n|---|---:|---:|\n");
    for (name,lo,hi,raw) in cases {
        let mut world=build(); let b=block(raw);
        for x in lo[0]..=hi[0] { for y in lo[1]..=hi[1] { for z in lo[2]..=hi[2] { world.set([x,y,z],b.clone()); } } }
        let states=sentence(&world,&parts(),false);
        assert_eq!(touch(&states),None,"removing {name} must break this sentence");
        text.push_str(&format!("| {name} | none | {} |\n",states.iter().map(|s|s.health as u32).min().unwrap()));
    }
    print!("{text}");
    if std::env::var_os("HARD_RECORD").is_some() { std::fs::write(root().join("jobs/hard-track-2/controls.md"),text).unwrap(); }
}

/// Diagnostic only: the existing mode-5 backward field and forward planner,
/// over the command-built world because no in-game fixture exists yet.
#[test]
#[ignore = "capped release planner diagnostic, not a correctness gate"]
fn current_planner_mode_5() {
    use plan::{Field,Goal,Options,Order,Planner,Problem,Reach};
    let world=build(); let timeline=Still(&world); let start=start(&world);
    let programs=words::tiny_programs();
    let body=([-109,-60,-109],[-61,-21,-61]);
    let extent=((49.0f64).powi(2)*2.0).sqrt();
    let reach=Reach::compile(world.block(A),&programs,extent,40.0).unwrap();
    let compile=reach.compile_us/1e6;
    let clock=std::time::Instant::now();
    let goal=Goal {cell:B};
    let field=Field::build(&world,&reach,body,&goal,start.main_supporting_block,(0,None));
    let field_seconds=clock.elapsed().as_secs_f64();
    let field_stats=(field.standable,field.reached,field.checks,field.cells.get(&A).map(|c|c.t));
    let mut table=words::ProposalTable::default();
    let problem=Problem {timeline:&timeline,start,goal,body,read:([-112,-64,-112],[-58,-17,-58]),a:Some(A)};
    let options=Options {order:Order::Field,arcs:true,gaits:true,audit:false,improve:false,nodes:1_500_000};
    let mut planner=Planner::new(problem,programs,&mut table,Some(field),options);
    let outcome=planner.run(&mut |seconds,_,_|seconds+field_seconds>10.0);
    let text=format!("# Current planner diagnostic\n\nExisting mode 5 (Field, arcs and gaits on, improve off, 1,500,000 nodes), command-built world. Run: `cargo test --offline --release -p plan --test hard_track current_planner_mode_5 -- --ignored --nocapture`. Budget: 10 seconds for field plus search; reach preparation recorded separately, as in the current flight.\n\n- Reach preparation: {compile:.6} s.\n- Field: {field_seconds:.6} s; (standable, reached, checks, T at A): {field_stats:?}.\n- Search: {:.6} s; stop {:?}; nodes {}; expansions {}; lower {}.\n- Sentence: {:?}.\n- Refusals: {:?}.\n- Uncertified candidates: {:?}.\n\nThis is a diagnostic, not a gate or an impossibility claim. No human T exists yet.\n",outcome.seconds,outcome.stopped,planner.stats.nodes,planner.stats.expansions,outcome.lower,outcome.sentence.as_ref().map(|(_,c)|c.ticks),planner.stats.refusals,planner.stats.uncertified);
    print!("{text}");
    if std::env::var_os("HARD_RECORD").is_some() { std::fs::write(root().join("jobs/hard-track-2/planner.md"),text).unwrap(); }
}

