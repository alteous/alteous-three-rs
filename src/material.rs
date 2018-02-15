//! Material parameters for mesh rendering.

use color;

use color::Color;
use texture::Texture;

#[doc(inline)]
pub use self::basic::Basic;

#[doc(inline)]
pub use self::line::Line;

/// Basic material API.
pub mod basic {
    use super::*;

    /// Parameters for a basic solid mesh material.
    #[derive(Clone, Debug, PartialEq)]
    pub struct Basic {
        /// Solid color applied in the absense of `map`.
        ///
        /// Default: `WHITE`.
        pub color: Color,

        /// Texture applied using the mesh texture co-ordinates.
        ///
        /// Default: `None`.
        pub map: Option<Texture>,
    }

    impl Default for Basic {
        fn default() -> Self {
            Self {
                color: color::WHITE,
                map: None,
            }
        }
    }
}

/// Parameters for a Gouraud shading model.
#[derive(Clone, Debug, PartialEq)]
pub struct Gouraud {
    /// Solid object color.
    ///
    /// Default: `WHITE`.
    pub color: Color,
}

impl Default for Gouraud {
    fn default() -> Self {
        Self {
            color: color::WHITE,
        }
    }
}

/// Parameters for a Lamberian diffusion reflection model.
#[derive(Clone, Debug, PartialEq)]
pub struct Lambert {
    /// Solid object color.
    ///
    /// Default: `WHITE`.
    pub color: Color,
}

impl Default for Lambert {
    fn default() -> Self {
        Self {
            color: color::WHITE,
        }
    }
}

/// Line material API.
pub mod line {
    use color;
    use gpu;
    
    /// Interpretation of vertex layout.
    #[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
    pub enum Layout {
        /// `[(start, end), (start, end), ...]`.
        Segments,

        /// `[start, end/start, ..., end/start, end]`.
        Strip,

        /// `[start, end/start, ..., end/start, end, <implicit_start>]`.
        Loop,
    }

    impl Layout {
        pub(crate) fn as_gpu_primitive(&self) -> gpu::draw_call::Primitive {
            match *self {
                Layout::Segments => gpu::draw_call::Primitive::Lines,
                Layout::Strip => gpu::draw_call::Primitive::LineStrip,
                Layout::Loop => gpu::draw_call::Primitive::LineLoop,
            }
        }
    }
    
    /// Parameters for a line material.
    #[derive(Clone, Debug, PartialEq)]
    pub struct Line {
        /// Solid line color.
        ///
        /// Default: `0xFFFFFF` (white).
        pub color: color::Color,

        /// Interpretation of vertex layout.
        ///
        /// Default: `Strip`.
        pub layout: Layout,
    }

    impl Default for Line {
        fn default() -> Self {
            Self {
                color: color::WHITE,
                layout: Layout::Strip,
            }
        }
    }
}

/// Parameters for a PBR (physically based rendering) lighting model.
#[derive(Clone, Debug, PartialEq)]
pub struct Pbr {
    /// Solid base color applied in the absense of `base_color_map`.
    ///
    /// Default: `WHITE`.
    pub base_color_factor: Color,

    /// Base color alpha factor applied in the absense of `base_color_map`.
    ///
    /// Default: `1.0` (opaque).
    pub base_color_alpha: f32,

    /// Metallic factor in the range [0.0, 1.0].
    ///
    /// Default: `1.0`.
    pub metallic_factor: f32,

    /// Roughness factor in the range [0.0, 1.0].
    ///
    /// * A value of 1.0 means the material is completely rough.
    /// * A value of 0.0 means the material is completely smooth.
    ///
    /// Default: `1.0`.
    pub roughness_factor: f32,

    /// Scalar multiplier in the range [0.0, 1.0] that controls the amount of
    /// occlusion applied in the presense of `occlusion_map`.
    ///
    /// Default: `1.0`.
    pub occlusion_strength: f32,

    /// Solid emissive color applied in the absense of `emissive_map`.
    ///
    /// Default: `BLACK`.
    pub emissive_factor: Color,

    /// Scalar multiplier applied to each normal vector of the `normal_map`.
    ///
    /// This value is ignored in the absense of `normal_map`.
    ///
    /// Default: `1.0`.
    pub normal_scale: f32,

    /// Base color texture.
    ///
    /// Default: `None`.
    pub base_color_map: Option<Texture>,

    /// Normal texture.
    ///
    /// Default: `None`.
    pub normal_map: Option<Texture>,

    /// Emissive texture.
    ///
    /// Default: `None`.
    pub emissive_map: Option<Texture>,

    /// Metallic-roughness texture.
    ///
    /// Default: `None`.
    pub metallic_roughness_map: Option<Texture>,

    /// Occlusion texture.
    ///
    /// Default: `None`.
    pub occlusion_map: Option<Texture>,
}

impl Default for Pbr {
    fn default() -> Self {
        Self {
            base_color_factor: color::WHITE,
            base_color_alpha: 1.0,
            metallic_factor: 1.0,
            roughness_factor: 1.0,
            occlusion_strength: 1.0,
            emissive_factor: color::BLACK,
            normal_scale: 1.0,
            base_color_map: None,
            normal_map: None,
            emissive_map: None,
            metallic_roughness_map: None,
            occlusion_map: None,
        }
    }
}

/// Parameters for a Phong reflection model.
#[derive(Clone, Debug, PartialEq)]
pub struct Phong {
    /// Solid color applied in the absense of `map`.
    ///
    /// Default: `WHITE`.
    pub color: Color,

    /// Determines the sharpness of specular highlights.
    ///
    /// Higher values result in sharper highlights to produce a glossy effect.
    ///
    /// Default: `30.0`.
    pub glossiness: f32,
}

impl Default for Phong {
    fn default() -> Self {
        Self {
            color: color::WHITE,
            glossiness: 30.0,
        }
    }
}

/// Texture for a 2D sprite.
#[derive(Clone, Debug, PartialEq)]
pub struct Sprite {
    /// The texture the apply to the sprite.
    pub map: Texture,
}

/// Parameters for mesh wireframe rasterization.
#[derive(Clone, Debug, PartialEq)]
pub struct Wireframe {
    /// Solid color applied to each wireframe edge.
    ///
    /// Default: `WHITE`.
    pub color: Color,
}

/// Specifies the appearance of a [`Mesh`](struct.Mesh.html).
#[derive(Clone, Debug, PartialEq)]
pub enum Material {
    /// Renders triangle meshes with a solid color or texture.
    Basic(Basic),

    /// Renders triangle meshes with the Gouraud illumination model.
    Gouraud(Gouraud),

    /// Renders line strip meshes with a solid color and unit width.
    Line(Line),

    /// Renders triangle meshes with the Lambertian illumination model.
    Lambert(Lambert),

    /// Renders triangle meshes with the Phong illumination model.
    Phong(Phong),

    /// Renders triangle meshes with a PBR (physically-based rendering)
    /// illumination model
    Pbr(Pbr),

    /// Renders [`Sprite`] objects with the given texture.
    ///
    /// [`Sprite`]: ../sprite/struct.Sprite.html
    Sprite(Sprite),

    /// Renders the edges of a triangle mesh with a solid color.
    Wireframe(Wireframe),
}

impl From<Basic> for Material {
    fn from(params: Basic) -> Material {
        Material::Basic(params)
    }
}


impl From<Gouraud> for Material {
    fn from(params: Gouraud) -> Material {
        Material::Gouraud(params)
    }
}

impl From<Lambert> for Material {
    fn from(params: Lambert) -> Material {
        Material::Lambert(params)
    }
}

impl From<Line> for Material {
    fn from(params: Line) -> Material {
        Material::Line(params)
    }
}

impl From<Phong> for Material {
    fn from(params: Phong) -> Material {
        Material::Phong(params)
    }
}

impl From<Pbr> for Material {
    fn from(params: Pbr) -> Material {
        Material::Pbr(params)
    }
}

impl From<Sprite> for Material {
    fn from(params: Sprite) -> Material {
        Material::Sprite(params)
    }
}

impl From<Wireframe> for Material {
    fn from(params: Wireframe) -> Material {
        Material::Wireframe(params)
    }
}
