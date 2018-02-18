//! Items in the scene heirarchy.

use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::mpsc;

use euler::{Quat, Vec3};
use hub::{Message, Operation};
use mesh::MAX_TARGETS;
use node::NodePointer;

//Note: no local state should be here, only remote links
/// `Base` represents a concrete entity that can be added to the scene.
///
/// One cannot construct `Base` directly. Wrapper types such as [`Camera`],
/// [`Mesh`], and [`Group`] are provided instead.
///
/// Any type that implements [`Object`] may be converted to its concrete
/// `Base` type with the method [`Object::upcast`]. This is useful for
/// storing generic objects in containers.
///
/// [`Camera`]: ../camera/struct.Camera.html
/// [`Mesh`]: ../mesh/struct.Mesh.html
/// [`Group`]: ../object/struct.Group.html
/// [`Object`]: ../object/trait.Object.html
/// [`Object::upcast`]: ../object/trait.Object.html#method.upcast
#[derive(Clone)]
pub struct Base {
    pub(crate) node: NodePointer,
    pub(crate) tx: mpsc::Sender<Message>,
}

/// Marks data structures that are able to added to the scene graph.
pub trait Object: AsRef<Base> {
    /// Converts into the base type.
    fn upcast(&self) -> Base {
        self.as_ref().clone()
    }

    /// Invisible objects are not rendered by cameras.
    fn set_visible(
        &self,
        visible: bool,
    ) {
        self.as_ref().set_visible(visible)
    }

    /// Rotates object in the specific direction of `target`.
    fn look_at(
        &self,
        eye: Vec3,
        target: Vec3,
        up: Option<Vec3>,
    ) {
        self.as_ref().look_at(eye, target, up)
    }

    /// Set both position, orientation and scale.
    fn set_transform(
        &self,
        pos: Vec3,
        rot: Quat,
        scale: f32,
    ) {
        self.as_ref().set_transform(pos, rot, scale)
    }

    /// Set position.
    fn set_position(
        &self,
        pos: Vec3,
    ) {
        self.as_ref().set_position(pos)
    }

    /// Set orientation.
    fn set_orientation(
        &self,
        rot: Quat,
    ) {
        self.as_ref().set_orientation(rot)
    }

    /// Set scale.
    fn set_scale(
        &self,
        scale: f32,
    ) {
        self.as_ref().set_scale(scale)
    }
}

impl PartialEq for Base {
    fn eq(
        &self,
        other: &Base,
    ) -> bool {
        self.node == other.node
    }
}

impl Eq for Base {}

impl Hash for Base {
    fn hash<H: Hasher>(
        &self,
        state: &mut H,
    ) {
        self.node.hash(state);
    }
}

impl fmt::Debug for Base {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        self.node.fmt(f)
    }
}

impl Base {
    /// Pass message to hub.
    pub(crate) fn send(&self, operation: Operation) {
        let _ = self.tx.send((self.node.downgrade(), operation));
    }

    /// Invisible objects are not rendered by cameras.
    pub fn set_visible(&self, visible: bool) {
        self.send(Operation::SetVisible(visible));
    }

    /// Rotates object in the specific direction of `target`.
    pub fn look_at(&self, eye: Vec3, target: Vec3, up: Option<Vec3>) {
        let dir = (eye - target).normalize();
        let z = vec3!(0, 0, 1);
        let up = match up {
            Some(v) => v.normalize(),
            None if dir.dot(z).abs() < 0.99 => z,
            None => vec3!(0, 1, 0),
        };
        let q = Quat::look_at(dir, up).inverse();
        self.set_transform(eye, q, 1.0);
    }

    /// Set both position, orientation and scale.
    pub fn set_transform(&self, pos: Vec3, rot: Quat, scale: f32) {
        self.send(Operation::SetTransform(Some(pos), Some(rot), Some(scale)));
    }

    /// Set position.
    pub fn set_position(&self, pos: Vec3) {
        self.send(Operation::SetTransform(Some(pos.into()), None, None));
    }

    /// Set orientation.
    pub fn set_orientation(&self, rot: Quat) {
        self.send(Operation::SetTransform(None, Some(rot.into()), None));
    }

    /// Set scale.
    pub fn set_scale(&self, scale: f32) {
        self.send(Operation::SetTransform(None, None, Some(scale)));
    }

    /// Set weights.
    pub fn set_weights(&self, weights: [f32; MAX_TARGETS]) {
        self.send(Operation::SetWeights(weights));
    }
}

impl AsRef<Base> for Base {
    fn as_ref(&self) -> &Base {
        self
    }
}

impl AsMut<Base> for Base {
    fn as_mut(&mut self) -> &mut Base {
        self
    }
}
