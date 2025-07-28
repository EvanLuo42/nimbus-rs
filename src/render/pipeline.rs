use crate::render::drawable::Drawable;
use std::collections::HashMap;
use wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindingResource, BlendState, ColorTargetState, ColorWrites, CompareFunction, DepthBiasState, DepthStencilState, Device, Face, FragmentState, FrontFace, MultisampleState, PipelineCompilationOptions, PipelineLayout, PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology, RenderPipeline, RenderPipelineDescriptor, Sampler, SamplerDescriptor, StencilState, SurfaceConfiguration, Texture, TextureFormat, TextureView, TextureViewDescriptor, VertexState};
use crate::render::material::{BaseColorType, MaterialType, MetallicRoughnessType};

#[derive(Clone)]
pub struct Pipeline {
    pub render_pipeline: RenderPipeline,
    pub bind_group_layout: BindGroupLayout,
    pub pipeline_layout: PipelineLayout,
    pub bind_group: BindGroup,
}

#[derive(Default, Clone)]
pub struct PipelineCache {
    pipelines: HashMap<String, Pipeline>
}

impl PipelineCache {
    pub fn get_or_create(&mut self, drawable: &Drawable, device: &Device, config: &SurfaceConfiguration) -> &Pipeline {
        self.pipelines
            .entry(drawable.material.name.clone())
            .or_insert_with(|| Self::create_render_pipeline(drawable, device, config))
    }

    fn create_render_pipeline(drawable: &Drawable, device: &Device, config: &SurfaceConfiguration) -> Pipeline {
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &drawable.material.ty.bind_group_layout_entries(),
            label: Some(&format!("{} Bind Group Layout", drawable.material.name.clone()))
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some(&format!("{} Render Pipeline Layout", drawable.material.name.clone())),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &drawable.material.ty.push_constant_ranges()
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some(&format!("{} Render Pipeline", drawable.material.name.clone())),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &drawable.material.vertex_shader,
                entry_point: Some("vertexMain"),
                buffers: &[drawable.mesh.vertex_buffer_layout()],
                compilation_options: PipelineCompilationOptions::default()
            },
            fragment: Some(FragmentState {
                module: &drawable.material.fragment_shader,
                entry_point: Some("fragmentMain"),
                targets: &[Some(ColorTargetState {
                    format: config.format,
                    blend: match drawable.material.is_transparent() {
                        true => Some(BlendState::ALPHA_BLENDING),
                        false => Some(BlendState::REPLACE),
                    },
                    write_mask: ColorWrites::ALL
                })],
                compilation_options: PipelineCompilationOptions::default(),
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Less,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState {
                count: 4,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        let mut entries = vec![];

        match &drawable.material.ty {
            MaterialType::Pbr { base_color, metallic_roughness } => {
                if let BaseColorType::Texture { texture_view, sampler, .. } = base_color {
                    entries.push(BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(texture_view),
                    });
                    entries.push(BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(sampler),
                    });
                }

                if let MetallicRoughnessType::Texture { texture_view, sampler, .. } = metallic_roughness {
                    entries.push(BindGroupEntry {
                        binding: 2,
                        resource: BindingResource::TextureView(texture_view),
                    });
                    entries.push(BindGroupEntry {
                        binding: 3,
                        resource: BindingResource::Sampler(sampler),
                    });
                }
            }

            MaterialType::Unlit { base_color } => {
                if let BaseColorType::Texture { texture_view, sampler, .. } = base_color {
                    entries.push(BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(texture_view),
                    });
                    entries.push(BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(sampler),
                    });
                }
            }

            MaterialType::Custom => {
                unimplemented!()
            }
        }

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some(&format!("{} Bind Group", drawable.material.name.clone())),
            layout: &bind_group_layout,
            entries: &entries
        });

        Pipeline {
            bind_group_layout,
            pipeline_layout,
            render_pipeline,
            bind_group
        }
    }
}
