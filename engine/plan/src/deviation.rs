#![allow(dead_code)]
#[path = "../../law/tests/support/json.rs"]
mod json;
use json::{Object, array, hex};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Difference { pub field: String, pub predicted_bits: String, pub recorded_bits: String }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Deviation { pub tick: u64, pub fields: Vec<Difference> }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report { pub first_deviation: Option<Deviation>, pub corrections: u64, pub pushes: u64, pub recording_end_tick: u64, pub plan_arrival_tick: u64, pub unobserved: Vec<String> }
impl Report {
    pub fn to_json(&self) -> String {
        let first = self.first_deviation.as_ref().map_or("null".into(), |d| format!("{{\"tick\":{},\"fields\":[{}]}}", d.tick, d.fields.iter().map(|f| format!("{{\"field\":\"{}\",\"predicted_bits\":\"{}\",\"recorded_bits\":\"{}\"}}", f.field, f.predicted_bits, f.recorded_bits)).collect::<Vec<_>>().join(",")));
        format!("{{\"first_deviation\":{},\"corrections\":{},\"pushes\":{},\"recording_end_tick\":{},\"plan_arrival_tick\":{},\"unobserved\":[{}]}}", first, self.corrections, self.pushes, self.recording_end_tick, self.plan_arrival_tick, self.unobserved.iter().map(|s| format!("\"{s}\"")).collect::<Vec<_>>().join(","))
    }
}
// PlayerState::bits' fixed prefix. t is a law clock, not the recording's tick index.
const FIELDS: &[(&str, &str, usize, u8)] = &[
 ("input.yaw","inputs.yawBits",1,8),("input.pitch","inputs.pitchBits",2,8),
 ("yaw","yRotBits",3,8),("pitch","xRotBits",4,8),("pose","pose",5,0),
 ("food_level","foodLevel",8,0),("ticks_frozen","ticksFrozen",12,0),
 ("input.forward","inputs.forward",13,0),("input.back","inputs.back",14,0),("input.left","inputs.left",15,0),("input.right","inputs.right",16,0),("input.jump","inputs.jump",17,0),("input.sneak","inputs.sneak",18,0),("input.sprint","inputs.sprint",19,0),
 ("on_ground","onGround",20,0),("horizontal_collision","horizontalCollision",21,0),("vertical_collision","verticalCollision",22,0),("vertical_collision_below","verticalCollisionBelow",23,0),("minor_horizontal_collision","minorHorizontalCollision",24,0),("sprinting","isSprinting",25,0),("sprint_speed_modifier","sprintSpeedModifier",26,0),("shift_key_down","isShiftKeyDown",28,0),("in_water","isInWater",29,0),("in_lava","isInLava",30,0),("on_climbable","onClimbable",31,0),("on_ground_no_blocks","onGroundNoBlocks",32,0),("crouching","crouching",33,0),("swimming","swimming",34,0),("underwater","underwater",35,0),("eye_in_water","eyeInWater",36,0),("was_eye_in_water","wasEyeInWater",37,0),("in_powder_snow","inPowderSnow",38,0),("was_in_powder_snow","wasInPowderSnow",39,0),("leather_boots","leatherBoots",40,0),
 ("health","healthBits",47,8),("saturation","saturationLevelBits",48,8),("exhaustion","exhaustionBits",49,8),("speed","speedBits",50,8),("step_height","stepHeightBits",51,8),("friction_modifier","frictionModifierBits",52,8),("air_drag_modifier","airDragModifierBits",53,8),("water_movement_efficiency","waterMovementEfficiencyBits",54,8),("movement_efficiency","movementEfficiencyBits",55,8),("sneaking_speed","sneakingSpeedBits",56,8),
 ("fall_distance","fallDistanceBits",57,16),("water_height","waterHeightBits",58,16),("lava_height","lavaHeightBits",59,16),("movement_speed_base","movementSpeedBaseBits",60,16),("jump_strength","jumpStrengthBits",61,16),("gravity","gravityBits",62,16),("safe_fall_distance","safeFallDistanceBits",63,16),("fall_damage_multiplier","fallDamageMultiplierBits",64,16),("bounciness","bouncinessBits",65,16),
 ("position.x","position.xBits",66,16),("position.y","position.yBits",67,16),("position.z","position.zBits",68,16),("velocity.x","velocity.xBits",69,16),("velocity.y","velocity.yBits",70,16),("velocity.z","velocity.zBits",71,16),("stuck_speed_multiplier.x","stuckSpeedMultiplier.xBits",72,16),("stuck_speed_multiplier.y","stuckSpeedMultiplier.yBits",73,16),("stuck_speed_multiplier.z","stuckSpeedMultiplier.zBits",74,16),
];
const NEVER: &[&str] = &["t", "hurt_cooldown.remaining_ticks", "hurt_cooldown.last_raw_hit", "food_tick_timer", "remaining_fire_ticks", "no_jump_delay", "no_physics", "fall_flying", "riding", "boat", "effects", "flying", "using_item", "movement_this_tick", "main_supporting_block"];
fn get<'a>(o: &Object<'a>, path: &str) -> Option<&'a str> { if let Some((parent, child)) = path.split_once('.') { o.get(parent).and_then(|s| Object::parse(s).get(child)) } else { o.get(path) } }
fn value(raw: &str, width: u8) -> u64 { if width != 0 { hex(raw, width as usize) } else { match raw { "true" => 1, "false" => 0, "\"STANDING\"" => 0, "\"CROUCHING\"" => 1, "\"SWIMMING\"" => 2, _ => raw.parse::<i64>().expect("invalid observed scalar") as u64 } } }
fn rows(text: &str) -> Vec<&str> { text.lines().filter(|s| !s.trim().is_empty()).collect() }
/// Compare complete JSONL documents. Invalid, incomplete, or unalignable data is refused.
pub fn compare(plan: &str, recording: &str) -> Result<Report, String> {
    std::panic::catch_unwind(|| compare_inner(plan, recording)).map_err(|e| e.downcast_ref::<String>().cloned().or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_else(|| "invalid input".into()))
}
fn compare_inner(plan: &str, recording: &str) -> Report {
    let p = rows(plan); let r = rows(recording);
    assert!(p.len() >= 2 && r.len() >= 3, "empty plan or recording");
    let header = Object::parse(p[0]); assert!(header.get("start").is_some() && header.get("start_bits").is_some(), "missing plan start");
    assert_eq!(Object::parse(r[0]).string("type"), "recording", "missing recording header");
    let end = Object::parse(r[r.len()-1]); assert_eq!(end.string("type"), "end"); assert!(end.boolean("complete"), "incomplete recording");
    let pn = p.len()-1; let rn = r.len()-2; assert_eq!(end.required("ticks").parse::<usize>().unwrap(), rn, "wrong trailer count");
    let mut unobserved: BTreeSet<String> = NEVER.iter().map(|s| s.to_string()).collect();
    let mut first = None; let mut corrections = 0; let mut pushes = 0;
    for (i, line) in r[1..r.len()-1].iter().enumerate() {
        let obs = Object::parse(line); assert_eq!(obs.required("t").parse::<usize>().unwrap(), i, "recording tick gap");
        if let Some(a) = obs.get("anomaly") { for item in array(a) { match Object::parse(item).string("type") { "ClientboundPlayerPositionPacket" | "ClientboundPlayerRotationPacket" => corrections += 1, "ClientboundSetEntityMotionPacket" | "ClientboundExplodePacket" => pushes += 1, _ => panic!("unknown anomaly type") } } }
        for &(name, path, _, _) in FIELDS { if get(&obs,path).is_none() { unobserved.insert(name.to_string()); } }
        if obs.get("speedModifiers").is_none() { unobserved.insert("speed_modifiers".into()); }
        if i < pn {
            let pred = Object::parse(p[i+1]); assert_eq!(pred.required("t").parse::<usize>().unwrap(), i, "plan tick gap");
            let bits: Vec<u64> = array(pred.required("state_bits")).iter().map(|x| x.parse().expect("invalid state bits")).collect(); assert!(bits.len() >= 78, "short state bits");
            let mut diffs = Vec::new();
            // FIELDS index the layout before 1.4b: from word 57 (13 scalars, 34 flags, 10 floats) the state now
            // carries use_item_speed_multiplier and use_item_can_sprint, each a presence word plus its value when present.
            let at = |index: usize| if index < 57 { index } else { let a = bits[57] as usize; index + 2 + a + bits[58 + a] as usize };
            for &(name, path, index, width) in FIELDS { if let Some(raw) = get(&obs,path) { let observed = value(raw,width); if bits[at(index)] != observed { let w = if width == 0 { 16 } else { width as usize }; diffs.push(Difference {field:name.into(), predicted_bits:format!("{:0w$x}",bits[at(index)]), recorded_bits:format!("{:0w$x}",observed)}); } } }
            if let Some(raw) = obs.get("speedModifiers") {
                let mut at = at(75);
                for _ in 0..2 { let present = bits[at]; assert!(present <= 1, "invalid optional state"); at += 1 + if present == 1 { 3 } else { 0 }; }
                let predicted = &bits[at..];
                let tuples = array(raw); let mut recorded = vec![tuples.len() as u64];
                for tuple in tuples {
                    let parts = array(tuple); assert_eq!(parts.len(), 3, "invalid speed modifier");
                    let id = parts[0].strip_prefix('"').and_then(|s| s.strip_suffix('"')).expect("invalid modifier id");
                    assert!(!id.contains('\\'), "escaped modifier id unsupported");
                    recorded.push(id.len() as u64); recorded.extend(id.bytes().map(u64::from));
                    let amount: f64 = parts[1].parse().expect("invalid modifier amount"); assert!(amount.is_finite()); recorded.push(amount.to_bits());
                    recorded.push(match parts[2].trim_matches('"') { "0" | "ADD_VALUE" | "add_value" => 0, "1" | "ADD_MULTIPLIED_BASE" | "add_multiplied_base" => 1, "2" | "ADD_MULTIPLIED_TOTAL" | "add_multiplied_total" => 2, _ => panic!("invalid modifier operation") });
                }
                if predicted != recorded { let render = |v: &[u64]| v.iter().map(|n| format!("{n:016x}")).collect::<Vec<_>>().join("-"); diffs.push(Difference { field: "speed_modifiers".into(), predicted_bits: render(predicted), recorded_bits: render(&recorded) }); }
            } else { unobserved.insert("speed_modifiers".into()); }
            if !diffs.is_empty() && first.is_none() { first = Some(Deviation {tick:i as u64,fields:diffs}); }
        }
    }
    for (i,line) in p[1..].iter().enumerate().skip(rn) { assert_eq!(Object::parse(line).required("t").parse::<usize>().unwrap(), i, "plan tick gap"); }
    Report {first_deviation:first, corrections, pushes, recording_end_tick:(rn-1) as u64, plan_arrival_tick:(pn-1) as u64, unobserved:unobserved.into_iter().collect()}
}
