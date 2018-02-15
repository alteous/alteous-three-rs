extern crate cgmath;
extern crate mint;
extern crate three;

use cgmath::{Angle, Decomposed, One, Quaternion, Rad, Rotation3, Transform, Vector3};
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
    orientation: Quaternion<f32>,
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
        group.set_position([0.0, 0.0, 1.0]);
        group.set_scale(2.0);
        group.add(&mesh);
        Cube {
            group,
            mesh,
            level_id: 0,
            orientation: Quaternion::one(),
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

    let axis = [
        Vector3::unit_z(),
        Vector3::unit_x(),
        -Vector3::unit_x(),
        Vector3::unit_y(),
        -Vector3::unit_y(),
    ];
    let children: Vec<_> = axis.iter()
        .map(|&axe| {
            Decomposed {
                disp: Vector3::new(0.0, 0.0, 1.0),
                rot: Quaternion::from_axis_angle(axe, Rad::turn_div_4()),
                scale: 1.0,
            }.concat(&Decomposed {
                disp: Vector3::new(0.0, 0.0, 1.0),
                rot: Quaternion::one(),
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
                orientation: child.rot,
            };
            let p: mint::Vector3<f32> = child.disp.into();
            cube.group.set_transform(p, child.rot, child.scale);
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

const COLORS: [three::Color; 7] = [0xffff80, 0x8080ff, 0x80ff80, 0xff8080, 0x80ffff, 0xff80ff, 0xFF0000];

const SPEEDS: [f32; 6] = [
    0.7,
    -1.0,
    1.3,
    -1.6,
    1.9,
    -2.2,
];

fn main() {
    let (mut window, mut input, mut renderer, mut factory) = three::Window::new("Three-rs group example");
    let mut scene = factory.scene();
    scene.background = three::Background::Color(0x204060);
    scene.ambient_light = 0xFFFF00;

    let camera = factory.perspective_camera(60.0, 1.0 .. 100.0);
    camera.look_at([-1.8, -8.0, 7.0], [0.0, 0.0, 3.5], None);
    scene.add(&camera);

    let light = factory.point_light(0xffffff, 1.0);
    light.set_position([0.0, -10.0, 10.0]);
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
            let angle = Rad(time * level.speed);
            let q = cube.orientation * cgmath::Quaternion::from_angle_z(angle);
            cube.group.set_orientation(q);
        }

        renderer.render(&scene, &camera, &window);
        window.swap_buffers();
    }
}
