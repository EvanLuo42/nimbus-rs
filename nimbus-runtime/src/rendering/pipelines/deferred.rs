use crate::core::errors::{NimbusError, NimbusResult};
use crate::rendering::context::RenderContext;
use crate::rendering::frame::FrameContext;
use crate::rendering::material::MaterialId;
use crate::rendering::mesh::Submesh;
use crate::rendering::pipeline::RenderPipeline;
use crate::scene::object::SceneObject;
use crate::scene::Scene;
use std::collections::HashMap;
use std::sync::Arc;
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferInheritanceInfo, CommandBufferInheritanceRenderPassInfo, CommandBufferInheritanceRenderPassType, CommandBufferUsage, PrimaryAutoCommandBuffer, SecondaryAutoCommandBuffer};
use vulkano::format::Format;
use vulkano::image::view::ImageView;
use vulkano::image::{Image, ImageCreateInfo, ImageType, ImageUsage};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter};
use vulkano::pipeline::graphics::vertex_input::{Vertex, VertexDefinition};
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::{GraphicsPipeline, Pipeline, PipelineBindPoint, PipelineLayout, PipelineShaderStageCreateInfo};
use vulkano::pipeline::graphics::color_blend::{ColorBlendAttachmentState, ColorBlendState};
use vulkano::pipeline::graphics::GraphicsPipelineCreateInfo;
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::multisample::MultisampleState;
use vulkano::pipeline::graphics::rasterization::RasterizationState;
use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass};
use crate::rendering::buffers::VertexBuffer;

pub struct DeferredPipeline {
    scene: Arc<Scene>,
    material_batches: HashMap<MaterialId, Vec<RenderInstance>>,

    deferred_pass: Arc<RenderPass>,
    
    gbuffer_framebuffers: Vec<Arc<Framebuffer>>,
    gbuffer_graphics_pipeline: Arc<GraphicsPipeline>
}

impl DeferredPipeline {
    pub fn new(ctx: &RenderContext, scene: Arc<Scene>) -> NimbusResult<Self> {
        let mut material_batches: HashMap<MaterialId, Vec<RenderInstance>> = HashMap::new();
        for object in &scene.objects {
            for submesh in &object.mesh.submeshes {
                material_batches.entry(submesh.material.id)
                    .or_default()
                    .push(RenderInstance {
                        object: object.clone(),
                        submesh: submesh.clone()
                    });
            }
        }

        let swapchain_image_count = ctx.swapchain.image_count();
        let deferred_pass = vulkano::single_pass_renderpass!(
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
                deferred_pass.clone(),
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

        let extent = ctx.swapchain.image_extent();
        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: [extent[0] as f32, extent[1] as f32],
            depth_range: 0.0..=1.0,
        };
        let vs = vs::load(ctx.device.clone()).unwrap().entry_point("main").unwrap();
        let fs = fs::load(ctx.device.clone()).unwrap().entry_point("main").unwrap();
        
        let vertex_input_state = VertexBuffer::per_vertex()
            .definition(&vs)?;
        
        let stages = [
            PipelineShaderStageCreateInfo::new(vs),
            PipelineShaderStageCreateInfo::new(fs),
        ];
        
        let layout = PipelineLayout::new(
            ctx.device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                .into_pipeline_layout_create_info(ctx.device.clone()).unwrap()
        )?;
        
        let gbuffer_subpass = Subpass::from(deferred_pass.clone(), 0).unwrap();
        
        let gbuffer_graphics_pipeline = GraphicsPipeline::new(
            ctx.device.clone(),
            None,
            GraphicsPipelineCreateInfo {
                stages: stages.into_iter().collect(),
                vertex_input_state: Some(vertex_input_state),
                input_assembly_state: Some(InputAssemblyState::default()),
                viewport_state: Some(ViewportState {
                    viewports: [viewport].into_iter().collect(),
                    ..Default::default()
                }),
                rasterization_state: Some(RasterizationState::default()),
                multisample_state: Some(MultisampleState::default()),
                color_blend_state: Some(ColorBlendState::with_attachment_states(
                    gbuffer_subpass.num_color_attachments(),
                    ColorBlendAttachmentState::default()
                )),
                subpass: Some(gbuffer_subpass.into()),
                ..GraphicsPipelineCreateInfo::layout(layout)
            }
        )?;
        
        Ok(
            Self {
                scene,
                material_batches,
                gbuffer_framebuffers,
                deferred_pass,
                gbuffer_graphics_pipeline,
            }
        )
    }

    pub fn gbuffer_subpass(&self, ctx: &RenderContext, frame: &FrameContext) -> NimbusResult<Arc<SecondaryAutoCommandBuffer>> {
        let mut secondary = AutoCommandBufferBuilder::secondary(
            ctx.command_allocator.clone(),
            ctx.graphics_queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
            CommandBufferInheritanceInfo {
                render_pass: Some(CommandBufferInheritanceRenderPassType::BeginRenderPass(CommandBufferInheritanceRenderPassInfo {
                    subpass: Subpass::from(self.deferred_pass.clone(), 0).unwrap(),
                    framebuffer: Some(self.gbuffer_framebuffers[frame.image_index as usize].clone()),
                }
                )),
                ..Default::default()
            }
        )?;
        
        for instances in self.material_batches.values() {
            secondary.bind_pipeline_graphics(self.gbuffer_graphics_pipeline.clone())?;
            
            for instance in instances {
                let mesh = &instance.object.mesh;
                let submesh = &instance.submesh;

                secondary.bind_descriptor_sets(
                    PipelineBindPoint::Graphics,
                    self.gbuffer_graphics_pipeline.layout().clone(),
                    0,
                    vec![
                        submesh.material.descriptor_set.clone()
                    ]
                )?;
                
                secondary.push_constants(
                    self.gbuffer_graphics_pipeline.layout().clone(),
                    0,
                    instance.object.model
                )?;
                
                secondary.bind_vertex_buffers(0, mesh.vertex_buffer.clone())?;
                secondary.bind_index_buffer(mesh.index_buffer.clone())?;
               unsafe {
                   secondary.draw_indexed(
                       submesh.index_count,
                       1,
                       submesh.index_offset,
                       0,
                       0
                   )?;
               }
            }
        }

        secondary.build().map_err(NimbusError::from)
    }

    pub fn lighting_subpass(&self, _ctx: &RenderContext) -> NimbusResult<Arc<SecondaryAutoCommandBuffer>> {
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

        primary.build().map_err(NimbusError::from)
    }
}

struct RenderInstance {
    object: Arc<SceneObject>,
    submesh: Arc<Submesh>
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "shaders/gbuffer.frag"
    }
}

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "shaders/gbuffer.vert"
    }
}
