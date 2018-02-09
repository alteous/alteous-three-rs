//! Basic rendering pipeline.
//!
//! Useful for rendering meshes with a solid color or rendering mesh wireframes.

use gpu::program;
use super::*;

use texture::Texture;

/// Basic pipeline bindings.
const BINDINGS: program::Bindings = program::Bindings {
    uniform_blocks: [
        program::UniformBlockBinding::Required(b"b_Locals\0"),
        program::UniformBlockBinding::Required(b"b_Globals\0"),
        program::UniformBlockBinding::None,
        program::UniformBlockBinding::None,
    ],
    samplers: [program::SamplerBinding::None; program::MAX_SAMPLERS],
};

/// Locals uniform block binding.
const LOCALS: UniformBlockBinding<Locals> = UniformBlockBinding {
    name: b"b_Locals\0",
    index: 0,
    init: Locals {
        u_Color: [0.0; 4],
        u_World: IDENTITY,
    },
};

/// Globals uniform block binding.
const GLOBALS: UniformBlockBinding<Globals> = UniformBlockBinding {
    name: b"b_Globals\0",
    index: 1,
    init: Globals {
        u_ViewProjection: IDENTITY,
        u_InverseProjection: IDENTITY,
        u_View: IDENTITY,
        u_NumLights: 0,
    },
};

/// Per-world variables.
#[allow(non_snake_case)]
#[derive(Clone, Debug)]
#[repr(C)]
struct Globals {
    /// Combined world-to-view and view-to-projection matrix.
    u_ViewProjection: [[f32; 4]; 4],

    /// Inverse of view-to-projection matrix.
    u_InverseProjection: [[f32; 4]; 4],

    /// World-to-view matrix.
    u_View: [[f32; 4]; 4],

    /// Number of lights to apply to the rendered object.
    u_NumLights: u32,
}

/// Per-instance variables.
#[allow(non_snake_case)]
#[derive(Clone, Debug)]
#[repr(C)]
struct Locals {
    /// Model-to-world matrix.
    u_World: [[f32; 4]; 4],

    /// Solid rendering color.
    u_Color: [f32; 4],
}

/// Basic rendering pipeline.
pub struct Basic {
    /// The program.
    program: gpu::Program,

    /// Locals uniform buffer.
    locals: gpu::Buffer,

    /// Globals uniform buffer.
    globals: gpu::Buffer,
}

impl Basic {
    /// Create an invocation of the basic program.
    pub fn invoke<'a>(
        &'a self,
        backend: &gpu::Factory,
        mx_view_projection: [[f32; 4]; 4],
        mx_world: [[f32; 4]; 4],
        color: [f32; 4],
        map: Option<&'a Texture>,
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
                    .. GLOBALS.init
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
                map.map(|tex| tex.to_param()),
                None,
                None,
                None,
            ],
        }
    }

    /// Creates the basic program.
    pub fn new(factory: &gpu::Factory) -> Self {
        let locals = make_uniform_buffer(factory, &LOCALS);
        let globals = make_uniform_buffer(factory, &GLOBALS);
        let program = make_program(factory, "basic", &BINDINGS);
        Basic { program, locals, globals }
    }
}
