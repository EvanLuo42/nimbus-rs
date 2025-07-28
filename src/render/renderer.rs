use crate::render::drawable::Drawable;
use crate::render::pipeline::PipelineCache;
use std::sync::Arc;
use wgpu::wgt::{CommandEncoderDescriptor, TextureViewDescriptor};
use wgpu::{
    Adapter, Backends, Color, CommandEncoder, Device, IndexFormat, Instance, InstanceDescriptor,
    LoadOp, Operations, PowerPreference, Queue, RenderPassColorAttachment, RenderPassDescriptor,
    RequestAdapterOptions, StoreOp, Surface, SurfaceConfiguration, SurfaceTexture, TextureUsages,
    TextureView,
};
use winit::dpi::PhysicalSize;
use winit::window::Window;

pub struct Renderer<'window> {
    instance: Instance,
    surface: Surface<'window>,
    surface_config: SurfaceConfiguration,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    pipeline_cache: PipelineCache,
    render_queue: Vec<Drawable>,
}

impl<'window> Renderer<'window> {
    pub async fn new(window: Arc<Window>) -> crate::Result<Renderer<'window>> {
        let PhysicalSize { width, height } = window.inner_size();

        let instance = Instance::new(&InstanceDescriptor {
            backends: Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone())?;

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;

        Ok(Renderer {
            instance,
            surface,
            surface_config,
            adapter,
            device,
            queue,
            pipeline_cache: Default::default(),
            render_queue: Default::default(),
        })
    }

    pub fn begin_frame(&mut self, frame_context: &mut FrameContext) -> crate::Result<()> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());
        let encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Main Encoder"),
            });
        frame_context.view = Some(view);
        frame_context.encoder = Some(encoder);
        frame_context.output = Some(output);
        Ok(())
    }

    pub fn end_frame(&mut self, frame_context: &mut FrameContext) {
        let mut main_render_pass =
            frame_context
                .encoder
                .as_mut()
                .unwrap()
                .begin_render_pass(&RenderPassDescriptor {
                    label: Some("Main Render Pass"),
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: frame_context.view.as_ref().unwrap(),
                        depth_slice: None,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Clear(Color {
                                r: 0.1,
                                g: 0.2,
                                b: 0.3,
                                a: 1.0,
                            }),
                            store: StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

        for drawable in self.render_queue.iter() {
            let pipeline =
                self.pipeline_cache
                    .get_or_create(&drawable, &self.device, &self.surface_config);
            main_render_pass.set_pipeline(&pipeline.render_pipeline);
            main_render_pass.set_bind_group(0, &pipeline.bind_group, &[]);

            main_render_pass.set_vertex_buffer(0, drawable.mesh.vertex_buffer.slice(..));
            match &drawable.mesh.index_buffer {
                None => {
                    main_render_pass.draw(0..drawable.mesh.vertex_count, 0..1);
                }
                Some(index_buffer) => {
                    main_render_pass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint16);
                    main_render_pass.draw_indexed(0..drawable.mesh.index_count, 0, 0..1)
                }
            }
        }

        drop(main_render_pass);

        let encoder = frame_context.encoder.take().unwrap();
        let output = frame_context.output.take().unwrap();
        self.queue.submit(Some(encoder.finish()));
        output.present();

        self.render_queue.clear();
    }

    pub fn submit(&mut self, drawable: Drawable) {
        self.render_queue.push(drawable)
    }
}

#[derive(Default)]
pub struct FrameContext {
    encoder: Option<CommandEncoder>,
    view: Option<TextureView>,
    output: Option<SurfaceTexture>,
}
