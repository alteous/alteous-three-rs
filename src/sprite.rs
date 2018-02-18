use hub::Operation;
use object;

/// Two-dimensional bitmap that is integrated into a larger scene.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Sprite {
    pub(crate) object: object::Base,
}
three_object!(Sprite::object);

impl Sprite {
    pub(crate) fn new(object: object::Base) -> Self {
        Sprite { object }
    }

    /// Set area of the texture to render.
    /// It can be used in sequential animations.
    pub fn set_texel_range(&self, base: [i16; 2], size: [u16; 2]) {
        self.object.send(Operation::SetTexelRange(base, size));
    }
}
