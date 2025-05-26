use nimbus_runtime::{Engine, WindowAttributes};

fn main() {
    let mut engine = Engine::with_window_attributes(
        WindowAttributes::default()
            .with_title("Nimbus Engine")
            .with_resizable(true)
    );
    engine.run();
}
