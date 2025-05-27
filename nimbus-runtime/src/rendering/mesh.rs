use crate::rendering::buffers::VertexBuffer;
use crate::rendering::material::Material;
use std::sync::Arc;
use vulkano::buffer::Subbuffer;

pub struct Mesh {
    pub vertex_buffer: Subbuffer<Vec<VertexBuffer>>,
    pub index_buffer: Subbuffer<[u32]>,
    pub submeshes: Vec<Arc<Submesh>>
}

pub struct Submesh {
    pub index_offset: u32,
    pub index_count: u32,
    pub material: Arc<Material>
}
