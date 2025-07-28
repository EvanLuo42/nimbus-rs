use cgmath::{Matrix4, SquareMatrix};
use crate::render::camera::Camera;
use crate::render::drawable::Drawable;
use crate::render::renderer::{FrameContext, Renderer};

pub struct SceneNode {
    pub name: Option<String>,
    pub local_transform: Matrix4<f32>,
    pub drawable: Option<Drawable>,
    pub children: Vec<usize>,
    pub parent: Option<usize>,
}

pub struct Scene {
    pub nodes: Vec<SceneNode>,
    pub root_nodes: Vec<usize>,
    pub camera: Camera
}

impl Scene {
    pub fn new() -> Self {
        Self {
            nodes: vec![],
            root_nodes: vec![],
            camera: Camera::default()
        }
    }

    pub fn add_node(&mut self, node: SceneNode) -> usize {
        let index = self.nodes.len();
        self.nodes.push(node);
        index
    }

    pub fn add_child(&mut self, parent: usize, child: usize) {
        self.nodes[parent].children.push(child);
        self.nodes[child].parent = Some(parent);
    }
    
    pub fn render(&self, renderer: &mut Renderer, frame_ctx: &mut FrameContext) {
        for &root in &self.root_nodes {
            self.render_node_recursive(renderer, frame_ctx, root, Matrix4::identity());
        }
    }

    fn render_node_recursive(
        &self,
        renderer: &mut Renderer,
        frame_ctx: &mut FrameContext,
        node_index: usize,
        parent_transform: Matrix4<f32>,
    ) {
        let node = &self.nodes[node_index];
        let world_transform = parent_transform * node.local_transform;

        if let Some(mut drawable) = node.drawable.clone() {
            drawable.model_matrix = world_transform;
            renderer.submit(drawable);
        }

        for &child in &node.children {
            self.render_node_recursive(renderer, frame_ctx, child, world_transform);
        }
    }
}
