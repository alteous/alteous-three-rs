#[macro_use]
extern crate euler;
extern crate three;

use three::Object;

fn main() {
    let (mut window, mut input, mut renderer, mut factory) = three::init();

    let mut scene = factory.scene();
    let camera = factory.perspective_camera(75.0, 1.0 .. 50.0);
    camera.set_position(vec3!(0.0, 0.0, 10.0));
    scene.add(&camera);
    
    let cuboid = {
        let geometry = three::Geometry::cuboid(3.0, 2.0, 1.0);
        let material = three::material::Wireframe { color: 0x00FF00 };
        factory.mesh(geometry, material)
    };
    cuboid.set_position(vec3!(-3.0, -3.0, 0.0));
    scene.add(&cuboid);

    let cylinder = {
        let geometry = three::Geometry::cylinder(1.0, 2.0, 2.0, 5);
        let material = three::material::Wireframe { color: 0xFF0000 };
        factory.mesh(geometry, material)
    };
    cylinder.set_position(vec3!(3.0, -3.0, 0.0));
    scene.add(&cylinder);

    let sphere = {
        let geometry = three::Geometry::uv_sphere(2.0, 5, 5);
        let material = three::material::Wireframe { color: 0xFF0000 };
        factory.mesh(geometry, material)
    };
    sphere.set_position(vec3!(-3.0, 3.0, 0.0));
    scene.add(&sphere);

    let line = {
        let geometry = three::Geometry::with_vertices(vec![
            [-2.0, -1.0, 0.0].into(),
            [0.0, 1.0, 0.0].into(),
            [2.0, -1.0, 0.0].into(),
        ]);
        let material = three::material::Line {
            color: 0x0000FF,
            layout: three::material::line::Layout::Loop,
        };
        factory.mesh(geometry, material)
    };
    line.set_position(vec3!(3.0, 3.0, 0.0));
    scene.add(&line);

    let mut angle = 0.0;
    while !input.quit_requested() && !input.hit(three::KEY_ESCAPE) {
        input.update();
        if let Some(diff) = input.timed(three::AXIS_LEFT_RIGHT) {
            angle += 1.5 * diff;
        } else {
            angle += 0.5 * input.delta_time();
        }
        let q = euler::Quat::axis_angle(vec3!(0, 1, 0), angle);
        cuboid.set_orientation(q);
        cylinder.set_orientation(q);
        sphere.set_orientation(q);
        line.set_orientation(q);
        renderer.render(&scene, &camera, &window);
        window.swap_buffers();
    }
}
