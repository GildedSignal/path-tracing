use law::{Vec3, entity_contact::{ContactEntity, push_pair}};
fn e(x:f64,z:f64)->ContactEntity { ContactEntity { position:Vec3::new(x,0.0,z),velocity:Vec3::new(0.0,0.0,0.0),no_physics:false,vehicle:false,pushable:true } }
#[test]
fn direct_runtime_witness_bits_and_guards() {
    // Frozen from direct staged-jar PairPush and CheckAdv witnesses.
    let c=0.01_f32 as f64;
    for (x,z,bits) in [(f64::from_bits(c.to_bits()-1),0.0,0), (c,0.0,0x3f747ae148f5c28f), (f64::from_bits(c.to_bits()+1),0.0,0x3f747ae148f5c28f), (0.25,0.0,0x3f999999a0000000), (1.0,0.0,0x3fa99999a0000000), (4.0,0.0,0x3fa99999a0000000), (0.0,1.0,0x3fa99999a0000000)] {
        let (mut a,mut b)=(e(0.0,0.0),e(x,z)); push_pair(&mut a,&mut b,false);
        assert_eq!(if x==0.0 { b.velocity.z.to_bits() } else { b.velocity.x.to_bits() },bits,"x={x} z={z}");
    }
    for mask in 0..128 { let (mut a,mut b)=(e(0.0,0.0),e(1.0,0.0)); a.velocity=Vec3::new(2.0,3.0,4.0); b.velocity=Vec3::new(-2.0,-3.0,-4.0); a.no_physics=mask&1!=0; b.no_physics=mask&2!=0; a.vehicle=mask&4!=0; b.vehicle=mask&8!=0; a.pushable=mask&16!=0; b.pushable=mask&32!=0; let same=mask&64!=0; push_pair(&mut a,&mut b,same); let gate=mask&3==0&&!same; let impulse=0.05_f32 as f64; assert_eq!(a.velocity.x.to_bits(),(if gate&&!a.vehicle&&a.pushable {2.0-impulse} else {2.0}).to_bits()); assert_eq!(b.velocity.x.to_bits(),(if gate&&!b.vehicle&&b.pushable {-2.0+impulse} else {-2.0}).to_bits()); assert_eq!(a.velocity.y,3.0); assert_eq!(b.velocity.y,-3.0); }
}

#[test]
fn direct_mojang_oracle() {
    let path = std::env::var("ENTITY_CONTACT_ORACLE").unwrap_or_else(|_| format!("{}/../../jobs/1.9i-entity-contact/oracle.txt", env!("CARGO_MANIFEST_DIR")));
    let data = std::fs::read_to_string(path).unwrap();
    let mut count=0;
    for line in data.lines() {
        let p:Vec<_>=line.split_whitespace().collect(); assert_eq!(p.len(),10);
        let id:usize=p[0].parse().unwrap();
        let f=|i:usize| f64::from_bits(u64::from_str_radix(p[i],16).unwrap());
        let m:u32=p[3].parse().unwrap();
        let (mut a,mut b)=(e(0.0,0.0),e(f(1),f(2)));
        a.velocity=Vec3::new(0.125,-0.0,-0.375); b.velocity=Vec3::new(-0.75,2.5,0.25);
        a.no_physics=m&1!=0; b.no_physics=m&2!=0; a.vehicle=m&4!=0; b.vehicle=m&8!=0;
        a.pushable=m&16!=0; b.pushable=m&32!=0;
        push_pair(&mut a,&mut b,m&64!=0);
        for (i,v) in [a.velocity.x,a.velocity.y,a.velocity.z,b.velocity.x,b.velocity.y,b.velocity.z].into_iter().enumerate() {
            assert_eq!(v.to_bits(),f(i+4).to_bits(),"first difference case {id} component {i}");
        }
        count+=1;
    }
    assert_eq!(count,170);
}
