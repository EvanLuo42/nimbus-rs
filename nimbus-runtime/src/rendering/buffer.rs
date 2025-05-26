use std::sync::Arc;
use vulkano::buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator};

pub fn upload_buffer<T: BufferContents>(
    allocator: Arc<StandardMemoryAllocator>,
    usage: BufferUsage,
    memory_type_filter: MemoryTypeFilter,
    value: T,
) -> Subbuffer<T> {
    Buffer::from_data(
        allocator.clone(),
        BufferCreateInfo {
            usage,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter,
            ..Default::default()
        },
        value,
    ).unwrap()
}