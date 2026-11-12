//! The class: the facts the law decides on (vocabulary.md, section 3). It names a piece so
//! that it is recognised and nothing else: no bit of it reaches the law, and the search
//! never merges states by it.
//!
//! One rule per field the segment read before writing it:
//! - a field whose values come from the block table or the toolkit enters by its bits;
//! - a field that accumulates enters only as the answers of the comparisons the law makes
//!   on it, each named below by its line in `engine/law/src`. None is ours.
//!
//! The answers are taken on the start state. The law makes some of them later in the tick,
//! on the value it has by then; the class only proposes, so that costs a replay at most.
use law::{BlockPos, PlayerState, pattern::Start, state_reads::{StateField as F, StateReads}};

/// What a table names states by. The table takes it as a value so that a lab can measure
/// another class against this one.
pub type ClassOf = fn(&PlayerState, StateReads) -> Box<[u64]>;

pub fn body_block(state: &PlayerState) -> BlockPos {
    [state.position.x, state.position.y, state.position.z].map(law::math::floor)
}

fn answers(bits: &[bool]) -> u64 { bits.iter().enumerate().fold(0, |word, (i, &bit)| word | (bit as u64) << i) }

/// The class of `state` under the fields in `reads`: the mask, then each field's word.
pub fn class(state: &PlayerState, reads: StateReads) -> Box<[u64]> {
    let s = state;
    let mut words = vec![reads.0 as u64, (reads.0 >> 64) as u64];
    for field in reads.fields() {
        match field {
            // Only counted.
            F::t => {}
            // Resources are guards: the two thresholds the movement tick branches on. The
            // food clock's own comparisons (movement.rs:1250-1276) decide deltas, which a
            // segment carries in `uses`.
            F::food_level => words.push(answers(&[s.food_level > 6])), // movement.rs:209, 215
            F::health => words.push(answers(&[s.health > 0.0])), // movement.rs:166, 182, 1008
            F::saturation | F::exhaustion | F::food_tick_timer => {}
            // Every other comparison on position is against a cell, and its answer is a
            // flag of the state (on ground, the collisions, in fluid, climbable) or lives in
            // the world part. What is left is the clamp at the world's edge.
            F::position => {
                let p = s.position;
                words.push(answers(&[p.x < -29999999.0, p.x > 29999999.0, p.z < -29999999.0, p.z > 29999999.0])); // movement.rs:195, 196
            }
            F::velocity => {
                let (v, g) = (s.velocity, s.gravity);
                let honey = v.y / 0.98f32 as f64 + 0.08; // movement.rs:1059
                words.push(answers(&[
                    v.horizontal_sqr() < 9.0e-6, // movement.rs:157
                    v.y.abs() < 0.003, // movement.rs:161
                    v.y < 0.0, // movement.rs:387, 629
                    v.y <= 0.0, // movement.rs:445
                    v.y > 0.0, // movement.rs:540; shapes.rs:404
                    v.x != 0.0, // movement.rs:550, 594
                    v.z != 0.0, // movement.rs:557, 594
                    v.x.abs() <= 0.05, // movement.rs:551
                    v.z.abs() <= 0.05, // movement.rs:558
                    v.length_sqr() == 0.0, // movement.rs:624
                    v.x.abs() < v.z.abs(), // shapes.rs:88
                    v.x > 0.0, // shapes.rs:404
                    v.z > 0.0, // shapes.rs:404
                    v.x.abs() < 1.0e-7, // shapes.rs:390; movement.rs:597
                    v.y.abs() < 1.0e-7, // shapes.rs:390; movement.rs:597
                    v.z.abs() < 1.0e-7, // shapes.rs:390; movement.rs:597
                    -v.y < g, // movement.rs:764
                    v.x.abs() < 0.003, // movement.rs:915
                    v.z.abs() < 0.003, // movement.rs:915
                    v.y.abs() < 0.1, // movement.rs:957
                    (v.y - 0.005).abs() >= 0.003, // movement.rs:496
                    (v.y - g / 16.0).abs() < 0.003, // movement.rs:496
                    honey < -0.08, // movement.rs:1061
                    honey < -0.13, // movement.rs:1063
                ]));
            }
            F::fall_distance => words.push(answers(&[
                s.fall_distance != 0.0, // movement.rs:730
                s.fall_distance > 0.0, // movement.rs:822
                s.fall_distance < s.step_height as f64, // movement.rs:540
                s.fall_distance > 2.5, // world.rs:446
            ])),
            F::no_jump_delay => words.push(answers(&[s.no_jump_delay > 0, s.no_jump_delay == 0])), // movement.rs:154, 175
            F::ticks_frozen => words.push(answers(&[s.ticks_frozen + 1 < 140])), // movement.rs:192
            F::remaining_fire_ticks => words.push(answers(&[s.remaining_fire_ticks > 0])), // movement.rs:1032
            F::hurt_cooldown => words.push(answers(&[s.hurt_cooldown.remaining_ticks > 0, s.hurt_cooldown.remaining_ticks > 10])), // movement.rs:1137, 1218
            F::water_height => words.push(answers(&[s.water_height > 0.0, s.water_height > 0.4])), // movement.rs:172, 173
            F::lava_height => words.push(answers(&[s.lava_height > 0.0, s.lava_height > 0.4])), // movement.rs:172, 173, 474
            // The support's cell follows the position; the law asks whether there is one.
            F::main_supporting_block => words.push(answers(&[s.main_supporting_block.is_some()])), // movement.rs:317, 712
            F::movement_this_tick => words.push(answers(&[s.movement_this_tick.is_some()])), // movement.rs:980
            // The block table and the toolkit: parameters of the generator, by their bits.
            // The flags are already yes-or-no answers.
            F::input | F::yaw | F::pitch | F::on_ground | F::horizontal_collision | F::vertical_collision
            | F::vertical_collision_below | F::minor_horizontal_collision | F::sprinting | F::sprint_speed_modifier
            | F::no_physics | F::shift_key_down | F::pose | F::in_water | F::in_lava | F::on_climbable
            | F::stuck_speed_multiplier | F::on_ground_no_blocks | F::crouching | F::swimming | F::underwater
            | F::eye_in_water | F::was_eye_in_water | F::in_powder_snow | F::was_in_powder_snow | F::leather_boots
            | F::movement_speed_base | F::speed_modifiers | F::speed | F::jump_strength | F::gravity | F::step_height
            | F::friction_modifier | F::air_drag_modifier | F::water_movement_efficiency | F::movement_efficiency
            | F::sneaking_speed | F::safe_fall_distance | F::fall_damage_multiplier | F::bounciness | F::fall_flying
            | F::riding | F::boat | F::effects | F::flying | F::using_item => s.push_field_bits(field, &mut words),
        }
    }
    words.into()
}

/// A class as a pattern-table start, anchored at the body's block: the world part of a
/// proposal is then the pattern key of the cells its replay read.
pub fn start(state: &PlayerState, words: Box<[u64]>) -> Start {
    Start { origin: body_block(state), words, context: law::CollisionContext::of(state) }
}
