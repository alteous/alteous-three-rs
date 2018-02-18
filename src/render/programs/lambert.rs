//! Lambert/Gouraud rendering pipeline.

use euler::{Mat4, Vec3};
use gpu::{self, framebuffer as fbuf, program};
use std::marker;
use super::*;

use arraymap::ArrayMap;

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
    direction: Vec3,
    
    // 12
    _0: u32,
    
    // 16
    color: Vec3,
    
    // 28
    intensity: f32,
    
    // 32
}

/// Point light parameters.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct PointLight {
    // 0
    position: Vec3,

    // 12
    _0: u32,
    
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
    u_World: Mat4,

    // 64
    /// Material color.
    u_Color: Vec3,

    // 76
    /// 1.0 for lighting interpolation and 0.0 otherwise.
    u_Smooth: f32,

    // 80
    /// Local point lights.
    u_PointLights: [PointLight; MAX_POINT_LIGHTS],

    // 336
}

/// Lambert/Gouraud rendering pipeline.
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
        let program = make_program(factory, "lambert", &BINDINGS);
        Self { program, locals, globals }
    }

    /// Create an invocation of the Lambert/Gouraud program.
    pub fn invoke(
        &self,
        backend: &gpu::Factory,
        mx_view_projection: Mat4,
        mx_world: Mat4,
        lighting: &Lighting,
        color: Vec3,
        smooth: bool,
    ) -> gpu::Invocation {
        backend.overwrite_buffer(
            self.locals.as_slice(),
            &[
                Locals {
                    u_World: mx_world,
                    u_Color: color,
                    u_Smooth: if smooth { 1.0 } else { 0.0 },
                    u_PointLights: lighting.points.map(|entry| {
                        PointLight {
                            position: entry.position.into(),
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
                    u_ViewProjection: mx_view_projection,
                    u_AmbientLight: AmbientLight {
                        color: color::to_linear_rgb(lighting.ambient.color),
                        intensity: lighting.ambient.intensity,
                    },
                    u_DirectionalLight: DirectionalLight {
                        direction: lighting.direct.direction,
                        color: color::to_linear_rgb(lighting.direct.color),
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
