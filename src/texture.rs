use gpu;
use mint;

use std::path::Path;

/// Texture sampling magnification/minification filter.
#[allow(dead_code)]
pub type Filter = gpu::sampler::Filter;

/// Texture co-ordinate wrapping mode.
#[allow(dead_code)]
pub type Wrap = gpu::sampler::Wrap;

/// Sampling properties for a `Texture`.
pub type Sampler = gpu::Sampler2;

/// An image applied (mapped) to the surface of a shape or polygon.
#[derive(Clone, Debug, PartialEq)]
pub struct Texture {
    inner: gpu::Texture2,
    total_size: [u32; 2],
    tex0: [f32; 2],
    tex1: [f32; 2],

    /// Texture sampling properties.
    pub sampler: Sampler,
}

impl Texture {
    pub(crate) fn new(inner: gpu::Texture2, width: u32, height: u32) -> Self {
        Texture {
            inner,
            total_size: [width, height],
            sampler: Default::default(),
            tex0: [0.0; 2],
            tex1: [width as f32, height as f32],
        }
    }

    /// Conversion into a program invocation parameter.
    pub(crate) fn to_param<'a>(&'a self) -> (&'a gpu::Texture2, gpu::Sampler2) {
        (&self.inner, self.sampler)
    }

    /// See [`Sprite::set_texel_range`](struct.Sprite.html#method.set_texel_range).
    pub fn set_texel_range(
        &mut self,
        base: mint::Point2<i16>,
        size: mint::Vector2<u16>,
    ) {
        self.tex0 = [
            base.x as f32,
            self.total_size[1] as f32 - base.y as f32 - size.y as f32,
        ];
        self.tex1 = [
            base.x as f32 + size.x as f32,
            self.total_size[1] as f32 - base.y as f32,
        ];
    }

    /// Returns normalized UV rectangle (x0, y0, x1, y1) of the current texel range.
    pub fn uv_range(&self) -> [f32; 4] {
        [
            self.tex0[0] / self.total_size[0] as f32,
            self.tex0[1] / self.total_size[1] as f32,
            self.tex1[0] / self.total_size[0] as f32,
            self.tex1[1] / self.total_size[1] as f32,
        ]
    }
}

/// Represents paths to cube map texture, useful for loading
/// [`CubeMap`](struct.CubeMap.html).
#[derive(Clone, Debug)]
pub struct CubeMapPath<P: AsRef<Path>> {
    /// "Front" image. `Z+`.
    pub front: P,
    /// "Back" image. `Z-`.
    pub back: P,
    /// "Left" image. `X-`.
    pub left: P,
    /// "Right" image. `X+`.
    pub right: P,
    /// "Up" image. `Y+`.
    pub up: P,
    /// "Down" image. `Y-`.
    pub down: P,
}

impl<P: AsRef<Path>> CubeMapPath<P> {
    pub(crate) fn _as_array(&self) -> [&P; 6] {
        [
            &self.right,
            &self.left,
            &self.up,
            &self.down,
            &self.front,
            &self.back,
        ]
    }
}

/// Cubemap is six textures useful for
/// [`Cubemapping`](https://en.wikipedia.org/wiki/Cube_mapping).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Cube;

impl Cube {
    pub(crate) fn _new() -> Self {
        Cube
    }
}
