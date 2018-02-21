#[macro_use]
extern crate euler;
extern crate three;

use three::Object;

fn main() {
    let (mut window, mut input, mut renderer, mut factory) = three::init();

    let mut scene = factory.scene();
    
    let camera = factory.perspective_camera(75.0, 1.0 .. 50.0);
    camera.set_position(vec3!(0.0, 0.0, 10.0));

    let light = factory.point_light(0xffffff, 0.5);
    let mut pos = vec3!(0.0, 5.0, 5.0);
    light.set_position(pos);
    scene.add(&light);

    let geometry = three::Geometry::cylinder(1.0, 2.0, 2.0, 5);
    let mut materials: Vec<three::Material> = vec![
        three::material::Basic { color: 0xFFFFFF, map: None }.into(),
        three::material::Lambert { color: 0xFFFFFF }.into(),
        three::material::Gouraud { color: 0xFFFFFF }.into(),
        three::material::Phong { color: 0xFFFFFF, glossiness: 80.0 }.into(),
    ];
    let count = materials.len();

    let _cubes: Vec<_> = materials
        .drain(..)
        .enumerate()
        .map(|(i, mat)| {
            let offset = 4.0 * (i as f32 + 0.5 - 0.5 * count as f32);
            let mesh = factory.mesh(geometry.clone(), mat);
            mesh.set_position(vec3!(offset, 0.0, 0.0));
            scene.add(&mesh);
            mesh
        })
        .collect();

    while !input.quit_requested() && !input.hit(three::KEY_ESCAPE) {
        input.update();
        if let Some(diff) = input.timed(three::AXIS_LEFT_RIGHT) {
            pos.x += 5.0 * diff;
            light.set_position(pos);
        }
        renderer.render(&scene, &camera, &window);
        window.swap_buffers();
    }
}
