use thiserror::Error;
use vulkano::device::{DeviceExtensions, QueueFlags};

#[derive(Error, Debug)]
pub enum NimbusError {
    #[error("Vulkan Error: {0}")]
    VulkanError(#[from] vulkano::VulkanError),
    
    #[error("Validated Vulkan Error: {0}")]
    ValidatedVulkanError(#[from] vulkano::Validated<vulkano::VulkanError>),
    
    #[error("Failed to load vulkan library: {0}")]
    LoadVulkanLibraryError(#[from] vulkano::LoadingError),
    
    #[error("Failed to create surface from window")]
    CreateSurfaceFromWindowError(#[from] vulkano::swapchain::FromWindowError),
    
    #[error("Couldn't find a physical device with required extensions: {:?}", .0)]
    PhysicalDeviceNotFound(Box<DeviceExtensions>),
    
    #[error("Couldn't find queue family within the given physical device: {:?}", .0)]
    QueueFamilyNotFound(QueueFlags)
}