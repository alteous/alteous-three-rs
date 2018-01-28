use gpu;
use super::*;

/// Locals uniform block binding.
pub const LOCALS: UniformBlockBinding<Locals> = UniformBlockBinding {
    name: b"b_Locals\0",
    index: 0,
    init: Locals {
        u_Color: [0.0; 4],
        u_World: IDENTITY,
    },
};

/// Globals uniform block binding.
pub const GLOBALS: UniformBlockBinding<Globals> = UniformBlockBinding {
    name: b"b_Globals\0",
    index: 1,
    init: Globals {
        u_ViewProjection: IDENTITY,
    },
};

/// Per-world variables.
#[allow(non_snake_case)]
#[derive(Clone, Debug)]
#[repr(C)]
pub struct Globals {
    /// Combined world-to-view and view-to-projection matrix.
    pub u_ViewProjection: [[f32; 4]; 4],
}

/// Per-instance variables.
#[allow(non_snake_case)]
#[derive(Clone, Debug)]
#[repr(C)]
pub struct Locals {
    /// Model-to-world matrix.
    pub u_World: [[f32; 4]; 4],

    /// Solid rendering color.
    pub u_Color: [f32; 4],
}

/// Solid pipeline.
pub struct Pipeline {
    /// Linked program.
    pub program: gpu::Program,

    /// Locals uniform buffer.
    pub locals: gpu::Buffer,

    /// Globals uniform buffer.
    pub globals: gpu::Buffer,
}

impl Pipeline {
    /// Initialize the solid pipeline.
    pub fn new(factory: &gpu::Factory) -> Self {
        let program = make_program(factory, "shader.vert", "shader.frag");
        let locals = make_uniform_buffer(factory, &program, &LOCALS);
        let globals = make_uniform_buffer(factory, &program, &GLOBALS);
        Pipeline { program, locals, globals }
    }
}
