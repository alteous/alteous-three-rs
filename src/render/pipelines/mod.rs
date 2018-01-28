/// Forward render pipelines.
pub mod forward;

use gpu::{self, buffer as buf};
use util::{cstr, read_file_to_cstring};

pub use self::forward::Forward;

/// 4x4 identity matrix.
pub const IDENTITY: [[f32; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

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
    vertex_shader_path: &str,
    fragment_shader_path: &str,
) -> gpu::Program {
    let vertex_shader = {
        let mut source = read_file_to_cstring(vertex_shader_path).unwrap();
        factory.program_object(gpu::program::Kind::Vertex, &source)
    };
    let fragment_shader = {
        let source = read_file_to_cstring(fragment_shader_path).unwrap();
        factory.program_object(gpu::program::Kind::Fragment, &source)
    };
    factory.program(&vertex_shader, &fragment_shader)
}

/// Create a uniform buffer for a uniform block in a program.
pub fn make_uniform_buffer<T: 'static + Clone>(
    factory: &gpu::Factory,
    program: &gpu::Program,
    binding: &UniformBlockBinding<T>,
) -> gpu::Buffer {
    factory.set_uniform_block_binding(
        program,
        cstr(binding.name),
        binding.index,
    );
    let buffer = factory.buffer(buf::Kind::Uniform, buf::Usage::DynamicDraw);
    factory.initialize_buffer(&buffer, &[binding.init.clone()]);
    factory.set_uniform_block_binding(program, cstr(binding.name), binding.index);
    buffer
}
