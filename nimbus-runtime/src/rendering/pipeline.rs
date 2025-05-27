use crate::core::errors::NimbusResult;
use crate::rendering::context::RenderContext;
use crate::rendering::frame::FrameContext;
use std::sync::Arc;
use vulkano::command_buffer::PrimaryAutoCommandBuffer;

pub trait RenderPipeline {
    fn record_commands(&self, ctx: &RenderContext, frame: &FrameContext) -> NimbusResult<Arc<PrimaryAutoCommandBuffer>>;
}
