use crate::core::errors::NimbusError;
use crate::rendering::context::RenderContext;
use std::sync::Arc;
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassBeginInfo, SubpassContents, SubpassEndInfo};
use vulkano::format::Format;
use vulkano::image::view::ImageView;
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass};
use crate::rendering::frame::FrameContext;

pub trait RenderPipeline {
    fn record_commands(&self, ctx: &RenderContext, frame: &FrameContext) -> Result<Arc<PrimaryAutoCommandBuffer>, NimbusError>;
}

pub struct BasicPipeline {
    clear_pass: Arc<RenderPass>,
    framebuffers: Vec<Arc<Framebuffer>>
}

impl BasicPipeline {
    pub fn new(ctx: &RenderContext) -> Result<Self, NimbusError> {
        let swapchain_format = ctx.swapchain.image_format();
        let clear_pass = vulkano::single_pass_renderpass!(
            ctx.device.clone(),
            attachments: {
                color: {
                    format: swapchain_format,
                    samples: 1,
                    load_op: Clear,
                    store_op: Store
                },
            },
            pass: {
                color: [color],
                depth_stencil: {},
            }
        )?;
        
        let image_views: Vec<Arc<ImageView>> = ctx.images
            .iter()
            .map(|image| ImageView::new_default(image.clone()).unwrap())
            .collect();
        let framebuffers: Vec<Arc<Framebuffer>> = image_views
            .iter()
            .map(|view| {
                Framebuffer::new(
                    clear_pass.clone(),
                    FramebufferCreateInfo {
                        attachments: vec![view.clone()],
                        ..Default::default()
                    }
                ).unwrap()
            })
            .collect();
        
        Ok(
            Self {
                clear_pass,
                framebuffers
            }
        )
    }
}

impl RenderPipeline for BasicPipeline {
    fn record_commands(&self, ctx: &RenderContext, frame: &FrameContext) -> Result<Arc<PrimaryAutoCommandBuffer>, NimbusError> {
        let mut primary = AutoCommandBufferBuilder::primary(
            ctx.command_allocator.clone(),
            ctx.graphics_queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit
        )?;
        primary.begin_render_pass(
            RenderPassBeginInfo {
                clear_values: vec![Some([0.0, 0.0, 1.0, 1.0].into())],
                ..RenderPassBeginInfo::framebuffer(self.framebuffers[frame.image_index as usize].clone())
            },
            SubpassBeginInfo {
                contents: SubpassContents::Inline,
                ..Default::default()
            }
        )?.end_render_pass(SubpassEndInfo::default())?;
        primary.build().map_err(NimbusError::from)
    }
}

pub struct DeferredPipeline {

}
