use crate::rendering::mesh::Mesh;
use glam::Mat4;
use std::sync::Arc;

pub struct SceneObject {
    pub mesh: Arc<Mesh>,
    pub model: Mat4
}
