//! Rendering pipelines.

pub use self::basic::Basic;
pub use self::lambert::Lambert;
pub use self::phong::Phong;
pub use self::shadow::Shadow;

pub mod basic;
pub mod lambert;
pub mod phong;
pub mod shadow;

use color;
use gpu::{self, buffer as buf};
use std::{marker, mem};

use render::Source;

/// The maximum number of point lights for any forward rendered program.
pub const MAX_POINT_LIGHTS: usize = 8;

/// Built-in programs.
pub struct Programs {
    pub(crate) basic: Basic,
    pub(crate) lambert: Lambert,
    pub(crate) phong: Phong,
    pub(crate) shadow: Shadow,
}

pub mod light {
    use camera;
    use color;

    use euler::{Vec3, Vec4};
    
    pub type Shadow = camera::Projection;

    #[derive(Clone, Debug)]
    pub struct Ambient {
        pub color: color::Color,
        pub intensity: f32,
    }

    impl Default for Ambient {
        fn default() -> Self {
            Self {
                color: color::BLACK,
                intensity: 0.0,
            }
        }
    }

    #[derive(Clone, Debug)]
    pub struct Direct {
        pub color: color::Color,
        pub intensity: f32,
        pub origin: Vec4,
        pub direction: Vec3,
        pub shadow: Option<Shadow>,
    }

    impl Default for Direct {
        fn default() -> Self {
            Self {
                color: color::BLACK,
                intensity: 0.0,
                origin: vec4!(0.0, 0.0, 0.0, 0.0),
                direction: vec3!(0.0, 0.0, 1.0),
                shadow: None,
            }
        }
    }

    #[derive(Clone, Debug)]
    pub struct Point {
        pub color: color::Color,
        pub intensity: f32,
        pub position: Vec3,
        pub shadow: Option<Shadow>,
    }

    impl Default for Point {
        fn default() -> Self {
            Self {
                color: color::BLACK,
                intensity: 0.0,
                position: vec3!(),
                shadow: None,
            }
        }
    }
}

/// Illumination data.
#[derive(Clone, Debug, Default)]
pub struct Lighting {
    /// Global ambient lighting.
    pub ambient: light::Ambient,

    /// Global directional lighting.
    pub direct: light::Direct,

    /// Local point lights.
    pub points: [light::Point; MAX_POINT_LIGHTS],
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
    let basic = Basic::new(factory)
    let lambert = Lambert::new(factory);
    let phong = Phong::new(factory);
    let shadow = Shadow::new(factory);
    Programs { basic, lambert, phong, shadow }
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
    _binding: &UniformBlockBinding<T>,
) -> gpu::Buffer {
    let size = mem::size_of::<T>();
    let kind = buf::Kind::Uniform;
    let usage = buf::Usage::DynamicDraw;
    factory.uninitialized_buffer(size, kind, usage)
}
