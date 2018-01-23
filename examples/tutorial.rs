extern crate three;

fn main() {
    let (mut window, mut input, mut renderer, mut factory) =
        three::Window::new("Getting started with three-rs");

    let mut scene = factory.scene();
    let vertices = vec![
        [-0.5, -0.5, -0.5].into(),
        [0.5, -0.5, -0.5].into(),
        [0.0, 0.5, -0.5].into(),
    ];
    let geometry = three::Geometry::with_vertices(vertices);
    let material = three::material::Basic {
        color: 0xFFFF00,
        map: None,
    };
    let mesh = factory.mesh(geometry, material);
    scene.add(&mesh);
    scene.background = three::Background::Color(0xC6F0FF);

    let center = [0.0, 0.0];
    let yextent = 1.0;
    let zrange = -1.0 .. 1.0;
    let camera = factory.orthographic_camera(center, yextent, zrange);

    while !input.quit_requested() {
        input.update();
        renderer.render(&scene, &camera);
        window.swap_buffers();
    }
}
