pub mod rendering;
pub mod physics;
pub mod core;
pub mod audio;
pub mod resources;
pub mod scene;
pub mod scripting;
pub mod ui;

pub use nalgebra;

use crate::rendering::renderer::Renderer;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
pub use winit::window::WindowAttributes;
use winit::window::{Window, WindowId};

#[derive(Default)]
pub struct Engine {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    window_attributes: WindowAttributes
}

impl Engine {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn with_window_attributes(window_attributes: WindowAttributes) -> Self {
        Self {
            window_attributes,
            ..Default::default()
        }
    }
    
    pub fn run(&mut self) {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop.run_app(self).unwrap()
    }
}

impl ApplicationHandler for Engine {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(event_loop.create_window(self.window_attributes.clone()).unwrap());
        self.window = Some(window.clone());
        
        self.renderer = Some(Renderer::new(window).unwrap());
        
        self.window.as_ref().unwrap().request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                if let Some(renderer) = &mut self.renderer {
                    let _ = renderer.render();
                }
                self.window.as_ref().unwrap().request_redraw();
            }
            _ => {}
        }
    }
}
