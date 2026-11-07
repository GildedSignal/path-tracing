#![allow(dead_code)]
#[path = "../../engine/law/src/math.rs"]
mod math;
#[path = "../../engine/law/src/shapes.rs"]
mod shapes;

use shapes::Vec3;
use std::{env, fs, process::ExitCode};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Empty,
    Water,
    Lava,
}

struct Cell {
    kind: Kind,
    amount: u8,
    blocks_motion: bool,
    sturdy_horizontal_face: bool,
}

impl Cell {
    fn decode(name: &str) -> Self {
        let (kind, amount, blocks_motion, sturdy_horizontal_face) = match name {
            "air" => (Kind::Empty, 0, false, false),
            "stone" => (Kind::Empty, 0, true, true),
            "ice" | "slab" => (Kind::Empty, 0, true, false),
            "wet-slab" => (Kind::Water, 8, true, false),
            _ => {
                let kind = match name.as_bytes()[0] {
                    b'w' => Kind::Water,
                    b'l' => Kind::Lava,
                    _ => panic!("unknown cell {name}"),
                };
                (kind, name[1..].parse().unwrap(), false, false)
            }
        };
        Self { kind, amount, blocks_motion, sturdy_horizontal_face }
    }

    fn height(&self) -> f32 {
        self.amount as f32 / 9.0
    }
}

fn flow(kind: Kind, amount: u8, falling: bool, cells: &[Cell; 12], ignore_above: bool) -> Vec3 {
    let current = amount as f32 / 9.0;
    let mut velocity = Vec3::default();
    let directions = [(0.0f32, -1.0f32), (1.0, 0.0), (0.0, 1.0), (-1.0, 0.0)];
    for (index, (x, z)) in directions.into_iter().enumerate() {
        let neighbor = &cells[index];
        let below = &cells[index + 4];
        let difference = if neighbor.kind == kind {
            current - neighbor.height()
        } else if neighbor.kind == Kind::Empty && !neighbor.blocks_motion && below.kind == kind {
            current - (below.height() - 0.8888889f32)
        } else {
            continue;
        };
        if difference != 0.0 {
            velocity.x += (x * difference) as f64;
            velocity.z += (z * difference) as f64;
        }
    }

    let wall = |cell: &Cell| cell.kind != kind && cell.sturdy_horizontal_face;
    if falling && (cells[..4].iter().any(wall) || (!ignore_above && cells[8..].iter().any(wall))) {
        velocity = velocity.normalize().plus(Vec3::new(0.0, -6.0, 0.0));
    }
    velocity.normalize()
}

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let path = args.next().expect("oracle CSV");
    let ignore_above = match args.next().as_deref() {
        None => false,
        Some("ignore-above") => true,
        Some(other) => panic!("unknown argument: {other}"),
    };
    let raw = fs::read_to_string(path).expect("read oracle CSV");
    let mut rows = raw.lines();
    assert_eq!(rows.next(), Some("case,kind,amount,falling,n,e,s,w,n_below,e_below,s_below,w_below,n_above,e_above,s_above,w_above,x,y,z"));
    let mut first = None;
    let mut differences = 0;
    let mut axes = [0; 3];
    let mut cases = 0;
    for row in rows {
        let fields: [&str; 19] = row.split(',').collect::<Vec<_>>().try_into().expect("nineteen fields");
        let kind = match fields[1] {
            "water" => Kind::Water,
            "lava" => Kind::Lava,
            _ => panic!("unexpected fluid"),
        };
        let cells = std::array::from_fn(|i| Cell::decode(fields[4 + i]));
        let actual = flow(kind, fields[2].parse().unwrap(), fields[3].parse().unwrap(), &cells, ignore_above);
        let actual = [actual.x.to_bits(), actual.y.to_bits(), actual.z.to_bits()];
        let expected = std::array::from_fn(|i| u64::from_str_radix(fields[16 + i], 16).unwrap());
        if actual != expected {
            differences += 1;
            for (i, (a, e)) in actual.into_iter().zip(expected).enumerate() {
                axes[i] += usize::from(a != e);
            }
            first.get_or_insert((row, actual));
        }
        cases += 1;
    }
    assert_eq!(cases, 1034);
    println!("cases={cases} mismatches={differences} x/y/z={axes:?}");
    if let Some((row, actual)) = first {
        println!("first mismatch oracle={row} Rust={actual:016x?}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
