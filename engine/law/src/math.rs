const SIN: &[u8; 262144] = include_bytes!("sin.bin");
const SIN_SCALE: f64 = 10430.378350470453;

// Mth.sin
pub fn sin(value: f64) -> f32 {
    table(((value * SIN_SCALE) as i64 & 65535) as usize)
}

// Mth.cos
pub fn cos(value: f64) -> f32 {
    table(((value * SIN_SCALE + 16384.0) as i64 & 65535) as usize)
}

// Mth.lambda$static$0
fn table(index: usize) -> f32 {
    f32::from_le_bytes(SIN[index * 4..index * 4 + 4].try_into().unwrap())
}

// Mth.floor
pub fn floor(value: f64) -> i32 {
    value.floor() as i32
}

// Mth.lfloor
pub fn lfloor(value: f64) -> i64 {
    value.floor() as i64
}

// Mth.sqrt
pub fn sqrt(value: f32) -> f32 {
    (value as f64).sqrt() as f32
}

// Mth.equal
pub fn equal(a: f64, b: f64) -> bool {
    (b - a).abs() < 1.0e-5
}

// LivingEntity.computeModifiedFriction
pub fn modified_friction(friction: f32, modifier: f32) -> f32 {
    (1.0f32 - (1.0f32 - friction) * modifier).clamp(0.0, 1.0)
}

// Math.round
pub fn round(value: f32) -> i32 {
    ((value as f64) + 0.5).floor() as i32
}
