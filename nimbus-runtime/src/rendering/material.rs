use std::sync::Arc;
use vulkano::descriptor_set::DescriptorSet;

#[derive(Eq, Hash, PartialEq, Copy, Clone)]
pub struct MaterialId(pub u64);

pub struct Material {
    pub id: MaterialId,
    pub descriptor_set: Arc<DescriptorSet>
}