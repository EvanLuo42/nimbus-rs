use wgpu::{Buffer, BufferAddress, VertexAttribute, VertexBufferLayout, VertexStepMode};

#[derive(Clone)]
pub struct Mesh {
    pub vertex_buffer: Buffer,
    pub index_buffer: Option<Buffer>,
    pub index_count: u32,
    pub vertex_count: u32,

    pub vertex_attributes: Vec<VertexAttribute>,
    pub array_stride: BufferAddress
}

impl Mesh {
    pub fn vertex_buffer_layout(&self) -> VertexBufferLayout {
        VertexBufferLayout {
            array_stride: self.array_stride,
            step_mode: VertexStepMode::Vertex,
            attributes: &self.vertex_attributes,
        }
    }
}

pub type RawMesh<'a> = gltf::Mesh<'a>;
