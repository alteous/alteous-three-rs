//! Rendering pipelines.

pub mod basic;
pub mod gouraud;
pub mod lambert;
pub mod phong;

use color;
use gpu::{self, buffer as buf};
use mint;
use render::Source;
use std::{marker, mem};

pub use self::basic::Basic;
pub use self::gouraud::Gouraud;
pub use self::lambert::Lambert;
pub use self::phong::Phong;

/// The maximum number of point lights for any forward rendered program.
pub const MAX_POINT_LIGHTS: usize = 8;

/// Built-in programs.
pub struct Programs {
    pub(crate) basic: Basic,
    pub(crate) gouraud: Gouraud,
    pub(crate) lambert: Lambert,
    pub(crate) phong: Phong,
}

/// Illumination data.
#[derive(Clone, Debug)]
pub struct Lighting {
    /// Global ambient lighting.
    pub ambient: (color::Color, f32),

    /// Global directional lighting.
    pub directional: (mint::Vector3<f32>, color::Color, f32),

    /// Local point lights.
    pub points: [(mint::Point3<f32>, color::Color, f32); MAX_POINT_LIGHTS],
}

impl Default for Lighting {
    fn default() -> Self {
        Self {
            ambient: (0xFFFFFF, 0.2),
            directional: ([0.0, -1.0, 0.0].into(), 0xFFFFFF, 0.0),
            points: [([0.0; 3].into(), 0xFFFFFF, 0.0); MAX_POINT_LIGHTS],
        }
    }
}

/// 4x4 identity matrix.
pub const IDENTITY: [[f32; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

/// Create ALL the programs.
pub fn init(factory: &gpu::Factory) -> Programs {
    Programs {
        basic: Basic::new(factory),
        gouraud: Gouraud::new(factory),
        lambert: Lambert::new(factory),
        phong: Phong::new(factory),
    }
}

/// Represents a uniform block in a program.
///
/// ```rust
/// #[repr(C)]
/// struct MyLocals {
///     mx_world: [[f32; 4]; 4],
/// }
///
/// const MY_LOCALS: UniformBlockBinding<MyLocals> = MyLocals {
///     name: b"b_MyLocals\0",
///     index: 0,
///     init: MyLocals {
///         [
///             [1.0, 0.0, 0.0, 0.0],
///             [0.0, 1.0, 0.0, 0.0],
///             [0.0, 0.0, 1.0, 0.0],
///             [0.0, 0.0, 0.0, 1.0],
///         ],
///     },
/// };
/// ```
pub struct UniformBlockBinding<T> {
    /// The uniform block name which must be a C string.
    pub name: &'static [u8],

    /// The uniform block binding index.
    pub index: u32,

    /// Consumes the buffer data type `T`.
    pub marker: marker::PhantomData<T>,
}

/// Make a vertex shader + fragment shader program.
pub fn make_program(
    factory: &gpu::Factory,
    name: &str,
    bindings: &gpu::program::Bindings,
) -> gpu::Program {
    let vertex_shader = {
        let source = Source::default(name, "vs").unwrap();
        factory.shader(gpu::shader::Kind::Vertex, &source)
    };
    let fragment_shader = {
        let source = Source::default(name, "ps").unwrap();
        factory.shader(gpu::shader::Kind::Fragment, &source)
    };
    factory.program(&vertex_shader, &fragment_shader, bindings)
}

/// Create a uniform buffer for a uniform block in a program.
pub fn make_uniform_buffer<T>(
    factory: &gpu::Factory,
    binding: &UniformBlockBinding<T>,
) -> gpu::Buffer {
    let size = mem::size_of::<T>();
    let kind = buf::Kind::Uniform;
    let usage = buf::Usage::DynamicDraw;
    factory.uninitialized_buffer(size, kind, usage)
}
