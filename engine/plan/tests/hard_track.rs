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
    let mut world=BoxWorld::default();
    world.difficulty=0;
    world.fill([-120,-64,-120],[-50,-64,-50],block("bedrock"));
    world.fill([-120,-63,-120],[-50,-62,-50],block("dirt"));
    world.fill([-120,-61,-120],[-50,-61,-50],block("grass_block[snowy=false]"));
    for line in include_str!("../../../lab/tracks/hardtrack.build.txt").lines() {
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
#[test]
fn post_ladder_shortcut() {
    let world=build();
    let mut state=settle(&Still(&world),[-105.47,-28.0,-105.56],0).unwrap();
    let goal=plan::Goal {cell:B};
    let mut first=None;
    // The five words from the independent post-ladder search (KT10).
    for (sprint,jump,yaw,ticks) in [(true,false,298.125,29),(true,true,270.0,23),
        (true,true,348.75,24),(true,true,354.375,9),(false,false,0.0,73)] {
        let input=Input {forward:true,sprint,jump,yaw,..Default::default()};
        for _ in 0..ticks {
            let before=state.bounding_box();
            law::try_tick(&mut state,&world,&input).unwrap();
            if goal.touched_boxes(before,state.bounding_box()) {first.get_or_insert(state.t);}
        }
    }
    assert_eq!(first,Some(158));
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
    assert_eq!(touch(&states),Some(583),"the sentence must reach B for the first time at its last tick");
    assert!(states.iter().all(|s|s.health==20.0),"no damage along the witness");
    assert_eq!(states[0].main_supporting_block,Some(A));
    assert_eq!(digest(&states),0xd0f6b1ddbc5cacc8,"every tick's complete state bits are frozen");
    // This compact spelling is also a sentence of the EXISTING tiny programs,
    // not a new executor capability. Certify that sentence through the existing
    // planner replay interface and compare its exact endpoint with the raw replay.
    let steps=parts().iter().map(|part|plan::Step {
        program:match part.keys.as_str() {"F"=>18,"FJ"=>21,"FS"=>27,"FSJ"=>30,"FSG"=>33,keys=>panic!("no existing program for {keys}")},
        args:words::Arguments {refined_yaw:Some(part.yaw),..Default::default()},ticks:part.ticks,lane:plan::Lane::Replay,
    }).collect::<Vec<_>>();
    let certificate=plan::certify(&Still(&world),&words::tiny_programs(),&states[0],&steps,&plan::Goal {cell:B},Some(A),false).unwrap();
    assert_eq!((certificate.ticks,certificate.clock),(583,Some(580)));
    assert_eq!(certificate.end.bits(),states[583].bits());
    let origin=states.iter().rposition(|s| s.on_ground && s.main_supporting_block==Some(A)).unwrap();
    let peak=states[382..=401].iter().map(|s|s.position.y).fold(f64::NEG_INFINITY,f64::max);
    let mut record=format!("# Witness record\n\nGenerated by `hard_track_witness` with HARD_RECORD=1.\n\n- Total from settled A to first B touch: {} ticks.\n- Last sample on A: {origin}; detector clock: {} ticks (same subtraction as plan::certify).\n- Complete state FNV-1a digest, including the start and every tick: `{:016x}`. A second full replay compares every state bit.\n- Existing tiny-program mapping: F=18, FJ=21, FS=27, FSJ=30, FSG=33. The same sentence passes plan::certify and has the identical complete endpoint bits.\n- Minimum health: 20; damage: 0; refusals: none; world writes: none.\n- Main rebound apex: {peak:.15}; exit support top -37; vertical margin {:.15} blocks.\n\n| Event | Tick | Position | Velocity |\n|---|---:|---|---|\n",states.len()-1,states.len()-1-origin,digest(&states),peak+37.0);
    for (name,tick) in [("settled A",0),("launch preparation",315),("ice launch",316),("landing",328),("south drop",360),("slime contact",382),("rebound exit",401),("ladder launch",448),("suspended ladder catch",461),("B",583)] {
        let s=&states[tick];
        record.push_str(&format!("| {name} | {tick} | {:?} | {:?} |\n",s.position,s.velocity));
    }
    if std::env::var_os("HARD_RECORD").is_some() { std::fs::write(root().join("jobs/hard-track/witness.md"),record).unwrap(); }
}

/// Measure local timing slack by moving just the named switch/pulse, preserving
/// every later phase boundary and the mission horizon. This is replay, not search.
#[test]
fn timing_certificates() {
    let world=build();
    let mut record=String::from("# Timing replays\n\nGenerated by `cargo test --offline -p plan --test hard_track timing_certificates -- --nocapture` with HARD_RECORD=1.\n\nShift moves the named input boundary, keeping all later boundaries fixed. Success means B by tick 583 with full health. These are local perturbations of one sentence, not a population sample or a proof about every possible control.\n\n| Input boundary | Shift | B tick | Health minimum | Slime exit support | Final ladder caught |\n|---|---:|---:|---:|---|---|\n");
    let mut arrivals=Vec::new(); let mut misses=0;
    for (name,before,after,expected) in [("climb to accelerator",1,2,[Some(583),Some(583),Some(583)]),("ice takeoff pulse",3,5,[None,Some(583),Some(583)]),("slime west turn",6,7,[None,Some(583),Some(579)]),("ladder takeoff pulse",8,10,[None,Some(583),Some(580)]),("final climb release",10,11,[Some(583),Some(583),Some(583)])] {
        for (index,shift) in [-1i32,0,1].into_iter().enumerate() {
            let mut p=parts();
            p[before].ticks=(p[before].ticks as i32+shift) as u32;
            p[after].ticks=(p[after].ticks as i32-shift) as u32;
            let states=sentence(&world,&p,false);
            assert_eq!(touch(&states),expected[index],"frozen timing outcome for {name} {shift:+}");
            match touch(&states) {Some(t)=>arrivals.push(t),None=>misses+=1};
            let hp=states.iter().map(|s|s.health as u32).min().unwrap();
            let exit=states.iter().any(|s|s.on_ground && s.position.y==-37.0 && s.position.x < -74.0);
            let ladder=states.iter().any(|s|s.on_climbable && s.position.z > -93.0 && s.position.y > -36.0);
            let row=format!("| {name} | {shift:+} | {} | {hp} | {exit} | {ladder} |\n",touch(&states).map_or("none".into(),|t|t.to_string()));
            print!("{row}"); record.push_str(&row);
        }
    }
    arrivals.sort();
    let percentile=|p:usize|arrivals[(p*arrivals.len()).div_ceil(100).saturating_sub(1)];
    record.push_str(&format!("\nSuccessful perturbations: n={}, min={}, median={}, p90={}, p99={}, max={} ticks; failed perturbations: {misses}. Duplicate zero shifts are intentional positive controls. No mean is used.\n",arrivals.len(),arrivals[0],percentile(50),percentile(90),percentile(99),arrivals.last().unwrap()));
    if std::env::var_os("HARD_RECORD").is_some() { std::fs::write(root().join("jobs/hard-track/timing.md"),record).unwrap(); }
}
fn root()->std::path::PathBuf { std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..") }

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
    let world=build();
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
    if std::env::var_os("HARD_RECORD").is_some() { std::fs::write(root().join("jobs/hard-track/bounds.md"),text).unwrap(); }
}

#[test]
fn the_required_mechanics_cannot_be_removed_from_the_witness() {
    let cases=[("blue ice",[-104,-29,-107],[-84,-29,-106],"stone"),
        ("low ceiling",[-104,-26,-107],[-86,-26,-106],"air"),
        ("slime rebound",[-74,-48,-99],[-71,-48,-96],"stone"),
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
    if std::env::var_os("HARD_RECORD").is_some() { std::fs::write(root().join("jobs/hard-track/controls.md"),text).unwrap(); }
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
    if std::env::var_os("HARD_RECORD").is_some() { std::fs::write(root().join("jobs/hard-track/planner.md"),text).unwrap(); }
}

