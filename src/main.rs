use crate::app::App;
use crate::errors::NimbusError;
use winit::event_loop::EventLoop;

mod app;
mod render;
mod errors;

pub type Result<T> = std::result::Result<T, NimbusError>;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    
    let event_loop = EventLoop::with_user_event().build()?;
    let mut app = App::new(&event_loop);
    event_loop.run_app(&mut app)?;
    Ok(())
}