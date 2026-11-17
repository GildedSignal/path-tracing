use law::Input;
#[test]
fn stair_recurrence_is_measured() {
    let stair=world::model(&world::BlockState {name:"minecraft:stone_stairs".into(),properties:vec![("facing".into(),"east".into()),("half".into(),"bottom".into()),("shape".into(),"straight".into()),("waterlogged".into(),"false".into())]}).unwrap().clone();
    let mut w=law::BoxWorld::default();w.difficulty=0;w.fill([-5,62,-2],[0,63,2],law::Block::full());
    for x in 1..=24 {w.set([x,63+x-1,0],stair.clone());}
    let mut state=plan::settle(&plan::Still(&w),[0.5,64.0,0.5],0).unwrap();let counted=plan::source::MeteredWorld::new(&w);
    let mut seen=std::collections::HashMap::new();let mut recurrence=None;let mut transitions=0;let mut previous=0;
    for _ in 0..300 {
        law::try_tick(&mut state,&counted,&Input {forward:true,sprint:true,yaw:270.0,..Default::default()}).unwrap();
        let step=state.position.x.floor() as i32;if step>24 || state.position.y<64.0 {break;}
        if step!=previous && step>0 {
            transitions+=1;previous=step;
            let mut relative=state.clone();relative.t=0;relative.position.x-=step as f64;relative.position.y-=step as f64;
            relative.main_supporting_block=relative.main_supporting_block.map(|p|[p[0]-step,p[1]-step,p[2]]);
            if let Some((tick,old_step))=seen.insert(relative.bits(),(state.t,step)) {recurrence=Some((state.t-tick,step-old_step));break;}
        }
    }
    assert!(transitions>1,"staircase measurement never advanced");
    println!("U2_STAIRS transitions={transitions} native_ticks={} bit_exact_recurrence={recurrence:?} gait_certified=false reason=requires_recurrence_and_corridor_certificate",counted.ticks());
}
