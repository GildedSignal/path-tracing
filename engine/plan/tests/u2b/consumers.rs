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
