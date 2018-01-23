use hub::{Hub, Operation, SubNode};
use object::Base;

/// Groups are used to combine several other objects or groups to work with
/// them as with a single entity.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Group {
    object: Base,
}
three_object!(Group::object);

impl Group {
    pub(crate) fn new(hub: &mut Hub) -> Self {
        let sub = SubNode::Group { first_child: None };
        let object = hub.spawn(sub);
        Group {
            object,
        }
    }

    /// Add new [`Base`](struct.Base.html) to the group.
    pub fn add<P>(
        &self,
        child: P,
    ) where
        P: AsRef<Base>,
    {
        let msg = Operation::AddChild(child.as_ref().node.clone());
        let _ = self.object.tx.send((self.object.node.downgrade(), msg));
    }

    /// Removes a child [`Base`](struct.Base.html) from the group.
    pub fn remove<P>(
        &self,
        child: P,
    ) where
        P: AsRef<Base>,
    {
        let msg = Operation::RemoveChild(child.as_ref().node.clone());
        let _ = self.object.tx.send((self.object.node.downgrade(), msg));
    }
}
