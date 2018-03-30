use gpu;
use object;

use geometry::Geometry;
use hub::Operation;
use material::Material;
use render::Vertex;
use skeleton::Skeleton;

use std::hash::{Hash, Hasher};

/// The maximum number of [`Target`]s able to influence a [`Mesh`].
///
/// [`Target`]: enum.Target.html
/// [`Mesh`]: struct.Mesh.html
pub const MAX_TARGETS: usize = 8;

/// Defines a target of displacement.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Target {
    /// Target the position attribute.
    Position,

    /// Target the normal attribute,
    Normal,

    /// Target the tangent attribute.
    Tangent,

    /// Leave attribute unchanged.
    None,
}

impl Default for Target {
    fn default() -> Self {
        Target::None
    }
}

/// [`Geometry`](struct.Geometry.html) with some [`Material`](struct.Material.html).
///
/// # Examples
///
/// Creating a solid red triangle.
///
/// ```rust,no_run
/// # let mut win = three::Window::new("Example");
/// # let factory = &mut win.factory;
/// let vertices = vec![
///     [-0.5, -0.5, 0.0].into(),
///     [ 0.5, -0.5, 0.0].into(),
///     [ 0.5, -0.5, 0.0].into(),
/// ];
/// let geometry = three::Geometry::with_vertices(vertices);
/// let red_material = three::material::Basic { color: three::color::RED, map: None };
/// let mesh = factory.mesh(geometry, red_material);
/// # let _ = mesh;
/// ```
///
/// Duplicating a mesh.
///
/// ```rust,no_run
/// # let mut win = three::Window::new("Example");
/// # let factory = &mut win.factory;
/// # let vertices = vec![
/// #     [-0.5, -0.5, 0.0].into(),
/// #     [ 0.5, -0.5, 0.0].into(),
/// #     [ 0.5, -0.5, 0.0].into(),
/// # ];
/// # let geometry = three::Geometry::with_vertices(vertices);
/// # let red_material = three::material::Basic { color: three::color::RED, map: None };
/// # let mesh = factory.mesh(geometry, red_material);
/// use three::Object;
/// let mut duplicate = factory.mesh_instance(&mesh);
/// // Duplicated meshes share their geometry but may be transformed individually.
/// duplicate.set_position([1.2, 3.4, 5.6]);
/// ```
///
/// Duplicating a mesh with a different material.
///
/// ```rust,no_run
/// # let mut win = three::Window::new("Example");
/// # let factory = &mut win.factory;
/// # let vertices = vec![
/// #     [-0.5, -0.5, 0.0].into(),
/// #     [ 0.5, -0.5, 0.0].into(),
/// #     [ 0.5, -0.5, 0.0].into(),
/// # ];
/// # let geometry = three::Geometry::with_vertices(vertices);
/// # let red_material = three::material::Basic { color: three::color::RED, map: None };
/// # let mesh = factory.mesh(geometry, red_material);
/// let yellow_material = three::material::Wireframe { color: three::color::YELLOW };
/// # use three::Object;
/// let mut duplicate = factory.mesh_instance_with_material(&mesh, yellow_material);
/// duplicate.set_position([1.2, 3.4, 5.6]);
/// ```
///
/// # Notes
///
/// * Meshes are removed from the scene when dropped.
/// * Hence, meshes must be kept in scope in order to be displayed.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Mesh {
    pub(crate) object: object::Base,
}
three_object!(Mesh::object);

/// A dynamic version of a mesh allows changing the geometry on CPU side
/// in order to animate the mesh.
#[derive(Clone, Debug)]
pub struct Dynamic {
    pub(crate) object: object::Base,
    pub(crate) vbuf: gpu::Buffer,
    pub(crate) geometry: Geometry,
    pub(crate) vertices: Vec<Vertex>,
}
three_object!(Dynamic::object);

impl PartialEq for Dynamic {
    fn eq(
        &self,
        other: &Dynamic,
    ) -> bool {
        self.object == other.object
    }
}

impl Eq for Dynamic {}

impl Hash for Dynamic {
    fn hash<H: Hasher>(
        &self,
        state: &mut H,
    ) {
        self.object.hash(state);
    }
}

impl Mesh {
    /// Set mesh material.
    pub fn set_material(
        &mut self,
        material: Material,
    ) {
        let msg = Operation::SetMaterial(material);
        let _ = self.object.tx.send((self.object.node.downgrade(), msg));
    }

    /// Bind a skeleton to the mesh.
    pub fn set_skeleton(
        &mut self,
        skeleton: Skeleton,
    ) {
        let msg = Operation::SetSkeleton(skeleton);
        let _ = self.object.tx.send((self.object.node.downgrade(), msg));
    }

    /// Set the morph target weights of a mesh.
    pub fn set_weights(
        &mut self,
        weights: [f32; MAX_TARGETS],
    ) {
        let msg = Operation::SetWeights(weights);
        let _ = self.object.tx.send((self.object.node.downgrade(), msg));
    }
}

impl Dynamic {
    /// Returns the number of vertices of the geometry base shape.
    pub fn vertex_count(&self) -> usize {
        self.geometry.vertices.len()
    }

    /// Set mesh material.
    pub fn set_material(
        &mut self,
        material: Material,
    ) {
        let msg = Operation::SetMaterial(material);
        let _ = self.object.tx.send((self.object.node.downgrade(), msg));
    }
}
