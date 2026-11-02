use color_eyre::eyre::{Ok, Result};
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

// Estructura mínima para guardar la ventana
struct GameApp {
    window: Option<Arc<Window>>,
}

impl ApplicationHandler for GameApp {
    // Se ejecuta al arrancar para crear la ventana física
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title("Dungeon")
                .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0));

            if let std::result::Result::Ok(window) = event_loop.create_window(window_attributes) {
                self.window = Some(Arc::new(window));
            }
        }
    }

    // Manejador de eventos de la ventana (cerrar y teclado)
    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            // Cerrar con la X
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            // Cerrar con la tecla Escape
            WindowEvent::KeyboardInput { event: key_event, .. } => {
                if key_event.state.is_pressed() {
                    if let winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape) = key_event.logical_key {
                        event_loop.exit();
                    }
                }
            }
            _ => {}
        }
    }
}

// Función que se invoca desde el main.rs al pulsar PLAY
pub fn run_game() -> Result<()> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Wait); // Modo eficiente para ventanas básicas

    let mut app = GameApp { window: None };
    event_loop.run_app(&mut app)?;

    Ok(())
}
