//! Rendering pipelines.

pub mod basic;
pub mod gouraud;
pub mod phong;
pub mod lambert;

use gpu::{self, buffer as buf};
use render::Source;

pub use self::basic::Basic;
pub use self::gouraud::Gouraud;
pub use self::phong::Phong;
pub use self::lambert::Lambert;

/// Built-in programs.
pub struct Programs {
    pub(crate) basic: Basic,
    pub(crate) lambert: Lambert,
    pub(crate) gouraud: Gouraud,
    pub(crate) phong: Phong,
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
        lambert: Lambert::new(factory),
        gouraud: Gouraud::new(factory),
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
pub struct UniformBlockBinding<T: 'static + Clone> {
    /// The uniform block name which must be a C string.
    pub name: &'static [u8],

    /// The uniform block binding index.
    pub index: u32,

    /// An initial value for the uniform buffer data.
    pub init: T,
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
pub fn make_uniform_buffer<T: 'static + Clone>(
    factory: &gpu::Factory,
    binding: &UniformBlockBinding<T>,
) -> gpu::Buffer {
    let buffer = factory.buffer(buf::Kind::Uniform, buf::Usage::DynamicDraw);
    factory.initialize_buffer(&buffer, &[binding.init.clone()]);
    buffer
}
