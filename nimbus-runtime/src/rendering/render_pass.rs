use crate::core::errors::NimbusError;
use crate::rendering::context::RenderContext;
use std::sync::Arc;
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer, SecondaryAutoCommandBuffer};

pub struct RenderPass {
    pub name: String,
    pub subpasses: Vec<Arc<dyn Subpass>>
}

impl RenderPass {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            subpasses: vec![]
        }
    }
    
    pub fn add_pass<S: Subpass + 'static>(&mut self, subpass: Arc<S>) {
        self.subpasses.push(subpass);
    }
    
    pub fn record_commands(&self, ctx: &RenderContext) -> Result<Arc<PrimaryAutoCommandBuffer>, NimbusError> {
        let mut primary = AutoCommandBufferBuilder::primary(
              ctx.command_allocator.clone(),
              ctx.graphics_queue.queue_family_index(),
              CommandBufferUsage::OneTimeSubmit
        )?;
        
        for secondary_command in self.record_subpasses_commands(ctx)? {
               primary.execute_commands(secondary_command)?;
        }
        
        primary.build().map_err(NimbusError::from)
    }
    
    fn record_subpasses_commands(&self, ctx: &RenderContext) -> Result<Vec<Arc<SecondaryAutoCommandBuffer>>, NimbusError> {
        todo!()
    }
}

pub trait Subpass: Send + Sync {
    fn name(&self) -> &str;
    fn dependencies(&self) -> &[&str];
    fn record_commands(&self) -> Result<Arc<SecondaryAutoCommandBuffer>, NimbusError>;
}
