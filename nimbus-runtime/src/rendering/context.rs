use crate::core::errors::NimbusError;
use std::sync::Arc;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType};
use vulkano::device::{Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags};
use vulkano::image::{Image, ImageUsage};
use vulkano::instance::{Instance, InstanceCreateFlags, InstanceCreateInfo, InstanceExtensions};
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::swapchain::{Surface, Swapchain, SwapchainCreateInfo};
use vulkano::VulkanLibrary;
use winit::window::Window;

#[derive(Clone)]
pub struct RenderContext {
    pub instance: Arc<Instance>,
    pub surface: Arc<Surface>,
    pub physical_device: Arc<PhysicalDevice>,
    pub device: Arc<Device>,
    pub graphics_queue: Arc<Queue>,
    pub swapchain: Arc<Swapchain>,
    pub images: Vec<Arc<Image>>,
    pub memory_allocator: Arc<StandardMemoryAllocator>,
    pub command_allocator: Arc<StandardCommandBufferAllocator>
}

impl RenderContext {
    pub fn new(window: Arc<Window>) -> Result<Self, NimbusError> {
        let library = VulkanLibrary::new()?;

        let instance_extensions = InstanceExtensions {
            #[cfg(target_os = "macos")]
            khr_portability_enumeration: true,
            #[cfg(target_os = "macos")]
            ext_metal_surface: true,
            #[cfg(target_os = "windows")]
            khr_win32_surface: true,
            ..Default::default()
        };

        let instance = Instance::new(library, InstanceCreateInfo {
            #[cfg(target_os = "macos")]
            flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
            enabled_extensions: instance_extensions,
            ..Default::default()
        })?;
        let surface = Surface::from_window(instance.clone(), window.clone())?;

        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };
        let queue_flags = QueueFlags::GRAPHICS;

        let (physical_device, queue_family_index) = select_physical_device(
            &instance,
            &surface,
            &device_extensions,
            queue_flags
        )?;

        let (device, mut queues) = Device::new(
            physical_device.clone(),
            DeviceCreateInfo {
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                enabled_extensions: device_extensions,
                ..Default::default()
            }
        )?;

        let graphics_queue = queues.next().unwrap();

        let caps = physical_device
            .surface_capabilities(&surface, Default::default())?;
        let image_extent = window.inner_size().into();
        let composite_alpha = caps.supported_composite_alpha.into_iter().next().unwrap();
        let image_format = physical_device
            .surface_formats(&surface, Default::default())
            ?[0].0;

        let (swapchain, images) = Swapchain::new(
            device.clone(),
            surface.clone(),
            SwapchainCreateInfo {
                min_image_count: caps.min_image_count + 1,
                image_format,
                image_extent,
                image_usage: ImageUsage::COLOR_ATTACHMENT,
                composite_alpha,
                ..Default::default()
            }
        )?;
        
        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        
        let command_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            Default::default()
        ));

        Ok(
            Self {
                instance,
                surface,
                physical_device,
                device,
                graphics_queue,
                swapchain,
                images,
                memory_allocator,
                command_allocator,
            }
        )
    }
}

fn select_physical_device(
    instance: &Arc<Instance>,
    surface: &Arc<Surface>,
    required_extensions: &DeviceExtensions,
    required_flags: QueueFlags,
) -> Result<(Arc<PhysicalDevice>, u32), NimbusError> {
    let physical_devices = instance.enumerate_physical_devices()?;

    let suitable_device = physical_devices
        .filter(|device| device.supported_extensions().contains(required_extensions))
        .filter_map(|device| {
            find_suitable_queue_family(&device, surface, required_flags)
                .map(|queue_index| (device, queue_index))
        })
        .min_by_key(|(device, _)| device_score(device));

    suitable_device.ok_or_else(|| {
        NimbusError::PhysicalDeviceNotFound(Box::new(*required_extensions))
    })
}

fn find_suitable_queue_family(
    device: &PhysicalDevice,
    surface: &Arc<Surface>,
    required_flags: QueueFlags,
) -> Option<u32> {
    device.queue_family_properties()
        .iter()
        .enumerate()
        .find(|(index, props)| {
            props.queue_flags.contains(required_flags)
                && device.surface_support(*index as u32, surface).unwrap_or(false)
        })
        .map(|(index, _)| index as u32)
}

fn device_score(device: &PhysicalDevice) -> u8 {
    match device.properties().device_type {
        PhysicalDeviceType::DiscreteGpu => 0,
        PhysicalDeviceType::IntegratedGpu => 1,
        PhysicalDeviceType::VirtualGpu => 2,
        PhysicalDeviceType::Cpu => 3,
        _ => 4,
    }
}
