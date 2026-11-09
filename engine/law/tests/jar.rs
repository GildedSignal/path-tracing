use law::{
    Aabb, 
    Vec3, 
    VoxelShape, 
    shapes::collide_with_shapes
};

#[test]
fn jar_collision_corpus(){
    for (line, row) in include_str!("data/shapes.tsv").lines().enumerate(){
        let v:Vec<_>=row.split_whitespace().map(|s|f64::from_bits(u64::from_str_radix(s, 16).unwrap())).collect();
        assert_eq!(v.len(), 31);
        let body=Aabb::new(v[0], v[1], v[2], v[3], v[4], v[5]);
        let boxes:Vec<_>=v[9..27].chunks(6).map(|a|Aabb::new(a[0], a[1], a[2], a[3], a[4], a[5])).collect();
        assert_eq!(VoxelShape::boxes(&boxes).intersects(body), v[30]==1.0, "jar overlap case {line}");
        let actual=collide_with_shapes(Vec3::new(v[6], v[7], v[8]), body, &[VoxelShape::boxes(&boxes)]);
        for (a, e) in [actual.x, actual.y, actual.z].into_iter().zip(&v[27..30]){
            assert_eq!(a.to_bits(), e.to_bits(), "jar shape case {line}: {actual:?}");
        }
    }
}

#[test]
fn jar_intersection_corpus() {
    for (line, row) in include_str!("data/intersections.tsv").lines().enumerate() {
        let (values, visits)=row.split_once('|').unwrap();
        let values: Vec<_>=values.split_whitespace().map(|v|f64::from_bits(u64::from_str_radix(v, 16).unwrap())).collect();
        assert_eq!(values.len(), 12);
        let expected: Vec<[i32; 4]>=visits.split(';').filter(|s|!s.is_empty()).map(|s| {
            let p:Vec<i32>=s.split(',').map(|v|v.parse().unwrap()).collect();
            p.try_into().unwrap()
        }).collect();
        let mut actual=Vec::new();
        law::blocks::for_each_intersected(Vec3::new(values[0], values[1], values[2]), Vec3::new(values[3], values[4], values[5]), 
        Aabb::new(values[6], values[7], values[8], values[9], values[10], values[11]), |p, step| {
            actual.push([p[0], p[1], p[2], step]);
            true
        });
        assert_eq!(actual, expected, "jar block intersection case {line}");
    }
}

#[test]
fn jar_epsilon_boundaries() {
    for row in include_str!("data/epsilons.tsv").lines() {
        let fields: Vec<_> = row.split_whitespace().collect();
        let value = f64::from_bits(u64::from_str_radix(fields[0], 16).unwrap());
        assert_eq!(law::math::equal(0.0, value), fields[1] == "true", "equal {value:?}");
        let expected = u64::from_str_radix(fields[2], 16).unwrap();
        assert_eq!(Vec3::new(value, 0.0, 0.0).normalize().x.to_bits(), expected, "normalize {value:?}");
    }
}

#[test]
fn jar_ray_traversal_and_clipping() {
    use law::{Block, BoxWorld, Fluid, FluidKind, World};
    for (line, row) in include_str!("data/rays.tsv").lines().enumerate() {
        let (values, visits) = row.split_once('|').unwrap();
        let fields: Vec<_> = values.split_whitespace().collect();
        let coordinates: Vec<_> = fields[..6].iter().map(|v| f64::from_bits(u64::from_str_radix(v, 16).unwrap())).collect();
        let from = Vec3::new(coordinates[0], coordinates[1], coordinates[2]);
        let to = Vec3::new(coordinates[3], coordinates[4], coordinates[5]);
        let target = [fields[6].parse().unwrap(), fields[7].parse().unwrap(), fields[8].parse().unwrap()];
        let height = f32::from_bits(u32::from_str_radix(fields[9], 16).unwrap());
        let expected: Vec<[i32; 3]> = visits.split(';').filter(|s| !s.is_empty()).map(|s| {
            s.split(',').map(|v| v.parse().unwrap()).collect::<Vec<_>>().try_into().unwrap()
        }).collect();
        let mut actual = Vec::new();
        law::blocks::traverse_blocks(from, to, |p| { actual.push(p); false });
        assert_eq!(actual, expected, "jar ray visits {line}");
        let mut world = BoxWorld::default();
        world.set(target, Block {
            air: false, fluid: Fluid { kind: FluidKind::Water, height, ..Fluid::default() }, ..Block::default()
        });
        assert_eq!(world.fall_reset_clip(from, to), fields[10] == "true", "jar ray hit {line}");
    }
}

#[test]
fn jar_landing_hooks() {
    use law::{Block, BoxWorld, Input, PlayerState};
    for row in include_str!("data/landings.tsv").lines() {
        let fields: Vec<_> = row.split_whitespace().collect();
        let distance: f64 = fields[1].parse().unwrap();
        let effective_distance = f64::from_bits(u64::from_str_radix(fields[2], 16).unwrap());
        let multiplier = f32::from_bits(u32::from_str_radix(fields[3], 16).unwrap());
        let mut block = Block::full();
        block.bed = fields[0] == "BedBlock";
        block.fall_on_multiplier = if block.bed { 1.0 } else { 0.2 };
        let mut world = BoxWorld::default();
        world.fill([-1, -1, -1], [1, -1, 1], block);
        let mut state = PlayerState {
            on_ground: true,
            fall_distance: distance,
            velocity: Vec3::new(0.0, -0.08*0.98f32 as f64, 0.0),
            health: 100.0,
            ..PlayerState::default()
        };
        assert_eq!(effective_distance, if fields[0] == "BedBlock" { distance*0.5 } else { distance });
        assert_eq!(multiplier, if fields[0] == "BedBlock" { 1.0 } else { 0.2 });
        let expected: i32 = fields[4].parse::<i32>().unwrap().max(0);
        law::tick(&mut state, &world, &Input::default());
        assert_eq!(100.0-state.health, expected as f32, "{row}");
    }
}
#[test]
fn jar_movement_speed_attribute_order() {
    for (line, row) in include_str!("data/attributes.tsv").lines().enumerate() {
        let (header, modifiers) = row.split_once('|').unwrap();
        let header: Vec<_> = header.split_whitespace().collect();
        let from_hex = |s| f64::from_bits(u64::from_str_radix(s, 16).unwrap());
        let mut state = law::PlayerState {
            movement_speed_base: from_hex(header[0]),
            sprinting: header[1] == "true",
            sprint_speed_modifier: header[1] == "true",
            ..law::PlayerState::default()
        };
        state.speed_modifiers = modifiers.split(';').filter(|s| !s.is_empty()).map(|entry| {
            let fields: Vec<_> = entry.split_whitespace().collect();
            law::SpeedModifier {
                id: fields[0].to_owned(),
                amount: from_hex(fields[1]),
                operation: match fields[2] {
                    "0" => law::SpeedOperation::AddValue,
                    "1" => law::SpeedOperation::AddMultipliedBase,
                    "2" => law::SpeedOperation::AddMultipliedTotal,
                    _ => panic!("attribute operation"),
                },
            }
        }).collect();
        assert_eq!(state.movement_speed_value().to_bits(), from_hex(header[2]).to_bits(),
            "jar attribute case {line}");
    }
}
