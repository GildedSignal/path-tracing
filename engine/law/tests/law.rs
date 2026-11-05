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

#[test]
fn braking_recurrence() {
    let expected=[[8, 5, 8, 10, 11, 12, 13], 
    [8, 6, 8, 10, 11, 12, 13], 
    [38, 36, 35, 33, 30, 25, 7], 
    [40, 39, 38, 36, 35, 32, 27]];
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
                s.speed=0.1;
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
            kind:FluidKind::Water, height:1.0, flow:Vec3::default()
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
fn braking_player_tick() {
    let expected=[[8, 5, 8, 10, 11, 12, 13], 
    [8, 5, 8, 10, 11, 12, 13], 
    [38, 36, 35, 33, 30, 25, 7], 
    [40, 39, 38, 36, 34, 31, 26]];
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
fn speed_refresh_follows_travel() {
    let world=floor(0.6, false);
    let mut state=player();
    tick(&mut state, &world, &Input {
        forward:true, sprint:true, ..Input::default()
    });
    let acceleration=0.1f32*(0.21600002f32/(0.6f32*0.6f32*0.6f32));
    assert_eq!(state.position.z.to_bits(), (0.98f32 as f64*acceleration as f64).to_bits());
    assert_eq!(state.speed, ((0.1f32 as f64)*(1.0+0.3f32 as f64)) as f32);
}
