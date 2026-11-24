use crate::codec::{Enc, MAXC};
use crate::gait::{eval_gait, GaitCycle, SPRINT_JUMP_INPUT};
use crate::program::{Prog, Program};
use crate::shelf::Ship;
use law::{
    entity_contact::{push_pair, ContactEntity},
    try_tick, BoxWorld, PlayerState,
};
use std::time::Instant;

pub struct DenseReport {
    pub promote_tick: u64,
    pub dense_ticks: usize,
    pub bot_count: usize,
    pub push_interactions: u64,
    pub oracle_mismatches: usize,
    pub dissolved_mismatches: usize,
    pub battle_bytes_during: usize,
    pub battle_bytes_after_dissolve: usize,
    pub dissolved_program_bytes: usize,
    pub total_inventory_after: u64,
    pub tick_ms: Vec<f64>,
}

struct DenseBot {
    lane: usize,
    ordinal: u64,
    kit_inv: u64,
    state: PlayerState,
}

fn step_dense_tick(bots: &mut [DenseBot], world: &BoxWorld) -> u64 {
    // Pit region: x in [0, 10], z in [0, 10]. Spatial bucket 12x12 for O(N) candidate lookup,
    // then apply push_pair in canonical (a < b) ordinal order for deterministic tie behavior.
    let mut active_pit: Vec<usize> = Vec::with_capacity(bots.len());
    for (idx, b) in bots.iter().enumerate() {
        let p = b.state.position;
        if p.x >= 0.0 && p.x <= 10.0 && p.z >= 0.0 && p.z <= 10.0 {
            active_pit.push(idx);
        }
    }

    let mut pairs: Vec<(usize, usize)> = Vec::new();
    // 1x1 cell grid over [-1..=11] x [-1..=11] (13x13 = 169 cells)
    let mut grid: Vec<Vec<usize>> = vec![Vec::new(); 169];
    for &idx in &active_pit {
        let p = bots[idx].state.position;
        let cx = (p.x.floor() as isize).clamp(0, 11) as usize;
        let cz = (p.z.floor() as isize).clamp(0, 11) as usize;
        grid[cz * 13 + cx].push(idx);
    }
    for &a in &active_pit {
        let pa = bots[a].state.position;
        let cx = (pa.x.floor() as isize).clamp(0, 11) as usize;
        let cz = (pa.z.floor() as isize).clamp(0, 11) as usize;
        for dz in -1isize..=1 {
            let nz = cz as isize + dz;
            if !(0..13).contains(&nz) {
                continue;
            }
            for dx in -1isize..=1 {
                let nx = cx as isize + dx;
                if !(0..13).contains(&nx) {
                    continue;
                }
                for &b in &grid[(nz as usize) * 13 + (nx as usize)] {
                    if a < b {
                        let pb = bots[b].state.position;
                        if (pa.x - pb.x).abs() < 0.6 && (pa.z - pb.z).abs() < 0.6 {
                            pairs.push((a, b));
                        }
                    }
                }
            }
        }
    }

    pairs.sort_unstable();
    let pushes = pairs.len() as u64;
    for (a, b) in pairs {
        let mut ca = ContactEntity {
            position: bots[a].state.position,
            velocity: bots[a].state.velocity,
            no_physics: false,
            vehicle: false,
            pushable: true,
        };
        let mut cb = ContactEntity {
            position: bots[b].state.position,
            velocity: bots[b].state.velocity,
            no_physics: false,
            vehicle: false,
            pushable: true,
        };
        push_pair(&mut ca, &mut cb, false);
        bots[a].state.velocity = ca.velocity;
        bots[b].state.velocity = cb.velocity;
    }

    for b in bots.iter_mut() {
        try_tick(&mut b.state, world, &SPRINT_JUMP_INPUT).expect("dense pit tick");
    }
    pushes
}

pub fn run_dense_pit_check(world: &BoxWorld, cycle: &GaitCycle) -> DenseReport {
    const LANES: usize = 50;
    const PER_LANE: usize = 10;
    const DENSE_TICKS: usize = 60;
    let z0 = -8.0f64;

    let mut enc = Enc::new(false);
    let streams: Vec<Program> = (0..LANES)
        .map(|lane| {
            let x_lane = 1.0 + (lane as f64) * 0.16;
            let prog = [
                Prog::Affine { a: 0, b: 1 }, // t_dep(i) = i
                Prog::Const(x_lane.to_bits()),
                Prog::Const(z0.to_bits()),
                Prog::Const(0),  // gait_id
                Prog::Const(64), // kit_inv
                Prog::Const(0),
                Prog::Const(0),
                Prog::Const(0),
            ];
            Program::seal_from(PER_LANE as u64, 5, prog, &[], &mut enc)
        })
        .collect();

    let promote_tick = cycle
        .map_z
        .first_cross(z0, 0.0, 200)
        .expect("bot crosses pit entrance z=0");
    assert!(
        promote_tick >= PER_LANE as u64,
        "all 500 bots should have departed by promote_tick={promote_tick}"
    );

    // 1. Brute-force reference stepped tick-by-tick from t = 0 through promote_tick + DENSE_TICKS.
    let mut brute: Vec<DenseBot> = Vec::with_capacity(LANES * PER_LANE);
    for (lane, stream) in streams.iter().enumerate() {
        let mut ship = [0u64; MAXC];
        for i in 0..PER_LANE as u64 {
            stream.eval(i, &mut ship);
            let t_dep = ship[0];
            let x = f64::from_bits(ship[1]);
            let z_start = f64::from_bits(ship[2]);
            let kit_inv = ship[4];
            let mut state = cycle.anchor_state_at(x, z_start);
            for t in t_dep..promote_tick {
                assert!(
                    state.position.z < 0.0,
                    "brute tick t={t} entered pit early at z={}",
                    state.position.z
                );
                try_tick(&mut state, world, &SPRINT_JUMP_INPUT).expect("brute pre-pit tick");
            }
            brute.push(DenseBot {
                lane,
                ordinal: i,
                kit_inv,
                state,
            });
        }
    }
    for _ in 0..DENSE_TICKS {
        step_dense_tick(&mut brute, world);
    }

    // 2. Tier-0 lazy until promote_tick, then promote into Tier-2 Battle Vec<DenseBot>.
    let mut battle_opt: Option<Vec<DenseBot>> = Some({
        let mut v = Vec::with_capacity(LANES * PER_LANE);
        let mut ship = [0u64; MAXC];
        for (lane, stream) in streams.iter().enumerate() {
            for i in 0..PER_LANE as u64 {
                stream.eval(i, &mut ship);
                let t_dep = ship[0];
                let x = f64::from_bits(ship[1]);
                let z_start = f64::from_bits(ship[2]);
                let kit_inv = ship[4];
                let elapsed = promote_tick - t_dep;
                let state = cycle.materialize_state(x, z_start, elapsed);
                v.push(DenseBot {
                    lane,
                    ordinal: i,
                    kit_inv,
                    state,
                });
            }
        }
        v
    });

    let battle_bytes_during = battle_opt
        .as_ref()
        .map_or(0, |v| v.capacity() * std::mem::size_of::<DenseBot>());

    let mut push_interactions = 0u64;
    let mut tick_ms = Vec::with_capacity(DENSE_TICKS);
    {
        let battle = battle_opt.as_mut().unwrap();
        for _ in 0..DENSE_TICKS {
            let t0 = Instant::now();
            push_interactions += step_dense_tick(battle, world);
            tick_ms.push(t0.elapsed().as_secs_f64() * 1_000.0);
        }
    }

    let battle = battle_opt.as_ref().unwrap();
    let mut oracle_mismatches = 0usize;
    for (b, r) in battle.iter().zip(brute.iter()) {
        let same = b.state.position.x.to_bits() == r.state.position.x.to_bits()
            && b.state.position.y.to_bits() == r.state.position.y.to_bits()
            && b.state.position.z.to_bits() == r.state.position.z.to_bits()
            && b.state.velocity.x.to_bits() == r.state.velocity.x.to_bits()
            && b.state.velocity.y.to_bits() == r.state.velocity.y.to_bits()
            && b.state.velocity.z.to_bits() == r.state.velocity.z.to_bits()
            && b.state.on_ground == r.state.on_ground
            && b.state.sprinting == r.state.sprinting;
        if !same {
            oracle_mismatches += 1;
        }
    }

    // 3. Dissolve Battle back into outgoing Form-P Programs + Ef scars, then drop Battle Vec.
    let end_tick = promote_tick + DENSE_TICKS as u64;
    let mut out_progs: Vec<Program> = Vec::with_capacity(LANES);
    for lane in 0..LANES {
        let x_lane = 1.0 + (lane as f64) * 0.16;
        let base_state = eval_gait(cycle, x_lane, z0, end_tick);
        let prog = [
            Prog::Const(x_lane.to_bits()),
            Prog::Const(base_state.z.to_bits()),
            Prog::Const(0u64), // vx_bits
            Prog::Const(base_state.vz.to_bits()),
            Prog::Const(64), // kit_inv
            Prog::Const(0),
            Prog::Const(0),
            Prog::Const(0),
        ];
        let lane_bots: Vec<&DenseBot> = battle.iter().filter(|b| b.lane == lane).collect();
        let scars: Vec<(u64, Ship)> = lane_bots
            .iter()
            .map(|b| {
                let mut s = [0u64; MAXC];
                s[0] = b.state.position.x.to_bits();
                s[1] = b.state.position.z.to_bits();
                s[2] = b.state.velocity.x.to_bits();
                s[3] = b.state.velocity.z.to_bits();
                s[4] = b.kit_inv;
                (b.ordinal, s)
            })
            .filter(|(_, s)| (0..5).any(|f| s[f] != prog[f].at(0)))
            .collect();
        out_progs.push(Program::seal_from(PER_LANE as u64, 5, prog, &scars, &mut enc));
    }

    let expected_post: Vec<(usize, u64, [u64; 5])> = battle
        .iter()
        .map(|b| {
            (
                b.lane,
                b.ordinal,
                [
                    b.state.position.x.to_bits(),
                    b.state.position.z.to_bits(),
                    b.state.velocity.x.to_bits(),
                    b.state.velocity.z.to_bits(),
                    b.kit_inv,
                ],
            )
        })
        .collect();

    // Dissolve dense container completely.
    drop(battle_opt.take());
    let battle_bytes_after_dissolve = battle_opt.as_ref().map_or(0, |v| v.capacity());

    let mut dissolved_mismatches = 0usize;
    let mut total_inventory_after = 0u64;
    let mut ship = [0u64; MAXC];
    for (lane, ord, want) in expected_post {
        out_progs[lane].eval(ord, &mut ship);
        if ship[..5] != want {
            dissolved_mismatches += 1;
        }
        total_inventory_after += ship[4];
    }
    let dissolved_program_bytes: usize = out_progs.iter().map(Program::bytes).sum();

    DenseReport {
        promote_tick,
        dense_ticks: DENSE_TICKS,
        bot_count: LANES * PER_LANE,
        push_interactions,
        oracle_mismatches,
        dissolved_mismatches,
        battle_bytes_during,
        battle_bytes_after_dissolve,
        dissolved_program_bytes,
        total_inventory_after,
        tick_ms,
    }
}
