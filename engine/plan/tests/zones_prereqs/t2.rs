use law::{PlayerState, Vec3};
use plan::goal::Goal;

// P2's literal 0.3 slab premise, checked against the actual detector predicate.
#[test]
fn t2_binade() {
    let mut state = PlayerState::default();
    state.position = Vec3::new(-0.301000005, 0.0, 0.5);
    let goal = Goal { cell: [0, 0, 0] };
    let literal_slab = state.position.x + 0.3f64 > goal.zone().min.x;
    let actual = goal.touched(&state, &state);
    println!("x={:?} literal_radius={:?} actual_radius={:?} literal_slab={} touched={} body_max={:?} zone_min={:?}",
        state.position.x, 0.3f64, (0.6f32 / 2.0f32) as f64,
        literal_slab, actual, state.bounding_box().max.x, goal.zone().min.x);
    assert_eq!(literal_slab, actual, "P2 inverse-stop slab premise");
}
