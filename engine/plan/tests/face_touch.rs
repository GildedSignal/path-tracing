use law::Aabb;
use plan::{Face, Goal};

#[test]
fn shared_face_vectors() {
    for line in include_str!("../../../bot/tests/fixtures/face-touch.txt").lines() {
        let v: Vec<_> = line.split_whitespace().collect();
        let n: Vec<f64> = v[..12].iter().map(|s| s.parse().unwrap()).collect();
        let a = Aabb::new(n[0], n[1], n[2], n[3], n[4], n[5]);
        let b = Aabb::new(n[6], n[7], n[8], n[9], n[10], n[11]);
        let cell = [v[12].parse().unwrap(), v[13].parse().unwrap(), v[14].parse().unwrap()];
        let face = if v[15] == "any" { None } else { Some(Face::parse(v[15]).unwrap()) };
        assert_eq!(Goal { cell, face }.touches_box(plan::goal::hull(a, b)), v[16].parse::<bool>().unwrap(), "{line}");
    }
}

#[test]
fn distance_is_to_the_selected_square() {
    let state = law::PlayerState { position: law::Vec3::new(2.0, 3.0, 0.5), ..Default::default() };
    let body = state.bounding_box();
    let west = Goal { cell: [0, 0, 0], face: Some(Face::West) };
    let east = Goal { face: Some(Face::East), ..west };
    assert_eq!(west.gap(&state), (body.min.x.powi(2) + 2.0_f64.powi(2)).sqrt());
    assert_eq!(east.gap(&state), ((body.min.x - 1.0).powi(2) + 2.0_f64.powi(2)).sqrt());
    assert!(west.gap(&state) > east.gap(&state));
    assert_eq!(Goal { face: Some(Face::Up), ..west }.center(), law::Vec3::new(0.5, 1.0, 0.5));
}
