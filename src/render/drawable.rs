use cgmath::Matrix4;
use crate::render::material::Material;
use crate::render::mesh::Mesh;

#[derive(Clone)]
pub struct Drawable {
    pub mesh: Mesh,
    pub material: Material,
    pub model_matrix: Matrix4<f32>,
    pub drawable_type: DrawableType
}

#[derive(Copy, Clone)]
pub enum DrawableType {
    Opaque,
    Transparent
}
