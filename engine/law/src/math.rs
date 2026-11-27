const SIN: &[u8; 262144] = include_bytes!("sin.bin");
const SIN_SCALE: f64 = 10430.378350470453;

/// Raw 65,536-entry `f32` little-endian Mojang sine LUT bytes for GPU upload.
pub fn sin_table_bytes() -> &'static [u8; 262144] {
    SIN
}

// Mth.sin
#[inline(always)]
pub fn sin(value: f64) -> f32 {
    table(((value * SIN_SCALE) as i64 & 65535) as usize)
}

// Mth.cos
#[inline(always)]
pub fn cos(value: f64) -> f32 {
    table(((value * SIN_SCALE + 16384.0) as i64 & 65535) as usize)
}

// Mth.lambda$static$0
#[inline(always)]
fn table(index: usize) -> f32 {
    let base = (index & 65535) * 4;
    f32::from_le_bytes([SIN[base], SIN[base + 1], SIN[base + 2], SIN[base + 3]])
}

// Mth.floor
#[inline(always)]
pub fn floor(value: f64) -> i32 {
    if value >= -2147483648.0 && value < 2147483648.0 {
        let i = value as i32;
        i - i32::from(value < i as f64)
    } else {
        value.floor() as i32
    }
}

// Mth.lfloor
#[inline(always)]
pub fn lfloor(value: f64) -> i64 {
    value.floor() as i64
}

// Mth.ceil
#[inline(always)]
pub fn ceil(value: f64) -> i32 {
    if value > -2147483648.0 && value <= 2147483647.0 {
        let i = value as i32;
        i + i32::from(value > i as f64)
    } else {
        value.ceil() as i32
    }
}

// Mth.floor(float): f2d, Math.floor, d2i
#[inline(always)]
pub fn floor_f32(value: f32) -> i32 {
    floor(value as f64)
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

pub(crate) fn sin_cos_degrees(angle: f32) -> (f32, f32) {
    let rad = (angle * 0.017453292f32) as f64;
    (sin(rad), cos(rad))
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
/// `Mth.fastInvSqrt(double) @0-41`.
pub fn fast_inv_sqrt(x: f64) -> f64 {
    let half = 0.5 * x;
    let bits = 6910469410427058090i64.wrapping_sub((x.to_bits() as i64) >> 1);
    let est = f64::from_bits(bits as u64);
    est * (1.5 - half * est * est)
}

/// `Mth.atan2(double y, double x)` using `FRAC_BIAS`, `ASIN_TAB`, and `COS_TAB`.
pub fn atan2(mut y: f64, mut x: f64) -> f64 {
    const FRAC_BIAS: f64 = f64::from_bits(4805340802404319232);
    static TABS: std::sync::OnceLock<([f64; 257], [f64; 257])> = std::sync::OnceLock::new();
    let (asin_tab, cos_tab) = TABS.get_or_init(|| {
        let mut a = [0.0; 257];
        let mut c = [0.0; 257];
        for i in 0..257 {
            let asin = ((i as f64) / 256.0).asin();
            c[i] = asin.cos();
            a[i] = asin;
        }
        (a, c)
    });

    let d0 = x * x + y * y;
    if d0.is_nan() {
        return f64::NAN;
    }
    let neg_y = y < 0.0;
    if neg_y {
        y = -y;
    }
    let neg_x = x < 0.0;
    if neg_x {
        x = -x;
    }
    let y_gt_x = y > x;
    if y_gt_x {
        std::mem::swap(&mut x, &mut y);
    }
    let inv = fast_inv_sqrt(d0);
    x *= inv;
    y *= inv;
    let biased = FRAC_BIAS + y;
    let idx = (biased.to_bits() as u32) as usize;
    let tab_asin = asin_tab[idx];
    let tab_cos = cos_tab[idx];
    let tab_sin = biased - FRAC_BIAS;
    let diff = y * tab_cos - x * tab_sin;
    let corr = (6.0 + diff * diff) * diff * 0.16666666666666666;
    let mut angle = tab_asin + corr;
    if y_gt_x {
        angle = 1.5707963267948966 - angle;
    }
    if neg_x {
        angle = 3.141592653589793 - angle;
    }
    if neg_y {
        angle = -angle;
    }
    angle
}

