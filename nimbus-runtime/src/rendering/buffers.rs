use glam::{Mat4, Vec2, Vec3, Vec4};
use vulkano::buffer::BufferContents;
use vulkano::pipeline::graphics::vertex_input::Vertex;

#[repr(C)]
#[derive(Copy, Clone, Default, Debug, BufferContents)]
pub struct CameraUbo {
    pub view: Mat4,
    pub projection: Mat4,
    pub camera_position: Vec4
}

#[repr(C)]
#[derive(Copy, Clone, Default, Debug, BufferContents, Vertex)]
pub struct VertexBuffer {
    #[format(R32G32B32_SFLOAT)]
    pub position: Vec3,
    #[format(R32G32B32_SFLOAT)]
    pub normal: Vec3,
    #[format(R32G32_SFLOAT)]
    pub uv: Vec2
}
