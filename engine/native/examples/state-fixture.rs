use law::{PlayerState,Input,Pose,HurtCooldown,Vec3,SpeedModifier,SpeedOperation};
fn main() {
    let s = PlayerState {
        t:123,input:Input { forward:true,back:true,left:true,right:true,jump:true,sneak:true,sprint:true,yaw:3.5,pitch:-2.0 },
        position:Vec3::new(-0.0,-60.0,123.25),velocity:Vec3::new(1.0,2.0,3.0),yaw:4.5,pitch:-3.0,
        on_ground:true,horizontal_collision:true,vertical_collision:true,vertical_collision_below:true,minor_horizontal_collision:true,
        sprinting:true,sprint_speed_modifier:true,no_physics:true,shift_key_down:true,pose:Pose::Swimming,fall_distance:1.25,
        in_water:true,in_lava:true,on_climbable:true,health:1.25,hurt_cooldown:HurtCooldown { remaining_ticks:-7,last_raw_hit:2.25 },
        food_level:7,saturation:1.25,exhaustion:1.25,food_tick_timer:1234,remaining_fire_ticks:-20,no_jump_delay:3,
        stuck_speed_multiplier:Vec3::new(1.0,2.0,3.0),movement_this_tick:Some(Vec3::new(1.0,2.0,3.0)),main_supporting_block:Some([-1,-61,2]),
        on_ground_no_blocks:true,crouching:true,swimming:true,underwater:true,eye_in_water:true,was_eye_in_water:true,
        water_height:1.25,lava_height:1.25,in_powder_snow:true,was_in_powder_snow:true,ticks_frozen:2,leather_boots:true,
        movement_speed_base:1.25,speed_modifiers:vec![SpeedModifier { id:"minecraft:é".into(),amount:1.25,operation:SpeedOperation::AddMultipliedTotal }],
        speed:1.25,jump_strength:1.25,gravity:1.25,step_height:1.25,friction_modifier:1.25,air_drag_modifier:1.25,
        water_movement_efficiency:1.25,movement_efficiency:1.25,sneaking_speed:1.25,safe_fall_distance:1.25,fall_damage_multiplier:1.25,
        bounciness:1.25,fall_flying:true,riding:true,boat:true,effects:true,flying:true,using_item:true,
        use_item_speed_multiplier:Some(1.25),use_item_can_sprint:Some(false),
    };
    println!("{}",s.bits().iter().map(|n|format!("{n:016x}")).collect::<Vec<_>>().join(" "));
}
