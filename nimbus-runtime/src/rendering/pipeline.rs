use crate::core::errors::NimbusResult;
use crate::rendering::context::RenderContext;
use crate::rendering::frame::FrameContext;
use std::sync::Arc;
use vulkano::command_buffer::PrimaryAutoCommandBuffer;
use vulkano::pipeline::GraphicsPipeline;

pub trait RenderPipeline {
    fn record_commands(&self, ctx: &RenderContext, frame: &FrameContext) -> NimbusResult<Arc<PrimaryAutoCommandBuffer>>;
    
    fn get_graphics_pipeline(&self) -> Arc<GraphicsPipeline>;
}
