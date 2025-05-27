use crate::core::errors::NimbusError;
use crate::rendering::context::RenderContext;
use crate::rendering::frame::FrameManager;
use crate::rendering::pipeline::RenderPipeline;
use crate::rendering::pipelines::deferred::DeferredPipeline;
use crate::resources::material::MaterialManager;
use crate::scene::Scene;
use rayon::{ThreadPool, ThreadPoolBuilder};
use std::cmp::max;
use std::sync::{Arc, Mutex};
use winit::window::Window;

pub struct Renderer {
    pub render_context: RenderContext,
    pub render_thread_pool: ThreadPool,
    pub frame_manager: FrameManager,
    pub pipeline: Arc<Mutex<dyn RenderPipeline>>,
    pub material_manager: Arc<Mutex<MaterialManager>>
}

impl Renderer {
    pub fn new(window: Arc<Window>) -> Result<Self, NimbusError> {
        let render_context = RenderContext::new(window)?;
        let render_thread_pool = ThreadPoolBuilder::new()
            .num_threads(max(1, (num_cpus::get() as isize - 2) as usize))
            .thread_name(|i| format!("RenderWorker-{i}"))
            .build()?;
        let material_manager = Arc::new(Mutex::new(MaterialManager::new()));
        let pipeline = Arc::new(Mutex::new(DeferredPipeline::new(&render_context)?));
        let scene = Arc::new(Scene::load("nimbus-runtime/models/Box.gltf", &render_context, material_manager.clone(), pipeline.lock().unwrap().get_graphics_pipeline()));
        pipeline.lock().unwrap().add_scene(scene);
        
        Ok(
            Self {
                render_context,
                render_thread_pool,
                frame_manager: FrameManager,
                material_manager,
                pipeline,
            }
        )
    }

    pub fn render(&self) -> Result<(), NimbusError> {
        let frame = self.frame_manager.begin_frame(&self.render_context)?;

        let commands = self.pipeline.lock().unwrap().record_commands(&self.render_context, &frame)?;

        self.frame_manager.end_frame(&self.render_context, frame, commands)?;
        Ok(())
    }
}
