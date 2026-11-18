//! Check whether a fixed label, gait phase and yaw determine the contact outcome.
use law::{Block, Input, codec::Interner, contact::Outcome};
use plan::source::{label,GeneratorSuperflatSource};

#[test]
fn same_aim_and_phase_can_have_two_contact_outcomes() {
    let world=GeneratorSuperflatSource::new(-61,Block::full(),[-64,-64,-64],[64,20,64]);
    let start=plan::settle(&plan::Still(&world),[0.5,-60.0,0.5],0).unwrap();
    let mut codec=Interner::default();
    let expected_label=label(&mut codec,&world,&start).unwrap();
    let input=Input {forward:true,yaw:plan::lattice::yaw(0),..Default::default()};
    let mut outcomes=Vec::new();
    for vz in [0.0,0.25] {
        let mut state=start.clone();
        state.velocity.x=2.0f64.powi(-25);
        state.velocity.z=vz;
        assert_eq!(label(&mut codec,&world,&state).unwrap(),expected_label);
        let mut plain=state.clone();
        law::try_tick(&mut plain,&world,&input).unwrap();
        let (_,_,axes)=law::contact::tick(&mut state,&world,&input).unwrap();
        assert_eq!(plain.bits(),state.bits());
        let x=axes.iter().find(|a|a.axis==law::Axis::X).unwrap();
        println!("CONTACT label={expected_label:?} yaw_index=0 start_vz={vz} outcome={:?} vx_bits={:016x}",x.outcome,state.velocity.x.to_bits());
        outcomes.push(x.outcome);
    }
    assert!(matches!(outcomes[0],Outcome::Free));
    assert!(matches!(outcomes[1],Outcome::Hold{..}));
}

#[test]
fn retarget_stop_can_touch_outside_continuing_gait_window() {
    use plan::{Goal,Kit,PlanOutcome,store::{GeneratorStore,SPRINT_JUMP,SPRINT}};
    use words::Arguments;
    let world=GeneratorSuperflatSource::new(-61,Block::full(),
        [-3_000_000,-64,-3_000_000],[3_000_000,20,3_000_000]);
    let start=plan::settle(&plan::Still(&world),[0.5,-60.0,0.5],0).unwrap();
    let store=GeneratorStore::default();let mut codec=Interner::default();
    let first=plan::solve(&start,Goal {cell:[58006,-61,128405]},Kit::BareLegs,&world,&store,&mut codec,u64::MAX);
    let PlanOutcome::Ready(first)=first.outcome else {panic!("missing predecessor")};
    let mut state=first.words[0].end.materialize();
    let goal=Goal {cell:[-388,-61,-317]};
    let d=goal.center().minus(state.position);
    let yaw=plan::lattice::bracket(d.x,d.z)[0];
    let label=plan::source::label(&mut codec,&world,&state).unwrap();
    let k=store.prepare(label,&world,&state,SPRINT_JUMP,yaw).unwrap();
    let cycle=k.cycle_start(&state);
    let end=k.transient_ticks as u64+plan::events::horizon(&cycle,goal,k.cycle_increments.iter().copied(),k.cycle_displacement);
    let args=Arguments {refined_yaw:Some(plan::lattice::yaw(yaw)),..Default::default()};
    let programs=words::tiny_programs();
    let mut continued=state.clone();
    let mut projected=None;
    let mut hits=None;
    // The 1643-tick cut and the 23-tick stop are the original command's
    // output. Compare both continuations at native precision.
    for t in 1..=end.max(1666) {
        let before=continued.bounding_box();
        let input=programs[SPRINT_JUMP as usize].select(&world,&continued,(t-1) as u32,args).unwrap().1;
        law::try_tick(&mut continued,&world,&input).unwrap();
        if [0,2].iter().all(|&a|goal.axis_sides(before,continued.bounding_box(),a)==(true,true)) {
            projected.get_or_insert(t);
        }
        if t<=1666 {
            let before=state.clone();
            let program=if t<=1643 {SPRINT_JUMP} else {SPRINT};
            let tick=if t<=1643 {t-1} else {t-1644};
            let input=programs[program as usize].select(&world,&state,tick as u32,args).unwrap().1;
            law::try_tick(&mut state,&world,&input).unwrap();
            if goal.touched(&before,&state) {hits.get_or_insert(t);}
        }
    }
    println!("RETARGET_APPROACH yaw={yaw} through={end} continuing_horizontal_touch={projected:?} switched_native_touch={hits:?}");
    assert_eq!(hits,Some(1666));
    assert_eq!(projected,None);
}
