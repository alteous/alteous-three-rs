//! The renderer.

use gfx;
use gfx::format::I8Norm;
use gpu;
use glutin;
use mint;
use itertools::Either;

pub mod source;

use std::{iter, mem};

pub use self::source::Source;

use camera::Camera;
use factory::Factory;
use geometry::Geometry;
use hub:: SubNode;
//use light::{ShadowMap, ShadowProjection};
use material::Material;
use mesh::MAX_TARGETS;
use scene::Scene;
//use text::Font;

/// Default values for type `Vertex`.
pub const DEFAULT_VERTEX: Vertex = Vertex {
    a_Position: [0.0, 0.0, 0.0, 1.0],
    a_TexCoord: [0.0, 0.0],
    a_Normal: [I8Norm(0), I8Norm(127), I8Norm(0), I8Norm(0)],
    a_Tangent: [I8Norm(127), I8Norm(0), I8Norm(0), I8Norm(0)],

    a_JointIndices: [0, 0, 0, 0],
    a_JointWeights: [1.0, 1.0, 1.0, 1.0],
};

impl Default for Vertex {
    fn default() -> Self {
        DEFAULT_VERTEX
    }
}

macro_rules! offset_of {
    ($T:ty, $field:ident) => {
        unsafe {
            let obj: $T = ::std::mem::uninitialized();
            let obj_ptr: *const u8 = ::std::mem::transmute(&obj);
            let member_ptr: *const u8 = ::std::mem::transmute(&obj.$field);

            ::std::mem::forget(obj);

            (member_ptr as usize) - (obj_ptr as usize)
        }
    }
}

#[allow(non_snake_case)]
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Vertex {
    /// Vertex position in local co-ordinate space.
    pub a_Position: [f32; 4],

    /// Vertex texture co-ordiante in 2D texture space.
    pub a_TexCoord: [f32; 2],

    /// Vertex normal in local co-ordinate space.
    pub a_Normal: [gfx::format::I8Norm; 4],

    /// Vertex tangent in local co-ordinate space
    /// in the form `[x, y, z, w]` where `w` defines
    /// handedness of the tangent.
    pub a_Tangent: [gfx::format::I8Norm; 4],

    /// Indices of joint matrices.
    pub a_JointIndices: [u32; 4],

    /// Weights of joint matrix contributions.
    pub a_JointWeights: [f32; 4],
}

use gpu::buffer::Format;

const NORMAL_Z: [I8Norm; 4] = [
    I8Norm(0), I8Norm(0), I8Norm(1), I8Norm(0)
];

const TANGENT_X: [I8Norm; 4] = [
    I8Norm(1), I8Norm(0), I8Norm(0),  I8Norm(1)
];

use factory::f2i;

pub fn make_vertices(geometry: &Geometry) -> Vec<Vertex> {
    let position_iter = geometry.vertices.iter();
    let normal_iter = if geometry.normals.is_empty() {
        Either::Left(iter::repeat(NORMAL_Z))
    } else {
        Either::Right(
            geometry
                .normals
                .iter()
                .map(|n| [f2i(n.x), f2i(n.y), f2i(n.z), I8Norm(0)]),
        )
    };
    let tex_coord_iter = if geometry.tex_coords.is_empty() {
        Either::Left(iter::repeat([0.0, 0.0]))
    } else {
        Either::Right(geometry.tex_coords.iter().map(|uv| [uv.x, uv.y]))
    };
    let tangent_iter = if geometry.tangents.is_empty() {
        Either::Left(iter::repeat(TANGENT_X))
    } else {
        Either::Right(
            geometry
                .tangents
                .iter()
                .map(|t| [f2i(t.x), f2i(t.y), f2i(t.z), f2i(t.w)]),
        )
    };
    let joint_indices_iter = if geometry.joints.indices.is_empty() {
        Either::Left(iter::repeat([0, 0, 0, 0]))
    } else {
        Either::Right(
            geometry.joints.indices
                .iter()
                .map(|x| [x[0] as u32, x[1] as u32, x[2] as u32, x[3] as u32])
        )
    };
    let joint_weights_iter = if geometry.joints.weights.is_empty() {
        Either::Left(iter::repeat([1.0, 1.0, 1.0, 1.0]))
    } else {
        Either::Right(geometry.joints.weights.iter().cloned())
    };

    izip!(
        position_iter,
        normal_iter,
        tangent_iter,
        tex_coord_iter,
        joint_indices_iter,
        joint_weights_iter
    )
        .map(|(pos, normal, tangent, uv, joint_indices, joint_weights)| {
            Vertex {
                a_Position: [pos.x, pos.y, pos.z, 1.0],
                a_Normal: normal,
                a_TexCoord: uv,
                a_Tangent: tangent,
                a_JointIndices: joint_indices,
                a_JointWeights: joint_weights,
                .. Default::default()
            }
        })
        .collect()
}

use gpu::buffer as buf;

pub fn make_vertex_array(
    factory: &gpu::Factory,
    indices: Option<&[[u32; 3]]>,
    vertices: &[Vertex],
) -> gpu::VertexArray {
    let vertex_buffer = factory.buffer(buf::Kind::Array, buf::Usage::StaticDraw);
    factory.initialize_buffer(&vertex_buffer, vertices);

    let position = gpu::Accessor::new(
        vertex_buffer.clone(),
        Format::Float { bits: 32, size: 3 },
        offset_of!(Vertex, a_Position),
        mem::size_of::<Vertex>(),
    );
    let tex_coord = gpu::Accessor::new(
        vertex_buffer.clone(),
        Format::Float { bits: 32, size: 2 },
        offset_of!(Vertex, a_TexCoord),
        mem::size_of::<Vertex>(),
    );
    let normal = gpu::Accessor::new(
        vertex_buffer.clone(),
        Format::Signed { bits: 8, size: 4, norm: true },
        offset_of!(Vertex, a_Normal),
        mem::size_of::<Vertex>(),
    );
    let tangent = gpu::Accessor::new(
        vertex_buffer.clone(),
        Format::Signed { bits: 8, size: 4, norm: true },
        offset_of!(Vertex, a_Tangent),
        mem::size_of::<Vertex>(),
    );
    let joint_indices = gpu::Accessor::new(
        vertex_buffer.clone(),
        Format::Unsigned { bits: 32, size: 4, norm: false },
        offset_of!(Vertex, a_JointIndices),
        mem::size_of::<Vertex>(),
    );
    let joint_weights = gpu::Accessor::new(
        vertex_buffer.clone(),
        Format::Float { bits: 32, size: 4 },
        offset_of!(Vertex, a_JointWeights),
        mem::size_of::<Vertex>(),
    );
    let index_buffer = indices.map(|slice| {
        let buffer = factory.buffer(buf::Kind::Index, buf::Usage::StaticDraw);
        factory.initialize_buffer(&buffer, slice);
        buffer
    });

    let mut builder = gpu::VertexArray::builder();
    builder.attribute(0, position);
    builder.attribute(1, tex_coord);
    builder.attribute(2, normal);
    builder.attribute(3, tangent);
    builder.attribute(4, joint_indices);
    builder.attribute(5, joint_weights);
    {
        if let Some(buf) = index_buffer {
            builder.indices(
                gpu::Accessor::new(
                    buf,
                    Format::Unsigned { bits: 32, norm: false, size: 1 },
                    0,
                    mem::size_of::<u32>(),
                )
            );  
        }
    }

    factory.vertex_array(builder)
}

/// Enables/disables weight contributions to vertex inputs.
#[derive(Clone, Debug)]
pub struct DisplacementContribution {
    /// Set to `1.0` if weights should influence `a_Position`.
    pub position: f32,
    /// Set to `1.0` if weights should influence `a_Normal`.
    pub normal: f32,
    /// Set to `1.0` if weights should influence `a_Tangent`.
    pub tangent: f32,
    /// The displacement weight.
    pub weight: f32,
}

/// Set of zero valued displacement contribution which cause vertex attributes
/// to be unchanged by morph targets.
pub const ZEROED_DISPLACEMENT_CONTRIBUTION: [DisplacementContribution; MAX_TARGETS] = [
    DisplacementContribution { position: 0.0, normal: 0.0, tangent: 0.0, weight: 0.0 },
    DisplacementContribution { position: 0.0, normal: 0.0, tangent: 0.0, weight: 0.0 },
    DisplacementContribution { position: 0.0, normal: 0.0, tangent: 0.0, weight: 0.0 },
    DisplacementContribution { position: 0.0, normal: 0.0, tangent: 0.0, weight: 0.0 },
    DisplacementContribution { position: 0.0, normal: 0.0, tangent: 0.0, weight: 0.0 },
    DisplacementContribution { position: 0.0, normal: 0.0, tangent: 0.0, weight: 0.0 },
    DisplacementContribution { position: 0.0, normal: 0.0, tangent: 0.0, weight: 0.0 },
    DisplacementContribution { position: 0.0, normal: 0.0, tangent: 0.0, weight: 0.0 },
];

/*
/// The format of the back buffer color requested from the windowing system.
pub type ColorFormat = gfx::format::Rgba8;
/// The format of the depth stencil buffer requested from the windowing system.
pub type DepthFormat = gfx::format::DepthStencil;
/// The format of the shadow buffer.
pub type ShadowFormat = gfx::format::Depth32F;
/// The concrete type of a basic pipeline.
pub type BasicPipelineState = gfx::PipelineState<back::Resources, basic_pipe::Meta>;

const MAX_LIGHTS: usize = 4;

const STENCIL_SIDE: gfx::state::StencilSide = gfx::state::StencilSide {
    fun: gfx::state::Comparison::Always,
    mask_read: 0,
    mask_write: 0,
    op_fail: gfx::state::StencilOp::Keep,
    op_depth_fail: gfx::state::StencilOp::Keep,
    op_pass: gfx::state::StencilOp::Keep,
};

#[cfg_attr(rustfmt, rustfmt_skip)]
quick_error! {
    #[doc = "Error encountered when building pipelines."]
    #[derive(Debug)]
    pub enum PipelineCreationError {
        #[doc = "GLSL compiler/linker error."]
        Compilation(err: gfx::shade::ProgramError) {
            from()
            description("GLSL program compilation error")
            display("GLSL program compilation error")
            cause(err)
        }

        #[doc = "Pipeline state error."]
        State(err: gfx::PipelineStateError<String>) {
            from()
            description("Pipeline state error")
            display("Pipeline state error")
            cause(err)
        }

        #[doc = "Standard I/O error."]
        Io(err: io::Error) {
            from()
            description("I/O error")
            display("I/O error")
            cause(err)
        }
    }
}

/// Default values for type `Vertex`.
pub const DEFAULT_VERTEX: Vertex = Vertex {
    pos: [0.0, 0.0, 0.0, 1.0],
    uv: [0.0, 0.0],
    normal: [I8Norm(0), I8Norm(127), I8Norm(0), I8Norm(0)],
    tangent: [I8Norm(127), I8Norm(0), I8Norm(0), I8Norm(0)],
    joint_indices: [0.0, 0.0, 0.0, 0.0],
    joint_weights: [1.0, 1.0, 1.0, 1.0],

    displacement0: [0.0, 0.0, 0.0, 0.0],
    displacement1: [0.0, 0.0, 0.0, 0.0],
    displacement2: [0.0, 0.0, 0.0, 0.0],
    displacement3: [0.0, 0.0, 0.0, 0.0],
    displacement4: [0.0, 0.0, 0.0, 0.0],
    displacement5: [0.0, 0.0, 0.0, 0.0],
    displacement6: [0.0, 0.0, 0.0, 0.0],
    displacement7: [0.0, 0.0, 0.0, 0.0],
};

impl Default for Vertex {
    fn default() -> Self {
        DEFAULT_VERTEX
    }
}

#[cfg_attr(rustfmt, rustfmt_skip)]
gfx_defines! {
    vertex Vertex {
        pos: [f32; 4] = "a_Position",
        uv: [f32; 2] = "a_TexCoord",
        normal: [gfx::format::I8Norm; 4] = "a_Normal",
        tangent: [gfx::format::I8Norm; 4] = "a_Tangent",
        joint_indices: [f32; 4] = "a_JointIndices",
        joint_weights: [f32; 4] = "a_JointWeights",

        displacement0: [f32; 4] = "a_Displacement0",
        displacement1: [f32; 4] = "a_Displacement1",
        displacement2: [f32; 4] = "a_Displacement2",
        displacement3: [f32; 4] = "a_Displacement3",
        displacement4: [f32; 4] = "a_Displacement4",
        displacement5: [f32; 4] = "a_Displacement5",
        displacement6: [f32; 4] = "a_Displacement6",
        displacement7: [f32; 4] = "a_Displacement7",
    }

    constant Locals {
        color: [f32; 4] = "u_Color",
        mat_params: [f32; 4] = "u_MatParams",
        uv_range: [f32; 4] = "u_UvRange",
        mx_world: [[f32; 4]; 4] = "u_World",
    }

    constant LightParam {
        projection: [[f32; 4]; 4] = "projection",
        pos: [f32; 4] = "pos",
        dir: [f32; 4] = "dir",
        focus: [f32; 4] = "focus",
        color: [f32; 4] = "color",
        color_back: [f32; 4] = "color_back",
        intensity: [f32; 4] = "intensity",
        shadow_params: [i32; 4] = "shadow_params",
    }

    constant Globals {
        mx_vp: [[f32; 4]; 4] = "u_ViewProj",
        mx_inv_proj: [[f32; 4]; 4] = "u_InverseProj",
        mx_view: [[f32; 4]; 4] = "u_View",
        num_lights: u32 = "u_NumLights",
    }

    pipeline basic_pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        cb_locals: gfx::ConstantBuffer<Locals> = "b_Locals",
        cb_lights: gfx::ConstantBuffer<LightParam> = "b_Lights",
        cb_globals: gfx::ConstantBuffer<Globals> = "b_Globals",
        tex_map: gfx::TextureSampler<[f32; 4]> = "t_Map",
        shadow_map0: gfx::TextureSampler<f32> = "t_Shadow0",
        shadow_map1: gfx::TextureSampler<f32> = "t_Shadow1",
        out_color: gfx::BlendTarget<ColorFormat> =
            ("Target0", gfx::state::ColorMask::all(), gfx::preset::blend::REPLACE),
        out_depth: gfx::DepthStencilTarget<DepthFormat> =
            (gfx::preset::depth::LESS_EQUAL_WRITE, gfx::state::Stencil {
                front: STENCIL_SIDE, back: STENCIL_SIDE,
            }),
    }

    pipeline shadow_pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        cb_locals: gfx::ConstantBuffer<Locals> = "b_Locals",
        cb_globals: gfx::ConstantBuffer<Globals> = "b_Globals",
        target: gfx::DepthTarget<ShadowFormat> =
            gfx::preset::depth::LESS_EQUAL_WRITE,
    }

    constant QuadParams {
        rect: [f32; 4] = "u_Rect",
        depth: f32 = "u_Depth",
    }

    pipeline quad_pipe {
        params: gfx::ConstantBuffer<QuadParams> = "b_Params",
        globals: gfx::ConstantBuffer<Globals> = "b_Globals",
        resource: gfx::RawShaderResource = "t_Input",
        sampler: gfx::Sampler = "t_Input",
        target: gfx::RenderTarget<ColorFormat> = "Target0",
        depth_target: gfx::DepthTarget<DepthFormat> =
            gfx::preset::depth::LESS_EQUAL_TEST,
    }

    constant PbrParams {
        base_color_factor: [f32; 4] = "u_BaseColorFactor",
        camera: [f32; 3] = "u_Camera",
        _padding0: f32 = "_padding0",
        emissive_factor: [f32; 3] = "u_EmissiveFactor",
        _padding1: f32 = "_padding1",
        metallic_roughness: [f32; 2] = "u_MetallicRoughnessValues",
        normal_scale: f32 = "u_NormalScale",
        occlusion_strength: f32 = "u_OcclusionStrength",
        pbr_flags: i32 = "u_PbrFlags",
    }

    constant DisplacementContribution {
        position: f32 = "position",
        normal: f32 = "normal",
        tangent: f32 = "tangent",
        weight: f32 = "weight",
    }

    pipeline pbr_pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),

        locals: gfx::ConstantBuffer<Locals> = "b_Locals",
        globals: gfx::ConstantBuffer<Globals> = "b_Globals",
        params: gfx::ConstantBuffer<PbrParams> = "b_PbrParams",
        lights: gfx::ConstantBuffer<LightParam> = "b_Lights",
        displacement_contributions: gfx::ConstantBuffer<DisplacementContribution> = "b_DisplacementContributions",
        joint_transforms: gfx::ShaderResource<[f32; 4]> = "b_JointTransforms",

        base_color_map: gfx::TextureSampler<[f32; 4]> = "u_BaseColorSampler",

        normal_map: gfx::TextureSampler<[f32; 4]> = "u_NormalSampler",

        emissive_map: gfx::TextureSampler<[f32; 4]> = "u_EmissiveSampler",

        metallic_roughness_map: gfx::TextureSampler<[f32; 4]> = "u_MetallicRoughnessSampler",

        occlusion_map: gfx::TextureSampler<[f32; 4]> = "u_OcclusionSampler",

        color_target: gfx::RenderTarget<ColorFormat> = "Target0",
        depth_target: gfx::DepthTarget<DepthFormat> = gfx::preset::depth::LESS_EQUAL_WRITE,
    }
}

impl Default for DisplacementContribution {
    fn default() -> Self {
        Self { position: 0.0, normal: 0.0, tangent: 0.0, weight: 0.0 }
    }
}

//TODO: private fields?
#[derive(Clone, Debug)]
pub(crate) struct GpuData {
    pub slice: gfx::Slice<back::Resources>,
    pub vertices: gfx::handle::Buffer<back::Resources, Vertex>,
    pub constants: gfx::handle::Buffer<back::Resources, Locals>,
    pub pending: Option<DynamicData>,
    pub displacement_contributions: [DisplacementContribution; MAX_TARGETS],
}

#[derive(Clone, Debug)]
pub(crate) struct DynamicData {
    pub num_vertices: usize,
    pub buffer: gfx::handle::Buffer<back::Resources, Vertex>,
}

/// Shadow type is used to specify shadow's rendering algorithm.
pub enum ShadowType {
    /// Force no shadows.
    Off,
    /// Basic (and fast) single-sample shadows.
    Basic,
    /// Percentage-closest filter (PCF).
    Pcf,
}

bitflags! {
    struct PbrFlags: i32 {
        const BASE_COLOR_MAP         = 1 << 0;
        const NORMAL_MAP             = 1 << 1;
        const METALLIC_ROUGHNESS_MAP = 1 << 2;
        const EMISSIVE_MAP           = 1 << 3;
        const OCCLUSION_MAP          = 1 << 4;
    }
}

struct DebugQuad {
    resource: gfx::handle::RawShaderResourceView<back::Resources>,
    pos: [i32; 2],
    size: [i32; 2],
}

/// All pipeline state objects used by the `three` renderer.
pub struct PipelineStates {
    /// Corresponds to `Material::Basic`.
    mesh_basic_fill: BasicPipelineState,

    /// Corresponds to `Material::Line`.
    line_basic: BasicPipelineState,

    /// Corresponds to `Material::Wireframe`.
    mesh_basic_wireframe: BasicPipelineState,

    /// Corresponds to `Material::Gouraud`.
    mesh_gouraud: BasicPipelineState,

    /// Corresponds to `Material::Phong`.
    mesh_phong: BasicPipelineState,

    /// Corresponds to `Material::Sprite`.
    sprite: BasicPipelineState,

    /// Used internally for shadow casting.
    shadow: gfx::PipelineState<back::Resources, shadow_pipe::Meta>,

    /// Used internally for rendering sprites.
    quad: gfx::PipelineState<back::Resources, quad_pipe::Meta>,

    /// Corresponds to `Material::Pbr`.
    pbr: gfx::PipelineState<back::Resources, pbr_pipe::Meta>,

    /// Used internally for rendering `Background::Skybox`.
    skybox: gfx::PipelineState<back::Resources, quad_pipe::Meta>,
}

impl PipelineStates {
    /// Creates the set of pipeline states needed by the `three` renderer.
    pub fn new(
        src: &source::Set,
        factory: &mut Factory,
    ) -> Result<Self, PipelineCreationError> {
        Self::init(src, &mut factory.backend)
    }

    /// Implementation of `PipelineStates::new`.
    pub(crate) fn init(
        src: &source::Set,
        backend: &mut back::Factory,
    ) -> Result<Self, PipelineCreationError> {
        let basic = backend.create_shader_set(&src.basic.vs, &src.basic.ps)?;
        let gouraud = backend.create_shader_set(&src.gouraud.vs, &src.gouraud.ps)?;
        let phong = backend.create_shader_set(&src.phong.vs, &src.phong.ps)?;
        let sprite = backend.create_shader_set(&src.sprite.vs, &src.sprite.ps)?;
        let shadow = backend.create_shader_set(&src.shadow.vs, &src.shadow.ps)?;
        let quad = backend.create_shader_set(&src.quad.vs, &src.quad.ps)?;
        let pbr = backend.create_shader_set(&src.pbr.vs, &src.pbr.ps)?;
        let skybox = backend.create_shader_set(&src.skybox.vs, &src.skybox.ps)?;

        let rast_quad = gfx::state::Rasterizer::new_fill();
        let rast_fill = gfx::state::Rasterizer::new_fill().with_cull_back();
        let rast_wire = gfx::state::Rasterizer {
            method: gfx::state::RasterMethod::Line(1),
            ..rast_fill
        };
        let rast_shadow = gfx::state::Rasterizer {
            offset: Some(gfx::state::Offset(2, 2)),
            ..rast_fill
        };

        let pso_mesh_basic_fill = backend.create_pipeline_state(
            &basic,
            gfx::Primitive::TriangleList,
            rast_fill,
            basic_pipe::new(),
        )?;
        let pso_line_basic = backend.create_pipeline_state(
            &basic,
            gfx::Primitive::LineStrip,
            rast_fill,
            basic_pipe::new(),
        )?;
        let pso_mesh_basic_wireframe = backend.create_pipeline_state(
            &basic,
            gfx::Primitive::TriangleList,
            rast_wire,
            basic_pipe::new(),
        )?;
        let pso_mesh_gouraud = backend.create_pipeline_state(
            &gouraud,
            gfx::Primitive::TriangleList,
            rast_fill,
            basic_pipe::new(),
        )?;
        let pso_mesh_phong = backend.create_pipeline_state(
            &phong,
            gfx::Primitive::TriangleList,
            rast_fill,
            basic_pipe::new(),
        )?;
        let pso_sprite = backend.create_pipeline_state(
            &sprite,
            gfx::Primitive::TriangleStrip,
            rast_fill,
            basic_pipe::Init {
                out_color: ("Target0", gfx::state::ColorMask::all(), gfx::preset::blend::ALPHA),
                ..basic_pipe::new()
            },
        )?;
        let pso_shadow = backend.create_pipeline_state(
            &shadow,
            gfx::Primitive::TriangleList,
            rast_shadow,
            shadow_pipe::new(),
        )?;
        let pso_quad = backend.create_pipeline_state(
            &quad,
            gfx::Primitive::TriangleStrip,
            rast_quad,
            quad_pipe::new(),
        )?;
        let pso_skybox = backend.create_pipeline_state(
            &skybox,
            gfx::Primitive::TriangleStrip,
            rast_quad,
            quad_pipe::new(),
        )?;
        let pso_pbr = backend.create_pipeline_state(
            &pbr,
            gfx::Primitive::TriangleList,
            rast_fill,
            pbr_pipe::new(),
        )?;

        Ok(PipelineStates {
            mesh_basic_fill: pso_mesh_basic_fill,
            line_basic: pso_line_basic,
            mesh_basic_wireframe: pso_mesh_basic_wireframe,
            mesh_gouraud: pso_mesh_gouraud,
            mesh_phong: pso_mesh_phong,
            sprite: pso_sprite,
            shadow: pso_shadow,
            quad: pso_quad,
            pbr: pso_pbr,
            skybox: pso_skybox,
        })
    }
}

/// Handle for additional viewport to render some relevant debug information.
/// See [`Renderer::debug_shadow_quad`](struct.Renderer.html#method.debug_shadow_quad).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DebugQuadHandle(froggy::Pointer<DebugQuad>);

/// Renders [`Scene`](struct.Scene.html) by [`Camera`](struct.Camera.html).
///
/// See [Window::render](struct.Window.html#method.render).
pub struct OldRenderer {
    device: back::Device,
    encoder: gfx::Encoder<back::Resources, back::CommandBuffer>,
    const_buf: gfx::handle::Buffer<back::Resources, Globals>,
    quad_buf: gfx::handle::Buffer<back::Resources, QuadParams>,
    light_buf: gfx::handle::Buffer<back::Resources, LightParam>,
    pbr_buf: gfx::handle::Buffer<back::Resources, PbrParams>,
    displacement_contributions_buf: gfx::handle::Buffer<back::Resources, DisplacementContribution>,
    out_color: gfx::handle::RenderTargetView<back::Resources, ColorFormat>,
    out_depth: gfx::handle::DepthStencilView<back::Resources, DepthFormat>,
    default_joint_buffer_view: gfx::handle::ShaderResourceView<back::Resources, [f32; 4]>,
    pso: PipelineStates,
    map_default: Texture<[f32; 4]>,
    shadow_default: Texture<f32>,
    debug_quads: froggy::Storage<DebugQuad>,
    size: (u32, u32),
    font_cache: HashMap<PathBuf, Font>,
    /// `ShadowType` of this `Renderer`.
    pub shadow: ShadowType,
}
*/

#[repr(C)]
struct TriangleVertex {
    position: [f32; 3],
}

#[repr(C)]
struct UniformBlock {
    color: [f32; 4],
}

const POSITION_FORMAT: gpu::buffer::Format = gpu::buffer::Format::Float { size: 3, bits: 32 };

const TRIANGLE_DATA: &'static [TriangleVertex] = &[
    TriangleVertex { position: [ -0.5, -0.5, 0.0 ] },
    TriangleVertex { position: [ 0.5, -0.5, 0.0 ] },
    TriangleVertex { position: [ 0.0, 0.5, 0.0 ] },
];

const YELLOW: &'static [UniformBlock] = &[
    UniformBlock { color: [1.0, 1.0, 0.0, 1.0] },
];

const GREEN_PIXEL: &'static [[u8; 4]] = &[
    [0, 255, 0, 255],
];

pub struct Renderer {
    factory: Factory,
}

/*
        let vertex_shader = {
            let mut source = read_file_to_end("gpu/triangle.vert").unwrap();
            source.push(0);
            factory.program_object(
                gpu::program::Kind::Vertex,
                cstr(&source),
            )
        };
        let fragment_shader = {
            let mut source = read_file_to_end("gpu/triangle.frag").unwrap();
            source.push(0);
            factory.program_object(
                gpu::program::Kind::Fragment,
                cstr(&source),
            )
        };
        let (program, block_binding, sampler_binding) = {
            let prog = factory.program(
                &vertex_shader,
                &fragment_shader,
            );
            let bname = cstr(b"UniformBlock\0");
            let bbinding = factory.query_uniform_block_index(&prog, bname);
            let sname = cstr(b"u_Sampler\0");
            let sbinding = factory.query_uniform_index(&prog, sname);
            (prog, bbinding.unwrap() as usize, sbinding.unwrap() as usize)
        };

        let vertex_buffer = factory.buffer(gpu::buffer::Kind::Array, gpu::buffer::Usage::StaticDraw);
        factory.initialize_buffer(&vertex_buffer, TRIANGLE_DATA);

        let uniform_buffer = factory.buffer(gpu::buffer::Kind::Uniform, gpu::buffer::Usage::DynamicDraw);
        factory.initialize_buffer(&uniform_buffer, YELLOW);
        
        let position_accessor = gpu::buffer::Accessor::new(vertex_buffer, POSITION_FORMAT, 0, 0);
        let mut vertex_array_builder = gpu::VertexArray::builder();
        vertex_array_builder.attributes.insert(0, position_accessor);
        let vertex_array = factory.vertex_array(vertex_array_builder);

        let texture = factory.texture2(Default::default());
        factory.initialize_texture2(
            &texture,
            true,
            0x1908, // gl::RGBA8,
            1,
            1,
            0x1908, // gl::RGBA,
            0x1401, // gl::UNSIGNED_BYTE,
            GREEN_PIXEL,
        );
            
        let sampler = gpu::Sampler::from_texture2(texture);
*/

impl Renderer {
    pub fn new(factory: Factory) -> Self {
        Self { factory }
    }

    fn make_invocation<'a>(
        program: &'a gpu::Program,
        _material: Material,
    ) -> gpu::program::Invocation<'a> {
        let uniforms = [None, None, None, None];
        let samplers = [None, None, None, None];
        gpu::program::Invocation {
            program,
            uniforms,
            samplers,
        }
    }

    /// Render everything in the scene as viewed by the given camera.
    pub fn render(
        &mut self,
        scene: &Scene,
        _camera: &Camera,
    ) {
        let mut hub = scene.hub.lock().expect("acquire hub lock");
        let scene_id = hub.nodes[&scene.object.node].scene_id;

        hub.process_messages();
        hub.update_graph();

        for node in hub.nodes.iter_mut() {
            if !node.visible || node.scene_id != scene_id {
                continue;
            }
            if let SubNode::Visual(ref data) = node.sub_node {
                let invocation = Self::make_invocation(
                    &data.program,
                    data.material.clone(),
                );
                let draw_call = gpu::DrawCall {
                    primitive: gpu::Primitive::Triangles,
                    mode: data.mode,
                    offset: data.range.start,
                    count: data.range.end - data.range.start,
                };
                self.factory.draw(
                    &data.vertex_array,
                    &draw_call,
                    &invocation,
                );
            }
        }
    }

    pub(crate) fn resize(
        &mut self,
        _window: &glutin::GlWindow,
    ) {
        println!("resize");
    }

    /// Returns current viewport aspect ratio, i.e. width / height.
    pub fn aspect_ratio(&self) -> f32 {
        1.0 // self.size.0 as f32 / self.size.1 as f32
    }

    /// Map screen pixel coordinates to Normalized Display Coordinates.
    /// The lower left corner corresponds to (-1,-1), and the upper right corner
    /// corresponds to (1,1).
    pub fn map_to_ndc<P: Into<mint::Point2<f32>>>(
        &self,
        _point: P,
    ) -> mint::Point2<f32> {
        /*
        let point = point.into();
        mint::Point2 {
            x: 2.0 * point.x / self.size.0 as f32 - 1.0,
            y: 1.0 - 2.0 * point.y / self.size.1 as f32,
        }
         */
        mint::Point2 { x: 0.0, y: 0.0 }
    }
}

/*
impl OldRenderer {
    #[cfg(feature = "opengl")]
    pub(crate) fn new(
        builder: glutin::WindowBuilder,
        context: glutin::ContextBuilder,
        event_loop: &glutin::EventsLoop,
        source: &source::Set,
    ) -> (Self, glutin::GlWindow, Factory) {
        use gfx::texture as t;
        use glutin::GlContext;
        
        let (window, device, mut gl_factory, out_color, out_depth) = gfx_window_glutin::init(builder, context, event_loop);
        let (_, srv_white) = gl_factory
            .create_texture_immutable::<gfx::format::Rgba8>(t::Kind::D2(1, 1, t::AaMode::Single), t::Mipmap::Provided, &[&[[0xFF; 4]]])
            .unwrap();
        let (_, srv_shadow) = gl_factory
            .create_texture_immutable::<(gfx::format::R32, gfx::format::Float)>(t::Kind::D2(1, 1, t::AaMode::Single), t::Mipmap::Provided, &[&[0x3F800000]])
            .unwrap();
        let sampler = gl_factory.create_sampler_linear();
        let sampler_shadow = gl_factory.create_sampler(t::SamplerInfo {
            comparison: Some(gfx::state::Comparison::Less),
            border: t::PackedColor(!0), // clamp to 1.0
            ..t::SamplerInfo::new(t::FilterMethod::Bilinear, t::WrapMode::Border)
        });
        let default_joint_buffer = gl_factory
            .create_buffer_immutable(
                &[
                    [1.0, 0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0, 1.0],
                ],
                gfx::buffer::Role::Constant,
                gfx::memory::Bind::SHADER_RESOURCE,
            )
            .unwrap();
        let default_joint_buffer_view = gl_factory
            .view_buffer_as_shader_resource(&default_joint_buffer)
            .unwrap();
        let encoder = gl_factory.create_command_buffer().into();
        let const_buf = gl_factory.create_constant_buffer(1);
        let quad_buf = gl_factory.create_constant_buffer(1);
        let light_buf = gl_factory.create_constant_buffer(MAX_LIGHTS);
        let pbr_buf = gl_factory.create_constant_buffer(1);
        let displacement_contributions_buf = gl_factory.create_constant_buffer(MAX_TARGETS);
        let pso = PipelineStates::init(source, &mut gl_factory).unwrap();
        let renderer = OldRenderer {
            device,
            encoder,
            const_buf,
            quad_buf,
            light_buf,
            pbr_buf,
            displacement_contributions_buf,
            out_color,
            out_depth,
            pso,
            default_joint_buffer_view,
            map_default: Texture::new(srv_white, sampler, [1, 1]),
            shadow_default: Texture::new(srv_shadow, sampler_shadow, [1, 1]),
            shadow: ShadowType::Basic,
            debug_quads: froggy::Storage::new(),
            font_cache: HashMap::new(),
            size: window.get_inner_size().unwrap(),
        };
        let factory = Factory::new(gl_factory);
        (renderer, window, factory)
    }

    /// Reloads the shaders.
    pub fn reload(
        &mut self,
        pipeline_states: PipelineStates,
    ) {
        self.pso = pipeline_states;
    }

    pub(crate) fn resize(
        &mut self,
        window: &glutin::GlWindow,
    ) {
        let size = window.get_inner_size().unwrap();

        // skip updating view and self size if some
        // of the sides equals to zero (fixes crash on minimize on Windows machines)
        if size.0 == 0 || size.1 == 0 {
            return;
        }

        self.size = size;
        gfx_window_glutin::update_views(window, &mut self.out_color, &mut self.out_depth);
    }

    /// Returns current viewport aspect ratio, i.e. width / height.
    pub fn aspect_ratio(&self) -> f32 {
        self.size.0 as f32 / self.size.1 as f32
    }

    /// Map screen pixel coordinates to Normalized Display Coordinates.
    /// The lower left corner corresponds to (-1,-1), and the upper right corner
    /// corresponds to (1,1).
    pub fn map_to_ndc<P: Into<mint::Point2<f32>>>(
        &self,
        point: P,
    ) -> mint::Point2<f32> {
        let point = point.into();
        mint::Point2 {
            x: 2.0 * point.x / self.size.0 as f32 - 1.0,
            y: 1.0 - 2.0 * point.y / self.size.1 as f32,
        }
    }

    /// See [`Window::render`](struct.Window.html#method.render).
    pub fn old_render(
        &mut self,
        scene: &Scene,
        camera: &Camera,
    ) {
        self.device.cleanup();

        let mut hub = scene.hub.lock().expect("acquire hub lock");
        let scene_id = hub.nodes[&scene.object.node].scene_id;

        hub.process_messages();
        hub.update_graph();
        {
            // Update joint transforms of skeletons
            let mut cursor = hub.nodes.cursor();
            while let Some((left, mut item, right)) = cursor.next() {
                let world_transform = item.world_transform.clone();
                match &mut item.sub_node {
                    &mut SubNode::Skeleton(ref mut skeleton) => {
                        skeleton.cpu_buffer.clear();
                        for (bone, ibm) in skeleton.bones.iter().zip(skeleton.inverse_bind_matrices.iter()) {
                            let bone_transform = Matrix4::from(left.get(&bone.object.node).or_else(|| right.get(&bone.object.node)).unwrap().world_transform);
                            let inverse_world_transform = Matrix4::from(world_transform).invert().unwrap();
                            let mx = inverse_world_transform * bone_transform * Matrix4::from(ibm.clone());
                            skeleton.cpu_buffer.push(mx.x.into());
                            skeleton.cpu_buffer.push(mx.y.into());
                            skeleton.cpu_buffer.push(mx.z.into());
                            skeleton.cpu_buffer.push(mx.w.into());
                        }

                        self.encoder
                            .update_buffer(
                                &skeleton.gpu_buffer,
                                &skeleton.cpu_buffer[..],
                                0,
                            )
                            .expect("upload to GPU target buffer");
                    }
                    _ => {}
                }
            }
        }
        // update dynamic meshes
        for node in hub.nodes.iter_mut() {
            if !node.visible || node.scene_id != scene_id {
                continue;
            }
            if let SubNode::Visual(_, ref mut gpu_data, _) = node.sub_node {
                if let Some(dynamic) = gpu_data.pending.take() {
                    self.encoder
                        .copy_buffer(
                            &dynamic.buffer,
                            &gpu_data.vertices,
                            0,
                            0,
                            dynamic.num_vertices,
                        )
                        .unwrap();
                }
            }
        }

        // gather lights
        struct ShadowRequest {
            target: gfx::handle::DepthStencilView<back::Resources, ShadowFormat>,
            resource: gfx::handle::ShaderResourceView<back::Resources, f32>,
            mx_view: Matrix4<f32>,
            mx_proj: Matrix4<f32>,
        }
        let mut lights = Vec::new();
        let mut shadow_requests = Vec::new();
        for node in hub.nodes.iter() {
            if !node.visible || node.scene_id != scene_id {
                continue;
            }
            if let SubNode::Light(ref light) = node.sub_node {
                if lights.len() == MAX_LIGHTS {
                    error!("Max number of lights ({}) reached", MAX_LIGHTS);
                    break;
                }
                let shadow_index = if let Some((ref map, ref projection)) = light.shadow {
                    let target = map.to_target();
                    let dim = target.get_dimensions();
                    let aspect = dim.0 as f32 / dim.1 as f32;
                    let mx_proj = match projection {
                        &ShadowProjection::Orthographic(ref p) => p.matrix(aspect),
                    };
                    let mx_view = Matrix4::from(node.world_transform.inverse_transform().unwrap());
                    shadow_requests.push(ShadowRequest {
                        target,
                        resource: map.to_resource(),
                        mx_view: mx_view,
                        mx_proj: mx_proj.into(),
                    });
                    shadow_requests.len() as i32 - 1
                } else {
                    -1
                };
                let mut color_back = 0;
                let mut p = node.world_transform.disp.extend(1.0);
                let d = node.world_transform.rot * Vector3::unit_z();
                let intensity = match light.sub_light {
                    SubLight::Ambient => [light.intensity, 0.0, 0.0, 0.0],
                    SubLight::Directional => {
                        p = d.extend(0.0);
                        [0.0, light.intensity, 0.0, 0.0]
                    }
                    SubLight::Hemisphere { ground } => {
                        color_back = ground | 0x010101; // can't be 0
                        p = d.extend(0.0);
                        [light.intensity, 0.0, 0.0, 0.0]
                    }
                    SubLight::Point => [0.0, light.intensity, 0.0, 0.0],
                };
                let projection = if shadow_index >= 0 {
                    let request = &shadow_requests[shadow_index as usize];
                    let matrix = request.mx_proj * request.mx_view;
                    matrix.into()
                } else {
                    [[0.0; 4]; 4]
                };
                lights.push(LightParam {
                    projection,
                    pos: p.into(),
                    dir: d.extend(0.0).into(),
                    focus: [0.0, 0.0, 0.0, 0.0],
                    color: {
                        let rgb = color::to_linear_rgb(light.color);
                        [rgb[0], rgb[1], rgb[2], 0.0]
                    },
                    color_back: {
                        let rgb = color::to_linear_rgb(color_back);
                        [rgb[0], rgb[1], rgb[2], 0.0]
                    },
                    intensity,
                    shadow_params: [shadow_index, 0, 0, 0],
                });
            }
        }

        // render shadow maps
        for request in &shadow_requests {
            self.encoder.clear_depth(&request.target, 1.0);
            let mx_vp = request.mx_proj * request.mx_view;
            self.encoder.update_constant_buffer(
                &self.const_buf,
                &Globals {
                    mx_vp: mx_vp.into(),
                    mx_view: request.mx_view.into(),
                    mx_inv_proj: request.mx_proj.into(),
                    num_lights: 0,
                },
            );
            for node in hub.nodes.iter() {
                if !node.visible || node.scene_id != scene_id {
                    continue;
                }
                let gpu_data = match node.sub_node {
                    SubNode::Visual(_, ref data, _) => data,
                    _ => continue,
                };
                self.encoder.update_constant_buffer(
                    &gpu_data.constants,
                    &Locals {
                        mx_world: Matrix4::from(node.world_transform).into(),
                        color: [0.0; 4],
                        mat_params: [0.0; 4],
                        uv_range: [0.0; 4],
                    },
                );
                //TODO: avoid excessive cloning
                let data = shadow_pipe::Data {
                    vbuf: gpu_data.vertices.clone(),
                    cb_locals: gpu_data.constants.clone(),
                    cb_globals: self.const_buf.clone(),
                    target: request.target.clone(),
                };
                self.encoder.draw(&gpu_data.slice, &self.pso.shadow, &data);
            }
        }

        // prepare target and globals
        let (mx_inv_proj, mx_view, mx_vp) = {
            let p: [[f32; 4]; 4] = camera.matrix(self.aspect_ratio()).into();
            let node = &hub.nodes[&camera.object.node];
            let w = match node.scene_id {
                Some(id) if Some(id) == scene_id => node.world_transform,
                Some(_) => panic!("Camera does not belong to this scene"),
                None => node.transform,
            };
            let mx_view = Matrix4::from(w.inverse_transform().unwrap());
            let mx_vp = Matrix4::from(p) * mx_view;
            (Matrix4::from(p).invert().unwrap(), mx_view, mx_vp)
        };

        self.encoder.update_constant_buffer(
            &self.const_buf,
            &Globals {
                mx_vp: mx_vp.into(),
                mx_view: mx_view.into(),
                mx_inv_proj: mx_inv_proj.into(),
                num_lights: lights.len() as u32,
            },
        );
        self.encoder
            .update_buffer(&self.light_buf, &lights, 0)
            .unwrap();

        self.encoder.clear_depth(&self.out_depth, 1.0);
        self.encoder.clear_stencil(&self.out_depth, 0);

        if let Background::Color(color) = scene.background {
            let rgb = color::to_linear_rgb(color);
            self.encoder
                .clear(&self.out_color, [rgb[0], rgb[1], rgb[2], 0.0]);
        }

        // render everything
        let (shadow_default, shadow_sampler) = self.shadow_default.to_param();
        let shadow0 = match shadow_requests.get(0) {
            Some(ref request) => request.resource.clone(),
            None => shadow_default.clone(),
        };
        let shadow1 = match shadow_requests.get(1) {
            Some(ref request) => request.resource.clone(),
            None => shadow_default.clone(),
        };
        for node in hub.nodes.iter() {
            if !node.visible || node.scene_id != scene_id {
                continue;
            }
            let (material, gpu_data, skeleton) = match node.sub_node {
                SubNode::Visual(ref mat, ref data, ref skeleton) => (mat, data, skeleton),
                _ => continue,
            };

            let joint_buffer_view = if let &Some(ref object) = skeleton {
                let data = match hub.get(object).sub_node {
                    hub::SubNode::Skeleton(ref data) => data,
                    _ => unreachable!(),
                };
                data.gpu_buffer_view.clone()
            } else {
                self.default_joint_buffer_view.clone()
            };

            //TODO: batch per PSO
            match *material {
                Material::Pbr(ref params) => {
                    self.encoder.update_constant_buffer(
                        &gpu_data.constants,
                        &Locals {
                            mx_world: Matrix4::from(node.world_transform).into(),
                            ..unsafe { mem::zeroed() }
                        },
                    );
                    let mut pbr_flags = PbrFlags::empty();
                    if params.base_color_map.is_some() {
                        pbr_flags.insert(BASE_COLOR_MAP);
                    }
                    if params.normal_map.is_some() {
                        pbr_flags.insert(NORMAL_MAP);
                    }
                    if params.metallic_roughness_map.is_some() {
                        pbr_flags.insert(METALLIC_ROUGHNESS_MAP);
                    }
                    if params.emissive_map.is_some() {
                        pbr_flags.insert(EMISSIVE_MAP);
                    }
                    if params.occlusion_map.is_some() {
                        pbr_flags.insert(OCCLUSION_MAP);
                    }
                    let bcf = color::to_linear_rgb(params.base_color_factor);
                    let emf = color::to_linear_rgb(params.emissive_factor);
                    self.encoder.update_constant_buffer(
                        &self.pbr_buf,
                        &PbrParams {
                            base_color_factor: [bcf[0], bcf[1], bcf[2], params.base_color_alpha],
                            camera: [0.0, 0.0, 1.0],
                            emissive_factor: [emf[0], emf[1], emf[2]],
                            metallic_roughness: [params.metallic_factor, params.roughness_factor],
                            normal_scale: params.normal_scale,
                            occlusion_strength: params.occlusion_strength,
                            pbr_flags: pbr_flags.bits(),
                            _padding0: unsafe { mem::uninitialized() },
                            _padding1: unsafe { mem::uninitialized() },
                        },
                    );
                    self.encoder.update_buffer(
                        &self.displacement_contributions_buf,
                        &gpu_data.displacement_contributions,
                        0,
                    ).expect("update displacement contributons buffer");
                    let data = pbr_pipe::Data {
                        vbuf: gpu_data.vertices.clone(),
                        locals: gpu_data.constants.clone(),
                        globals: self.const_buf.clone(),
                        lights: self.light_buf.clone(),
                        params: self.pbr_buf.clone(),
                        base_color_map: {
                            params
                                .base_color_map
                                .as_ref()
                                .unwrap_or(&self.map_default)
                                .to_param()
                        },
                        normal_map: {
                            params
                                .normal_map
                                .as_ref()
                                .unwrap_or(&self.map_default)
                                .to_param()
                        },
                        emissive_map: {
                            params
                                .emissive_map
                                .as_ref()
                                .unwrap_or(&self.map_default)
                                .to_param()
                        },
                        metallic_roughness_map: {
                            params
                                .metallic_roughness_map
                                .as_ref()
                                .unwrap_or(&self.map_default)
                                .to_param()
                        },
                        occlusion_map: {
                            params
                                .occlusion_map
                                .as_ref()
                                .unwrap_or(&self.map_default)
                                .to_param()
                        },
                        displacement_contributions: self.displacement_contributions_buf.clone(),
                        joint_transforms: joint_buffer_view,
                        color_target: self.out_color.clone(),
                        depth_target: self.out_depth.clone(),
                    };
                    self.encoder.draw(&gpu_data.slice, &self.pso.pbr, &data);
                }
                ref other => {
                    let (pso, color, param0, map) = match *other {
                        Material::Pbr(_) => unreachable!(),
                        Material::Basic(ref params) => (
                            &self.pso.mesh_basic_fill,
                            params.color,
                            0.0,
                            params.map.as_ref(),
                        ),
                        Material::CustomBasic(ref params) => (&params.pipeline, params.color, 0.0, params.map.as_ref()),
                        Material::Lambert(ref params) => (
                            &self.pso.mesh_gouraud,
                            params.color,
                            if params.flat { 0.0 } else { 1.0 },
                            None,
                        ),
                        Material::Line(ref params) => (&self.pso.line_basic, params.color, 0.0, None),
                        Material::Phong(ref params) => (&self.pso.mesh_phong, params.color, params.glossiness, None),
                        Material::Sprite(ref params) => (&self.pso.sprite, !0, 0.0, Some(&params.map)),
                        Material::Wireframe(ref params) => (&self.pso.mesh_basic_wireframe, params.color, 0.0, None),
                    };
                    let uv_range = match map {
                        Some(ref map) => map.uv_range(),
                        None => [0.0; 4],
                    };
                    self.encoder.update_constant_buffer(
                        &gpu_data.constants,
                        &Locals {
                            mx_world: Matrix4::from(node.world_transform).into(),
                            color: {
                                let rgb = color::to_linear_rgb(color);
                                [rgb[0], rgb[1], rgb[2], 0.0]
                            },
                            mat_params: [param0, 0.0, 0.0, 0.0],
                            uv_range,
                        },
                    );
                    //TODO: avoid excessive cloning
                    let data = basic_pipe::Data {
                        vbuf: gpu_data.vertices.clone(),
                        cb_locals: gpu_data.constants.clone(),
                        cb_lights: self.light_buf.clone(),
                        cb_globals: self.const_buf.clone(),
                        tex_map: map.unwrap_or(&self.map_default).to_param(),
                        shadow_map0: (shadow0.clone(), shadow_sampler.clone()),
                        shadow_map1: (shadow1.clone(), shadow_sampler.clone()),
                        out_color: self.out_color.clone(),
                        out_depth: (self.out_depth.clone(), (0, 0)),
                    };
                    self.encoder.draw(&gpu_data.slice, pso, &data);
                }
            };
        }

        let quad_slice = gfx::Slice {
            start: 0,
            end: 4,
            base_vertex: 0,
            instances: None,
            buffer: gfx::IndexBuffer::Auto,
        };

        // draw background (if any)
        match scene.background {
            Background::Texture(ref texture) => {
                // TODO: Reduce code duplication (see drawing debug quads)
                self.encoder.update_constant_buffer(
                    &self.quad_buf,
                    &QuadParams {
                        rect: [-1.0, -1.0, 1.0, 1.0],
                        depth: 1.0,
                    },
                );
                let data = quad_pipe::Data {
                    params: self.quad_buf.clone(),
                    globals: self.const_buf.clone(),
                    resource: texture.to_param().0.raw().clone(),
                    sampler: texture.to_param().1,
                    target: self.out_color.clone(),
                    depth_target: self.out_depth.clone(),
                };
                self.encoder.draw(&quad_slice, &self.pso.quad, &data);
            }
            Background::Skybox(ref cubemap) => {
                self.encoder.update_constant_buffer(
                    &self.quad_buf,
                    &QuadParams {
                        rect: [-1.0, -1.0, 1.0, 1.0],
                        depth: 1.0,
                    },
                );
                let data = quad_pipe::Data {
                    params: self.quad_buf.clone(),
                    resource: cubemap.to_param().0.raw().clone(),
                    sampler: cubemap.to_param().1,
                    globals: self.const_buf.clone(),
                    target: self.out_color.clone(),
                    depth_target: self.out_depth.clone(),
                };
                self.encoder.draw(&quad_slice, &self.pso.skybox, &data);
            }
            Background::Color(_) => {}
        }

        // draw ui text
        for node in hub.nodes.iter() {
            if let SubNode::UiText(ref text) = node.sub_node {
                text.font.queue(&text.section);
                if !self.font_cache.contains_key(&text.font.path) {
                    self.font_cache
                        .insert(text.font.path.clone(), text.font.clone());
                }
            }
        }
        for (_, font) in &self.font_cache {
            font.draw(&mut self.encoder, &self.out_color, &self.out_depth);
        }

        // draw debug quads
        self.debug_quads.sync_pending();
        for quad in self.debug_quads.iter() {
            let pos = [
                if quad.pos[0] >= 0 {
                    quad.pos[0]
                } else {
                    self.size.0 as i32 + quad.pos[0] - quad.size[0]
                },
                if quad.pos[1] >= 0 {
                    quad.pos[1]
                } else {
                    self.size.1 as i32 + quad.pos[1] - quad.size[1]
                },
            ];
            let p0 = self.map_to_ndc([pos[0] as f32, pos[1] as f32]);
            let p1 = self.map_to_ndc([
                (pos[0] + quad.size[0]) as f32,
                (pos[1] + quad.size[1]) as f32,
            ]);
            self.encoder.update_constant_buffer(
                &self.quad_buf,
                &QuadParams {
                    rect: [p0.x, p0.y, p1.x, p1.y],
                    depth: -1.0,
                },
            );
            let data = quad_pipe::Data {
                params: self.quad_buf.clone(),
                globals: self.const_buf.clone(),
                resource: quad.resource.clone(),
                sampler: self.map_default.to_param().1,
                target: self.out_color.clone(),
                depth_target: self.out_depth.clone(),
            };
            self.encoder.draw(&quad_slice, &self.pso.quad, &data);
        }

        self.encoder.flush(&mut self.device);
    }

    /// Draw [`ShadowMap`](struct.ShadowMap.html) for debug purposes.
    pub fn debug_shadow_quad(
        &mut self,
        map: &ShadowMap,
        _num_components: u8,
        pos: [i16; 2],
        size: [u16; 2],
    ) -> DebugQuadHandle {
        DebugQuadHandle(self.debug_quads.create(DebugQuad {
            resource: map.to_resource().raw().clone(),
            pos: [pos[0] as i32, pos[1] as i32],
            size: [size[0] as i32, size[1] as i32],
        }))
    }
}
*/
