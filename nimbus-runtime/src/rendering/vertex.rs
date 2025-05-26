use crate::core::math::{Vec2, Vec3};
use vulkano::buffer::BufferContents;
use vulkano::pipeline::graphics::vertex_input::Vertex;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, BufferContents, Vertex)]
pub struct VertexStructure {
    #[format(R32G32B32_SFLOAT)]
    pub position: Vec3,
    #[format(R32G32B32_SFLOAT)]
    pub normal: Vec3,
    #[format(R32G32_SFLOAT)]
    pub texcoord: Vec2
}
