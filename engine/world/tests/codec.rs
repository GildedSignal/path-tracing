use law::codec::{Interner, LegsRole};
use world::{BlockState, model};

fn state(raw: &str) -> BlockState {
    let (name, tail) = raw.split_once('[').unwrap_or((raw, ""));
    let mut properties = tail.trim_end_matches(']').split(',').filter(|p| !p.is_empty())
        .map(|p| { let (k,v) = p.split_once('=').unwrap(); (k.to_owned(), v.to_owned()) }).collect::<Vec<_>>();
    properties.sort();
    BlockState { name: format!("minecraft:{name}"), properties }
}

#[test]
fn native_air_buttons_and_plates_are_leg_equivalent() {
    let mut codec = Interner::with_digest(|_| 0);
    let air = model(&state("air")).unwrap();
    for name in ["stone_button[face=floor,facing=north,powered=false]", "oak_button[face=wall,facing=east,powered=true]",
        "stone_pressure_plate[powered=false]", "oak_pressure_plate[powered=true]", "light_weighted_pressure_plate[power=7]"] {
        let block = model(&state(name)).unwrap();
        for role in [LegsRole::Support02, LegsRole::Support05, LegsRole::BodyFeet, LegsRole::Inside] {
            assert_eq!(codec.legs(air, true, role).unwrap(), codec.legs(block, true, role).unwrap(), "{name} {role:?}");
        }
    }
}

#[test]
fn native_local_inside_query_ignores_a_distant_cobweb() {
    use law::codec::planes::SectionRoles;
    let mut codec = Interner::with_digest(|_| 0);
    let roles = [LegsRole::Support02, LegsRole::Support05, LegsRole::BodyFeet, LegsRole::Inside];
    let air = model(&state("air")).unwrap();
    let flower = model(&state("dandelion")).unwrap();
    let web = model(&state("cobweb")).unwrap();
    let plain = roles.map(|role| codec.legs(air, true, role).unwrap());
    let flowers = roles.map(|role| codec.legs(flower, true, role).unwrap());
    let webs = roles.map(|role| codec.legs(web, true, role).unwrap());
    assert_eq!(plain, flowers);
    let section = SectionRoles::from_classes(plain, |cell| Some((if cell == 4000 { webs } else { flowers }, true)));
    assert_eq!(section.query(LegsRole::Inside, &[(0, 1)]), Ok(vec![]));
    assert_eq!(section.query(LegsRole::Inside, &[(4000/64, 1 << (4000%64))]), Ok(vec![(4000/64, 1 << (4000%64))]));
}
