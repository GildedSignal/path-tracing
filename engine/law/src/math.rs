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

// Mth.ceil
pub fn ceil(value: f64) -> i32 {
    value.ceil() as i32
}

// Mth.floor(float): f2d, Math.floor, d2i
pub fn floor_f32(value: f32) -> i32 {
    (value as f64).floor() as i32
}

// Mth.lerp(DDD): start + delta * (end - start)
pub fn lerp(delta: f64, start: f64, end: f64) -> f64 {
    start + delta * (end - start)
}

// Mth.sqrt
pub fn sqrt(value: f32) -> f32 {
    (value as f64).sqrt() as f32
}

// Mth.equal
pub fn equal(a: f64, b: f64) -> bool {
    (b - a).abs() < 1.0e-5f32 as f64
}

// LivingEntity.computeModifiedFriction
pub fn modified_friction(friction: f32, modifier: f32) -> f32 {
    (1.0f32 - (1.0f32 - friction) * modifier).clamp(0.0, 1.0)
}

// Math.round
pub fn round(value: f32) -> i32 {
    ((value as f64) + 0.5).floor() as i32
}

// Mth.clamp(FFF): value < min ? min : Math.min(value, max)
pub fn clamp_f32(value: f32, min: f32, max: f32) -> f32 {
    if value < min { min } else { value.min(max) }
}

// Mth.wrapDegrees(F)
pub fn wrap_degrees(value: f32) -> f32 {
    let mut f = value % 360.0f32;
    if f >= 180.0f32 {
        f -= 360.0f32;
    }
    if f < -180.0f32 {
        f += 360.0f32;
    }
    f
}

// Entity.calculateViewVector(FF), argument order (yaw, pitch)
pub fn calculate_view_vector(yaw: f32, pitch: f32) -> crate::Vec3 {
    let f = pitch * 0.017453292f32;
    let g = -yaw * 0.017453292f32;
    let (sin_g, cos_g, sin_f, cos_f) = (sin(g as f64), cos(g as f64), sin(f as f64), cos(f as f64));
    crate::Vec3::new((sin_g * cos_f) as f64, (-sin_f) as f64, (cos_g * cos_f) as f64)
}

/// `RandomSource.create(long)` = `LegacyRandomSource` / `SingleThreadedRandomSource`: the 48-bit
/// java.util.Random LCG, `seed = (s ^ 0x5DEECE66D) & (2^48 - 1)`, `next(bits) = seed' >>> (48 - bits)`,
/// with `MarsagliaPolarGaussian` for gaussians.
#[derive(Clone, Debug)]
pub struct JavaRandom {
    seed: u64,
    next_gaussian: f64,
    has_gaussian: bool,
}

impl JavaRandom {
    pub fn new(seed: i64) -> Self {
        Self { seed: (seed as u64 ^ 0x5DEECE66D) & 0xFFFF_FFFF_FFFF, next_gaussian: 0.0, has_gaussian: false }
    }

    fn next(&mut self, bits: u32) -> i32 {
        self.seed = self.seed.wrapping_mul(0x5DEECE66D).wrapping_add(0xB) & 0xFFFF_FFFF_FFFF;
        (self.seed >> (48 - bits)) as i32
    }

    /// RandomSource.nextFloat default: `next(24) / (1 << 24)`.
    pub fn next_float(&mut self) -> f32 {
        self.next(24) as f32 / (1 << 24) as f32
    }

    /// RandomSource.nextDouble default: `((next(26) << 27) + next(27)) * 2^-53`.
    pub fn next_double(&mut self) -> f64 {
        (((self.next(26) as i64) << 27) + self.next(27) as i64) as f64 / (1u64 << 53) as f64
    }

    /// MarsagliaPolarGaussian.nextGaussian 17-108.
    pub fn next_gaussian(&mut self) -> f64 {
        if self.has_gaussian {
            self.has_gaussian = false;
            return self.next_gaussian;
        }
        loop {
            let d0 = 2.0 * self.next_double() - 1.0;
            let d1 = 2.0 * self.next_double() - 1.0;
            let s = d0 * d0 + d1 * d1;
            if s < 1.0 && s > 0.0 {
                let g = (-2.0 * s.ln() / s).sqrt();
                self.next_gaussian = d1 * g;
                self.has_gaussian = true;
                return d0 * g;
            }
        }
    }
}
