extern crate three;

use three::Object;

struct Animator {
    cell_size: [u16; 2],
    cell_counts: [u16; 2],
    duration: f32,
    repeat: bool,
    sprite: three::Sprite,
    current: [u16; 2],
    timer: three::Timer,
}

impl Animator {
    fn update_uv(&mut self) {
        let base = [
            (self.current[0] * self.cell_size[0]) as i16,
            (self.current[1] * self.cell_size[1]) as i16,
        ];
        self.sprite.set_texel_range(base, self.cell_size);
    }

    fn update(
        &mut self,
        switch_row: Option<u16>,
        input: &three::Input,
    ) {
        if let Some(row) = switch_row {
            self.current = [0, row];
            self.timer = input.time();
            self.update_uv();
        } else if self.timer.get(input) >= self.duration && (self.repeat || self.current[0] < self.cell_counts[0]) {
            self.timer = input.time();
            self.current[0] += 1;
            if self.current[0] < self.cell_counts[0] {
                self.update_uv();
            } else if self.repeat {
                self.current[0] = 0;
                self.update_uv();
            }
        }
    }
}

fn main() {
    let (mut window, mut input, mut renderer, mut factory) = three::init();
    let camera = factory.orthographic_camera([0.0, 0.0], 10.0, -10.0 .. 10.0);

    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/test_data/pikachu_anim.png",
    );
    let map = factory.load_texture(path);
    let sprite = factory.sprite(map);
    let mut scene = factory.scene();
    sprite.set_scale(8.0);
    scene.add(&sprite);

    let mut anim = Animator {
        cell_size: [96, 96],
        cell_counts: [5, 13],
        duration: 0.1,
        repeat: true,
        current: [0, 0],
        timer: input.time(),
        sprite,
    };
    anim.update_uv();

    // Specify background image. Remove `if` to enable.
    if false {
        let background = factory.load_texture("test_data/texture.png");
        scene.background = three::Background::Texture(background);
    }

    while !input.quit_requested() && !input.hit(three::KEY_ESCAPE) {
        input.update();
        let row = input.delta(three::AXIS_LEFT_RIGHT).map(|mut diff| {
            let total = anim.cell_counts[1] as i8;
            while diff < 0 {
                diff += total
            }
            (anim.current[1] + diff as u16) % total as u16
        });
        anim.update(row, &input);
        renderer.render(&scene, &camera, &window);
        window.swap_buffers();
    }
}
