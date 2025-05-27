use crate::core::errors::NimbusError;
use crate::rendering::context::RenderContext;
use std::sync::Arc;
use vulkano::command_buffer::PrimaryAutoCommandBuffer;
use vulkano::swapchain::{acquire_next_image, SwapchainAcquireFuture, SwapchainPresentInfo};
use vulkano::sync::GpuFuture;

pub struct FrameManager;

impl FrameManager {
    pub fn begin_frame(&self, ctx: &RenderContext) -> Result<FrameContext, NimbusError> {
        let (image_index, _suboptimal, acquire_future) =
            acquire_next_image(ctx.swapchain.clone(), None)?;
        Ok(
            FrameContext {
                image_index,
                future: acquire_future,
            }
        )
    }
    
    pub fn end_frame(&self, ctx: &RenderContext, frame: FrameContext, commands: Arc<PrimaryAutoCommandBuffer>) -> Result<(), NimbusError> {
        frame.future
            .then_execute(ctx.graphics_queue.clone(), commands)?
            .then_swapchain_present(
                ctx.graphics_queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(
                    ctx.swapchain.clone(),
                    frame.image_index
                )
            )
            .then_signal_fence_and_flush()?
            .wait(None)?;
        
        Ok(())
    }
}

pub struct FrameContext {
    pub image_index: u32,
    pub future: SwapchainAcquireFuture,
}
