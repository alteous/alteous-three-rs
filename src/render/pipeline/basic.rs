//! Basic rendering pipeline.
//!
//! Useful for rendering meshes with a solid color or rendering mesh wireframes.

use gpu::{self, framebuffer as fbuf, pipeline as pipe, program};
use super::*;

/// Basic pipeline bindings.
pub const BINDINGS: program::Bindings = program::Bindings {
    uniform_blocks: [
        program::UniformBlockBinding::Required(b"b_Locals\0"),
        program::UniformBlockBinding::Required(b"b_Globals\0"),
        program::UniformBlockBinding::None,
        program::UniformBlockBinding::None,
    ],
    samplers: [program::SamplerBinding::None; program::MAX_SAMPLERS],
};

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

/// Clear operation for the basic pipeline.
pub const CLEAR_OP: fbuf::ClearOp = fbuf::ClearOp {
    color: fbuf::ClearColor::Yes { r: 0.0, g: 0.0, b: 0.0, a: 0.0 },
    depth: fbuf::ClearDepth::Yes { z: 1.0 },
};

/// State transition for the wireframe pipeline.
pub const WIREFRAME: pipe::State = pipe::DEFAULT_STATE;

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

/// Pipeline states for the basic rendering pipeline.
#[derive(Clone, Debug)]
pub struct States {
    /// Render as a solid.
    pub solid: gpu::State,

    /// Render as a wireframe.
    pub wireframe: gpu::State,
}

/// Basic rendering pipeline.
pub struct Basic {
    /// Linked program.
    pub program: gpu::Program,

    /// Locals uniform buffer.
    pub locals: gpu::Buffer,

    /// Globals uniform buffer.
    pub globals: gpu::Buffer,

    /// Pipeline states.
    pub states: States,
}

impl Basic {
    /// Creates a solid rendering pipeline.
    pub fn new(factory: &gpu::Factory) -> Self {
        let program = make_program(factory, "shader.vert", "shader.frag", &BINDINGS);
        let locals = make_uniform_buffer(factory, &LOCALS);
        let globals = make_uniform_buffer(factory, &GLOBALS);
        let states = States {
            solid: gpu::State::default(),
            wireframe: gpu::State {
                culling: gpu::pipeline::Culling::None,
                polygon_mode: gpu::pipeline::PolygonMode::Line(1),
                .. Default::default()
            },
        };
        Basic {
            program,
            states,
            locals,
            globals,
        }
    }
}
