use law::{PlayerState, World, codec::state_answers, state_reads::{StateField as F, StateReads}};
use plan::{flight::track, settle, Still};
use words::{Arguments, tiny_programs};

#[test]
fn t4_food() {
    let lines = include_str!("../../../world/tests/fixtures/tracks.txt");
    let track = track(lines.lines().find(|l| l.split_whitespace().next()==Some("straight800")).unwrap()).unwrap();
    let world = &track.world;
    assert_eq!(world.difficulty(), 0);
    let a=track.a;
    let base=settle(&Still(world), [a[0] as f64+0.5,a[1] as f64+1.0,a[2] as f64+0.5],0).unwrap();
    let mask=StateReads(1u128 << F::food_level as u8);
    let programs=tiny_programs(); let program=&programs[30];
    let mut answers=Vec::new();
    for (food,sat,exh) in [(7,0.0,3.99),(6,0.0,0.0),(20,5.0,0.0)] {
        let mut start=base.clone(); start.food_level=food; start.saturation=sat; start.exhaustion=exh; start.health=20.0;
        let bits=start.bits();
        let mut known=vec![true;bits.len()]; known[8]=false;
        assert_eq!(PlayerState::from_observed_bits(&bits,&known).unwrap_err().reason,"unknown word");
        let mut state=PlayerState::from_observed_bits(&bits,&vec![true;bits.len()]).unwrap();
        answers.push(state_answers(&state,mask));
        let mut wraps=0;
        for n in 0..240 {
            let prior=state.exhaustion;
            let input=program.select(world,&state,n,Arguments::default()).unwrap().1;
            law::try_tick(&mut state,world,&input).unwrap();
            if state.exhaustion < prior {wraps+=1;}
            assert_eq!(state.food_level,food,"food tick {n}");
            assert_eq!(state.sprinting,food>6,"guard tick {n}");
            assert_eq!(state_answers(&state,mask),answers.last().unwrap().clone());
        }
        if food>6 { assert!(wraps>0,"no exhaustion wrap"); }
        println!("food={food} ticks=240 wraps={wraps} guard={} PASS",food>6);
    }
    assert_ne!(answers[0],answers[1]); assert_eq!(answers[0],answers[2]);
    for health in [0.0,20.0] { for sat in [0.0,20.0] { for exh in [0.0,3.99] {
        let mut s=base.clone(); s.health=health; s.saturation=sat; s.exhaustion=exh;
        let b=s.bits(); let round=PlayerState::from_observed_bits(&b,&vec![true;b.len()]).unwrap(); assert_eq!(round.bits(),b);
    } } }
    println!("observed resource perturbations and unknown food refusal PASS");
}
