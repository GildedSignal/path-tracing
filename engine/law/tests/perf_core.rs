//! Release microbenchmarks for the ported `engine/law` core modules (`--release -- --ignored --nocapture`).

use std::collections::HashMap;
use std::hint::black_box;
use std::time::Instant;

use law::combat::{resolve_player_attack, AttackInput};
use law::entity_contact::{ContactEntity, SpatialCrowdIndex};
use law::explosion::{explosion_entity_effect, ExplosionProfile, ExplosionTarget};
use law::falling::flight_tick;
use law::fluid::{compute_spread_directions, FlowKind};
use law::neighbor::{
    powered_toggle_on_signal, redstone_wire_target_strength, update_flags,
    NeighborQueue, Direction, UpdateEvent,
};
use law::piston::{BlockKind, PistonBlock, PistonWorld, Resolver};
use law::ticks::{LevelTicks, SubTickCounter, ScheduledTick, TickPriority, MAX_TICKS_PER_TICK};
use law::{Aabb, BlockPos, Vec3};

fn ns_per_op(iters: usize, elapsed: std::time::Duration) -> f64 {
    elapsed.as_nanos() as f64 / iters as f64
}

#[test]
#[ignore = "release microbenchmark; run with cargo test --release -p law --test perf_core -- --ignored --nocapture"]
fn benchmark_ported_core_laws() {
    // 1. SpatialCrowdIndex: 10,000-entity 2-block stride parade & dense cluster (warm steady-state).
    {
        let mut parade = Vec::with_capacity(10_000);
        for i in 0..10_000usize {
            let x = (i % 100) as f64 * 2.0;
            let z = (i / 100) as f64 * 2.0;
            parade.push(ContactEntity::player(Vec3::new(x, 64.0, z), Vec3::new(0.1, 0.0, 0.0)));
        }
        let mut dense = Vec::with_capacity(10_000);
        for i in 0..10_000usize {
            // Clusters of 8 colliding entities spaced every 2 blocks.
            let c = i / 8;
            let local = i % 8;
            let x = (c % 40) as f64 * 2.0 + (local & 1) as f64 * 0.35;
            let z = (c / 40) as f64 * 2.0 + ((local >> 1) & 1) as f64 * 0.35;
            let mut e = ContactEntity::player(Vec3::new(x, 64.0, z), Vec3::new(0.05, 0.0, 0.05));
            if local >= 4 {
                e.position.x += 0.1;
            }
            dense.push(e);
        }

        let mut idx = SpatialCrowdIndex::new();
        idx.step_crowd_push(&mut parade, 24, |_, _| false, |_| 1, |_, _| {});
        let t0 = Instant::now();
        for _ in 0..50 {
            idx.step_crowd_push(black_box(&mut parade), 24, |_, _| false, |_| 1, |_, _| {});
        }
        let parade_ns = ns_per_op(50, t0.elapsed());

        idx.step_crowd_push(&mut dense, 24, |_, _| false, |_| 1, |_, _| {});
        let t1 = Instant::now();
        for _ in 0..50 {
            idx.step_crowd_push(black_box(&mut dense), 24, |_, _| false, |_| 1, |_, _| {});
        }
        let dense_ns = ns_per_op(50, t1.elapsed());
        println!(
            "PERF SpatialCrowdIndex 10k: parade_2block={:.3} us/tick ({:.1} ns/entity), dense_clusters8={:.3} us/tick ({:.1} ns/entity)",
            parade_ns / 1000.0,
            parade_ns / 10_000.0,
            dense_ns / 1000.0,
            dense_ns / 10_000.0,
        );
    }

    // 2. LevelTicks: schedule + drain 65,536 ticks across 16 chunks.
    {
        let mut lt = LevelTicks::new();
        let mut out = Vec::with_capacity(MAX_TICKS_PER_TICK);
        // Warmup capacity
        let mut sub = SubTickCounter::default();
        for i in 0..MAX_TICKS_PER_TICK {
            let chunk_x = ((i & 15) as i32) * 16;
            let pos = [chunk_x + ((i >> 4) & 15) as i32, (i >> 12) as i32, ((i >> 8) & 15) as i32];
            lt.schedule(ScheduledTick {
                kind: 1u16,
                pos,
                trigger_tick: 1,
                priority: TickPriority::Normal,
                sub_tick_order: sub.next(),
            });
        }
        lt.collect_due(1, MAX_TICKS_PER_TICK, |_| true, &mut out);

        let mut total_sched = std::time::Duration::ZERO;
        let mut total_drain = std::time::Duration::ZERO;
        let runs = 5usize;
        for r in 0..runs {
            let gt = (r + 2) as i64;
            let ts = Instant::now();
            for i in 0..MAX_TICKS_PER_TICK {
                let chunk_x = ((i & 15) as i32) * 16;
                let pos = [chunk_x + ((i >> 4) & 15) as i32, (i >> 12) as i32, ((i >> 8) & 15) as i32];
                lt.schedule(ScheduledTick {
                    kind: 1u16,
                    pos,
                    trigger_tick: gt,
                    priority: TickPriority::Normal,
                    sub_tick_order: sub.next(),
                });
            }
            total_sched += ts.elapsed();
            let td = Instant::now();
            lt.collect_due(gt, MAX_TICKS_PER_TICK, |_| true, &mut out);
            black_box(out.len());
            total_drain += td.elapsed();
        }
        println!(
            "PERF LevelTicks 65536 (16 chunks): schedule={:.3} ms ({:.1} ns/tick), drain={:.3} ms ({:.1} ns/tick)",
            (total_sched.as_secs_f64() / runs as f64) * 1000.0,
            (total_sched.as_nanos() as f64 / runs as f64) / 65_536.0,
            (total_drain.as_secs_f64() / runs as f64) * 1000.0,
            (total_drain.as_nanos() as f64 / runs as f64) / 65_536.0,
        );
    }

    // 3. Fluid compute_spread_directions (radius-4 open + wall detour).
    {
        let iters = 200_000usize;
        let t0 = Instant::now();
        let mut acc = 0u32;
        for k in 0..iters {
            let ox = (k & 3) as i32;
            let (mask, dist) = compute_spread_directions(
                FlowKind::Water,
                (ox, 0),
                |x, z| !(z == 1 && (1..=2).contains(&x)),
                |x, z| x == ox + 3 && z == 1,
                |_| true,
            );
            acc ^= (mask as u32) + (dist as u32);
        }
        black_box(acc);
        println!("PERF fluid::compute_spread_directions: {:.1} ns/op", ns_per_op(iters, t0.elapsed()));
    }

    // 4. Piston Resolver::resolve (12-block sticky + collision reorder).
    {
        struct StickyWorld {
            blocks: HashMap<BlockPos, PistonBlock>,
        }
        impl PistonWorld for StickyWorld {
            fn block(&self, position: BlockPos) -> PistonBlock {
                *self.blocks.get(&position).unwrap_or(&PistonBlock::air())
            }
        }
        let mut world = StickyWorld { blocks: HashMap::new() };
        let piston = [0, 64, 0];
        let stone = PistonBlock::pushable();
        let slime = PistonBlock { kind: BlockKind::SlimeBlock, ..stone };
        world.blocks.insert([1, 64, 0], slime);
        for i in 2..=6 {
            world.blocks.insert([i, 64, 0], stone);
        }
        world.blocks.insert([1, 64, -1], slime);
        for i in 2..=5 {
            world.blocks.insert([i, 64, -1], stone);
        }
        let mut resolver = Resolver::new(piston, Direction::East, true);
        let iters = 200_000usize;
        let t0 = Instant::now();
        let mut acc = 0usize;
        for _ in 0..iters {
            assert!(resolver.resolve(black_box(&world)));
            acc += resolver.to_push().len();
        }
        black_box(acc);
        println!("PERF piston::Resolver (10-block branched): {:.1} ns/op", ns_per_op(iters, t0.elapsed()));
    }

    // 5. NeighborQueue (8-wire cascade with flat fixed array world).
    {
        let mut updater = NeighborQueue::new(100_000);
        let mut powers = [0u8; 16];
        let iters = 20_000usize;
        let t0 = Instant::now();
        for step in 0..iters {
            powers[1..=8].fill(0);
            let mut door_open = false;
            let lever_on = (step & 1) == 0;
            updater.drain_with(
                |q| {
                    q.update_neighbors_except([0, 1, 0], 1u8, None);
                },
                |q, ev| {
                    if let UpdateEvent::Simple { pos, .. } = ev {
                        if pos[1] == 1 && pos[2] == 0 && (1..=8).contains(&pos[0]) {
                            let ix = pos[0] as usize;
                            let sig = if ix == 1 && lever_on { 15 } else { 0 };
                            let left = powers[ix - 1];
                            let right = powers[ix + 1];
                            let target = redstone_wire_target_strength(sig, [0, right, 0, left]);
                            if target != powers[ix] {
                                powers[ix] = target;
                                q.update_neighbors_except(pos, 2u8, None);
                                q.neighbor_changed(pos, 2u8, false);
                            }
                        } else if pos == [9, 1, 0] {
                            if let Some((n, _)) = powered_toggle_on_signal(door_open, powers[8] > 0) {
                                door_open = n;
                            }
                        }
                    }
                },
            );
            black_box((powers[8], door_open, update_flags::CLIENTS));
        }
        println!("PERF CollectingNeighborUpdater (8-wire+door cascade): {:.1} ns/cascade", ns_per_op(iters, t0.elapsed()));
    }

    // 6. Scalar/value hot paths: falling::flight_tick, combat::resolve_player_attack, explosion::explosion_entity_effect.
    {
        let iters = 2_000_000usize;
        let t0 = Instant::now();
        let mut y = 1000.0;
        let mut vy = 0.0;
        for _ in 0..iters {
            (y, vy) = flight_tick(black_box(y), black_box(vy));
        }
        black_box((y, vy));
        println!("PERF falling::flight_tick: {:.2} ns/op", ns_per_op(iters, t0.elapsed()));

        let atk_in = AttackInput {
            base_attack_damage: 1.0,
            enchanted_full_damage: 1.0,
            item_damage_bonus: 3.0,
            attack_strength_ticker: 10,
            attack_speed: 4.0,
            attacker_velocity: Vec3::new(0.2, 0.0, 0.3),
            attacker_known_movement: Vec3::default(),
            attacker_speed: 0.13,
            attacker_yaw: 37.5,
            attacker_on_ground: true,
            attacker_on_climbable: false,
            attacker_in_water: false,
            attacker_mobility_restricted: false,
            attacker_is_passenger: false,
            attacker_fall_distance: 0.0,
            attacker_sprinting: true,
            holding_sword: true,
            weapon_knockback: 0.0,
            target_is_living: true,
            target_is_server_player: false,
            target_on_ground: true,
            target_velocity: Vec3::new(0.05, -0.0784, -0.05),
            target_kb_resistance: 0.0,
            hurt_succeeded: true,
        };
        let iters_atk = 1_000_000usize;
        let t1 = Instant::now();
        let mut dmg_acc = 0.0f64;
        for _ in 0..iters_atk {
            let r = resolve_player_attack(black_box(atk_in), || 0.5);
            dmg_acc += r.total_damage as f64 + r.knockback.target_packet_velocity.x;
        }
        black_box(dmg_acc);
        println!("PERF combat::resolve_player_attack: {:.2} ns/op", ns_per_op(iters_atk, t1.elapsed()));

        let tgt = ExplosionTarget {
            feet_pos: Vec3::new(0.5, 64.0, 0.5),
            eye_y: 65.62,
            bb: Aabb::player(Vec3::new(0.5, 64.0, 0.5), 1.8),
            is_living: true,
            is_player: true,
            is_spectator: false,
            is_creative_flying: false,
            is_flying_ability: false,
            is_primed_tnt: false,
            ignore_explosion: false,
            explosion_knockback_resistance: 0.0,
        };
        let iters_exp = 200_000usize;
        let t2 = Instant::now();
        let mut exp_acc = 0.0f64;
        for _ in 0..iters_exp {
            if let Some(hit) = explosion_entity_effect(
                ExplosionProfile::wind_charge(),
                black_box(Vec3::new(0.5, 64.25, 0.5)),
                black_box(tgt),
                |_, _| true,
            ) {
                exp_acc += hit.knockback_impulse.y;
            }
        }
        black_box(exp_acc);
        println!("PERF explosion::explosion_entity_effect (wind charge 27-ray exposure): {:.1} ns/op", ns_per_op(iters_exp, t2.elapsed()));
    }
}
