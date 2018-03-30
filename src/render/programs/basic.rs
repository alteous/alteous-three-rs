//! Basic rendering pipeline.
//!
//! Useful for rendering meshes with a solid color or rendering mesh wireframes.

use gpu::program;
use std::marker;
use super::*;

use euler::{Mat4, Vec4};
use texture::Texture;

/// Basic pipeline bindings.
const BINDINGS: program::Bindings = program::Bindings {
    uniform_blocks: [
        program::UniformBlockBinding::Required(b"b_Locals\0"),
        program::UniformBlockBinding::Required(b"b_Globals\0"),
        program::UniformBlockBinding::None,
        program::UniformBlockBinding::None,
    ],
    samplers: [
        program::SamplerBinding::Optional(b"t_Map\0"),
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

/// Globals uniform block binding.
const GLOBALS: UniformBlockBinding<Globals> = UniformBlockBinding {
    name: b"b_Globals\0",
    index: 1,
    marker: marker::PhantomData,
};

/// Per-world variables.
#[allow(non_snake_case)]
#[derive(Clone, Debug)]
#[repr(C)]
struct Globals {
    /// Combined world-to-view and view-to-projection matrix.
    u_ViewProjection: Mat4,

    /// Inverse of view-to-projection matrix.
    u_InverseProjection: Mat4,

    /// World-to-view matrix.
    u_View: Mat4,

    /// Number of lights to apply to the rendered object.
    u_NumLights: u32,
}

/// Per-instance variables.
#[allow(non_snake_case)]
#[derive(Clone, Debug)]
#[repr(C)]
struct Locals {
    /// Model-to-world matrix.
    u_World: Mat4,

    /// Solid rendering color.3
    u_Color: Vec4,

    /// Texture co-ordinate range.
    u_UvRange: [f32; 4],
}

/// Basic rendering pipeline.
pub struct Basic {
    /// Program with texture.
    without_texture: gpu::Program,

    /// Program without texture.
    with_texture: gpu::Program,
 
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
        mx_view_projection: Mat4,
        mx_world: Mat4,
        color: Vec4,
        map: Option<&'a Texture>,
    ) -> gpu::Invocation {
        backend.overwrite_buffer(
            self.locals.as_slice(),            &[
                Locals {
                    u_World: mx_world,
                    u_Color: color,
                    u_UvRange: {
                        map
                            .map(|tex| tex.uv_range())
                            .unwrap_or([0.0, 1.0, 0.0, 1.0])
                    },
                },
            ],
        );
        backend.overwrite_buffer(
            self.globals.as_slice(),
            &[
                Globals {
                    u_ViewProjection: mx_view_projection,
                    u_InverseProjection: mat4!(),
                    u_View: mat4!(),
                    u_NumLights: 0,
                },
            ],
        );
        gpu::Invocation {
            program: if map.is_some() {
                &self.with_texture
            } else {
                &self.without_texture
            },
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
        let without_texture = make_program(factory, "basic", &BINDINGS);
        let with_texture = make_program(factory, "basic_with_texture", &BINDINGS);
        Basic { with_texture, without_texture, locals, globals }
    }
}
