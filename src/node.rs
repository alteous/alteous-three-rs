use cgmath;
use froggy;
use mint;

use hub::SubNode;

/// Pointer to a Node
pub(crate) type NodePointer = froggy::Pointer<NodeInternal>;
pub(crate) type TransformInternal = cgmath::Decomposed<cgmath::Vector3<f32>, cgmath::Quaternion<f32>>;

// Fat node of the scene graph.
//
// `NodeInternal` is used by `three-rs` internally,
// client code uses [`object::Base`](struct.Base.html) instead.
#[derive(Debug)]
pub(crate) struct NodeInternal {
    /// `true` if this node (and its subnodes) are visible to cameras.
    pub(crate) visible: bool,
    /// For internal use.
    pub(crate) world_visible: bool,
    /// The transform relative to the node's parent.
    pub(crate) transform: TransformInternal,
    /// The transform relative to the world origin.
    pub(crate) world_transform: TransformInternal,
    /// Context specific-data, for example, `UiText`, `Visual` or `Light`.
    pub(crate) sub_node: SubNode,    
    /// Pointer to the next sibling.
    pub(crate) next_sibling: Option<NodePointer>,
}

/*
impl NodeInternal {
    pub(crate) fn to_node(&self) -> Node {
        Node {
            visible: self.visible,
            world_visible: self.world_visible,
            transform: self.transform.into(),
            world_transform: self.world_transform.into(),
        }
    }
}
*/

/// Position, rotation, and scale of the scene `Node`.
#[derive(Clone, Debug, PartialEq)]
pub struct Transform {
    /// Position.
    pub position: mint::Point3<f32>,
    /// Orientation.
    pub orientation: mint::Quaternion<f32>,
    /// Scale.
    pub scale: f32,
}

impl From<TransformInternal> for Transform {
    fn from(tf: TransformInternal) -> Self {
        let pos: mint::Vector3<f32> = tf.disp.into();
        Transform {
            position: pos.into(),
            orientation: tf.rot.into(),
            scale: tf.scale,
        }
    }
}

/// General information about scene `Node`.
#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    /// Relative to parent transform.
    pub transform: Transform,
    /// World transform (relative to the world's origin).
    pub world_transform: Transform,
    /// Is `Node` visible by cameras or not?
    pub visible: bool,
    /// The same as `visible`, used internally.
    pub world_visible: bool,
}

impl From<SubNode> for NodeInternal {
    fn from(sub: SubNode) -> Self {
        NodeInternal {
            visible: true,
            world_visible: false,
            transform: cgmath::Transform::one(),
            world_transform: cgmath::Transform::one(),
            sub_node: sub,
            next_sibling: None,
        }
    }
}
