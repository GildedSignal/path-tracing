use super::*;
#[test]
fn u2_family_piece_wave_and_consume() {
    let w=plan::source::GeneratorSuperflatSource::new(63,law::Block::full(),[80,60,-10],[400,100,10]);
    let mut start=plan::settle(&plan::Still(&w),[95.5,64.0,0.5],0).unwrap();start.velocity.x=0.1;
    let mut codec=Interner::default();let mut store=plan::joints::JointStore::default();
    let args=Arguments {max_ticks:4,refined_yaw:Some(270.0),..Default::default()};let id=program_id(false,false);
    let edges=store.learn_family(&w,&mut codec,&start,0,(95.5,95.50001),0,id,args,|_,t|t==4).unwrap();
    assert_eq!(edges.len(),1);
    let prediction=store.consume(edges[0],&start,0).unwrap().mapped_exits().unwrap();
    assert_eq!(prediction.len(),1);let middle=prediction[0].clone();
    let next=store.learn_family(&w,&mut codec,&middle,0,(middle.position.x,middle.position.x+0.00002),0,id,args,|_,t|t==4).unwrap();
    let end=store.consume(next[0],&middle,0).unwrap().mapped_exits().unwrap()[0].clone();
    let goal=Goal {cell:[end.position.x.floor() as i32,64,0]};
    let ticks=store.work.law_ticks;let field=store.wave(goal).clone();
    assert_eq!(store.work.law_ticks,ticks,"acting wave must not replay the law");
    let row=field.families.iter().find(|r|r.pattern==0 && r.lo<=plan::joints::ordered(start.position.x) && plan::joints::ordered(start.position.x)<=r.hi).unwrap();
    assert_eq!((row.lower,row.upper),(8,Some(8)));assert_eq!(row.next,Some(next[0]));
    let mut shifted=start.clone();shifted.position.x+=0.000005;
    let read=store.consume(edges[0],&shifted,0).unwrap();
    let mut native=shifted.clone();for _ in 0..4 {law::try_tick(&mut native,&w,&Input {forward:true,yaw:270.0,..Default::default()}).unwrap();}
    assert!(read.mapped_exits().unwrap().iter().any(|s|s.bits()==native.bits()));
    let mut aim=shifted.clone();aim.yaw=71.0;aim.pitch=23.0;aim.input=Input {back:true,..Default::default()};
    assert!(store.consume(edges[0],&aim,0).is_ok(),"aim history is not an entry key");
    aim.velocity.x=aim.velocity.x.next_up();assert!(store.consume(edges[0],&aim,0).is_err());
    let mut edited=law::BoxWorld::default();edited.difficulty=0;
    let changed=SnapshotSource {world:&edited,ticks:(0,u64::MAX),min:[80,60,-10],max:[400,100,10]};
    assert!(store.edit(&codec,[95,63,0],(0,u64::MAX),&changed)>0);
    assert!(store.consume(edges[0],&start,0).is_err());
    assert!(store.wave(goal).families.iter().all(|r|r.pattern!=0));
    println!("U2B_FAMILY interval_pullback=8 points_and_families=true shifted_exact=true dead_aim_reused=true velocity_distinct=true local_invalidation=true acting_ticks=0");
}
#[test]
fn u2_point_route_entry_has_no_aim_memory() {
    let w=plan::source::GeneratorSuperflatSource::new(63,law::Block::full(),[80,60,-10],[400,100,10]);
    let start=plan::settle(&plan::Still(&w),[95.5,64.0,0.5],0).unwrap();let goal=Goal {cell:[98,64,0]};
    let mut codec=Interner::default();let mut store=plan::joints::JointStore::default();
    let id=program_id(false,false);let args=Arguments {max_ticks:80,refined_yaw:Some(270.0),..Default::default()};
    store.learn(&w,&mut codec,&start,start.position,0,id,&program(false,false),args,|a,b,_|goal.touched(a,b)).unwrap();
    let expected=store.route(&start,goal).unwrap();
    let mut changed=start.clone();changed.yaw=71.0;changed.pitch=23.0;changed.input=Input {back:true,..Default::default()};
    assert_eq!(EntryTemplate::new(&start,start.position,0).key,EntryTemplate::new(&changed,start.position,0).key);
    let actual=store.route(&changed,goal).expect("point route entry must ignore dead aim history");
    assert_eq!(actual.ticks,expected.ticks);
    for _ in 0..actual.ticks {law::try_tick(&mut changed,&w,&Input {forward:true,yaw:270.0,..Default::default()}).unwrap();}
    assert_eq!(actual.end.bits(),changed.bits());
    println!("U2B_POINT_ENTRY dead_aim_reused=true exact_native=true T={}",actual.ticks);
}
#[test]
fn u2_point_wave_joins_aim_aliases_with_numeric_guards() {
    let w=plan::source::GeneratorSuperflatSource::new(63,law::Block::full(),[80,60,-10],[400,100,10]);
    let start=plan::settle(&plan::Still(&w),[95.5,64.0,0.5],0).unwrap();
    let mut codec=Interner::default();let mut store=plan::joints::JointStore::default();
    let id=program_id(false,false);let args=Arguments {max_ticks:80,refined_yaw:Some(270.0),..Default::default()};
    let mid=Goal {cell:[98,64,0]};let goal=Goal {cell:[101,64,0]};
    let first=store.learn(&w,&mut codec,&start,start.position,0,id,&program(false,false),args,|a,b,_|mid.touched(a,b)).unwrap();
    let mut alias=store.end(first).clone();alias.yaw=71.0;alias.pitch=23.0;alias.input=Input {back:true,..Default::default()};
    store.learn(&w,&mut codec,&alias,alias.position,0,id,&program(false,false),args,|a,b,_|goal.touched(a,b)).unwrap();
    let route=store.route(&start,goal).expect("the wave must join equivalent point entries");assert_eq!(route.words.len(),2);
    let mut native=start.clone();for _ in 0..route.ticks {law::try_tick(&mut native,&w,&Input {forward:true,yaw:270.0,..Default::default()}).unwrap();}
    assert_eq!(route.end.bits(),native.bits());assert!(goal.touched(&native,&native));
    let mut guarded=start.clone();guarded.food_level-=1;assert!(store.route(&guarded,goal).is_none(),"exact numeric guard must survive aim aliasing");
    println!("U2B_POINT_WAVE alias_join=true numeric_guard_exact=true native_equal=true T={}",route.ticks);
}
#[test]
fn u2_family_rounding_tie_is_not_identity() {
    let mut w=law::BoxWorld::default();w.difficulty=0;
    let mut start=PlayerState::default();start.position=Vec3::new(100.0,64.0,0.5);start.on_ground=false;start.main_supporting_block=None;
    start.velocity=Vec3::new(2.0f64.powi(-47),0.0,0.1);
    let hi=f64::from_bits(start.position.x.to_bits()+64);
    let family=plan::families::Family::learn(&w,&start,0,(start.position.x,hi),Input::default(),1,|_,t|t==1).unwrap();
    assert_eq!(family.pieces.len(),1);
    for offset in 0..=64 {
        let mut native=start.clone();native.position.x=f64::from_bits(start.position.x.to_bits()+offset);
        law::try_tick(&mut native,&w,&Input::default()).unwrap();
        let bits=plan::joints::ordered(f64::from_bits(start.position.x.to_bits()+offset));
        assert!(family.pieces[0].predict(bits).iter().any(|p|p.bits()==native.bits()),"rounding tie at offset {offset}: an unchanged endpoint cannot imply identity");
    }
    println!("U2B_FAMILY_TIE consecutive_ulps=65 native_equal=true one_piece=true");
}
