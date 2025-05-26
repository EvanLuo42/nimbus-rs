use std::sync::Arc;
use vulkano::command_buffer::{CommandBufferExecFuture, PrimaryAutoCommandBuffer};
use vulkano::swapchain::{acquire_next_image, SwapchainAcquireFuture, SwapchainPresentInfo};
use vulkano::sync::GpuFuture;
use crate::core::errors::NimbusError;
use crate::rendering::context::RenderContext;

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
    
    pub fn end_frame(&self, ctx: &RenderContext, frame: FrameContext, commands: Vec<Arc<PrimaryAutoCommandBuffer>>) -> Result<(), NimbusError> {
        let mut future = frame.future.boxed();

        for command in commands {
            future = future
                .then_execute(ctx.graphics_queue.clone(), command.clone())?
                .boxed();
        }
        
        future
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
