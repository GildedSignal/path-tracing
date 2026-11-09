use crate::Vec3;

/// Inputs for one side of Entity.push(Entity), at the instant of the call.
#[derive(Clone, Copy, Debug)]
pub struct ContactEntity {
    pub position: Vec3,
    pub velocity: Vec3,
    pub no_physics: bool,
    pub vehicle: bool,
    pub pushable: bool,
}

/// Applies the base Entity pair law in caller/other order. The caller must handle
/// virtual overrides and supply the pair in the order vanilla invokes it.
pub fn push_pair(this: &mut ContactEntity, other: &mut ContactEntity, same_vehicle: bool) {
    if same_vehicle || other.no_physics || this.no_physics { return; }
    let mut x = other.position.x - this.position.x;
    let mut z = other.position.z - this.position.z;
    // Mth.absMax uses Math.max(Math.abs(x), Math.abs(z)).
    let ax = x.abs();
    let az = z.abs();
    let mut distance = if ax.is_nan() || az.is_nan() { f64::NAN } else { ax.max(az) };
    if distance < 0.01_f32 as f64 { return; }
    distance = distance.sqrt();
    x /= distance;
    z /= distance;
    let mut scale = 1.0 / distance;
    if scale > 1.0 { scale = 1.0; }
    x *= scale;
    z *= scale;
    x *= 0.05_f32 as f64;
    z *= 0.05_f32 as f64;
    if x.is_finite() && z.is_finite() {
        if !this.vehicle && this.pushable { this.velocity = this.velocity.plus(Vec3::new(-x, 0.0, -z)); }
        if !other.vehicle && other.pushable { other.velocity = other.velocity.plus(Vec3::new(x, 0.0, z)); }
    }
}
