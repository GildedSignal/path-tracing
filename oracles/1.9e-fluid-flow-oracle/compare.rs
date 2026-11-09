use law::{Block, BoxWorld, Fluid, FluidFlowBlock, FluidKind, Input, PlayerState, Vec3, World, tick};
use std::{env, fs, process::ExitCode};

fn decode(name: &str) -> Block {
    let (kind, amount, blocks_motion, solid_face) = match name {
        "air" => (FluidKind::Empty, None, false, false),
        "stone" => (FluidKind::Empty, None, true, true),
        "ice" | "slab" => (FluidKind::Empty, None, true, false),
        "wet-slab" => (FluidKind::Water, Some(8), true, false),
        _ => {
            let kind = match name.as_bytes()[0] {
                b'w' => FluidKind::Water,
                b'l' => FluidKind::Lava,
                _ => panic!("unknown cell {name}"),
            };
            (kind, Some(name[1..].parse().unwrap()), false, false)
        }
    };
    Block {
        fluid: Fluid { kind, raw_amount: amount, raw_falling: amount.map(|_| false), ..Fluid::default() },
        fluid_flow: Some(FluidFlowBlock { blocks_motion, solid_faces: [solid_face; 4] }),
        air: name == "air",
        ..Block::default()
    }
}

fn fixture(fields: &[&str; 19]) -> BoxWorld {
    let kind = match fields[1] {
        "water" => FluidKind::Water,
        "lava" => FluidKind::Lava,
        _ => panic!("unexpected fluid"),
    };
    let mut world = BoxWorld::default();
    world.set([0, 0, 0], Block {
        air: false,
        fluid: Fluid {
            kind,
            raw_amount: Some(fields[2].parse().unwrap()),
            raw_falling: Some(fields[3].parse().unwrap()),
            ..Fluid::default()
        },
        ..Block::default()
    });
    let sides = [[0, -1], [1, 0], [0, 1], [-1, 0]];
    for (layer, y) in [0, -1, 1].into_iter().enumerate() {
        for (index, [x, z]) in sides.into_iter().enumerate() {
            world.set([x, y, z], decode(fields[4 + layer * 4 + index]));
        }
    }
    world
}

fn bits(v: Vec3) -> [u64; 3] {
    [v.x.to_bits(), v.y.to_bits(), v.z.to_bits()]
}

fn movement_control(mut world: BoxWorld) {
    let mut center = world.block([0, 0, 0]).clone();
    center.fluid.height = 4.0f32 / 9.0;
    center.fluid.flow = Vec3::new(-1.0, 0.0, 0.0);
    world.set([0, 0, 0], center.clone());
    let check = |world: &BoxWorld, push: f64| {
        let mut player = PlayerState { position: Vec3::new(0.5, 0.0, 0.5), ..PlayerState::default() };
        tick(&mut player, world, &Input::default());
        assert!(player.in_water);
        assert_eq!(player.water_height.to_bits(), ((4.0f32 / 9.0) as f64).to_bits());
        assert_eq!(bits(player.position), bits(Vec3::new(0.5 + push, 0.0, 0.5)));
        assert_eq!(player.velocity.x.to_bits(), (push * 0.8f32 as f64).to_bits());
        assert_eq!(player.velocity.z.to_bits(), 0);
    };
    check(&world, 0.014);

    // The legacy snapshot supplies the evaluated height and a westward current.
    let mut evaluated = BoxWorld::default();
    center.fluid.raw_amount = None;
    center.fluid.raw_falling = None;
    evaluated.set([0, 0, 0], center);
    check(&evaluated, -0.014);
}

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let path = args.next().expect("oracle CSV");
    assert!(args.next().is_none(), "expected only the oracle CSV path");
    let raw = fs::read_to_string(path).expect("read oracle CSV");
    let mut rows = raw.lines();
    assert_eq!(rows.next(), Some("case,kind,amount,falling,n,e,s,w,n_below,e_below,s_below,w_below,n_above,e_above,s_above,w_above,x,y,z"));
    let mut first = None;
    let mut differences = 0;
    let mut axes = [0; 3];
    let mut cases = 0;
    let mut gradient = None;
    for row in rows {
        let fields: [&str; 19] = row.split(',').collect::<Vec<_>>().try_into().expect("nineteen fields");
        let world = fixture(&fields);
        let actual = bits(world.block([0, 0, 0]).fluid.current(&world, [0, 0, 0]));
        let expected = std::array::from_fn(|i| u64::from_str_radix(fields[16 + i], 16).unwrap());
        if actual != expected {
            differences += 1;
            for (i, (a, e)) in actual.into_iter().zip(expected).enumerate() {
                axes[i] += usize::from(a != e);
            }
            first.get_or_insert((row, actual));
        }
        if fields[0] == "level-gradient" {
            assert!(gradient.replace(world).is_none());
        }
        cases += 1;
    }
    assert_eq!(cases, 1034);
    println!("cases={cases} mismatches={differences} x/y/z={axes:?}");
    if let Some((row, actual)) = first {
        println!("first mismatch oracle={row} Rust={actual:016x?}");
        ExitCode::FAILURE
    } else {
        movement_control(gradient.expect("level-gradient row"));
        println!("ordinary raw and evaluated movement controls passed");
        ExitCode::SUCCESS
    }
}
