pub mod rendering;
pub mod physics;
pub mod core;
pub mod audio;
mod resources;
mod scene;
mod scripting;
mod ui;

use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};
use crate::rendering::renderer::Renderer;

pub struct Engine {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>
}

impl ApplicationHandler for Engine {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window = Some(Arc::new(event_loop.create_window(Window::default_attributes()).unwrap()));
        self.renderer = Some(Renderer::new(self.window.clone().unwrap()).unwrap());
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => self.window.as_ref().unwrap().request_redraw(),
            _ => {}
        }
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    pub fn new() -> Self {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);
        
        let mut engine = Self {
            window: None,
            renderer: None
        };
        event_loop.run_app(&mut engine).unwrap();
        engine
    }
}
