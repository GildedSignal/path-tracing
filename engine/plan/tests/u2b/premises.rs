use super::*;
use law::state_reads::{StateField as F, NON_MONOTONE_FIELDS};

pub(super) fn fixture(step:bool,yaw:f32)->(law::BoxWorld,PlayerState,Input) {
    let source=plan::source::GeneratorSuperflatSource::new(63,law::Block::full(),[80,60,-10],[400,100,10]);
    let held=Input {forward:true,sprint:true,jump:true,yaw,..Default::default()};
    let mut warm=plan::settle(&plan::Still(&source),[90.5,64.0,0.5],0).unwrap();
    let mut phase=warm.clone();
    for _ in 0..600 {law::try_tick(&mut warm,&source,&held).unwrap();if warm.on_ground {phase=warm.clone();}}
    let mut w=law::BoxWorld::default();w.difficulty=0;
    w.fill([90,60,-5],[120,63,5],law::Block::full());
    if step {w.fill([100,64,-5],[100,64,5],law::Block::full());}
    else {w.fill([100,63,-5],[100,63,5],law::Block::default());}
    phase.position.x=95.5;phase.position.z=0.5;phase.main_supporting_block=Some([95,63,0]);
    (w,phase,held)
}

fn pre_pin(w:&impl World,start:&PlayerState,x:f64,input:&Input)->(Vec<u128>,PlayerState,bool) {
    let mut s=start.clone();s.position.x=x;let mut signature=Vec::new();
    let mask=NON_MONOTONE_FIELDS.iter().fold(0u128,|m,(f,_)|m | (1u128<<*f as u8));
    for _ in 0..80 {
        let before=s.clone();law::try_tick(&mut s,w,input).unwrap();
        signature.push(words::segment::joint_events(&before,&s).0 & !mask);
        if s.horizontal_collision {signature.push(u128::MAX);return (signature,s,true);}
        if s.on_ground && s.position.x>100.0 {return (signature,s,false);}
    }
    panic!("fixture did not terminate")
}

// Only the four ruled resources leave the exact state; support is relative.
pub(super) fn pin_key(s:&PlayerState)->Vec<u64> {
    let mut out=Vec::new();
    for &f in F::ALL {
        if matches!(f,F::exhaustion|F::saturation|F::food_level|F::fall_distance)
            || NON_MONOTONE_FIELDS.iter().any(|(declared,_)|*declared==f) {continue;}
        if f==F::main_supporting_block {
            out.push(s.main_supporting_block.is_some() as u64);
            if let Some(p)=s.main_supporting_block {
                let body=[s.position.x.floor() as i32,s.position.y.floor() as i32,s.position.z.floor() as i32];
                out.extend((0..3).map(|a|(p[a]-body[a]) as u64));
            }
        } else {s.push_field_bits(f,&mut out);}
    }
    out
}

#[test]
fn u2_monotone_premise() {
    let mut w=law::BoxWorld::default();w.difficulty=0;w.fill([90,63,-3],[110,63,3],law::Block::full());
    let mut start=plan::settle(&plan::Still(&w),[100.0,64.0,0.5],0).unwrap();
    start.velocity.x=0.25+2.0f64.powi(-47);
    let map=plan::events::Map::new(vec![start.velocity.x]);
    let mut previous=None;let mut steps=std::collections::BTreeSet::new();
    for offset in 0..64 {
        let x=f64::from_bits(start.position.x.to_bits()+offset);let mut s=start.clone();s.position.x=x;
        law::try_tick(&mut s,&w,&Input::default()).unwrap();
        assert_eq!(map.advance(x,1).to_bits(),s.position.x.to_bits());
        if let Some(before)=previous {assert!(s.position.x.to_bits()>=before);steps.insert(s.position.x.to_bits()-before);}
        previous=Some(s.position.x.to_bits());
    }
    assert_eq!(steps,std::collections::BTreeSet::from([0,2]));
    println!("U2B_MONOTONE tie_steps={steps:?} worst_decrease_ulp=0 map_error_ulp=0");
}

#[test]
fn u2_pin_erases_coordinate_premise() {
    let mut failures=Vec::new();
    for step in [false,true] {for yaw in [270.0,275.625] {
        let (w,start,input)=fixture(step,yaw);
        let mut pins=std::collections::HashMap::<Vec<u128>,(f64,Vec<u64>,PlayerState)>::new();
        for i in 0..=1u64<<18 {
            let x=95.5+4.0*i as f64/(1u64<<18) as f64;
            let (signature,s,pinned)=pre_pin(&w,&start,x,&input);
            if !pinned {continue;}
            let key=pin_key(&s);
            if let Some((first,old,representative))=pins.get(&signature) {
                if *old!=key {
                    let mut different=Vec::new();
                    for &f in F::ALL {
                        let mut a=Vec::new();let mut b=Vec::new();
                        representative.push_field_bits(f,&mut a);s.push_field_bits(f,&mut b);
                        if a!=b {different.push((f,a,b));}
                    }
                    println!("U2B_PIN_FAILURE step={step} yaw={yaw} first={first:.17} x={x:.17} first_bits={:x} x_bits={:x} tick={} fields={different:?}",first.to_bits(),x.to_bits(),s.t-start.t);
                    failures.push((step,yaw));break;
                }
            } else {pins.insert(signature,(x,key,s));}
        }
        println!("U2B_PIN step={step} yaw={yaw} sampled_pin_classes={} failed={}",pins.len(),failures.contains(&(step,yaw)));
    }}
    assert!(failures.is_empty(),"a pin retains undeclared state information: {failures:?}");
}

#[test]
#[ignore = "Frozen U2b pin-closure falsifier: run explicitly and record RED"]
fn u2_pin_cut_neighbourhood_witness() {
    let (w,start,input)=fixture(false,275.625);
    let a=f64::from_bits(0x405818d3c15831d9);
    let b=f64::from_bits(0x40581c7285449b7c);
    let (sa,pa,ap)=pre_pin(&w,&start,a,&input);
    let (sb,pb,bp)=pre_pin(&w,&start,b,&input);
    assert!(ap && bp);assert_eq!(sa,sb);assert_eq!(pa.t,pb.t);
    println!("U2B_PIN_WITNESS inputs={:x},{:x} tick={} x={:x},{:x} flag={},{}",a.to_bits(),b.to_bits(),pa.t-start.t,pa.position.x.to_bits(),pb.position.x.to_bits(),pa.minor_horizontal_collision,pb.minor_horizontal_collision);
    assert_eq!(pin_key(&pa),pin_key(&pb),"proposal requires pin endpoints equal except declared flag");
}
