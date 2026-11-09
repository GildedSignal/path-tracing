#![allow(dead_code)]
#[path = "../../engine/law/src/math.rs"]
mod math;
#[path = "../../engine/law/src/shapes.rs"]
mod shapes;

use shapes::Vec3;
use std::{env, fs, process::ExitCode};

fn glide(mut velocity: Vec3, yaw: f32, pitch: f32, cosine: fn(f64) -> f64) -> Vec3 {
    // ElytraOracle sets the requested pitch through Entity.setXRot.
    let pitch = (pitch % 360.0).clamp(-90.0, 90.0) * 0.017453292f32;
    let yaw = -yaw * 0.017453292f32;
    let look_cos = math::cos(pitch as f64);
    let look_x = (math::sin(yaw as f64) * look_cos) as f64;
    let look_z = (math::cos(yaw as f64) * look_cos) as f64;
    let look_horizontal = (look_x * look_x + look_z * look_z).sqrt();
    let speed = velocity.horizontal_sqr().sqrt();
    let lift = cosine(pitch as f64);
    let lift = lift * lift;

    velocity = velocity.plus(Vec3::new(0.0, 0.08 * (-1.0 + 0.75 * lift), 0.0));
    if velocity.y < 0.0 && look_horizontal > 0.0 {
        let correction = velocity.y * -0.1 * lift;
        velocity = velocity.plus(Vec3::new(
            look_x * correction / look_horizontal,
            correction,
            look_z * correction / look_horizontal,
        ));
    }
    if pitch < 0.0 && look_horizontal > 0.0 {
        let correction = speed * (-math::sin(pitch as f64)) as f64 * 0.04;
        velocity = velocity.plus(Vec3::new(
            -look_x * correction / look_horizontal,
            correction * 3.2,
            -look_z * correction / look_horizontal,
        ));
    }
    if look_horizontal > 0.0 {
        velocity = velocity.plus(Vec3::new(
            (look_x / look_horizontal * speed - velocity.x) * 0.1,
            0.0,
            (look_z / look_horizontal * speed - velocity.z) * 0.1,
        ));
    }
    velocity.multiply(Vec3::new(0.99f32 as f64, 0.98f32 as f64, 0.99f32 as f64))
}

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let path = args.next().expect("oracle CSV");
    let cosine: fn(f64) -> f64 = match args.next().as_deref() {
        None => f64::cos,
        Some("table-cosine") => |value| math::cos(value) as f64,
        Some(argument) => panic!("unknown argument: {argument}"),
    };
    let raw = fs::read_to_string(path).expect("read oracle CSV");
    let mut rows = raw.lines();
    assert_eq!(rows.next(), Some("yaw,pitch,vx,vy,vz,next_vx,next_vy,next_vz"));
    let mut first = None;
    let mut differences = 0;
    let mut component_differences = [0; 3];
    let mut cases = 0;
    for (index, row) in rows.enumerate() {
        let fields: [u64; 8] = row.split(',')
            .map(|bits| u64::from_str_radix(bits, 16).expect("raw hexadecimal field"))
            .collect::<Vec<_>>().try_into().expect("eight fields");
        let [yaw, pitch, x, y, z, ex, ey, ez] = fields;
        let input = Vec3::new(f64::from_bits(x), f64::from_bits(y), f64::from_bits(z));
        let actual = glide(input, f32::from_bits(yaw.try_into().unwrap()), f32::from_bits(pitch.try_into().unwrap()), cosine);
        let observed = [actual.x.to_bits(), actual.y.to_bits(), actual.z.to_bits()];
        let expected = [ex, ey, ez];
        if observed != expected {
            differences += 1;
            for (axis, (a, e)) in observed.into_iter().zip(expected).enumerate() {
                component_differences[axis] += usize::from(a != e);
            }
            first.get_or_insert((index + 1, row, observed));
        }
        cases += 1;
    }
    assert_eq!(cases, 8204, "incomplete oracle output");
    println!("cases={cases} mismatches={differences} x/y/z={component_differences:?}");
    if let Some((index, row, observed)) = first {
        println!("first mismatch at case {index}: oracle row={row}, Rust={observed:016x?}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
