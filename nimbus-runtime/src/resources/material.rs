use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use vulkano::descriptor_set::DescriptorSet;

#[derive(Eq, Hash, PartialEq, Copy, Clone)]
pub struct MaterialId(pub u64);

pub struct Material {
    pub id: MaterialId,
    pub descriptor_set: Arc<DescriptorSet>
}

pub struct MaterialManager {
    next_id: AtomicU64,
    materials: HashMap<MaterialId, Arc<Material>>,
}

impl Default for MaterialManager {
    fn default() -> Self {
        Self::new()
    }
}

impl MaterialManager {
    pub fn new() -> Self {
        Self {
            next_id: AtomicU64::new(1),
            materials: HashMap::new(),
        }
    }

    pub fn insert(&mut self, descriptor_set: Arc<DescriptorSet>) -> MaterialId {
        let id = MaterialId(self.next_id.fetch_add(1, Ordering::Relaxed));
        self.materials.insert(id, Arc::new(Material { id, descriptor_set }));
        id
    }

    pub fn get(&self, id: MaterialId) -> Option<Arc<Material>> {
        self.materials.get(&id).cloned()
    }
}
