use crate::core::errors::NimbusError;
use crate::core::math::{Mat4, Vec4};
use crate::rendering::camera::Camera;
use crate::rendering::context::RenderContext;
use crate::rendering::drawable::Drawable;
use std::sync::Arc;
use nalgebra::Vector4;
use vulkano::buffer::{BufferContents, BufferUsage, Subbuffer};
use vulkano::command_buffer::PrimaryAutoCommandBuffer;
use vulkano::memory::allocator::MemoryTypeFilter;
use vulkano::swapchain::{acquire_next_image, SwapchainAcquireFuture, SwapchainPresentInfo};
use vulkano::sync::GpuFuture;
use crate::rendering::buffer::upload_buffer;

pub struct FrameManager {
    ctx: Arc<RenderContext>
}

impl FrameManager {
    pub fn new(ctx: Arc<RenderContext>) -> Self {
        Self {
            ctx
        }
    }
    
    pub fn begin_frame(&self, camera: Camera) -> Result<FrameContext, NimbusError> {
        let (image_index, _suboptimal, acquire_future) =
            acquire_next_image(self.ctx.swapchain.clone(), None)?;
        let global_uniform = GlobalUniform::from(camera);
        let global_ubo = upload_buffer(
            self.ctx.memory_allocator.clone(),
            BufferUsage::UNIFORM_BUFFER,
            MemoryTypeFilter::PREFER_HOST | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            global_uniform
        );
        Ok(
            FrameContext {
                image_index,
                future: acquire_future,
                visible_drawables: vec![],
                camera,
                global_ubo,
            }
        )
    }
    
    pub fn end_frame(&self, frame: FrameContext, primary_cmd: Arc<PrimaryAutoCommandBuffer>) -> Result<(), NimbusError> {
        let future = frame
            .future
            .then_execute(self.ctx.graphics_queue.clone(), primary_cmd)?
            .then_swapchain_present(
                self.ctx.graphics_queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(
                    self.ctx.swapchain.clone(),
                    frame.image_index
                )
            )
            .then_signal_fence_and_flush()?;
        future.wait(None)?;
        Ok(())
    }
}

pub struct FrameContext {
    pub image_index: u32,
    pub future: SwapchainAcquireFuture,
    pub visible_drawables: Vec<Arc<Drawable>>,
    pub camera: Camera,
    pub global_ubo: Subbuffer<GlobalUniform>
}

#[repr(C)]
#[derive(Copy, Clone, BufferContents)]
pub struct GlobalUniform {
    pub view_proj: Mat4,
    pub camera_pos: Vec4
}

impl From<Camera> for GlobalUniform {
    fn from(camera: Camera) -> Self {
        Self {
            view_proj: camera.view_proj_matrix().into(),
            camera_pos: Vector4::new(camera.position.x, camera.position.y, camera.position.z, 1.0).into(),
        }
    }
}
