use law::*;
use std::{alloc::{GlobalAlloc,Layout,System},cell::Cell};
thread_local! {
    static TRACK: Cell<bool> = const {Cell::new(false)};
    static ALLOCS: Cell<usize> = const {Cell::new(0)};
}
struct Allocator;
unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self,l:Layout)->*mut u8 {
        TRACK.with(|v|if v.get() {ALLOCS.with(|n|n.set(n.get()+1))});
        unsafe {System.alloc(l)}
    }
    unsafe fn dealloc(&self,p:*mut u8,l:Layout) {unsafe {System.dealloc(p,l)}}
    unsafe fn realloc(&self,p:*mut u8,l:Layout,n:usize)->*mut u8 {
        TRACK.with(|v|if v.get() {ALLOCS.with(|n|n.set(n.get()+1))});
        unsafe {System.realloc(p,l,n)}
    }
}
#[global_allocator] static ALLOCATOR:Allocator=Allocator;

#[test]
fn collision_queries_and_benchmark_ticks_do_not_allocate() {
    for case in 0..4 {
        let mut world=BoxWorld::default();
        world.fill([-10,-1,-10],[10,-1,20000],Block::full());
        if case==2 {world.fill([-10,2,-10],[10,2,20000],Block::full());}
        if case==3 {
            for z in (4..8000).step_by(4) {world.set([0,0,z],Block::boxes(&[Aabb::new(0.,0.,0.,1.,0.5,1.)]));}
        }
        let mut state=PlayerState{on_ground:true,velocity:Vec3::new(0.,-0.0784000015258789,0.),..PlayerState::default()};
        // Initialize shared empty/context shapes before the measured workload.
        tick(&mut state,&world,&Input::default());
        ALLOCS.with(|n|n.set(0)); TRACK.with(|v|v.set(true));
        for _ in 0..2000 {
            let input=Input{forward:true,sprint:case!=0,jump:case!=0&&state.on_ground,..Input::default()};
            tick(&mut state,&world,&input);
            state.food_level=20;state.saturation=5.;state.exhaustion=0.;
            let context=CollisionContext::of(&state);
            std::hint::black_box(world.no_collision(state.bounding_box(),context));
            std::hint::black_box(world.collides_with_suffocating_block(state.bounding_box(),context));
        }
        TRACK.with(|v|v.set(false));
        assert_eq!(ALLOCS.with(Cell::get),0,"case {case}");
    }
}

#[test]
fn borrowed_queries_match_owned_queries_and_shape_rounding() {
    let mut world=BoxWorld::default();
    world.fill([-3,-1,-3],[3,-1,3],Block::full());
    // A shape reaching outside its own cell must retain the original halo.
    world.set([1,0,0],Block{ suffocating:true,..Block::boxes(&[Aabb::new(-0.4,0.,0.,1.4,1.5,1.)]) });
    world.set([0,0,1],Block{above_shape:Some(VoxelShape::boxes(&[Aabb::new(0.,0.,0.,1.,0.5,1.)])),..Block::default()});
    for x in -4..5 { for y in -2..4 {
        let body=Aabb::new(x as f64*0.3,y as f64*0.3,-0.2,x as f64*0.3+0.6,y as f64*0.3+1.8,1.2);
        let context=CollisionContext{feet_y:body.min.y,descending:false,fall_distance:0.,leather_boots:false};
        let old=world.collisions(body,context);
        let mut index=0;
        world.visit_collisions(body,context,|shape| {
            for a in 0..3 {for i in 0..shape.shape.coords[a].len() {
                assert_eq!(shape.coord(a,i).to_bits(),old[index].coords[a][i].to_bits());
            }}
            index+=1;true
        });
        assert_eq!(index,old.len());
    }}
    let shape=VoxelShape::boxes(&[Aabb::new(0.,0.,0.,0.5,1.,1.),Aabb::new(0.5,0.,0.,1.,0.5,1.)]);
    for p in [0.,-0.,1e4,1e6,3e7] {
        let offset=Vec3::new(p,-p,p);
        let owned=shape.moved(offset);
        let view=shape.at(offset);
        let body=Aabb::new(p-0.3,-p+0.4,p+0.1,p+0.3,-p+2.2,p+0.7);
        assert_eq!(owned.intersects(body),view.intersects(body));
        for axis in [Axis::X,Axis::Y,Axis::Z] {for d in [-10.,-1e-8,0.,1e-8,10.] {
            assert_eq!(owned.collide(axis,body,d).to_bits(),view.collide(axis,body,d).to_bits());
        }}
    }
}

#[test]
fn collider_inline_capacity_is_not_a_geometry_limit() {
    let mut world=BoxWorld::default();
    world.fill([-10,-1,-10],[10,-1,10],Block::full());
    for _ in 0..80 {world.extra_shapes.push(VoxelShape::boxes(&[Aabb::new(-1.,0.,0.4,1.,2.,1.4)]));}
    let mut state=PlayerState{on_ground:true,velocity:Vec3::new(0.,0.,0.2),..PlayerState::default()};
    let b=state.bounding_box();
    let context=CollisionContext::of(&state);
    let wanted=Vec3::new(0.,-0.0784000015258789,0.2);
    let expected=law::shapes::collide_with_shapes(wanted,b,&world.collisions(b.expand(wanted),context));
    // tick_with_collision supplies the exact requested vector for comparison.
    ALLOCS.with(|n|n.set(0)); TRACK.with(|v|v.set(true));
    let (request,result)=tick_with_collision(&mut state,&world,&Input::default()).unwrap();
    TRACK.with(|v|v.set(false));
    let reference=law::shapes::collide_with_shapes(request,b,&world.collisions(b.expand(request),context));
    assert_eq!(result,reference);
    assert_eq!(expected.z,0.09999998807907107);
    assert_eq!(ALLOCS.with(Cell::get),0);
}
