use crate::scene::object::SceneObject;
use std::sync::Arc;

pub mod object;

#[derive(Default)]
pub struct Scene {
    pub objects: Vec<Arc<SceneObject>>
}
