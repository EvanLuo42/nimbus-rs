use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use vulkano::image::sampler::Sampler;
use vulkano::image::view::ImageView;

#[derive(Eq, Hash, PartialEq, Copy, Clone)]
pub struct MaterialId(pub u64);

pub struct Material {
    pub id: MaterialId,
    pub textures: Textures,
    pub sampler: Arc<Sampler>
}

#[derive(Clone)]
pub struct Textures {
    pub base_color: Arc<ImageView>,
    pub normal: Arc<ImageView>,
    pub albedo: Arc<ImageView>,
    pub specular: Arc<ImageView>,
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

    pub fn insert(&mut self, textures: Textures, sampler: Arc<Sampler>) -> MaterialId {
        let id = MaterialId(self.next_id.fetch_add(1, Ordering::Relaxed));
        self.materials.insert(id, Arc::new(Material { id, textures, sampler }));
        id
    }

    pub fn get(&self, id: MaterialId) -> Option<Arc<Material>> {
        self.materials.get(&id).cloned()
    }
}

pub fn load_texture() {
    
}
