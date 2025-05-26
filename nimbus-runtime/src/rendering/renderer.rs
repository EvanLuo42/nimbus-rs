use crate::core::errors::NimbusError;
use crate::rendering::context::RenderContext;
use crate::rendering::frame::FrameManager;
use crate::rendering::render_pass::RenderPass;
use rayon::{ThreadPool, ThreadPoolBuilder};
use std::cmp::max;
use std::sync::Arc;
use winit::window::Window;

pub struct Renderer {
    pub render_context: RenderContext,
    pub render_thread_pool: ThreadPool,
    pub frame_manager: FrameManager,
    pub render_passes: Vec<Arc<RenderPass>>
}

impl Renderer {
    pub fn new(window: Arc<Window>) -> Result<Self, NimbusError> {
        let render_context = RenderContext::new(window)?;
        let render_thread_pool = ThreadPoolBuilder::new()
            .num_threads(max(1, (num_cpus::get() as isize - 2) as usize))
            .thread_name(|i| format!("RenderWorker-{i}"))
            .build()?;

        todo!()
    }

    pub fn render(&self) -> Result<(), NimbusError> {
        let frame = self.frame_manager.begin_frame(&self.render_context)?;
        let commands = self.render_passes.iter()
            .map(|pass| pass.record_commands(&self.render_context))
            .collect::<Result<_, _>>()?;
        self.frame_manager.end_frame(&self.render_context, frame, commands)?;
        Ok(())
    }
}
