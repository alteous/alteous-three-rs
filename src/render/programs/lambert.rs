//! Basic rendering pipeline.
//!
//! Useful for rendering meshes with a solid color or rendering mesh wireframes.

use gpu::{self, framebuffer as fbuf, program};
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

/// Lambert rendering pipeline.
pub struct Lambert {
    /// The program.
    pub program: gpu::Program,

    /// Locals uniform buffer.
    pub locals: gpu::Buffer,

    /// Globals uniform buffer.
    pub globals: gpu::Buffer,
}

impl Lambert {
    /// Creates the basic rendering pipelines.
    pub fn new(factory: &gpu::Factory) -> Self {
        let locals = make_uniform_buffer(factory, &LOCALS);
        let globals = make_uniform_buffer(factory, &GLOBALS);
        let program = make_program(factory, "gouraud", &BINDINGS);
        Lambert { program, locals, globals }
    }

    /// Create an invocation of the basic program.
    pub fn invoke(
        &self,
        backend: &gpu::Factory,
        mx_view_projection: [[f32; 4]; 4],
        mx_world: [[f32; 4]; 4],
        color: [f32; 4],
    ) -> gpu::Invocation {
        backend.overwrite_buffer(
            self.locals.as_slice(),
            &[
                Locals {
                    u_World: mx_world,
                    u_Color: color,
                },
            ],
        );
        backend.overwrite_buffer(
            self.globals.as_slice(),
            &[
                Globals {
                    u_ViewProjection: mx_view_projection,
                },
            ],
        );
        gpu::Invocation {
            program: &self.program,
            uniforms: [
                Some(&self.locals),
                Some(&self.globals),
                None,
                None,
            ],
            samplers: [
                None,
                None,
                None,
                None,
            ],
        }
    }
}
