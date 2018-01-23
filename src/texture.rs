use std::path::Path;

use mint;

/// The sampling properties for a `Texture`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Sampler;

/// An image applied (mapped) to the surface of a shape or polygon.
#[derive(Clone, Debug, PartialEq)]
pub struct Texture { 
    total_size: [u32; 2],
    tex0: [f32; 2],
    tex1: [f32; 2],
}

impl Texture {
    pub(crate) fn _new(total_size: [u32; 2]) -> Self {
        Texture {
            total_size,
            tex0: [0.0; 2],
            tex1: [total_size[0] as f32, total_size[1] as f32],
        }
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
pub struct CubeMap;

impl CubeMap {
    pub(crate) fn _new() -> Self {
        CubeMap
    }
}
