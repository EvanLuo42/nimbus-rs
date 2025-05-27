use std::sync::Arc;
use crate::scene::object::SceneObject;

pub mod object;

#[derive(Default)]
pub struct Scene {
    pub objects: Vec<Arc<SceneObject>>
}

impl Scene {
    pub fn cull(&self) -> Vec<SceneObject> {
        todo!()
    }
}
