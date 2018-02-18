use gpu;
use image;

use image::GenericImage;
use std::path::Path;
use texture::Texture;

fn load_texture_impl(backend: &gpu::Factory, path: &Path) -> Texture {
    let image = image::open(path).expect("image loader failed");
    let (width, height) = image.dimensions();
    let texture_format = gpu::texture::format::U8::Rgba;
    let image_format = gpu::image::format::U8::Rgba;
    let mipmap = true;
    let pixels = image.flipv().to_rgba().into_raw();
    let inner = backend.texture2(width, height, mipmap, texture_format);
    backend.write_texture2(&inner, image_format, &pixels);
    Texture::new(inner, width, height)
}

impl super::Factory {
    /// Loads a texture.
    pub fn load_texture<P>(&mut self, path: P) -> Texture
        where P: AsRef<Path>
    {
        let path = path.as_ref();
        let key = path.to_string_lossy().into_owned();
        let backend = self.backend.clone(); // hack around borrow checker
        self.texture_cache
            .entry(key)
            .or_insert_with(|| load_texture_impl(&backend, path))
            .clone()
    }
}
