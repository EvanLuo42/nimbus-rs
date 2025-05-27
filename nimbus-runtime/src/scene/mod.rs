use crate::scene::object::GameObject;

pub mod object;

#[derive(Default)]
pub struct Scene {
    pub objects: Vec<GameObject>
}
