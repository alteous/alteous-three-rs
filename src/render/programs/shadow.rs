//! Shadow rendering programs.

use gpu::{self, program};
use std::marker;
use super::*;

/// Shadow pipeline bindings.
const BINDINGS: program::Bindings = program::Bindings {
    uniform_blocks: [
        program::UniformBlockBinding::Required(b"b_Locals\0"),
        program::UniformBlockBinding::None,
        program::UniformBlockBinding::None,
        program::UniformBlockBinding::None,
    ],
    samplers: [
        program::SamplerBinding::None,
        program::SamplerBinding::None,
        program::SamplerBinding::None,
        program::SamplerBinding::None,
    ],  
};

/// Locals uniform block binding.
const LOCALS: UniformBlockBinding<Locals> = UniformBlockBinding {
    name: b"b_Locals\0",
    index: 0,
    marker: marker::PhantomData,
};

/// Per-instance variables.
#[allow(non_snake_case)]
#[derive(Clone, Debug)]
#[repr(C)]
struct Locals {
    /// Combined world-to-view-to-projection-to-clip matrix.
    u_ModelViewProjection: [[f32; 4]; 4],
}

/// Shadow program.
pub struct Shadow {
    program: gpu::Program,
    locals: gpu::Buffer,
}

impl Shadow {
    /// Creates the shadow programs.
    pub fn new(factory: &gpu::Factory) -> Self {
        let locals = make_uniform_buffer(factory, &LOCALS);
        let program = make_program(factory, "shadow", &BINDINGS);
        Self { program, locals }
    }

    pub fn invoke<'a>(
        &'a self,
        backend: &gpu::Factory,
        mx_model_view_projection: [[f32; 4]; 4],
    ) -> gpu::Invocation {
        backend.overwrite_buffer(
            self.locals.as_slice(),
            &[Locals { u_ModelViewProjection: mx_model_view_projection }],
        );
        gpu::Invocation {
            program: &self.program,
            uniforms: [None; gpu::program::MAX_UNIFORM_BLOCKS],
            samplers: [None; gpu::program::MAX_SAMPLERS],
        }
    }
}
