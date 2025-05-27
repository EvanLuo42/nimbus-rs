use std::sync::Arc;
use glam::{Mat4, Vec4};
use vulkano::buffer::BufferContents;
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferInheritanceInfo, CommandBufferInheritanceRenderPassInfo, CommandBufferInheritanceRenderPassType, CommandBufferUsage, PrimaryAutoCommandBuffer, SecondaryAutoCommandBuffer};
use vulkano::format::Format;
use vulkano::image::{Image, ImageCreateInfo, ImageType, ImageUsage};
use vulkano::image::view::ImageView;
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter};
use vulkano::pipeline::GraphicsPipeline;
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass};
use crate::core::errors::{NimbusError, NimbusResult};
use crate::rendering::context::RenderContext;
use crate::rendering::frame::FrameContext;
use crate::rendering::pipeline::RenderPipeline;

pub struct DeferredPipeline {
    gbuffer_pass: Arc<RenderPass>,
    gbuffer_framebuffers: Vec<Arc<Framebuffer>>,
    gbuffer_graphics_pipeline: Arc<GraphicsPipeline>
}

impl DeferredPipeline {
    pub fn new(ctx: &RenderContext) -> NimbusResult<Self> {
        let swapchain_image_count = ctx.swapchain.image_count();
        let gbuffer_pass = vulkano::single_pass_renderpass!(
            ctx.device.clone(),
            attachments: {
                albedo: {
                    format: Format::R8G8B8A8_UNORM,
                    samples: 1,
                    load_op: Clear,
                    store_op: Store
                },
                normal: {
                    format: Format::R16G16B16A16_SFLOAT,
                    samples: 1,
                    load_op: Clear,
                    store_op: Store
                },
                material: {
                    format: Format::R8G8B8_UNORM,
                    samples: 1,
                    load_op: Clear,
                    store_op: Store
                },
                depth: {
                    format: Format::D32_SFLOAT,
                    samples: 1,
                    load_op: Clear,
                    store_op: Store
                },
            },
            pass: {
                color: [albedo, normal, material],
                depth_stencil: {depth},
            }
        )?;

        let mut gbuffer_framebuffers = Vec::new();
        for _ in 0..swapchain_image_count {
            let albedo_image = create_image(ctx, Format::R8G8B8A8_UNORM)?;
            let normal_image = create_image(ctx, Format::R16G16B16A16_SFLOAT)?;
            let material_image = create_image(ctx, Format::R8G8B8_UNORM)?;
            let depth_image = create_image(ctx, Format::D32_SFLOAT)?;

            let albedo_view = ImageView::new_default(albedo_image)?;
            let normal_view = ImageView::new_default(normal_image)?;
            let material_view = ImageView::new_default(material_image)?;
            let depth_view = ImageView::new_default(depth_image)?;

            let framebuffer = Framebuffer::new(
                gbuffer_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![
                        albedo_view,
                        normal_view,
                        material_view,
                        depth_view
                    ],
                    ..Default::default()
                }
            )?;
            gbuffer_framebuffers.push(framebuffer);
        }

        todo!("Construct graphics pipeline for gbuffer")
        // Ok(
        //     Self {
        //         gbuffer_pass,
        //         gbuffer_framebuffers
        //     }
        // )
    }

    pub fn gbuffer_subpass(&self, ctx: &RenderContext, frame: &FrameContext) -> NimbusResult<Arc<SecondaryAutoCommandBuffer>> {
        let secondary = AutoCommandBufferBuilder::secondary(
            ctx.command_allocator.clone(),
            ctx.graphics_queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
            CommandBufferInheritanceInfo {
                render_pass: Some(CommandBufferInheritanceRenderPassType::BeginRenderPass(CommandBufferInheritanceRenderPassInfo {
                    subpass: Subpass::from(self.gbuffer_pass.clone(), 0).unwrap(),
                    framebuffer: Some(self.gbuffer_framebuffers[frame.image_index as usize].clone()),
                }
                )),
                ..Default::default()
            }
        )?;
        
        secondary.build().map_err(NimbusError::from)
    }

    pub fn lighting_subpass(&self, ctx: &RenderContext) -> NimbusResult<Arc<SecondaryAutoCommandBuffer>> {
        todo!()
    }
}

fn create_image(ctx: &RenderContext, format: Format) -> NimbusResult<Arc<Image>> {
    let extent = ctx.swapchain.image_extent();
    Image::new(
        ctx.memory_allocator.clone(),
        ImageCreateInfo {
            image_type: ImageType::Dim2d,
            format,
            extent: [extent[0], extent[1], 1],
            usage: ImageUsage::TRANSFER_DST | ImageUsage::TRANSFER_SRC,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
            ..Default::default()
        }
    ).map_err(NimbusError::from)
}

impl RenderPipeline for DeferredPipeline {
    fn record_commands(&self, ctx: &RenderContext, frame: &FrameContext) -> NimbusResult<Arc<PrimaryAutoCommandBuffer>> {
        let gbuffer_commands = self.gbuffer_subpass(ctx, frame)?;
        let mut primary = AutoCommandBufferBuilder::primary(
              ctx.command_allocator.clone(),
              ctx.graphics_queue.queue_family_index(),
              CommandBufferUsage::OneTimeSubmit
        )?;
        
        primary.execute_commands(gbuffer_commands)?;
        
        todo!()
    }
}

#[repr(C)]
#[derive(Copy, Clone, Default, Debug, BufferContents)]
pub struct CameraUbo {
    pub view: Mat4,
    pub projection: Mat4,
    pub camera_position: Vec4
}
