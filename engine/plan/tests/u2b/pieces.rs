//! Frozen before the first run. Included by u2.rs once the compiler API exists.
//! The adapter may change; this oracle and its assertions must not.
use super::*;
use super::u2b_premises::{fixture,pin_key};

fn exact_without_uses(s:&PlayerState)->Vec<u64> {
    let mut key=pin_key(s);
    for &(field,_) in law::state_reads::NON_MONOTONE_FIELDS {s.push_field_bits(field,&mut key);}
    key
}
fn native(w:&impl World,start:&PlayerState,x:f64,held:&Input,stop:&impl Fn(&PlayerState,u32)->bool)->(Vec<u128>,PlayerState) {
    let mask=law::state_reads::NON_MONOTONE_FIELDS.iter().fold(0u128,|m,(f,_)|m | (1u128<<*f as u8));
    let mut state=start.clone();state.position.x=x;let mut signature=Vec::new();let mut pinned=false;
    for tick in 1..=80 {
        let before=state.clone();law::try_tick(&mut state,w,held).unwrap();
        if !pinned {
            signature.push(words::segment::joint_events(&before,&state).0 & !mask);
            if state.horizontal_collision {signature.push(u128::MAX);pinned=true;}
        }
        if stop(&state,tick) {return (signature,state);}
    }
    panic!("native word did not reach its exit")
}
#[test]
fn u2_pieces_computed_and_closed() {
    let mut patterns=Vec::new();
    for step in [false,true] {for yaw in [270.0,275.625] {
        let (w,start,input)=fixture(step,yaw);
        patterns.push((if step {"step"} else {"gap"},Scene(w),start,input,95.5,99.5,if step {8} else {5},0));
    }}
    let (mut ice,start,input)=fixture(false,270.0);
    ice.fill([100,63,-5],[120,63,5],block("ice"));
    patterns.push(("glm-ice",Scene(ice),start,input,95.5,99.5,2,1));
    let w=scene();let mut start=plan::settle(&plan::Still(&w),[-86.5,-28.0,-101.5],0).unwrap();
    start.velocity.x=0.24;start.velocity.z=0.06;
    patterns.push(("hardtrack-lane-wall",w,start,Input {forward:true,sprint:true,yaw:0.0,..Default::default()},-86.75,-86.25,2,2));
    for (name,w,start,input,lo,hi,expected,kind) in patterns {
        let stop=|s:&PlayerState,t:u32|match kind {
            0=>s.on_ground && s.position.x>100.0,
            1=>s.main_supporting_block.is_some_and(|p|p[0]>=100 && p[1]==63),
            _=>s.on_ground && t>1,
        };
        // learn_for_gate invokes the production compiler; it must not contain
        // native prediction, samples, a terrain-name switch, or stored answers.
        let family=super::learn_for_gate(&w,&start,(lo,hi),input,&stop);
        assert_eq!(family.pieces.len(),expected,"{name} yaw={}",input.yaw);
        assert_eq!(family.pieces.first().unwrap().lo,ordered(lo));
        assert_eq!(family.pieces.last().unwrap().hi,ordered(hi));
        for adjacent in family.pieces.windows(2) {assert_eq!(adjacent[0].hi+1,adjacent[1].lo);}
        for piece in &family.pieces {
            if let Some([a,b])=&piece.pin {assert_eq!(pin_key(a),pin_key(b),"pin has undeclared differences: {name}");}
        }
        for power in [6,8,10,12,14,16,18] {
            let n=1u64<<power;
            let mut inputs:Vec<_>=(0..=n).map(|i|ordered(lo+(hi-lo)*i as f64/n as f64)).collect();
            for piece in family.pieces.iter().skip(1) {for delta in -8i64..=8 {
                let b=piece.lo.checked_add_signed(delta).unwrap();
                if ordered(lo)<=b && b<=ordered(hi) {inputs.push(b);}
            }}
            inputs.sort_unstable();inputs.dedup();let mut partition=Vec::new();
            for bits in inputs {
                let x=unordered(bits);let (signature,end)=native(&w,&start,x,&input,&stop);
                let index=family.pieces.partition_point(|p|p.hi<bits);let piece=&family.pieces[index];
                assert!(piece.lo<=bits && bits<=piece.hi);
                assert_eq!(signature,piece.signature,"partition at {name} yaw={} x={x:.17}",input.yaw);
                if partition.last()!=Some(&signature) {partition.push(signature);}
                let image=piece.predict(bits);
                assert!(image.iter().any(|p|exact_without_uses(p)==exact_without_uses(&end)),"exit at {name} yaw={} x={x:.17}",input.yaw);
                assert!(piece.uses.contains(&start,&end),"resource interval at {name} x={x:.17}");
            }
            assert_eq!(partition,family.pieces.iter().map(|p|p.signature.clone()).collect::<Vec<_>>(),"sampled partition {name} 2^{power}");
            println!("U2B_PIECES pattern={name} yaw={} samples=2^{power} computed={} expected={expected} cut_neighbourhood_ulps=8 exit_error_ulp=0 pin_error_ulp=0",input.yaw,family.pieces.len());
        }
    }
}
