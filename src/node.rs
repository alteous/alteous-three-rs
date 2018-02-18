use euler;
use froggy;

use euler::{Mat4, Quat, Vec3};
use hub::SubNode;

/// Pointer to a Node
pub(crate) type NodePointer = froggy::Pointer<NodeInternal>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TransformInternal {
    pub disp: Vec3,
    pub rot: Quat,
    pub scale: f32,
}

impl TransformInternal {
    pub(crate) fn one() -> Self {
        Self {
            disp: vec3!(0, 0, 0),
            rot: Quat::identity(),
            scale: 1.0,
        }
    }

    pub(crate) fn concat(&self, other: Self) -> Self {
        Self {
            scale: self.scale * other.scale,
            rot: self.rot * other.rot,
            disp: self.disp + self.rot.rotate(other.disp * self.scale),
        }
    }

    pub(crate) fn inverse(&self) -> Self {
        let scale = 1.0 / self.scale;
        let rot = self.rot.inverse();
        let disp = -scale * rot.rotate(self.disp);
        Self { disp, rot, scale }
    }

    pub(crate) fn matrix(&self) -> Mat4 {
        euler::Trs {
            t: self.disp,
            r: self.rot,
            s: vec3!(self.scale),
        }.matrix()
    }
}

pub(crate) type Pointer = froggy::Pointer<NodeInternal>;

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

/// Position, rotation, and scale of the scene `Node`.
#[derive(Clone, Debug, PartialEq)]
pub struct Transform {
    /// Position.
    pub position: Vec3,
    /// Orientation.
    pub orientation: Quat,
    /// Scale.
    pub scale: f32,
}

impl Transform {
    /// Identity transform.
    pub fn identity() -> Self {
        Self {
            position: vec3!(0, 0, 0),
            orientation: Quat::identity(),
            scale: 1.0,
        }
    }
}

impl From<TransformInternal> for Transform {
    fn from(tf: TransformInternal) -> Self {
        Transform {
            position: tf.disp,
            orientation: tf.rot,
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
            transform: TransformInternal::one(),
            world_transform: TransformInternal::one(),
            sub_node: sub,
            next_sibling: None,
        }
    }
}
