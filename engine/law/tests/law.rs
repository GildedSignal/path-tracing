use law::*;
fn floor(friction: f32, ceiling: bool) -> BoxWorld {
    let mut w=BoxWorld::default();
    let mut b=Block::full();
    b.friction=friction;
    w.fill([-10, -1, -10], [10, -1, 100000], b);
    if ceiling {
        w.fill([-10, 2, -10], [10, 2, 100000], Block::full());
    }
    w
}
fn player() -> PlayerState {
    PlayerState {
        on_ground:true, 
        velocity:Vec3::new(0.0, -0.0784000015258789, 0.0), 
        ..PlayerState::default()
    }
}
fn advance(s: &mut PlayerState, w: &impl World, i: &Input) {
    tick(s, w, i);
    s.food_level=20;
    s.saturation=5.0;
    s.exhaustion=0.0;
}
fn speed(friction:f32, sprint:bool, jump:bool, ceiling:bool)->f64 {
    let w=floor(friction, ceiling);
    let mut s=player();
    for _ in 0..2000 {
        let i=Input{
            forward:true, 
            sprint, 
            jump:jump&&s.on_ground, 
            ..Input::default()
        };
        advance(&mut s, &w, &i);
    }
    let start=s.position.z;
    for _ in 0..2400 {
        let i=Input{
            forward:true, 
            sprint, 
            jump:jump&&s.on_ground, 
            ..Input::default()
        };
        advance(&mut s, &w, &i);
    }
    (s.position.z-start)/2400.0*20.0
}

#[test]
fn steady_speeds() {
    for (name, f, s, j, c, expected) in [
    ("walk", 0.6, false, false, false, "4.317"), 
    ("sprint", 0.6, true, false, false, "5.612"), 
    ("sprint-jump", 0.6, true, true, false, "7.127"), 
    ("ceiling", 0.6, true, true, true, "9.734"), 
    ("ice sprint", 0.98, true, false, false, "5.404"), 
    ("ice jump", 0.98, true, true, false, "9.241"), 
    ("ice ceiling", 0.98, true, true, true, "19.345"), 
    ("blue ice ceiling", 0.989, true, true, true, "19.903")
    ] {
        let measured=speed(f, s, j, c);
        println!("{name}: {measured:.9} m/s");
        assert_eq!(format!("{measured:.3}"), expected, "{name}");
    }
}

#[test]
fn ladder_terminal_and_fall() {
    let mut w=BoxWorld::default();
    w.fill([-1, -10, -1], [1, 1000, 1], Block{
        climbable:true, air:false, ..Block::default()
    });
    let mut s=player();
    for _ in 0..40 {
        advance(&mut s, &w, &Input{
            jump:true, ..Input::default()
        });
    }
    assert_eq!(format!("{:.3}", s.velocity.y*20.0), "2.352");
    let mut s=PlayerState{
        position:Vec3::new(0.0, 1000000.0, 0.0), 
        ..PlayerState::default()
    };
    for _ in 0..2000 {
        tick(&mut s, &BoxWorld::default(), &Input::default());
    }
    assert_eq!(format!("{:.1}", -s.velocity.y*20.0), "78.4");
    let mut s=PlayerState{
        position:Vec3::new(0.0, 23.0, 0.0), 
        ..PlayerState::default()
    };
    let w=floor(0.6, false);
    for _ in 0..100 {
        tick(&mut s, &w, &Input::default());
        if s.on_ground{
            break;
        }
    }
    assert!(s.on_ground);
    assert_eq!(s.health, 0.0);
    assert_eq!(calculate_fall_damage(&PlayerState::default(), 23.0, 1.0), 20);
}

fn braking_expectations(mode: &str) -> [[i32; 7]; 4] {
    let mut result = [[0; 7]; 4];
    for row in include_str!("data/braking.tsv").lines() {
        let fields: Vec<_> = row.split_whitespace().collect();
        if fields[0] != mode {
            continue;
        }
        let friction = f32::from_bits(u32::from_str_radix(fields[1], 16).unwrap());
        let sprint = fields[2] == "true";
        let index = [(0.6f32, false), (0.6, true), (0.98, false), (0.98, true)]
            .iter().position(|entry| *entry == (friction, sprint)).unwrap();
        let k: usize = fields[3].parse().unwrap();
        result[index][k] = fields[4].parse().unwrap();
    }
    assert!(result.iter().flatten().all(|ticks| *ticks > 0));
    result
}

#[test]
fn braking_recurrence_regression() {
    let expected = braking_expectations("recurrence");
    for (row, (friction, sprint)) in [(0.6, false), (0.6, true), (0.98, false), (0.98, true)].into_iter().enumerate(){
        let w=floor(friction, false);
        let mut steady=player();
        for _ in 0..2000{
            advance(&mut steady, &w, &Input{
                forward:true, sprint, ..Input::default()
            });
        }
        for k in 0..=6{
            let mut s=steady.clone();
            let mut stopped=None;
            for t in 1..=400{
                let zeroed=s.velocity.horizontal_sqr()<9.0e-6;
                advance(&mut s, &w, &Input{
                    back:t<=k, ..Input::default()
                });
                if zeroed{
                    stopped=Some(t);
                    break;
                }
            }
            println!("friction={friction}, sprint={sprint}, reverse={k}, threshold={stopped:?}");
            assert_eq!(stopped, Some(expected[row][k as usize]));
        }
    }
}

#[test]
fn thresholds_and_ceiling(){
    let w=floor(0.6, true);
    let mut s=PlayerState{
        velocity:Vec3::new(0.001, 0.42, 0.001), 
        ..player()
    };
    tick(&mut s, &w, &Input::default());
    assert_eq!(s.velocity.x.to_bits(), 0.0f64.to_bits());
    assert!(s.vertical_collision);
    assert_eq!(s.velocity.y.to_bits(), (-0.08*0.98f32 as f64).to_bits());
}

#[test]
fn axis_tie_contacts(){
    let shape=VoxelShape::boxes(&[Aabb::new(1.0, 0.0, 1.0, 2.0, 2.0, 2.0)]);
    let b=Aabb::new(0.0, 0.0, 0.0, 0.6, 1.8, 0.6);
    let moved=law::shapes::collide_with_shapes(Vec3::new(0.5, 0.0, 0.5), b, &[shape]);
    assert_eq!(moved, Vec3::new(0.5, 0.0, 0.4));
}

#[test]
fn fluids_hooks_and_food(){
    let mut w=BoxWorld::default();
    w.fill([-2, -2, -2], [2, 5, 2], Block{
        air:false, fluid:Fluid{
            kind:FluidKind::Water, height:1.0, ..Fluid::default()
        }, ..Block::default()
    });
    let mut s=PlayerState::default();
    tick(&mut s, &w, &Input{
        forward:true, jump:true, ..Input::default()
    });
    assert!(s.in_water);
    assert!(s.velocity.y>0.0);
    assert!(s.exhaustion>0.0);
    let mut w=BoxWorld::default();
    w.set([0, 0, 0], Block{
        air:false, inside:Inside::Cobweb, ..Block::default()
    });
    let mut s=PlayerState{
        position:Vec3::new(0.5, 0.0, 0.5), 
        ..PlayerState::default()
    };
    tick(&mut s, &w, &Input::default());
    assert_eq!(s.stuck_speed_multiplier, Vec3::new(0.25, 0.05f32 as f64, 0.25));
    let w=floor(0.6, false);
    let mut s=player();
    s.food_level=6;
    tick(&mut s, &w, &Input{
        forward:true, sprint:true, ..Input::default()
    });
    assert!(!s.sprinting);
    s.exhaustion=4.0;
    tick(&mut s, &w, &Input::default());
    assert_eq!(s.saturation, 5.0);
    s.exhaustion=4.01;
    tick(&mut s, &w, &Input::default());
    assert_eq!(s.saturation, 4.0);
}

#[test]
fn java_casts_and_table(){
    assert_eq!(math::floor(f64::NAN), 0);
    assert_eq!(math::floor(f64::INFINITY), i32::MAX);
    assert_eq!(math::lfloor(f64::NEG_INFINITY), i64::MIN);
    assert_eq!(math::floor(-0.1), -1);
    assert_eq!(math::sin(0.0).to_bits(), 0);
    assert_eq!(math::cos(0.0), 1.0);
    assert_eq!(math::sin(-0.000001).to_bits(), 0);
}

#[test]
fn braking_player_tick_regression() {
    let expected = braking_expectations("player");
    for (index, (friction, sprint)) in [(0.6, false), (0.6, true), (0.98, false), (0.98, true)].into_iter().enumerate() {
        let world=floor(friction, false);
        let mut steady=player();
        for _ in 0..2000 {
            advance(&mut steady, &world, &Input {
                forward:true, sprint, ..Input::default()
            });
        }
        let mut row=Vec::new();
        for k in 0..=6 {
            let mut state=steady.clone();
            let mut stopped=None;
            for t in 1..=400 {
                let threshold=state.velocity.horizontal_sqr()<9.0e-6;
                advance(&mut state, &world, &Input {
                    back:t<=k, ..Input::default()
                });
                if threshold {
                    stopped=Some(t);
                    break;
                }
            }
            row.push(stopped.expect("threshold event"));
        }
        println!("player tick friction={friction}, sprint={sprint}, threshold ticks={row:?}");
        assert_eq!(row, expected[index]);
    }
}

#[test]
fn step_height_boundary() {
    let mut world=floor(0.6, false);
    world.set([0, 0, 1], Block::boxes(&[Aabb::new(0.0, 0.0, 0.0, 1.0, 0.5, 1.0)]));
    let mut state=PlayerState {
        position:Vec3::new(0.5, 0.0, 0.5), 
        velocity:Vec3::new(0.0, -0.0784000015258789, 0.4), 
        ..player()
    };
    tick(&mut state, &world, &Input::default());
    assert_eq!(state.position, Vec3::new(0.5, 0.5, 0.9));
    let height=f32::from_bits(0.6f32.to_bits()+1) as f64;
    world.set([0, 0, 1], Block::boxes(&[Aabb::new(0.0, 0.0, 0.0, 1.0, height, 1.0)]));
    let mut state=PlayerState {
        position:Vec3::new(0.5, 0.0, 0.5), 
        velocity:Vec3::new(0.0, -0.0784000015258789, 0.4), 
        ..player()
    };
    tick(&mut state, &world, &Input::default());
    assert_eq!(state.position.y.to_bits(), 0.0f64.to_bits());
    assert!(state.horizontal_collision);
}

#[test]
fn sneak_edge_backoff() {
    let mut world=BoxWorld::default();
    world.set([0, -1, 0], Block::full());
    let mut state=PlayerState {
        position:Vec3::new(0.5, 0.0, 0.5), 
        velocity:Vec3::new(0.9, -0.0784000015258789, 0.0), 
        shift_key_down:true, 
        pose:Pose::Crouching, 
        ..player()
    };
    tick(&mut state, &world, &Input {
        sneak:true, ..Input::default()
    });
    let moved=0.9f64-0.05-0.05-0.05;
    assert_eq!(state.position.x.to_bits(), (0.5+moved).to_bits());
    assert!(state.on_ground);
}

#[test]
fn grounded_travel_uses_current_sprint_modifier() {
    let world = floor(0.6, false);
    let walk_speed = 0.1f32;
    let sprint_speed = (walk_speed as f64 * (1.0 + 0.3f32 as f64)) as f32;
    for sprinting in [true, false] {
        let mut state = PlayerState { sprinting, ..player() };
        state.speed = if sprinting { sprint_speed } else { walk_speed };
        let input = Input { forward: !sprinting, left: sprinting, sprint: true, ..Input::default() };
        tick(&mut state, &world, &input);
        assert_eq!(state.sprinting, !sprinting);
        let speed = if sprinting { walk_speed } else { sprint_speed };
        let acceleration = speed * (0.21600002f32 / (0.6f32 * 0.6f32 * 0.6f32));
        let expected = 0.98f32 as f64 * acceleration as f64;
        let wanted = state.movement_this_tick.expect("grounded collision request");
        let actual = if sprinting { wanted.x } else { wanted.z };
        assert_eq!(actual.to_bits(), expected.to_bits(),
            "sprint {sprinting} -> {} requested {actual:?} ({:016x}), expected {expected:?} ({:016x})",
            state.sprinting, actual.to_bits(), expected.to_bits());
        assert_eq!(state.speed.to_bits(), speed.to_bits());
    }
}

#[test]
fn sprint_latches_through_key_release_and_sneak() {
    let world = floor(0.6, false);
    for sneak in [false, true] {
        let mut state = player();
        tick(&mut state, &world, &Input { forward: true, sprint: true, ..Input::default() });
        assert!(state.sprinting);
        for _ in 0..3 {
            tick(&mut state, &world, &Input { forward: true, sneak, ..Input::default() });
            assert!(state.sprinting);
        }
        tick(&mut state, &world, &Input::default());
        assert!(!state.sprinting);
    }
}

#[test]
fn shallow_water_prevents_sprinting() {
    let mut world = floor(0.6, false);
    world.fill([-2, 0, -2], [2, 0, 10], Block {
        air: false,
        fluid: Fluid { kind: FluidKind::Water, height: 0.4, ..Fluid::default() },
        ..Block::default()
    });
    let mut state = player();
    for _ in 0..3 {
        tick(&mut state, &world, &Input { forward: true, sprint: true, ..Input::default() });
        assert!(state.in_water && !state.underwater);
        assert!(!state.sprinting);
    }
}

#[test]
fn bed_hay_and_honey_landings() {
    for (material, bed, multiplier, expected) in [("bed", true, 1.0, 2), ("hay", false, 0.2, 1), ("honey", false, 0.2, 1)] {
        let mut world = floor(0.6, false);
        world.set([0, -1, 0], Block { bed, fall_on_multiplier: multiplier, ..Block::full() });
        let mut state = PlayerState { fall_distance: 10.0, ..player() };
        tick(&mut state, &world, &Input::default());
        assert_eq!(20.0-state.health, expected as f32, "{material}");
    }
}

#[test]
fn four_corner_push_out_and_no_physics() {
    let mut world = BoxWorld::default();
    world.set([0, 0, 0], Block::full());
    for no_physics in [false, true] {
        let mut state = PlayerState {
            position: Vec3::new(0.9, 0.0, 0.5), no_physics, ..PlayerState::default()
        };
        tick(&mut state, &world, &Input::default());
        assert_eq!(state.position, Vec3::new(0.9, 0.0, if no_physics { 0.5 } else { 0.4 }));
    }
}

#[test]
#[should_panic(expected = "BlockState.entityInside: bubbleColumn")]
fn unsupported_inside_hook_fails_explicitly() {
    let mut world = BoxWorld::default();
    world.set([0, 0, 0], Block {
        air: false, inside: Inside::Unsupported("bubbleColumn".to_owned()), ..Block::default()
    });
    let mut state = PlayerState { position: Vec3::new(0.5, 0.0, 0.5), ..PlayerState::default() };
    tick(&mut state, &world, &Input::default());
}

#[test]
#[should_panic(expected = "FluidState.entityInside: LavaFluid")]
fn lava_inside_hook_fails_explicitly() {
    let mut world = BoxWorld::default();
    world.set([0, 0, 0], Block {
        air: false, fluid: Fluid { kind: FluidKind::Lava, height: 1.0, ..Fluid::default() }, ..Block::default()
    });
    let mut state = PlayerState { position: Vec3::new(0.5, 0.0, 0.5), ..PlayerState::default() };
    tick(&mut state, &world, &Input::default());
}

#[test]
fn fall_reset_end_portal_and_gateway() {
    let mut world = BoxWorld::default();
    world.set([0, 0, 0], Block { end_portal_or_gateway: true, air: false, ..Block::default() });
    assert!(world.fall_reset_clip(Vec3::new(0.5, 2.0, 0.5), Vec3::new(0.5, -2.0, 0.5)));
    assert!(world.fall_reset_clip(Vec3::new(0.5, 0.5, 0.5), Vec3::new(0.5, -2.0, 0.5)));
    assert!(!world.fall_reset_clip(Vec3::new(1.1, 2.0, 0.5), Vec3::new(1.1, -2.0, 0.5)));
}
#[test]
fn suffocation_full_cube_keeps_sub_epsilon_overlap() {
    let mut world = BoxWorld::default();
    world.set([0, 0, 0], Block::full());
    let bounds = Aabb::new(0.1, 1.0-5.0e-8, 0.1, 0.9, 2.0, 0.9);
    assert!(world.collides_with_suffocating_block(bounds, CollisionContext::of(&player())));
}
#[test]
fn client_freeze_hook_does_not_compute_frost_speed() {
    let mut world = floor(0.6, false);
    world.fill([-1, 0, -1], [1, 1, 1], Block {
        inside: Inside::PowderSnow,
        ..Block::boxes(&[])
    });
    let mut state = player();
    for frozen in 1..=140 {
        tick(&mut state, &world, &Input::default());
        assert!(state.in_powder_snow);
        assert_eq!(state.ticks_frozen, frozen);
        assert_eq!(state.speed.to_bits(), 0.1f32.to_bits());
    }
    tick(&mut state, &world, &Input::default());
    assert_eq!(state.ticks_frozen, 140);
    state.leather_boots = true;
    tick(&mut state, &world, &Input::default());
    assert_eq!(state.ticks_frozen, 140);
    tick(&mut state, &floor(0.6, false), &Input::default());
    assert_eq!(state.ticks_frozen, 140, "client must not run server decay");
}

#[test]
fn supplied_frost_applies_without_support_and_attribute_clamps() {
    let mut state = PlayerState {
        position: Vec3::new(0.5, 10.0, 0.5),
        ticks_frozen: 0,
        speed_modifiers: vec![SpeedModifier {
            id: "minecraft:powder_snow".to_owned(),
            amount: -0.05f32 as f64,
            operation: SpeedOperation::AddValue,
        }],
        ..PlayerState::default()
    };
    tick(&mut state, &BoxWorld::default(), &Input::default());
    assert_eq!(state.speed.to_bits(), 0.05f32.to_bits());
    state.movement_speed_base = 0.0;
    tick(&mut state, &BoxWorld::default(), &Input::default());
    assert_eq!(state.speed.to_bits(), 0.0f32.to_bits());
    state.movement_speed_base = 2048.0;
    tick(&mut state, &BoxWorld::default(), &Input::default());
    assert_eq!(state.speed.to_bits(), 1024.0f32.to_bits());
}

#[test]
fn corner_course_axis_tie_first_contact() {
    let mut world = BoxWorld::default();
    world.fill([10, -1, 0], [30, -1, 20], Block::full());
    world.fill([20, 0, 8], [20, 1, 8], Block::full());
    let mut state = PlayerState {
        position: Vec3::new(16.5, 0.0, 4.5),
        velocity: Vec3::default(),
        ..player()
    };
    let input = Input { forward: true, left: true, yaw: 0.0, ..Input::default() };
    for contact_tick in 0..100 {
        let before = state.position;
        tick(&mut state, &world, &input);
        if !state.horizontal_collision { continue; }
        let wanted = state.movement_this_tick.expect("contact movement");
        assert_eq!(wanted.x.abs().to_bits(), wanted.z.abs().to_bits(), "contact tick {contact_tick}");
        let free_x = before.x + wanted.x;
        let free_z = before.z + wanted.z;
        let clipped_x = state.position.x.to_bits() != free_x.to_bits();
        let clipped_z = state.position.z.to_bits() != free_z.to_bits();
        assert!(clipped_x ^ clipped_z, "exactly one horizontal axis must clip");
        let vanilla = Vec3::new(free_x, 0.0, 8.0 - 0.6f32 as f64 / 2.0);
        println!("corner first contact tick={contact_tick}, wanted={wanted:?}, position={:?}", state.position);
        if cfg!(feature = "mutant-axis-tie") {
            assert_ne!(state.position, vanilla, "STOP: axis-tie mutant is uncatchable on the corner course");
        }
        assert_eq!(state.position, vanilla, "vanilla clips Z after moving X on the tie");
        return;
    }
    panic!("corner course never made contact");
}
