use crate::core::errors::NimbusError;
use crate::rendering::context::RenderContext;
use crate::rendering::frame::FrameManager;
use crate::rendering::pipeline::{BasicPipeline, RenderPipeline};
use rayon::{ThreadPool, ThreadPoolBuilder};
use std::cmp::max;
use std::sync::Arc;
use winit::window::Window;

pub struct Renderer {
    pub render_context: RenderContext,
    pub render_thread_pool: ThreadPool,
    pub frame_manager: FrameManager,
    pub pipeline: Arc<dyn RenderPipeline>
}

impl Renderer {
    pub fn new(window: Arc<Window>) -> Result<Self, NimbusError> {
        let render_context = RenderContext::new(window)?;
        let render_thread_pool = ThreadPoolBuilder::new()
            .num_threads(max(1, (num_cpus::get() as isize - 2) as usize))
            .thread_name(|i| format!("RenderWorker-{i}"))
            .build()?;
        let pipeline = Arc::new(BasicPipeline::new(&render_context)?);
        Ok(
            Self {
                render_context,
                render_thread_pool,
                frame_manager: FrameManager,
                pipeline,
            }
        )
    }

    pub fn render(&self) -> Result<(), NimbusError> {
        let frame = self.frame_manager.begin_frame(&self.render_context)?;

        let commands = self.pipeline.record_commands(&self.render_context, &frame)?;

        self.frame_manager.end_frame(&self.render_context, frame, commands)?;
        Ok(())
    }
}
