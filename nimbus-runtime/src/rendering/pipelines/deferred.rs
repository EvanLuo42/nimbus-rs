use crate::core::errors::{NimbusError, NimbusResult};
use crate::rendering::buffers::VertexBuffer;
use crate::rendering::context::RenderContext;
use crate::rendering::frame::FrameContext;
use crate::rendering::mesh::Submesh;
use crate::rendering::pipeline::RenderPipeline;
use crate::resources::material::MaterialId;
use crate::scene::object::SceneObject;
use std::collections::HashMap;
use std::sync::Arc;
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferInheritanceInfo, CommandBufferInheritanceRenderPassInfo, CommandBufferInheritanceRenderPassType, CommandBufferUsage, PrimaryAutoCommandBuffer, SecondaryAutoCommandBuffer};
use vulkano::descriptor_set::{DescriptorSet, WriteDescriptorSet};
use vulkano::descriptor_set::layout::{DescriptorSetLayout, DescriptorSetLayoutCreateInfo};
use vulkano::format::Format;
use vulkano::image::view::ImageView;
use vulkano::image::{Image, ImageCreateInfo, ImageType, ImageUsage};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter};
use vulkano::pipeline::graphics::color_blend::{ColorBlendAttachmentState, ColorBlendState};
use vulkano::pipeline::graphics::depth_stencil::{DepthState, DepthStencilState};
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::multisample::MultisampleState;
use vulkano::pipeline::graphics::rasterization::RasterizationState;
use vulkano::pipeline::graphics::vertex_input::{Vertex, VertexDefinition};
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::graphics::GraphicsPipelineCreateInfo;
use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
use vulkano::pipeline::{GraphicsPipeline, Pipeline, PipelineBindPoint, PipelineLayout, PipelineShaderStageCreateInfo};
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass};

pub struct DeferredPipeline {
    material_batches: Option<HashMap<MaterialId, Vec<RenderInstance>>>,

    deferred_pass: Arc<RenderPass>,
    
    framebuffers: Vec<Arc<Framebuffer>>,

    gbuffer_graphics_pipeline: Arc<GraphicsPipeline>,
    lighting_graphics_pipeline: Arc<GraphicsPipeline>
}

impl DeferredPipeline {
    pub fn new(ctx: &RenderContext) -> NimbusResult<Self> {
        let swapchain_image_count = ctx.swapchain.image_count();
        let swapchain_image_format = ctx.swapchain.image_format();
        let deferred_pass = vulkano::ordered_passes_renderpass!(
            ctx.device.clone(),
            attachments: {
                final_color: {
                    format: swapchain_image_format,
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                },
                diffuse: {
                    format: Format::R8G8B8A8_UNORM,
                    samples: 1,
                    load_op: Clear,
                    store_op: DontCare
                },
                normal: {
                    format: Format::R16G16B16A16_SFLOAT,
                    samples: 1,
                    load_op: Clear,
                    store_op: DontCare
                },
                depth: {
                    format: Format::D32_SFLOAT,
                    samples: 1,
                    load_op: Clear,
                    store_op: DontCare
                },
            },
            passes: [
                {
                    color: [diffuse, normal],
                    depth_stencil: {depth},
                    input: []
                },
                {
                    color: [final_color],
                    depth_stencil: {},
                    input: [diffuse, normal, depth]
                }
            ]
        )?;

        let mut framebuffers = Vec::new();
        for _ in 0..swapchain_image_count {
            let final_color_image = create_image(ctx, swapchain_image_format, ImageUsage::COLOR_ATTACHMENT)?;
            let diffuse_image = create_image(ctx, Format::R8G8B8A8_UNORM, ImageUsage::COLOR_ATTACHMENT | ImageUsage::SAMPLED | ImageUsage::INPUT_ATTACHMENT)?;
            let normal_image = create_image(ctx, Format::R16G16B16A16_SFLOAT, ImageUsage::COLOR_ATTACHMENT | ImageUsage::SAMPLED | ImageUsage::INPUT_ATTACHMENT)?;
            let depth_image = create_image(ctx, Format::D32_SFLOAT, ImageUsage::DEPTH_STENCIL_ATTACHMENT | ImageUsage::INPUT_ATTACHMENT)?;

            let final_color_view = ImageView::new_default(final_color_image)?;
            let diffuse_view = ImageView::new_default(diffuse_image)?;
            let normal_view = ImageView::new_default(normal_image)?;
            let depth_view = ImageView::new_default(depth_image)?;

            let framebuffer = Framebuffer::new(
                deferred_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![
                        final_color_view,
                        diffuse_view,
                        normal_view,
                        depth_view
                    ],
                    ..Default::default()
                }
            )?;
            framebuffers.push(framebuffer);
        }

        let extent = ctx.swapchain.image_extent();
        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: [extent[0] as f32, extent[1] as f32],
            depth_range: 0.0..=1.0,
        };
        let gbuffer_vs = gbuffer_vs::load(ctx.device.clone()).unwrap().entry_point("main").unwrap();
        let gbuffer_fs = gbuffer_fs::load(ctx.device.clone()).unwrap().entry_point("main").unwrap();
        
        let gbuffer_vertex_input_state = VertexBuffer::per_vertex()
            .definition(&gbuffer_vs)?;
        
        let gbuffer_stages = [
            PipelineShaderStageCreateInfo::new(gbuffer_vs),
            PipelineShaderStageCreateInfo::new(gbuffer_fs),
        ];
        
        let gbuffer_layout = PipelineLayout::new(
            ctx.device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages(&gbuffer_stages)
                .into_pipeline_layout_create_info(ctx.device.clone()).unwrap()
        )?;
        
        let gbuffer_subpass = Subpass::from(deferred_pass.clone(), 0).unwrap();
        
        let gbuffer_graphics_pipeline = GraphicsPipeline::new(
            ctx.device.clone(),
            None,
            GraphicsPipelineCreateInfo {
                stages: gbuffer_stages.into_iter().collect(),
                vertex_input_state: Some(gbuffer_vertex_input_state),
                input_assembly_state: Some(InputAssemblyState::default()),
                viewport_state: Some(ViewportState {
                    viewports: [viewport.clone()].into_iter().collect(),
                    ..Default::default()
                }),
                rasterization_state: Some(RasterizationState::default()),
                multisample_state: Some(MultisampleState::default()),
                depth_stencil_state: Some(DepthStencilState {
                    depth: Some(DepthState::simple()),
                    ..Default::default()
                }),
                subpass: Some(gbuffer_subpass.into()),
                ..GraphicsPipelineCreateInfo::layout(gbuffer_layout)
            }
        )?;

        let lighting_vs = lighting_vs::load(ctx.device.clone()).unwrap().entry_point("main").unwrap();
        let lighting_fs = lighting_fs::load(ctx.device.clone()).unwrap().entry_point("main").unwrap();

        let lighting_vertex_input_state = VertexBuffer::per_vertex()
            .definition(&lighting_vs)?;

        let lighting_stages = [
            PipelineShaderStageCreateInfo::new(lighting_vs),
            PipelineShaderStageCreateInfo::new(lighting_fs),
        ];

        let lighting_layout = PipelineLayout::new(
            ctx.device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages(&lighting_stages)
                .into_pipeline_layout_create_info(ctx.device.clone()).unwrap()
        )?;

        let lighting_subpass = Subpass::from(deferred_pass.clone(), 1).unwrap();

        let lighting_graphics_pipeline = GraphicsPipeline::new(
            ctx.device.clone(),
            None,
            GraphicsPipelineCreateInfo {
                stages: lighting_stages.into_iter().collect(),
                vertex_input_state: Some(lighting_vertex_input_state),
                input_assembly_state: Some(InputAssemblyState::default()),
                viewport_state: Some(ViewportState {
                    viewports: [viewport].into_iter().collect(),
                    ..Default::default()
                }),
                rasterization_state: Some(RasterizationState::default()),
                multisample_state: Some(MultisampleState::default()),
                color_blend_state: Some(ColorBlendState::with_attachment_states(
                    lighting_subpass.num_color_attachments(),
                    ColorBlendAttachmentState::default()
                )),
                depth_stencil_state: None,
                subpass: Some(lighting_subpass.into()),
                ..GraphicsPipelineCreateInfo::layout(lighting_layout)
            }
        )?;
        
        Ok(
            Self {
                material_batches: None,
                framebuffers,
                deferred_pass,
                gbuffer_graphics_pipeline,
                lighting_graphics_pipeline,
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
                    framebuffer: Some(self.framebuffers[frame.image_index as usize].clone()),
                })),
                ..Default::default()
            }
        )?;

        // TODO: Multithreaded command recording
        for instances in self.material_batches.clone().unwrap().values() {
            secondary.bind_pipeline_graphics(self.gbuffer_graphics_pipeline.clone())?;
            
            for instance in instances {
                let mesh = &instance.object.mesh;
                let submesh = &instance.submesh;

                secondary.bind_descriptor_sets(
                    PipelineBindPoint::Graphics,
                    self.gbuffer_graphics_pipeline.layout().clone(),
                    0,
                    vec![
                        // TODO: Camera UBO descriptor set
                        DescriptorSet::new(
                            ctx.descriptor_set_allocator.clone(),
                            self.gbuffer_graphics_pipeline.layout().set_layouts()[0].clone(), 
                            [
                                WriteDescriptorSet::sampler(0, submesh.material.sampler.clone()),
                                WriteDescriptorSet::image_view(1, submesh.material.textures.base_color.clone()),
                                WriteDescriptorSet::image_view(2, submesh.material.textures.albedo.clone()),
                                WriteDescriptorSet::image_view(3, submesh.material.textures.specular.clone()),
                                WriteDescriptorSet::image_view(4, submesh.material.textures.normal.clone()),
                            ], 
                            []
                        )?
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

    pub fn lighting_subpass(&self, ctx: &RenderContext, frame: &FrameContext) -> NimbusResult<Arc<SecondaryAutoCommandBuffer>> {
        let mut secondary = AutoCommandBufferBuilder::secondary(
            ctx.command_allocator.clone(),
            ctx.graphics_queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
            CommandBufferInheritanceInfo {
                render_pass: Some(CommandBufferInheritanceRenderPassType::BeginRenderPass(CommandBufferInheritanceRenderPassInfo {
                    subpass: Subpass::from(self.deferred_pass.clone(), 1).unwrap(),
                    framebuffer: None,
                })),
                ..Default::default()
            }
        )?;
        todo!()
    }
}

fn create_image(ctx: &RenderContext, format: Format, usage: ImageUsage) -> NimbusResult<Arc<Image>> {
    let extent = ctx.swapchain.image_extent();
    Image::new(
        ctx.memory_allocator.clone(),
        ImageCreateInfo {
            image_type: ImageType::Dim2d,
            format,
            extent: [extent[0], extent[1], 1],
            usage,
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
        let lighting_commands = self.lighting_subpass(ctx, frame)?;
        
        let mut primary = AutoCommandBufferBuilder::primary(
              ctx.command_allocator.clone(),
              ctx.graphics_queue.queue_family_index(),
              CommandBufferUsage::OneTimeSubmit
        )?;

        primary.execute_commands(gbuffer_commands)?;
        primary.execute_commands(lighting_commands)?;

        primary.build().map_err(NimbusError::from)
    }

    fn get_graphics_pipeline(&self) -> Arc<GraphicsPipeline> {
        self.gbuffer_graphics_pipeline.clone()
    }
}

#[derive(Clone)]
struct RenderInstance {
    object: Arc<SceneObject>,
    submesh: Arc<Submesh>
}

mod gbuffer_fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "shaders/gbuffer.frag"
    }
}

mod gbuffer_vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "shaders/gbuffer.vert"
    }
}

mod lighting_fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "shaders/lighting.frag"
    }
}

mod lighting_vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "shaders/lighting.vert"
    }
}
