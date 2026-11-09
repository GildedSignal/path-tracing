use law::{Block, BoxWorld, Fluid, FluidFlowBlock, FluidKind, Input, PlayerState, Vec3, tick};

fn raw_water() -> Fluid {
    Fluid {
        kind: FluidKind::Water,
        raw_amount: Some(4),
        raw_falling: Some(false),
        ..Fluid::default()
    }
}

#[test]
#[should_panic(expected = "raw fluid current needs FALLING at [0, 0, 0]")]
fn tick_refuses_a_missing_falling_property() {
    let mut world = BoxWorld::default();
    world.set([0, 0, 0], Block {
        fluid: Fluid { raw_falling: None, ..raw_water() },
        ..Block::default()
    });
    let mut player = PlayerState { position: Vec3::new(0.5, 0.0, 0.5), ..PlayerState::default() };
    tick(&mut player, &world, &Input::default());
}

#[test]
#[should_panic(expected = "needs a valid amount at [0, 0, 0]")]
fn raw_falling_alone_cannot_use_evaluated_flow() {
    let fluid = Fluid { raw_amount: None, ..raw_water() };
    fluid.current(&BoxWorld::default(), [0, 0, 0]);
}

#[test]
#[should_panic(expected = "needs a valid amount at [1, -1, 0]")]
fn drop_refuses_evaluated_height_in_place_of_own_height() {
    let mut world = BoxWorld::default();
    world.set([1, -1, 0], Block {
        fluid: Fluid { kind: FluidKind::Water, height: 1.0, ..Fluid::default() },
        ..Block::default()
    });
    raw_water().current(&world, [0, 0, 0]);
}

#[test]
#[should_panic(expected = "raw fluid current needs block predicates at [0, 0, -1]")]
fn current_refuses_unknown_motion_blocking() {
    let mut world = BoxWorld::default();
    world.set([0, 0, -1], Block::full());
    raw_water().current(&world, [0, 0, 0]);
}

#[test]
#[should_panic(expected = "raw fluid current needs block predicates at [0, 1, -1]")]
fn falling_current_refuses_unknown_above_face() {
    let mut world = BoxWorld::default();
    world.set([0, 1, -1], Block::full());
    let falling = Fluid { raw_falling: Some(true), ..raw_water() };
    falling.current(&world, [0, 0, 0]);
}

#[test]
#[should_panic(expected = "raw fluid neighborhood is unavailable at [16, 0, 0]")]
fn current_refuses_unavailable_neighborhood() {
    let mut world = BoxWorld::default();
    world.unloaded_chunks.push([1, 0]);
    raw_water().current(&world, [15, 0, 0]);
}

#[test]
fn falling_current_reads_the_face_named_by_the_scan_direction() {
    let mut world = BoxWorld::default();
    let falling = Fluid { raw_falling: Some(true), ..raw_water() };
    let wall = |solid_faces| Block {
        fluid_flow: Some(FluidFlowBlock { blocks_motion: true, solid_faces }),
        ..Block::full()
    };
    world.set([0, 0, -1], wall([false, false, true, false]));
    assert_eq!(falling.current(&world, [0, 0, 0]), Vec3::default());
    world.set([0, 0, -1], wall([true, false, false, false]));
    assert_eq!(falling.current(&world, [0, 0, 0]), Vec3::new(0.0, -1.0, 0.0));
}
