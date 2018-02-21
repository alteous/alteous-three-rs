#[macro_use]
extern crate euler;
extern crate three;

use euler::Quat;
use std::f32::consts::PI;
use three::Object;

macro_rules! three_object {
    ($ty:ident::$id:ident) => {
        impl AsRef<::three::object::Base> for $ty {
            fn as_ref(&self) -> &::three::object::Base {
                self.$id.as_ref()
            }
        }

        impl ::three::Object for $ty {}
    };
}

struct Level {
    speed: f32,
}

struct Cube {
    group: three::Group,
    mesh: three::Mesh,
    level_id: usize,
    orientation: Quat,
}
three_object!(Cube::group);

fn create_cubes(
    factory: &mut three::Factory,
    materials: &[three::material::Gouraud],
    levels: &[Level],
) -> Vec<Cube> {
    let mut geometry = three::Geometry::cuboid(2.0, 2.0, 2.0);
    for v in geometry.vertices.iter_mut() {
        v.z += 1.0;
    }

    let root = {
        let mut group = factory.group();
        let mut mesh = factory.mesh(geometry.clone(), materials[0].clone());
        group.set_position(vec3!(0, 0, 1));
        group.set_scale(2.0);
        group.add(&mesh);
        Cube {
            group,
            mesh,
            level_id: 0,
            orientation: Quat::identity(),
        }
    };
    let mut list = vec![root];

    struct Stack {
        parent_id: usize,
        mat_id: usize,
        lev_id: usize,
    }
    let mut stack = vec![
        Stack {
            parent_id: 0,
            mat_id: 1,
            lev_id: 1,
        },
    ];

    let axes = [
        vec3!(0, 0, 1),
        vec3!(1, 0, 0),
        vec3!(-1, 0, 0),
        vec3!(0, 1, 0),
        vec3!(0, -1, 0),
    ];
    let children: Vec<_> = axes.iter()
        .map(|axis| {
            three::Transform {
                position: vec3!(0, 0, 1),
                orientation: Quat::axis_angle(*axis, PI / 2.0),
                scale: 1.0,
            }.concat(three::Transform {
                position: vec3!(0, 0, 1),
                orientation: Quat::identity(),
                scale: 0.4,
            })
        })
        .collect();

    while let Some(next) = stack.pop() {
        for child in &children {
            let mat = materials[next.mat_id].clone();
            let mut cube = Cube {
                group: factory.group(),
                mesh: factory.mesh_duplicate(&list[0].mesh, mat),
                level_id: next.lev_id,
                orientation: child.orientation,
            };
            let p = child.position;
            cube.group.set_transform(p, child.orientation, child.scale);
            list[next.parent_id].group.add(&cube.group);
            cube.group.add(&cube.mesh);
            if next.mat_id + 1 < materials.len() && next.lev_id + 1 < levels.len() {
                stack.push(Stack {
                    parent_id: list.len(),
                    mat_id: next.mat_id + 1,
                    lev_id: next.lev_id + 1,
                });
            }
            list.push(cube);
        }
    }

    list
}

const COLORS: [three::Color; 6] = [0xffff80, 0x8080ff, 0x80ff80, 0xff8080, 0x80ffff, 0xff80ff];

const SPEEDS: [f32; 5] = [
    0.7,
    -1.0,
    1.3,
    -1.6,
    1.9,
];

fn main() {
    let (mut window, mut input, mut renderer, mut factory) = three::init();
    let mut scene = factory.scene();
    scene.background = three::Background::Color(0x204060);

    let camera = factory.perspective_camera(60.0, 1.0 .. 100.0);
    camera.look_at(vec3!(-1.8, -8.0, 7.0), vec3!(0.0, 0.0, 3.5), None);
    scene.add(&camera);

    let light = factory.point_light(0xffffff, 1.0);
    light.set_position(vec3!(0.0, -10.0, 10.0));
    scene.add(&light);

    let materials: Vec<_> = COLORS
        .iter()
        .map(|&color| three::material::Gouraud { color })
        .collect();
    let levels: Vec<_> = SPEEDS.iter().map(|&speed| Level { speed }).collect();
    let mut cubes = create_cubes(&mut factory, &materials, &levels);
    scene.add(&cubes[0]);

    let timer = input.time();
    while !input.quit_requested() && !input.hit(three::KEY_ESCAPE) {
        input.update();
        let time = timer.get(&input);
        for cube in cubes.iter_mut() {
            let level = &levels[cube.level_id];
            let angle = time * level.speed;
            let q = cube.orientation * Quat::axis_angle(vec3!(0, 0, 1), angle);
            cube.group.set_orientation(q);
        }

        renderer.render(&scene, &camera, &window);
        window.swap_buffers();
    }
}
