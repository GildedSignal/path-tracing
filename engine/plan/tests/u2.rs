use law::{PlayerState,Vec3,Input,World};
use law::codec::{Interner,proof::{Source,Refusal,SnapshotSource}};
use plan::{Goal,source::{CorridorSource,CorridorResult,MeteredWorld},store::GeneratorStore,joints::{EntryTemplate,Status}};
use words::{Arguments,Program,Predicate,Row};

struct Scene(law::BoxWorld);
impl World for Scene {
    fn block(&self,p:law::BlockPos)->&law::Block {self.0.block(p)}
    fn loaded(&self,p:law::BlockPos)->bool {self.0.loaded(p)}
    fn difficulty(&self)->u8 {self.0.difficulty()}
    fn min_y(&self)->i32 {self.0.min_y()}
    fn has_fluid(&self)->bool {self.0.has_fluid()}
    fn has_inside_effects(&self)->bool {self.0.has_inside_effects()}
    fn visit_collision_cells(&self,b:law::Aabb,lo:law::BlockPos,hi:law::BlockPos,visit:impl FnMut(law::BlockPos)->bool)->bool {self.0.visit_collision_cells(b,lo,hi,visit)}
}
impl Source for Scene {
    fn cell(&self,t:u64,p:law::BlockPos,m:u8)->Result<law::codec::Canonical,Refusal> {
        SnapshotSource {world:self,ticks:(0,u64::MAX),min:[-120,-64,-120],max:[-50,20,-50]}.cell(t,p,m)
    }
    fn environment(&self,t:u64,q:&law::footprint::EnvironmentRead)->Result<law::codec::Canonical,Refusal> {
        SnapshotSource {world:self,ticks:(0,u64::MAX),min:[-120,-64,-120],max:[-50,20,-50]}.environment(t,q)
    }
}
impl CorridorSource for Scene {
    fn corridor_query(&self,_:&mut Interner,_:law::codec::LabelId,lo:law::BlockPos,_:law::BlockPos)->CorridorResult {
        CorridorResult::Exception {first_pos:lo,exceptions:1}
    }
}
fn block(raw:&str)->law::Block {
    let (name,tail)=raw.split_once('[').unwrap_or((raw,""));
    let mut properties=tail.trim_end_matches(']').split(',').filter(|p|!p.is_empty()).map(|p|{let(k,v)=p.split_once('=').unwrap();(k.into(),v.into())}).collect::<Vec<_>>();properties.sort();
    world::model(&world::BlockState {name:format!("minecraft:{}",name.trim_start_matches("minecraft:")),properties}).unwrap().clone()
}
fn scene()->Scene {
    let mut w=law::BoxWorld::default();w.difficulty=0;
    w.fill([-120,-64,-120],[-50,-64,-50],block("bedrock"));
    w.fill([-120,-63,-120],[-50,-62,-50],block("dirt"));
    w.fill([-120,-61,-120],[-50,-61,-50],block("grass_block[snowy=false]"));
    for line in include_str!("../../../lab/tracks/hardtrack.build.txt").lines() {
        let line=line.split('#').next().unwrap().trim();if line.is_empty(){continue;}
        let f:Vec<_>=line.split_whitespace().collect();let at=|i:usize|->law::BlockPos {std::array::from_fn(|a|f[i+a].parse().unwrap())};
        let (lo,hi,b)=match f[0] {"/fill"=>(at(1),at(4),f[7]),"/setblock"=>(at(1),at(1),f[4]),_=>panic!("command")};
        let b=block(b);for x in lo[0]..=hi[0] {for y in lo[1]..=hi[1] {for z in lo[2]..=hi[2] {w.set([x,y,z],b.clone());}}}
    }
    Scene(w)
}
fn program(sprint:bool,jump:bool)->Program {Program {rows:vec![Row {when:Predicate::Always,input:Input {forward:true,sprint,jump,..Default::default()}}]}}
fn program_id(sprint:bool,jump:bool)->u16 {words::program::tiny_program_id(0,sprint,if jump {words::program::HeldPredicate::Always} else {words::program::HeldPredicate::Never},words::program::HeldPredicate::Never)}
fn hold(w:&impl World,start:&PlayerState,sprint:bool,jump:bool,yaw:f32,budget:u32)->Option<(PlayerState,u32)> {
    let mut state=start.clone();let mut airborne=!state.on_ground;
    for t in 1..=budget {
        law::try_tick(&mut state,w,&Input {forward:true,sprint,jump,yaw,..Default::default()}).ok()?;
        if state.on_climbable && airborne {return Some((state,t));}
        if !state.on_ground {airborne=true;} else if airborne {return Some((state,t));}
    }None
}
fn on_wall(s:&PlayerState)->bool {
    let b=s.bounding_box();s.on_ground && s.position.y== -31.0 && b.min.x< -84.0 && b.max.x> -85.0 && b.max.z> -99.0 && b.min.z< -95.0
}
fn on_ladder(s:&PlayerState)->bool {s.on_climbable && (-84.0..-82.0).contains(&s.position.x) && s.position.z.floor()== -92.0}

#[test]
fn u2_two_speeds_and_exact_key() {
    let w=scene();let base=plan::settle(&plan::Still(&w),[-86.5,-28.0,-101.5],0).unwrap();
    let mut a=base.clone();a.velocity.x=0.24;a.velocity.z=0.06;
    let mut b=a.clone();b.velocity.x=0.36;
    let origin=Vec3::new(-87.0,-29.0,-102.0);
    let ka=EntryTemplate::new(&a,origin,0);let kb=EntryTemplate::new(&b,origin,0);
    assert_ne!(ka.key,kb.key);assert_eq!(ka.state(ka.s).unwrap().bits(),a.bits());assert_eq!(kb.state(kb.s).unwrap().bits(),b.bits());
    let ea=hold(&w,&a,true,false,0.0,80).unwrap().0;let eb=hold(&w,&b,true,false,0.0,80).unwrap().0;
    assert!(on_wall(&ea));assert!(!on_wall(&eb));
    println!("U2_SOUNDNESS keys_distinct=true endpoints_y={},{}",ea.position.y,eb.position.y);
    let mut codec=Interner::default();for i in 0..100_000u64 {codec.intern_state(&i.to_le_bytes()).unwrap();}
    assert!(codec.digests().is_empty());println!("U2_STATE_ARENA entries=100000 key_bytes={} cell_classes=0",codec.state_bytes_used());
}

#[test]
fn u2_pullback_prices_and_hash() {
    assert_eq!(plan::joints::pullback(0,100,(25,45),|s|2*s+5),Some((10,20)));
    assert_eq!(plan::joints::pullback(0,100,(25,45),|s|205-2*s),Some((80,90)));
    assert_eq!(plan::joints::price(None,Some(5),20,8),12);
    assert_eq!(plan::joints::price(Some(30),Some(8),20,8),22);
    let mut store=plan::joints::JointStore::default();let mut field=store.wave(Goal {cell:[0;3]}).clone();
    field.set_price(b"tool".to_vec(),None,Some(5),20,8);assert_eq!(field.prices[b"tool".as_slice()],12);
    field.partitions=vec![vec![plan::joints::InstanceInterval {lo:0.0,hi:1.0,t:Some(20),via:None},plan::joints::InstanceInterval {lo:1.0f64.next_up(),hi:2.0,t:Some(19),via:None}]];
    assert!(!field.slide_want(0,0.5,(0.0,1.0)));assert!(field.slide_want(0,0.5,(0.0,2.0)));
    for file in ["law/src/sin.bin","world/data/models.tsv","world/data/blocks.tsv","law/src/movement.rs"] {assert!(law::LAW_HASH_INPUTS.lines().any(|n|n==file));}
    assert_eq!(law::codec::SCHEMA,law::LAW_HASH);
    println!("U2_HASH value={:02x?} inputs={}",law::LAW_HASH,law::LAW_HASH_INPUTS.lines().count());
}

#[test]
fn u2_statuses_inventory_queue_and_one_mechanism() {
    let mut w=scene();let mut codec=Interner::default();let store=GeneratorStore::default();assert!(store.bind_codec(&mut codec));
    let start=plan::settle(&plan::Still(&w),[-86.5,-28.0,-101.5],0).unwrap();let args=Arguments {max_ticks:1,..Default::default()};
    let result=store.joints.borrow_mut().learn(&w,&mut codec,&start,start.position,0,program_id(false,false),&program(false,false),args,|_,_,_|false);
    assert!(matches!(result,Err(Status::Searched {work:1})));
    let mut refused=start.clone();refused.effects=true;
    let result=store.joints.borrow_mut().learn(&w,&mut codec,&refused,start.position,0,program_id(false,false),&program(false,false),args,|_,_,_|false);
    assert!(matches!(result,Err(Status::Refused {..})));assert_eq!(store.joints.borrow().attempts.len(),2);
    store.joints.borrow_mut().inventory(&w,&mut codec,[-91,-31,-99],[-89,-28,-97]).unwrap();
    let cell=[-90,-30,-98];w.0.set(cell,law::Block::full());store.joints.borrow_mut().world_edit(&w,&mut codec,cell,(0,u64::MAX)).unwrap();
    assert!(store.joints.borrow().queue.iter().any(|d|d.face==Some(cell)));
    let (source,from)={let source=plan::source::GeneratorSuperflatSource::new(-61,law::Block::full(),[-2000,-64,-2000],[2000,20,2000]);let from=plan::settle(&plan::Still(&source),[0.5,-60.0,0.5],0).unwrap();(source,from)};
    let flat_store=GeneratorStore::default();let mut flat_codec=Interner::default();let goal=Goal {cell:[1000,-61,0]};
    let flat=plan::solve(&from,goal,plan::Kit::BareLegs,&source,&flat_store,&mut flat_codec,u64::MAX);
    let plan::PlanOutcome::Ready(flat)=flat.outcome else {panic!("flat did not compose")};
    let work=flat_store.joints.borrow().work;assert!(work.waves>0 && work.edge_reads==flat.words.len() as u64 && work.settled>0);
    // The very same WordStep record and backward operation serve learned edges.
    let mut entry=start.clone();entry.velocity.x=0.24;entry.velocity.z=0.06;
    let (end,t)=hold(&w,&entry,true,false,0.0,80).unwrap();
    let id=store.joints.borrow_mut().learn(&w,&mut codec,&entry,entry.position,0,program_id(true,false),&program(true,false),Arguments {max_ticks:80,refined_yaw:Some(0.0),..Default::default()},|_,_,n|n==t).unwrap();
    let goal=Goal {cell:[end.position.x.floor() as i32,-32,end.position.z.floor() as i32]};let counted=MeteredWorld::new(&w);
    let result=plan::solve(&entry,goal,plan::Kit::BareLegs,&counted,&store,&mut codec,u64::MAX);
    assert!(matches!(result.outcome,plan::PlanOutcome::Ready(_)));assert_eq!(counted.ticks(),0);
    let borrowed=store.joints.borrow();assert!(borrowed.work.waves>0 && borrowed.work.edge_reads>0 && borrowed.work.settled>0);assert!(borrowed.consume(id,&entry,0).is_ok());
    println!("U2_ONE_MECHANISM flat={work:?} joint={:?} statuses_retained=2 inventory_enqueued=true",borrowed.work);
}

#[test]
fn u2_certified_wave_no_replay_and_invalidation() {
    let mut w=scene();let start=plan::settle(&plan::Still(&w),[-86.5,-28.0,-101.5],0).unwrap();
    let mut start=start;start.velocity.x=0.24;start.velocity.z=0.06;
    let mut codec=Interner::default();let store=GeneratorStore::default();assert!(store.bind_codec(&mut codec));
    let expected=hold(&w,&start,true,false,0.0,80).unwrap();let args=Arguments {max_ticks:80,refined_yaw:Some(0.0),..Default::default()};
    let id=store.joints.borrow_mut().learn(&w,&mut codec,&start,Vec3::new(-87.0,-29.0,-102.0),0,program_id(true,false),&program(true,false),args,|_,_,t|t==expected.1).unwrap();
    assert_eq!(store.joints.borrow().end(id).bits(),expected.0.bits());
    assert!(store.joints.borrow().consume(id,&start,0).is_ok());
    let mut wrong=start.clone();wrong.velocity.x=0.36;assert!(store.joints.borrow().consume(id,&wrong,0).is_err());
    let p=expected.0.position;let goal=Goal {cell:[p.x.floor() as i32,-32,p.z.floor() as i32]};
    let metered=MeteredWorld::new(&w);let result=plan::solve(&start,goal,plan::Kit::BareLegs,&metered,&store,&mut codec,u64::MAX);
    let plan::PlanOutcome::Ready(answer)=result.outcome else {panic!("no certified route")};
    assert_eq!(answer.ticks,expected.1 as u64);assert_eq!(metered.ticks(),0);assert_eq!(answer.words.len(),1);
    let mut previous=start.clone();for t in 0..answer.words[0].ticks {let input=program(true,false).select(&w,&previous,t,args).unwrap().1;law::try_tick(&mut previous,&w,&input).unwrap();}
    assert_eq!(previous.bits(),answer.end.bits());
    let outside=[-60,-21,-60];w.0.set(outside,law::Block::full());assert_eq!(store.joints.borrow_mut().edit(&codec,outside,(0,u64::MAX),&w),0);
    let support=expected.0.main_supporting_block.unwrap();w.0.set(support,law::Block::default());
    assert_eq!(store.joints.borrow_mut().edit(&codec,support,(0,u64::MAX),&w),1);
    assert!(matches!(store.joints.borrow().patterns[0].plateaus[0].status,Status::Searched {..}));
    assert!(store.joints.borrow_mut().route(&start,goal).is_none());
    println!("U2_CERTIFICATE exact_replay=true acting_ticks=0 outside_broken=0 inside_broken=1");
}

fn toolkit()->Vec<(u16,Program,Arguments)> {
    let mut out=Vec::new();
    for (sprint,jump) in [(false,false),(true,false),(true,true),(false,true)] {for yaw in 0..64 {
        out.push((program_id(sprint,jump),program(sprint,jump),Arguments {max_ticks:80,refined_yaw:Some(yaw as f32*360.0/64.0),..Default::default()}));
    }}out
}

#[test]
fn u2_kill10_face_relation() {
    let mut w=scene();let mut codec=Interner::default();let words=toolkit();let store=GeneratorStore::default();
    assert!(store.bind_codec(&mut codec));
    let mut inventory=plan::faces::FaceInventory::build(&w,&mut codec,[-106,-38,-108],[-81,-25,-87]).unwrap();
    let faces:Vec<_>=inventory.iter().cloned().collect();let mut counts=[0usize;2];let mut compiled=0;let mut native_ticks=0;
    // Every candidate top face in the inventory is compiled without a target.
    // Only after generation do the named regions select the required assertions.
    for face in &faces {
        if face.climbable {continue;}
        let p=(face.bounds.min.plus(face.bounds.max)).scale(0.5);
        let Ok(start)=plan::settle(&plan::Still(&w),[p.x,p.y,p.z],0) else {continue;};
        if !start.on_ground || start.position.y!=p.y {continue;}
        let table=plan::joints::MirroredReach::compile(&w,&start,p,0,&words);compiled+=1;native_ticks+=table.ticks;
        for arc in &table.arcs {
            for (side,(lo,hi)) in [([-90,-29,-103],[-84,-29,-102]),([-85,-32,-99],[-85,-32,-96])].into_iter().enumerate() {
                if !(0..3).all(|a|face.cell[a]>=lo[a] && face.cell[a]<=hi[a]) {continue;}
                if !(if side==0 {on_wall(&arc.end)} else {on_ladder(&arc.end)}) {continue;}
                assert!(faces.iter().any(|f|table.preimages(f).any(|a|std::ptr::eq(a,arc))),"known landing missing from mirrored table");
                let pgr=words.iter().find(|(id,_,args)|*id==arc.program && args==&arc.args).unwrap();
                let id=store.joints.borrow_mut().learn(&w,&mut codec,&start,p,0,arc.program,&pgr.1,arc.args,|_,_,t|t==arc.ticks).unwrap();
                assert_eq!(store.joints.borrow().end(id).bits(),arc.end.bits());counts[side]+=1;
            }
        }
    }
    assert!(counts.into_iter().all(|n|n>0));
    // Air was inventory too. Placing a block creates new joints and repairs
    // only that cell and its dependent neighbour, not the whole box.
    let added=[-90,-30,-98];w.0.set(added,law::Block::full());let before=inventory.reads;
    let repaired=inventory.edit(&w,&mut codec,added).unwrap();
    assert!(inventory.created.iter().any(|f|f.cell==added));assert!(repaired<=2);
    println!("U2_KT10 topology=PASS lane_wall={} wall_ladder={} templates={} native_ticks={} certified_edges={} inventory_local_repair={} repair_reads={}",counts[0],counts[1],compiled,native_ticks,store.joints.borrow().edge_count(),repaired,inventory.reads-before);
}

#[test]
fn u2_tick_meter_all_entries_and_refusal() {
    let world=law::BoxWorld::default();let counted=MeteredWorld::new(&world);let recorder=law::footprint::RecordingWorld::new(&counted);
    let mut s=PlayerState::default();let input=Input::default();
    law::try_tick(&mut s,&recorder,&input).unwrap();law::tick_with_collision(&mut s,&recorder,&input);law::tick_with_contact(&mut s,&recorder,&input);
    s.effects=true;assert!(law::try_tick(&mut s,&recorder,&input).is_err());assert_eq!(counted.ticks(),4);
    println!("U2_TICK_METER calls=4 observed=4 refused=1 nested_recording=true");
}

#[test]
#[ignore = "Offline KT10 bar search; explicit gate keeps the normal test loop short"]
fn u2_kill10_post_ladder_bar() {
    let w=scene();let start=plan::settle(&plan::Still(&w),[-105.47,-28.0,-105.56],0).unwrap();
    let goal=Goal {cell:[-84,-25,-88]};let words=toolkit();let mut codec=Interner::default();
    let inventory=plan::faces::FaceInventory::build(&w,&mut codec,[-106,-38,-108],[-81,-25,-87]).unwrap();
    // Select long uniform rows from the face inventory, independently of any
    // recorded route. Their two axes supply learning order, not a reach proof.
    let mut rows=std::collections::BTreeMap::<(i32,i32,u16),Vec<i32>>::new();
    for f in inventory.iter().filter(|f|!f.climbable && f.bounds.min.y==start.position.y) {
        let class=codec.legs(w.block(f.cell),true,law::codec::LegsRole::Support02).unwrap();
        rows.entry((f.cell[1],f.cell[2],class.index())).or_default().push(f.cell[0]);
    }
    let mut lanes=Vec::new();
    for ((_,z,_),mut xs) in rows {
        if xs.len()<2 {continue;}xs.sort();let mut connects=false;
        for &x in xs.iter().rev() {
            let Ok(s)=plan::settle(&plan::Still(&w),[x as f64+0.5,start.position.y,z as f64+0.5],0) else {continue;};
            let table=plan::joints::MirroredReach::compile(&w,&s,s.position,0,&words);
            if table.arcs.iter().any(|arc|on_wall(&arc.end)) {connects=true;break;}
        }
        if connects {lanes.push((xs[0] as f64,xs[xs.len()-1] as f64+1.0,z as f64,z as f64+1.0));}
    }
    assert!(!lanes.is_empty());
    let lane_yaw=if goal.center().x>start.position.x {270.0} else {90.0};
    let mut prefixes=Vec::new();
    for &(sprint,jump) in &[(false,false),(true,false),(true,true),(false,true)] {for k in 0..64 {
        let yaw=k as f32*360.0/64.0;let mut s=start.clone();
        for n in 1..=40 {
            law::try_tick(&mut s,&w,&Input {forward:true,sprint,jump,yaw,..Default::default()}).unwrap();
            if s.on_ground && s.position.y==start.position.y && lanes.iter().any(|&(lo,hi,zlo,zhi)|s.position.x>=lo && s.position.x<=hi && s.position.z>=zlo && s.position.z<=zhi) {
                prefixes.push((s.clone(),vec![(sprint,jump,yaw,n)]));
            }
        }
    }}
    println!("U2_KT10_BAR prefixes={} lanes={lanes:?} row_axis_yaw={lane_yaw} recording_inputs_read=0",prefixes.len());
    let mut best:Option<(u64,Vec<(bool,bool,f32,u32)>)>=None;
    let mut wall_hits=0;let mut ladder_hits=0;let mut sentences=0;
    // This is learning, with finite fair toolkit order. It is intentionally
    // separate from the certified-edge wave exercised below.
    for (p0,path) in &prefixes {
        let mut s1=p0.clone();
        for n1 in 1..=60 {
            law::try_tick(&mut s1,&w,&Input {forward:true,sprint:true,jump:true,yaw:lane_yaw,..Default::default()}).unwrap();
            if !lanes.iter().any(|&(_,hi,_,_)|s1.position.x>=hi-10.0) {continue;}
            if best.as_ref().is_some_and(|(t,_)|s1.t-start.t>=*t) {break;}
            for (id,pgr,args) in &words {
                let input=pgr.rows[0].input;let yaw=args.yaw();
                let Some((s2,t2))=hold(&w,&s1,input.sprint,input.jump,yaw,args.max_ticks) else {continue;};
                if !on_wall(&s2) {continue;}wall_hits+=1;
                for (_,pgr3,args3) in &words {
                    let input3=pgr3.rows[0].input;let yaw3=args3.yaw();
                    let Some((s3,t3))=hold(&w,&s2,input3.sprint,input3.jump,yaw3,args3.max_ticks) else {continue;};
                    if !on_ladder(&s3) {continue;}ladder_hits+=1;
                    let mut s4=s3.clone();
                    for t4 in 1..=200 {
                        let before=s4.clone();law::try_tick(&mut s4,&w,&Input {forward:true,yaw:0.0,..Default::default()}).unwrap();
                        if goal.touched(&before,&s4) {
                            sentences+=1;let total=s4.t-start.t;
                            if best.as_ref().is_none_or(|(t,_)|total<*t) {
                                let mut p=path.clone();p.extend([(true,true,lane_yaw,n1),(input.sprint,input.jump,yaw,t2),(input3.sprint,input3.jump,yaw3,t3),(false,false,0.0,t4)]);
                                println!("U2_KT10_BEST T={total} path={p:?} word={id}");best=Some((total,p));
                            }break;
                        }
                    }
                }
            }
        }
    }
    let (ticks,path)=best.expect("no post-ladder sentence");let store=GeneratorStore::default();assert!(store.bind_codec(&mut codec));
    let mut state=start.clone();
    for &(sprint,jump,yaw,ticks) in &path {
        let id=store.joints.borrow_mut().learn(&w,&mut codec,&state,state.position,0,program_id(sprint,jump),&program(sprint,jump),Arguments {max_ticks:ticks,refined_yaw:Some(yaw),..Default::default()},|_,_,t|t==ticks).unwrap();
        state=store.joints.borrow().end(id).clone();
    }
    let counted=MeteredWorld::new(&w);let result=plan::solve(&start,goal,plan::Kit::BareLegs,&counted,&store,&mut codec,u64::MAX);
    let plan::PlanOutcome::Ready(answer)=result.outcome else {panic!("wave lost learned chain")};
    assert_eq!(answer.ticks,ticks);assert_eq!(answer.end.bits(),state.bits());assert_eq!(counted.ticks(),0);
    println!("U2_KT10_BAR best={ticks} human=169 margin={} inferred_total={} wall_hits={wall_hits} ladder_hits={ladder_hits} sentences={sentences} certified_edges={} acting_ticks=0 path={path:?}",169i64-ticks as i64,259+ticks,store.joints.borrow().edge_count());
    assert!(ticks<=169,"post-ladder bar missed");
}

#[test]
#[ignore = "Replays the independent search artifact; explicit gate runs after KT10 search"]
fn u2_kill10_discovered_program_current_record() {
    let text=std::fs::read_to_string("D:/path-tracing-evidence/u2/kill10-bar.log").unwrap();
    let line=text.lines().filter(|l|l.starts_with("U2_KT10_BEST T=")).min_by_key(|l|l.split("T=").nth(1).unwrap().split_whitespace().next().unwrap().parse::<u64>().unwrap()).unwrap();
    let expected:u64=line.split("T=").nth(1).unwrap().split_whitespace().next().unwrap().parse().unwrap();
    let path=line.split("path=[").nth(1).unwrap().split(']').next().unwrap();
    let path:Vec<(bool,bool,f32,u32)>=path.split("), (").map(|p| {
        let v:Vec<_>=p.trim_matches(['(',')']).split(", ").collect();(v[0].parse().unwrap(),v[1].parse().unwrap(),v[2].parse().unwrap(),v[3].parse().unwrap())
    }).collect();
    let w=scene();let start=plan::settle(&plan::Still(&w),[-105.47,-28.0,-105.56],0).unwrap();let goal=Goal {cell:[-84,-25,-88]};
    let mut codec=Interner::default();let store=GeneratorStore::default();assert!(store.bind_codec(&mut codec));
    let mut s=start.clone();let mut first_touch=None;
    for &(sprint,jump,yaw,ticks) in &path {
        let args=Arguments {max_ticks:ticks,refined_yaw:Some(yaw),..Default::default()};
        let id=store.joints.borrow_mut().learn(&w,&mut codec,&s,s.position,0,program_id(sprint,jump),&program(sprint,jump),args,|_,_,n|n==ticks).unwrap();
        for _ in 0..ticks {let before=s.clone();law::try_tick(&mut s,&w,&Input {forward:true,sprint,jump,yaw,..Default::default()}).unwrap();if goal.touched(&before,&s) {first_touch.get_or_insert(s.t-start.t);}}
        assert_eq!(store.joints.borrow().end(id).bits(),s.bits());
    }
    let counted=MeteredWorld::new(&w);let result=plan::solve(&start,goal,plan::Kit::BareLegs,&counted,&store,&mut codec,u64::MAX);
    let plan::PlanOutcome::Ready(p)=result.outcome else {panic!("no witnessed route")};
    assert_eq!(first_touch,Some(expected));assert_eq!(p.ticks,expected);assert!(expected<=169);assert_eq!(p.end.bits(),s.bits());assert_eq!(counted.ticks(),0);
    println!("U2_KT10_REPLAY best={expected} human=169 margin={} law_hash={:x?} cold_template_bytes={} cold_state_bytes={} edges={} acting_native_ticks=0 first_touch_exact=true",169-expected,law::LAW_HASH,store.joints.borrow().template_bytes(),store.joints.borrow().cold_state_bytes(),store.joints.borrow().edge_count());
}

#[test]
#[ignore = "Falsifier of U2 spec rule 3; run explicitly and record red without weakening"]
fn u2_translation_premise() {
    let mut world=law::BoxWorld::default();world.difficulty=0;
    world.fill([90,63,-3],[110,63,3],law::Block::full());
    let mut a=plan::settle(&plan::Still(&world),[100.0,64.0,0.5],0).unwrap();
    a.velocity.x=0.25+2.0f64.powi(-47);
    let mut b=a.clone();b.position.x=a.position.x.next_up();
    let before=[a.clone(),b.clone()];
    law::try_tick(&mut a,&world,&Input::default()).unwrap();
    law::try_tick(&mut b,&world,&Input::default()).unwrap();
    let ea=words::segment::joint_events(&before[0],&a);
    let eb=words::segment::joint_events(&before[1],&b);
    assert_eq!(ea,eb,"different events would not falsify the premise");
    assert_eq!(before[0].position.x.to_bits()>>52,before[1].position.x.to_bits()>>52);
    println!("U2_TRANSLATION input={:x},{:x} output={:x},{:x} events={ea:?} entry_delta={} exit_delta={}",before[0].position.x.to_bits(),before[1].position.x.to_bits(),a.position.x.to_bits(),b.position.x.to_bits(),before[1].position.x-before[0].position.x,b.position.x-a.position.x);
    assert_eq!(b.position.x-a.position.x,before[1].position.x-before[0].position.x,"same signature and binade do not imply pure translation: parity matters");
}

fn refined_count(mut evaluate:impl FnMut(f64)->Vec<u128>,lo:f64,hi:f64,divisions:u32)->(usize,Vec<u64>,u64) {
    let mut calls=0;let mut last=evaluate(lo);calls+=1;let mut previous=lo;let mut cuts=Vec::new();
    for i in 1..=divisions {
        let s=lo+(hi-lo)*i as f64/divisions as f64;let signature=evaluate(s);calls+=1;
        if signature!=last {
            let mut a=ordered(previous);let mut b=ordered(s);
            while b-a>1 {let middle=a+(b-a)/2;calls+=1;if evaluate(unordered(middle))==last {a=middle;} else {b=middle;}}
            cuts.push(b);
        }
        last=signature;previous=s;
    }
    (cuts.len()+1,cuts,calls)
}
fn ordered(x:f64)->u64 {let b=x.to_bits();if b>>63!=0 {!b} else {b^(1<<63)}}
fn unordered(b:u64)->f64 {f64::from_bits(if b>>63!=0 {b^(1<<63)} else {!b})}
fn signature(w:&impl World,mut s:PlayerState,sprint:bool,jump:bool,yaw:f32,budget:u32,mut stop:impl FnMut(&PlayerState)->bool)->Vec<u128> {
    let mut result=Vec::new();
    for _ in 0..budget {
        let before=s.clone();if let Err(e)=law::try_tick(&mut s,w,&Input {forward:true,sprint,jump,yaw,..Default::default()}) {result.push(law::codec::digest(e.0.as_bytes()));break;}
        result.push(words::segment::joint_events(&before,&s).0);if stop(&s) {break;}
    }result
}

#[test]
#[ignore = "KT11 is red; explicit gate records plateau growth without weakening the assertion"]
fn u2_kill11_refined_bisection_diagnostic() {
    let source=plan::source::GeneratorSuperflatSource::new(63,law::Block::full(),[80,60,-10],[400,100,10]);
    let mut warm=plan::settle(&plan::Still(&source),[90.5,64.0,0.5],0).unwrap();let mut phase=warm.clone();
    for _ in 0..600 {law::try_tick(&mut warm,&source,&Input {forward:true,sprint:true,jump:true,yaw:270.0,..Default::default()}).unwrap();if warm.on_ground {phase=warm.clone();}}
    let mut patterns=Vec::new();
    for gap in [false,true] {
        let mut w=law::BoxWorld::default();w.difficulty=0;w.fill([90,60,-5],[120,63,5],law::Block::full());
        if gap {w.fill([100,63,-5],[100,63,5],law::Block::default());}
        else {w.fill([100,63,-5],[120,63,5],block("ice"));}
        let mut start=phase.clone();start.position.x=95.5;start.main_supporting_block=Some([95,63,0]);
        patterns.push((if gap {"glm-gap"} else {"glm-ice"},Scene(w),start,95.5,99.5,true,true,270.0,if gap {1}else{0}));
    }
    let w=scene();let mut start=plan::settle(&plan::Still(&w),[-86.5,-28.0,-101.5],0).unwrap();start.velocity.x=0.24;start.velocity.z=0.06;
    patterns.push(("hardtrack-lane-wall",w,start,-86.75,-86.25,true,false,0.0,2));
    let mut grew=Vec::new();
    for (name,w,start,lo,hi,sprint,jump,yaw,kind) in patterns {
        let mut previous=None;
        for divisions in [64,256,1024] {
            let eval=|s:f64| {let mut state=start.clone();state.position.x=s;
                signature(&w,state,sprint,jump,yaw,80,|s|match kind {0=>s.main_supporting_block.is_some_and(|p|p[0]>=100 && p[1]==63),1=>s.on_ground && s.position.x>100.0,_=>s.on_ground && s.t>start.t+1})};
            let (count,cuts,calls)=refined_count(eval,lo,hi,divisions);
            println!("U2_KT11 pattern={name} divisions={divisions} plateaus={count} calls={calls} cuts={cuts:x?}");
            if previous.is_some_and(|old|count>old) {grew.push((name,divisions,count));}previous=Some(count);
        }
    }
    println!("U2_KT11 observed_growth={grew:?} interval_certification=RED_same_signature_is_not_translation");
    // Sampled signature counts are a diagnostic only. The explicit premise
    // falsifier is the blocking gate for interval certificates, run separately.
    assert!(grew.is_empty(),"plateau count grows under refinement: {grew:?}");
}
