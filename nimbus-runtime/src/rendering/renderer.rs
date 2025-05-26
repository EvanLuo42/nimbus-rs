use std::sync::Arc;
use nalgebra::max;
use rayon::{ThreadPool, ThreadPoolBuilder};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage};
use winit::window::Window;
use crate::core::errors::NimbusError;
use crate::rendering::camera::Camera;
use crate::rendering::context::RenderContext;
use crate::rendering::frame::FrameManager;
use crate::rendering::pass::{RenderPass, RenderPassGraph};

pub struct Renderer {
    context: Arc<RenderContext>,
    frame_manager: FrameManager,
    render_graph: RenderPassGraph,
    render_thread_pool: ThreadPool,
    camera: Camera
}

impl Renderer {
    pub fn new(window: Arc<Window>) -> Result<Self, NimbusError> {
        let context = Arc::new(RenderContext::new(window.clone())?);
        let render_thread_pool = ThreadPoolBuilder::new()
            .num_threads(max(1, (num_cpus::get() as isize - 2) as usize))
            .thread_name(|i| format!("RenderWorker-{i}"))
            .build()?;
        let frame_manager = FrameManager::new(context.clone());
        let render_graph = RenderPassGraph::new();
        let camera = Camera::default();
        
        Ok(
            Self {
                context,
                frame_manager,
                render_graph,
                render_thread_pool,
                camera
            }
        )
    }
    
    pub fn add_pass<P: RenderPass + Clone + 'static>(&mut self, pass: P) {
        self.render_graph.add_pass(pass.name(), pass.dependencies(), pass.clone());
    }
    
    pub fn render(&mut self) -> Result<(), NimbusError> {
        let frame = self.frame_manager.begin_frame(self.camera)?;
        
        let secondary_cmds = self.render_thread_pool.install(|| {
            self.render_graph.execute_parallel_with_pool(
                &frame,
                &self.context,
                &self.render_thread_pool
            )
        });
        
        let mut primary = AutoCommandBufferBuilder::primary(
            self.context.command_allocator.clone(),
            self.context.graphics_queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit
        )?;
        
        for cmd in secondary_cmds {
            primary.execute_commands(cmd.clone())?;
        }
        
        let primary_cmd = primary.build()?;
        self.frame_manager.end_frame(frame, primary_cmd)?;
        
        Ok(())
    }
}
