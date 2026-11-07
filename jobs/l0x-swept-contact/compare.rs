use law::{Aabb, Vec3, VoxelShape};
use std::{collections::BTreeMap, env, fs};

fn aabb(values: &[f64]) -> Aabb {
    Aabb::new(values[0], values[1], values[2], values[3], values[4], values[5])
}

fn main() {
    let file = env::args().nth(1).expect("oracle path");
    let source = fs::read_to_string(file).unwrap();
    let mut groups: BTreeMap<&str, (usize, usize, usize, Option<usize>)> = BTreeMap::new();
    for (index, line) in source.lines().enumerate() {
        let mut fields: Vec<_> = line.split_ascii_whitespace().collect();
        let expected: bool = fields.pop().unwrap().parse().unwrap();
        let numbers: Vec<_> = fields[1..].iter().map(|v| {
            f64::from_bits(u64::from_str_radix(v, 16).unwrap())
        }).collect();
        let body = aabb(&numbers[..6]);
        let motion = Vec3::new(numbers[6], numbers[7], numbers[8]);
        let hazards: Vec<_> = numbers[9..].chunks_exact(6).map(aabb).collect();
        let shape = VoxelShape::boxes(&hazards);
        let actual = body.collided_along(motion, &shape);
        let group = groups.entry(fields[0]).or_default();
        group.0 += 1;
        group.1 += usize::from(expected);
        if actual != expected {
            group.2 += 1;
            group.3.get_or_insert(index + 1);
        }
    }
    let mut failed = false;
    for (label, (total, hits, mismatches, first)) in groups {
        println!("{label}: {total} cases, {hits} hits, {} misses, {mismatches} disagreements, first line {first:?}", total - hits);
        if let Some(line) = first {
            println!("  {}", source.lines().nth(line - 1).unwrap());
        }
        failed |= mismatches > 0 || hits == 0 || hits == total;
    }
    if failed {
        std::process::exit(1);
    }
}
