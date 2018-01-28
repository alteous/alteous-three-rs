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

/// Forward rendering pipeline.
pub struct Forward {
    /// Linked program.
    pub program: gpu::Program,

    /// Draw state.
    pub state: gpu::State,

    /// Locals uniform buffer.
    pub locals: gpu::Buffer,

    /// Globals uniform buffer.
    pub globals: gpu::Buffer,
}

/// Creates a solid rendering pipeline.
pub fn solid(factory: &gpu::Factory) -> Forward {
    let program = make_program(factory, "shader.vert", "shader.frag");
    let locals = make_uniform_buffer(factory, &program, &LOCALS);
    let globals = make_uniform_buffer(factory, &program, &GLOBALS);
    let state = gpu::State::default();
    Forward {
        program,
        state,
        locals,
        globals,
    }
}

/// Creates a wireframe rendering pipeline.
pub fn wireframe(factory: &gpu::Factory) -> Forward {
    let program = make_program(factory, "shader.vert", "shader.frag");
    let locals = make_uniform_buffer(factory, &program, &LOCALS);
    let globals = make_uniform_buffer(factory, &program, &GLOBALS);
    let state = gpu::State {
        culling: gpu::pipeline::Culling::None,
        polygon_mode: gpu::pipeline::PolygonMode::Line(1),
        .. Default::default()
    };
    Forward {
        program,
        state,
        locals,
        globals,
    }
}
