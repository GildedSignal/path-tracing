//! S203 gate: frozen after the native oracle measured ice=2, lane-wall=2.
//! Silent clamps are min(P2,W) inside a free piece (proposal section 3).
use super::*;
use super::s203::{fixtures,run,NativeKey};
use super::u2b_premises::pin_key;
use law::contact::Outcome;
fn exact(s:&PlayerState)->Vec<u64> {
    let mut b=pin_key(s);for &(f,_) in law::state_reads::NON_MONOTONE_FIELDS {s.push_field_bits(f,&mut b);}b
}
#[test]
fn u2_pins_point_and_hold() {
    let mut witness=Vec::new();
    for (name,w,start,input,lo,hi,expected,kind) in fixtures() {
        let stop=|s:&PlayerState,t:u32|match kind {0=>s.on_ground && s.position.x>100.0,
            1=>s.on_ground && t>1 && (s.position.x>100.0 || s.horizontal_collision),
            2=>s.main_supporting_block.is_some_and(|p|p[0]>=100 && p[1]==63),_=>s.on_ground && t>1};
        let family=plan::families::Family::learn(&w,&start,0,(lo,hi),input,80,stop).unwrap();
        assert_eq!(family.pieces.len(),expected,"{name} yaw={} z={}",input.yaw,start.position.z);
        assert_eq!(family.pieces.first().unwrap().lo,ordered(lo));assert_eq!(family.pieces.last().unwrap().hi,ordered(hi));
        for a in family.pieces.windows(2) {assert_eq!(a[0].hi+1,a[1].lo);}
        for power in [6,8,10,12,14,16,18] {
            let n=1u64<<power;let mut inputs:Vec<_>=(0..=n).map(|i|ordered(lo+(hi-lo)*i as f64/n as f64)).collect();
            for piece in &family.pieces {
                for edge in [piece.lo,piece.hi] {for d in -8i64..=8 {if let Some(b)=edge.checked_add_signed(d) {if ordered(lo)<=b && b<=ordered(hi) {inputs.push(b);}}}}
                if matches!(piece.pin_outcome,Some(Outcome::Hold {..})) {for d in 0..256 {if piece.lo+d<=piece.hi {inputs.push(piece.lo+d);}if piece.hi>=piece.lo+d {inputs.push(piece.hi-d);}}}
            }
            inputs.sort_unstable();inputs.dedup();let mut partition=Vec::new();let mut merged_x=None;
            let mut second=if name=="gap" && input.yaw==275.625 {Some(w.0.clone())} else {None};
            if let Some(w)=&mut second {w.fill([115,64,-5],[115,66,5],law::Block::full());}
            for bits in inputs {
                let native=run(&w,&start,unordered(bits),&input,kind,false);
                let piece=&family.pieces[family.pieces.partition_point(|p|p.hi<bits)];assert!(piece.lo<=bits && bits<=piece.hi);
                let key=NativeKey {events:piece.signature.clone(),outcomes:piece.outcomes.clone()};assert_eq!(native.key,key,"partition at {name} x={:.17}",unordered(bits));
                if partition.last()!=Some(&key) {partition.push(key);}
                assert!(piece.predict(bits).iter().any(|p|exact(p)==exact(&native.end)),"exit at {name} yaw={} x={:.17}",input.yaw,unordered(bits));
                assert!(piece.uses.contains(&start,&native.end),"uses at {name} x={:.17}",unordered(bits));
                if let Some((before,pinned,contact))=&native.pin {
                    match contact.outcome {
                        Outcome::Clamp {plane}=>{
                            let expected=if contact.wanted>0.0 {plane-(0.6f32/2.0) as f64} else {plane+(0.6f32/2.0) as f64};
                            assert_eq!(pinned.position.x.to_bits(),expected.to_bits());
                            if let Some([a,b])=&piece.pin {assert_eq!(pin_key(a),pin_key(b),"clamp endpoints {name}");}
                        },
                        Outcome::Hold {plane,..}=>{
                            assert_eq!(pinned.position.x.to_bits(),before.position.x.to_bits());
                            let plane=plane.expect("a pin hold must name its preceding collider plane");
                            let wall=if contact.wanted>0.0 {plane-(0.6f32/2.0) as f64} else {plane+(0.6f32/2.0) as f64};
                            assert!((pinned.position.x-wall).abs()<=law::contact::CONTACT_ZERO);
                        },
                        Outcome::Free=>panic!("horizontal pin has no free-axis contact"),
                    }
                    if let Some(second)=&second {
                        let mut after=native.end.clone();for _ in 0..60 {law::try_tick(&mut after,second,&input).unwrap();}
                        if let Some(x)=merged_x {assert_eq!(after.position.x.to_bits(),x,"second clamp did not merge");}else {merged_x=Some(after.position.x.to_bits());}
                    }
                }
            }
            assert_eq!(partition,family.pieces.iter().map(|p|NativeKey {events:p.signature.clone(),outcomes:p.outcomes.clone()}).collect::<Vec<_>>());
            println!("S203_GATE pattern={name} yaw={} z={} power={power} pieces={} expected={expected} exit_error_ulp=0",input.yaw,start.position.z,family.pieces.len());
        }
        if name=="gap" && input.yaw==275.625 {
            for (bits,kind,expected) in [(0x405818d3c15831d9,1,0x40592cccccc00000),(0x40581c7285449b7c,2,0x40592cccccbfffff)] {
                let coordinate=ordered(f64::from_bits(bits));let p=&family.pieces[family.pieces.partition_point(|p|p.hi<coordinate)];
                assert_eq!(p.pin_outcome.unwrap().kind(),kind);let native=run(&w,&start,f64::from_bits(bits),&input,0,false);
                assert_eq!(native.pin.as_ref().unwrap().1.position.x.to_bits(),expected);witness.push((bits,kind,expected));
            }
        }
    }
    assert_eq!(witness.len(),2);println!("S203_WITNESS {witness:x?}");
}
