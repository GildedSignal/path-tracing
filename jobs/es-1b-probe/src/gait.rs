use law::{
    footprint::{Footprint, RecordingWorld},
    tick_with_collision, try_tick, Block, BoxWorld, Input, PlayerState, Vec3,
};
use plan::events::Map;
use std::collections::HashMap;

pub const GAIT_PERIOD: usize = 12;
pub const SPRINT_JUMP_INPUT: Input = Input {
    forward: true,
    back: false,
    left: false,
    right: false,
    jump: true,
    sneak: false,
    sprint: true,
    yaw: 0.0,
    pitch: 0.0,
};

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct PhaseState {
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub vz: f64,
    pub on_ground: bool,
    pub sprinting: bool,
    pub sprint_speed_modifier: bool,
    pub no_jump_delay: i32,
}

pub struct GaitCycle {
    pub anchor_template: PlayerState,
    pub phase_templates: [PlayerState; GAIT_PERIOD],
    pub dz: [f64; GAIT_PERIOD],
    pub phases: [PhaseState; GAIT_PERIOD],
    pub map_z: Map,
    pub cycle_footprint: Footprint,
    pub settle_ticks: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct EvalState {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub vx: f64,
    pub vy: f64,
    pub vz: f64,
    pub on_ground: bool,
    pub sprinting: bool,
}

pub fn make_flat_stone_world() -> BoxWorld {
    let mut world = BoxWorld::default();
    world.difficulty = 0; // Peaceful: food exhaustion never disables sprint
    world.fill([-1024, -1, -1024], [100_000, -1, 10_000_000], Block::full());
    world
}

fn phase_key(s: &PlayerState) -> (u64, u64, u64, u64, bool, bool, bool, i32) {
    (
        s.position.y.to_bits(),
        s.velocity.x.to_bits(),
        s.velocity.y.to_bits(),
        s.velocity.z.to_bits(),
        s.on_ground,
        s.sprinting,
        s.sprint_speed_modifier,
        s.no_jump_delay,
    )
}

pub fn calibrate_sprint_jump(world: &BoxWorld) -> GaitCycle {
    let mut s = PlayerState {
        position: Vec3::new(0.5, 0.0, 0.5),
        velocity: Vec3::new(0.0, -0.0784000015258789, 0.0),
        on_ground: true,
        main_supporting_block: Some([0, -1, 0]),
        food_level: 20,
        saturation: 20.0,
        exhaustion: 0.0,
        ..PlayerState::default()
    };

    let mut seen: HashMap<_, usize> = HashMap::new();
    let mut history: Vec<PlayerState> = Vec::with_capacity(512);
    let mut dz_history: Vec<f64> = Vec::with_capacity(512);

    let (cycle_start, period) = loop {
        let key = phase_key(&s);
        if let Some(&prev) = seen.get(&key) {
            let period = history.len() - prev;
            if period == GAIT_PERIOD && s.on_ground && s.no_jump_delay == 0 {
                break (prev, period);
            }
        }
        seen.insert(key, history.len());
        history.push(s.clone());
        let _ = tick_with_collision(&mut s, world, &SPRINT_JUMP_INPUT);
        let z_before = history.last().unwrap().position.z;
        dz_history.push(s.position.z - z_before);
        assert!(history.len() < 2000, "failed to lock 12-tick sprint-jump cycle");
    };
    assert_eq!(period, GAIT_PERIOD);

    // Verify exact periodicity of dz across one more full cycle, and record its Footprint.
    let rec = RecordingWorld::new(world);
    let mut anchor = history[cycle_start].clone();
    anchor.t = 0;
    anchor.position.x = 0.5;
    anchor.position.z = 0.5;
    anchor.main_supporting_block = anchor.main_supporting_block.map(|_| [0, -1, 0]);
    anchor.exhaustion = 0.0;

    let mut check_s = anchor.clone();
    let mut dz = [0.0f64; GAIT_PERIOD];
    let mut phases: Vec<PhaseState> = Vec::with_capacity(GAIT_PERIOD);
    let mut phase_templates: Vec<PlayerState> = Vec::with_capacity(GAIT_PERIOD);
    for i in 0..GAIT_PERIOD {
        phase_templates.push(check_s.clone());
        phases.push(PhaseState {
            y: check_s.position.y,
            vx: check_s.velocity.x,
            vy: check_s.velocity.y,
            vz: check_s.velocity.z,
            on_ground: check_s.on_ground,
            sprinting: check_s.sprinting,
            sprint_speed_modifier: check_s.sprint_speed_modifier,
            no_jump_delay: check_s.no_jump_delay,
        });
        let (_wanted, actual) = tick_with_collision(&mut check_s, &rec, &SPRINT_JUMP_INPUT)
            .expect("flat stone tick");
        dz[i] = actual.z;
        assert_eq!(
            check_s.position.x.to_bits(),
            anchor.position.x.to_bits(),
            "yaw=0 must preserve x bit-exact"
        );
    }
    assert_eq!(
        phase_key(&check_s),
        phase_key(&anchor),
        "12-tick cycle must close bit-identically"
    );

    let cycle_footprint = rec.take();
    let map_z = Map::new(dz.to_vec());
    let phases: [PhaseState; GAIT_PERIOD] = phases.try_into().expect("12 phases");
    let phase_templates: [PlayerState; GAIT_PERIOD] =
        phase_templates.try_into().expect("12 templates");

    let cycle = GaitCycle {
        anchor_template: anchor,
        phase_templates,
        dz,
        phases,
        map_z,
        cycle_footprint,
        settle_ticks: cycle_start,
    };

    // Self-check against law::try_tick over 5,000 ticks across multiple binades.
    for &(x_test, z0_test) in &[(0.5, 0.5), (42.5, 1023.75), (128.5, 8191.25)] {
        let mut oracle = cycle.anchor_state_at(x_test, z0_test);
        for dt in 0..=5000u64 {
            let eval = eval_gait(&cycle, x_test, z0_test, dt);
            assert_eq!(eval.x.to_bits(), oracle.position.x.to_bits(), "x mismatch at dt={dt}");
            assert_eq!(eval.y.to_bits(), oracle.position.y.to_bits(), "y mismatch at dt={dt}");
            assert_eq!(eval.z.to_bits(), oracle.position.z.to_bits(), "z mismatch at dt={dt}");
            assert_eq!(eval.vx.to_bits(), oracle.velocity.x.to_bits(), "vx mismatch at dt={dt}");
            assert_eq!(eval.vy.to_bits(), oracle.velocity.y.to_bits(), "vy mismatch at dt={dt}");
            assert_eq!(eval.vz.to_bits(), oracle.velocity.z.to_bits(), "vz mismatch at dt={dt}");
            assert_eq!(eval.on_ground, oracle.on_ground, "on_ground mismatch at dt={dt}");
            assert_eq!(eval.sprinting, oracle.sprinting, "sprinting mismatch at dt={dt}");
            if dt < 5000 {
                try_tick(&mut oracle, world, &SPRINT_JUMP_INPUT).expect("oracle tick");
            }
        }
    }

    cycle
}

impl GaitCycle {
    pub fn anchor_state_at(&self, x: f64, z0: f64) -> PlayerState {
        let mut s = self.anchor_template.clone();
        s.position.x = x;
        s.position.z = z0;
        s.main_supporting_block = s
            .main_supporting_block
            .map(|_| [x.floor() as i32, -1, z0.floor() as i32]);
        s
    }

    pub fn materialize_state(&self, x: f64, z0: f64, elapsed: u64) -> PlayerState {
        let mut s = self.phase_templates[(elapsed % GAIT_PERIOD as u64) as usize].clone();
        let z = self.map_z.advance(z0, elapsed);
        s.position.x = x;
        s.position.z = z;
        s.main_supporting_block = s
            .main_supporting_block
            .map(|_| [x.floor() as i32, -1, z.floor() as i32]);
        s.t = elapsed;
        s
    }
}

#[inline]
pub fn eval_gait(cycle: &GaitCycle, x: f64, z0: f64, elapsed: u64) -> EvalState {
    let z = cycle.map_z.advance(z0, elapsed);
    let p = &cycle.phases[(elapsed % GAIT_PERIOD as u64) as usize];
    EvalState {
        x,
        y: p.y,
        z,
        vx: p.vx,
        vy: p.vy,
        vz: p.vz,
        on_ground: p.on_ground,
        sprinting: p.sprinting,
    }
}

pub fn step_law_oracle(
    world: &BoxWorld,
    cycle: &GaitCycle,
    x: f64,
    z0: f64,
    elapsed: u64,
) -> PlayerState {
    let mut s = cycle.anchor_state_at(x, z0);
    for _ in 0..elapsed {
        try_tick(&mut s, world, &SPRINT_JUMP_INPUT).expect("oracle step");
    }
    s
}
