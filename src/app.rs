use std::sync::Arc;
use tracing::info;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};
use winit::window::{Window, WindowAttributes, WindowId};
use crate::render::renderer::{FrameContext, Renderer};
use crate::render::scene::Scene;

pub struct App {
    proxy: Option<EventLoopProxy<State<'static>>>,
    state: Option<State<'static>>
}

impl App {
    pub fn new(event_loop: &EventLoop<State>) -> Self {
        let proxy = Some(event_loop.create_proxy());
        Self {
            state: None,
            proxy
        }
    }
}

impl ApplicationHandler<State<'static>> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes()
                    .with_title("Nimbus Renderer")
                    .with_resizable(true)
                )
                .unwrap(),
        );

        let state = pollster::block_on(State::new(window.clone())).unwrap();
        self.state = Some(state);

        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        let state = self.state.as_mut().unwrap();
        match event {
            WindowEvent::CloseRequested => {
                info!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                state.render().unwrap();
                state.window.request_redraw();
            }
            WindowEvent::Resized(PhysicalSize {width, height}) => {
                state.resize(width, height);
            }
            _ => (),
        }
    }
}

pub struct State<'window> {
    renderer: Renderer<'window>,
    scene: Scene,
    window: Arc<Window>,
    is_surface_configured: bool,
    frame_context: FrameContext
}

impl<'window> State<'window> {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let renderer = Renderer::new(window.clone()).await?;
        let scene = Scene::new();
        Ok(Self {
            renderer,
            scene,
            window,
            frame_context: FrameContext::default(),
            is_surface_configured: false
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.renderer.surface_config.width = width;
            self.renderer.surface_config.height = height;
            self.renderer.surface.configure(&self.renderer.device, &self.renderer.surface_config);
            self.is_surface_configured = true;
        }
    }

    pub fn render(&mut self) -> anyhow::Result<()> {
        if !self.is_surface_configured {
            return Ok(());
        }
        
        self.renderer.begin_frame(&mut self.frame_context)?;

        self.renderer.submit_camera(&self.scene.camera);
        self.scene.render(&mut self.renderer, &mut self.frame_context);

        self.renderer.end_frame(&mut self.frame_context);
        Ok(())
    }
}
