use crate::core::math::Mat4;
use crate::rendering::vertex::VertexStructure;
use std::sync::Arc;
use vulkano::buffer::{BufferContents, Subbuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer};
use vulkano::descriptor_set::DescriptorSet;
use vulkano::pipeline::graphics::GraphicsPipeline;
use vulkano::pipeline::Pipeline;
use vulkano::pipeline::PipelineBindPoint::Graphics;
use crate::core::errors::NimbusError;

pub struct Drawable {
    pub pipeline: Arc<GraphicsPipeline>,
    pub vertex_buffer: Subbuffer<[VertexStructure]>,
    pub index_buffer: Subbuffer<[u32]>,
    pub index_count: u32,
    pub descriptor_set: Arc<DescriptorSet>,
    pub model_ubo: Subbuffer<ModelUniform>
}

impl Drawable {
    pub fn draw(
        &self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>
    ) -> Result<(), NimbusError> {
        unsafe {
            builder
                .bind_pipeline_graphics(self.pipeline.clone())?
                .bind_vertex_buffers(0, self.vertex_buffer.clone())?
                .bind_index_buffer(self.index_buffer.clone())?
                .bind_descriptor_sets(
                    Graphics,
                    self.pipeline.layout().clone(),
                    0,
                    self.descriptor_set.clone()
                )?
                .draw_indexed(self.index_count, 1, 0, 0, 0)?;
        }
        Ok(())
    }
}

#[repr(C)]
#[derive(Copy, Clone, BufferContents)]
pub struct ModelUniform {
    pub model: Mat4
}
