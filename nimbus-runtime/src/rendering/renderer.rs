use std::sync::Arc;
use winit::window::Window;
use crate::core::errors::NimbusError;
use crate::rendering::context::RenderContext;

pub struct Renderer {
    context: RenderContext
}

impl Renderer {
    pub fn new(window: Arc<Window>) -> Result<Self, NimbusError> {
        let context = RenderContext::new(window.clone())?;
        
        Ok(
            Self {
                context,
            }
        )
    }
    
    pub fn render(&mut self) {
        todo!()
    }
}
