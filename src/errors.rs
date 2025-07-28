use thiserror::Error;

#[derive(Error, Debug)]
pub enum NimbusError {
    #[error("Create surface failed: {0}")]
    CreateSurfaceError(#[from] wgpu::CreateSurfaceError),
    
    #[error("Request adapter failed: {0}")]
    RequestAdapterError(#[from] wgpu::RequestAdapterError),

    #[error("Request device failed: {0}")]
    RequestDeviceError(#[from] wgpu::RequestDeviceError),

    #[error("Surface error: {0}")]
    SurfaceError(#[from] wgpu::SurfaceError),
}