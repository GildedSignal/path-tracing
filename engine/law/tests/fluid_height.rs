use law::{Block, BoxWorld, Fluid, FluidKind, Input, PlayerState, Vec3, World, tick};

fn cell(kind: FluidKind, raw_amount: u8) -> Block {
    Block {
        air: false,
        fluid: Fluid { kind, raw_amount: Some(raw_amount), ..Fluid::default() },
        ..Block::default()
    }
}

#[test]
fn vanilla_fluid_surface_height() {
    for row in include_str!("data/fluid-heights.csv").lines().skip(1) {
        let mut fields = row.split(',');
        let (kind, other) = match fields.next().unwrap() {
            "water" => (FluidKind::Water, FluidKind::Lava),
            "lava" => (FluidKind::Lava, FluidKind::Water),
            kind => panic!("unexpected fluid {kind}"),
        };
        let amount = fields.next().unwrap().parse().unwrap();
        let _form = fields.next().unwrap();
        let above = fields.next().unwrap();
        let expected = u32::from_str_radix(fields.next().unwrap(), 16).unwrap();
        let mut world = BoxWorld::default();
        world.set([0, 0, 0], cell(kind, amount));
        match above {
            "empty" => {}
            "same-source" => world.set([0, 1, 0], cell(kind, 8)),
            "same-flowing" => world.set([0, 1, 0], cell(kind, 1)),
            "different" => world.set([0, 1, 0], cell(other, 8)),
            above => panic!("unexpected above state {above}"),
        }
        let fluid = world.block([0, 0, 0]).fluid;
        assert_eq!(fluid.surface_height(&world, [0, 0, 0]).to_bits(), expected, "{row}");
    }
}

#[test]
fn raw_surface_drives_contact_and_fall_reset() {
    let mut world = BoxWorld::default();
    world.set([0, 0, 0], cell(FluidKind::Water, 1));
    let start = Vec3::new(0.5, 0.5, 0.5);
    let end = Vec3::new(0.7, 0.5, 0.5);
    let contacts_water = |world: &BoxWorld| {
        let mut player = PlayerState { position: start, ..PlayerState::default() };
        tick(&mut player, world, &Input::default());
        player.in_water
    };
    assert!(!world.fall_reset_clip(start, end));
    assert!(!contacts_water(&world));

    world.set([0, 1, 0], cell(FluidKind::Water, 8));
    assert!(world.fall_reset_clip(start, end));
    assert!(contacts_water(&world));

    world.set([0, 1, 0], Block::default());
    assert!(!world.fall_reset_clip(start, end));
    assert!(!contacts_water(&world));
}
