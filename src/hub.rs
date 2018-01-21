use audio::{AudioData, Operation as AudioOperation};
use color::Color;
//use light::{ShadowMap, ShadowProjection};
use material::{self, Material};
use mesh::MAX_TARGETS;
use node::{NodeInternal, NodePointer};
use object;
use render::DisplacementContribution;
use skeleton::{Bone, Skeleton};
// use text::{Operation as TextOperation, TextData};

use cgmath::Transform;
use froggy;
use gpu;
use mint;

use std::ops;
use std::sync::{Arc, Mutex};
use std::sync::{atomic, mpsc};

//TODO: private fields?
#[derive(Clone, Debug)]
pub(crate) struct GpuData {
    pub range: ops::Range<usize>,
    pub vertex_array: gpu::VertexArray,
    pub pending: Option<DynamicData>,
    pub displacement_contributions: [DisplacementContribution; MAX_TARGETS],
}

#[derive(Clone, Debug)]
pub(crate) struct DynamicData {
    pub num_vertices: usize,
    pub buffer: gpu::Buffer,
}

#[derive(Clone, Debug)]
pub(crate) enum SubLight {
    Ambient,
    Directional,
    Hemisphere { ground: Color },
    Point,
}

/*
#[derive(Clone, Debug)]
pub(crate) struct LightData {
    pub color: Color,
    pub intensity: f32,
    pub sub_light: SubLight,
    pub shadow: Option<(ShadowMap, ShadowProjection)>,
}
*/

#[derive(Clone, Debug)]
pub(crate) struct SkeletonData {
    pub bones: Vec<Bone>,
    pub inverse_bind_matrices: Vec<mint::ColumnMatrix4<f32>>,
    pub gpu_buffer: gpu::Buffer,
    pub cpu_buffer: Vec<[f32; 4]>,
}

#[derive(Clone, Debug)]
pub(crate) struct VisualData {
    pub material: Material,
    pub skeleton: Option<Skeleton>,

    // Draw parameters
    pub program: gpu::Program,
    pub range: ops::Range<usize>,
    pub mode: gpu::Mode,
    pub vertex_array: gpu::VertexArray,
}

#[derive(Debug)]
pub(crate) enum SubNode {
    /// No extra data, such as in the case of `Group`.
    Empty,

    /// Marks the root object of a `Scene`.
    Scene,

    /// Audio data.
    Audio(AudioData),

    // Renderable text for 2D user interface.
    //UiText(TextData),

    /// Renderable 3D content, such as a mesh.
    Visual(VisualData),

    // Lighting information for illumination and shadow casting.
    //Light(LightData),

    /// Array of `Bone` instances that may be bound to a `Skinned` mesh.
    Skeleton(SkeletonData),
}

pub(crate) type Message = (froggy::WeakPointer<NodeInternal>, Operation);
pub(crate) enum Operation {
    SetAudio(AudioOperation),
    SetParent(NodePointer),
    SetVisible(bool),
    // SetText(TextOperation),
    SetTransform(
        Option<mint::Point3<f32>>,
        Option<mint::Quaternion<f32>>,
        Option<f32>,
    ),
    SetMaterial(Material),
    SetSkeleton(Skeleton),
    // SetShadow(ShadowMap, ShadowProjection),
    SetTexelRange(mint::Point2<i16>, mint::Vector2<u16>),
    SetWeights([f32; MAX_TARGETS]),
}

pub(crate) type Pointer = Arc<Mutex<Hub>>;

pub(crate) struct Hub {
    pub(crate) nodes: froggy::Storage<NodeInternal>,
    pub(crate) message_tx: mpsc::Sender<Message>,
    message_rx: mpsc::Receiver<Message>,
}

impl Hub {
    pub(crate) fn new() -> Pointer {
        let (tx, rx) = mpsc::channel();
        let hub = Hub {
            nodes: froggy::Storage::new(),
            message_tx: tx,
            message_rx: rx,
        };
        Arc::new(Mutex::new(hub))
    }

    pub(crate) fn get<T>(
        &self,
        object: T,
    ) -> &NodeInternal
    where
        T: AsRef<object::Base>,
    {
        let base: &object::Base = object.as_ref();
        &self.nodes[&base.node]
    }

    pub(crate) fn get_mut<T>(
        &mut self,
        object: T,
    ) -> &mut NodeInternal
    where
        T: AsRef<object::Base>,
    {
        let base: &object::Base = object.as_ref();
        &mut self.nodes[&base.node]
    }

    fn spawn(
        &mut self,
        sub: SubNode,
    ) -> object::Base {
        object::Base {
            node: self.nodes.create(sub.into()),
            tx: self.message_tx.clone(),
        }
    }

    pub(crate) fn spawn_empty(&mut self) -> object::Base {
        self.spawn(SubNode::Empty)
    }

    pub(crate) fn spawn_visual(
        &mut self,
        visual_data: VisualData,
    ) -> object::Base {
        self.spawn(SubNode::Visual(visual_data))
    }
/*
    pub(crate) fn spawn_light(
        &mut self,
        data: LightData,
    ) -> object::Base {
        self.spawn(SubNode::Light(data))
    }

    pub(crate) fn spawn_ui_text(
        &mut self,
        text: TextData,
    ) -> object::Base {
        self.spawn(SubNode::UiText(text))
    }
*/
    pub(crate) fn spawn_audio_source(
        &mut self,
        data: AudioData,
    ) -> object::Base {
        self.spawn(SubNode::Audio(data))
    }

    pub(crate) fn spawn_scene(&mut self) -> object::Base {
        static SCENE_UID_COUNTER: atomic::AtomicUsize = atomic::ATOMIC_USIZE_INIT;
        let uid = SCENE_UID_COUNTER.fetch_add(1, atomic::Ordering::Relaxed);
        let tx = self.message_tx.clone();
        let node = self.nodes.create(NodeInternal {
            scene_id: Some(uid),
            ..SubNode::Scene.into()
        });
        object::Base { node, tx }
    }

    pub(crate) fn spawn_skeleton(
        &mut self,
        data: SkeletonData,
    ) -> object::Base {
        self.spawn(SubNode::Skeleton(data))
    }

    pub(crate) fn process_messages(&mut self) {
        while let Ok((weak_ptr, operation)) = self.message_rx.try_recv() {
            let ptr = match weak_ptr.upgrade() {
                Ok(ptr) => ptr,
                Err(_) => continue,
            };
            match operation {
                Operation::SetAudio(operation) => {
                    if let SubNode::Audio(ref mut data) = self.nodes[&ptr].sub_node {
                        Hub::process_audio(operation, data);
                    }
                },
                Operation::SetParent(parent) => {
                    self.nodes[&ptr].parent = Some(parent);
                }
                Operation::SetVisible(visible) => {
                    self.nodes[&ptr].visible = visible;
                }
                Operation::SetTransform(pos, rot, scale) => {
                    if let Some(pos) = pos {
                        self.nodes[&ptr].transform.disp = mint::Vector3::from(pos).into();
                    }
                    if let Some(rot) = rot {
                        self.nodes[&ptr].transform.rot = rot.into();
                    }
                    if let Some(scale) = scale {
                        self.nodes[&ptr].transform.scale = scale;
                    }
                }
                Operation::SetMaterial(material) => {
                    if let SubNode::Visual(ref mut data) = self.nodes[&ptr].sub_node {
                        data.material = material;
                    }
                },
                Operation::SetTexelRange(base, size) => {
                    if let SubNode::Visual(ref mut data) = self.nodes[&ptr].sub_node {
                        match &mut data.material {
                           &mut  material::Material::Sprite(ref mut params) => params.map.set_texel_range(base, size),
                            _ => panic!("Unsupported material for texel range request"),
                        }
                    }
                },
                /*
                Operation::SetText(operation) => {
                    if let SubNode::UiText(ref mut data) = self.nodes[&ptr].sub_node {
                        Hub::process_text(operation, data);
                    }
                },
                 */
                Operation::SetSkeleton(skeleton) => {
                    if let SubNode::Visual(ref mut data) = self.nodes[&ptr].sub_node {
                        data.skeleton = Some(skeleton);
                    }
                },
                /*
                Operation::SetShadow(map, proj) => {
                    if let SubNode::Light(ref mut data) = self.nodes[&ptr].sub_node {
                        data.shadow = Some((map, proj));
                    }
                },

                Operation::SetWeights(weights) => {
                    fn set_weights(data: &mut VisualData, weights: [f32; MAX_TARGETS]) {
                        for i in 0 .. MAX_TARGETS {
                            data.displacement_contributions[i].weight = weights[i];
                        }
                    }

                    // Hack around borrow checker rules:
                    // if
                    {
                        if let SubNode::Visual(ref mut data) = self.nodes[&ptr].sub_node {
                            set_weights(data, weights);
                            continue;
                        }
                    }
                    // else
                    {
                        for item in self.nodes.iter_mut() {
                            let update = item.parent.as_ref() == Some(&ptr);
                            if update {
                                if let SubNode::Visual(ref mut data) = item.sub_node {
                                    set_weights(data, weights);
                                }
                            }
                        }
                    }
                }
                 */
                _ => unimplemented!(),
            }
        }
        self.nodes.sync_pending();
    }

    fn process_audio(
        operation: AudioOperation,
        data: &mut AudioData,
    ) {
        match operation {
            AudioOperation::Append(clip) => data.source.append(clip),
            AudioOperation::Pause => data.source.pause(),
            AudioOperation::Resume => data.source.resume(),
            AudioOperation::Stop => data.source.stop(),
            AudioOperation::SetVolume(volume) => data.source.set_volume(volume),
        }
    }
/*
    fn process_text(
        operation: TextOperation,
        data: &mut TextData,
    ) {
        use gfx_glyph::Scale;
        match operation {
            TextOperation::Color(color) => {
                let rgb = color::to_linear_rgb(color);
                data.section.text[0].color = [rgb[0], rgb[1], rgb[2], 0.0];
            }
            TextOperation::Font(font) => data.font = font,
            TextOperation::Layout(layout) => data.layout = layout,
            TextOperation::Opacity(opacity) => data.section.text[0].color[3] = opacity,
            TextOperation::Pos(point) => data.section.screen_position = (point.x, point.y),
            // TODO: somehow grab window::hdpi_factor and multiply size
            TextOperation::Scale(scale) => data.section.text[0].scale = Scale::uniform(scale),
            TextOperation::Size(size) => data.section.bounds = (size.x, size.y),
            TextOperation::Text(text) => data.section.text[0].text = text,
        }
    }
*/
    pub(crate) fn update_graph(&mut self) {
        let mut cursor = self.nodes.cursor();
        while let Some((left, mut item, _)) = cursor.next() {
            if !item.visible {
                item.world_visible = false;
                continue;
            }
            let (visibility, affilation, transform) = match item.parent {
                Some(ref parent_ptr) => match left.get(parent_ptr) {
                    Some(parent) => (
                        parent.world_visible,
                        parent.scene_id,
                        parent.world_transform.concat(&item.transform),
                    ),
                    None => {
                        error!("Parent node was created after the child, ignoring");
                        (false, item.scene_id, item.transform)
                    }
                },
                None => (true, item.scene_id, item.transform),
            };
            item.world_visible = visibility;
            item.scene_id = affilation;
            item.world_transform = transform;
        }
    }
/*
    pub(crate) fn update_mesh(
        &mut self,
        mesh: &DynamicMesh,
    ) {
        match self.get_mut(&mesh).sub_node {
            SubNode::Visual(ref mut data, _) => data.pending = Some(mesh.dynamic.clone()),
            _ => unreachable!(),
        }
    }
*/
}
