//! Phong rendering pipeline.

use color;
use euler::{Mat4, Vec3, Vec4};
use gpu::{self, framebuffer as fbuf, program};
use std::{marker, mem};
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
    marker: marker::PhantomData,
};

/// Globals uniform block binding.
pub const GLOBALS: UniformBlockBinding<Globals> = UniformBlockBinding {
    name: b"b_Globals\0",
    index: 1,
    marker: marker::PhantomData,
};

/// Clear operation for the basic pipeline.
pub const CLEAR_OP: fbuf::ClearOp = fbuf::ClearOp {
    color: fbuf::ClearColor::Yes { r: 0.0, g: 0.0, b: 0.0, a: 0.0 },
    depth: fbuf::ClearDepth::Yes { z: 1.0 },
};

/// Ambient lighting parameters.
#[derive(Clone, Copy, Debug)]
struct AmbientLight {
    // 0
    color: Vec3,

    // 12
    intensity: f32,

    // 16
}

/// Directional lighting parameters.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct DirectionalLight {
    // 0
    position: Vec4,

    // 16
    direction: Vec3,
    
    // 28
    _28: u32,

    // 32
    color: Vec3,

    // 44
    intensity: f32,

    // 48
}

/// Point light parameters.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct PointLight {
    // 0
    position: Vec4,

    // 16
    color: Vec3,

    // 28
    intensity: f32,

    // 32
}

/// Per-world variables.
#[allow(non_snake_case)]
#[derive(Clone, Debug)]
pub struct Globals {
    // 0
    /// Combined world-to-view and view-to-projection matrix.
    u_ViewProjection: Mat4,

    // 64
    /// Global ambient lighting.
    u_AmbientLight: AmbientLight,

    // 80
    /// Global directional light.
    u_DirectionalLight: DirectionalLight,

    // 112
}

/// Per-instance variables.
#[allow(non_snake_case)]
#[derive(Clone, Debug)]
pub struct Locals {
    // 0
    /// Model-to-world matrix.
    u_World: [[f32; 4]; 4],

    // 64
    /// Material specular glossiness constant.
    u_Glossiness: f32,

    // 68
    _0: [u32; 3],

    // 80
    /// Local point lights.
    u_PointLights: [PointLight; MAX_POINT_LIGHTS],

    // 336
}

/// Basic rendering pipeline.
pub struct Phong {
    /// The program.
    program: gpu::Program,

    /// Locals uniform buffer.
    locals: gpu::Buffer,

    /// Globals uniform buffer.
    globals: gpu::Buffer,
}

impl Phong {
    /// Creates the basic rendering pipelines.
    pub fn new(factory: &gpu::Factory) -> Self {
        let locals = make_uniform_buffer(factory, &LOCALS);
        let globals = make_uniform_buffer(factory, &GLOBALS);
        let program = make_program(factory, "phong", &BINDINGS);
        Phong { program, locals, globals }
    }

    /// Create an invocation of the basic program.
    pub fn invoke(
        &self,
        backend: &gpu::Factory,
        mx_view_projection: [[f32; 4]; 4],
        mx_world: [[f32; 4]; 4],
        lighting: &Lighting,
        glossiness: f32,
    ) -> gpu::Invocation {
        use ::arraymap::ArrayMap;
        backend.overwrite_buffer(
            self.locals.as_slice(),
            &[
                Locals {
                    u_World: mx_world.into(),
                    u_Glossiness: glossiness.into(),
                    u_PointLights: lighting.points.map(|entry| {
                        PointLight {
                            position: vec4!(entry.position, 1.0),
                            color: color::to_linear_rgb(entry.color),
                            intensity: entry.intensity,
                            .. unsafe { mem::uninitialized() }
                        }
                    }),
                    .. unsafe { mem::uninitialized() }
                },
            ],
        );
        backend.overwrite_buffer(
            self.globals.as_slice(),
            &[
                Globals {
                    u_ViewProjection: mx_view_projection.into(),
                    u_AmbientLight: AmbientLight {
                        color: color::to_linear_rgb(lighting.ambient.color).into(),
                        intensity: lighting.ambient.intensity,
                    },
                    u_DirectionalLight: DirectionalLight {
                        position: lighting.direct.origin,
                        direction: lighting.direct.direction,
                        color: color::to_linear_rgb(lighting.direct.color).into(),
                        intensity: lighting.direct.intensity,
                        .. unsafe { mem::uninitialized() }
                    },
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
