use object;

use euler::Vec3;
use input::{Button, Input, MOUSE_LEFT};
use node::TransformInternal;
use object::Object;

/// Simple controls for Orbital Camera.
///
/// Camera is rotating around the fixed point without any restrictions.
/// By default, it uses left mouse button as control button (hold it to rotate) and mouse wheel
/// to adjust distance to the central point.
#[derive(Clone, Debug)]
pub struct Orbit {
    object: object::Base,
    transform: TransformInternal,
    target: Vec3,
    button: Button,
    speed: f32,
}

/// Helper struct to construct [`Orbit`](struct.Orbit.html) with desired settings.
#[derive(Clone, Debug)]
pub struct Builder {
    object: object::Base,
    position: Vec3,
    target: Vec3,
    button: Button,
    speed: f32,
}

impl Builder {
    /// Create new `Builder` with default values.
    pub fn new<T: Object>(object: &T) -> Self {
        Builder {
            object: object.upcast(),
            position: [0.0, 0.0, 0.0].into(),
            target: [0.0, 0.0, 0.0].into(),
            button: MOUSE_LEFT,
            speed: 1.0,
        }
    }

    /// Set the initial position.
    ///
    /// Defaults to the world origin.
    pub fn position(
        &mut self,
        position: Vec3,
    ) -> &mut Self {
        self.position = position.into();
        self
    }

    /// Set the target position.
    ///
    /// Defaults to the world origin.
    pub fn target(
        &mut self,
        target: Vec3,
    ) -> &mut Self {
        self.target = target.into();
        self
    }

    /// Setup the speed of the movements. Default value is 1.0
    pub fn speed(
        &mut self,
        speed: f32,
    ) -> &mut Self {
        self.speed = speed;
        self
    }

    /// Setup control button. Default is left mouse button (`MOUSE_LEFT`).
    pub fn button(
        &mut self,
        button: Button,
    ) -> &mut Self {
        self.button = button;
        self
    }

    /// Finalize builder and create new `OrbitControls`.
    pub fn build(&mut self) -> Orbit {
        unimplemented!()
        /*
        let dir = (self.position - self.target).normalize();
        let up = vec3!(0, 1, 0);
        let q = Quat::look_at(dir, up).inverse();
        let object = self.object.clone();
        object.set_transform(self.position, q, 1.0);

        Orbit {
            object,
            transform: TransformInternal {
                disp: self.position,
                rot: q,
                scale: 1.0,
            },
            target: self.target.into(),
            button: self.button,
            speed: self.speed,
        }
         */
    }
}

impl Orbit {
    /// Create new `Builder` with default values.
    pub fn builder<T: Object>(object: &T) -> Builder {
        Builder::new(object)
    }

    /// Update current position and rotation of the controlled object according to the last frame input.
    pub fn update(
        &mut self,
        _input: &Input,
    ) {
        /*
        if !input.hit(self.button) && input.mouse_wheel().abs() < 1e-6 {
            return;
        }

        if input.mouse_movements().len() > 0 {
            let mouse_delta = input.mouse_delta_ndc();
            let pre = TransformInternal {
                disp: -1.0 * self.target,
                .. TransformInternal::one()
            };
            let q_ver = Quat::axis_angle(
                vec3!(0, 1, 0),
                self.speed * mouse_delta.x,
            );
            let axis = self.transform.rot.rotate(vec3!(1, 0, 0));
            let q_hor = Quat::axis_angle(axis, self.speed * mouse_delta.y);
            let post = TransformInternal {
                scale: 1.0 + input.mouse_wheel() / 1000.0,
                rot: q_hor * q_ver,
                disp: self.target,
            };
            self.transform = post.concat(&pre.concat(&self.transform));
            self.object.set_transform(self.transform.disp, self.transform.rot, 1.0);
        }
         */
        unimplemented!()
    }
}
