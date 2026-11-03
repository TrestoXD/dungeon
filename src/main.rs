use std::{io::stdout, time::Duration};

use color_eyre::Result;
use crossterm::{
    event::{self, Event},
    ExecutableCommand,
};
use ratatui::{layout::Rect, DefaultTerminal};

// Módulos nuevos del menú
mod app;
mod display; // tu run_game() de antes
mod settings;
mod ui;

// Módulos del motor del juego (los que ya tienes)


use app::{Action, App};
use settings::Settings;

fn main() -> Result<()> {
    color_eyre::install()?;

    stdout().execute(event::EnableMouseCapture)?;
    let mut terminal = ratatui::init();
    terminal.resize(Rect::new(0, 0, 80, 24))?;

    let outcome = run(&mut terminal);

    // Un único sitio donde se restaura el terminal, pase lo que pase.
    let _ = stdout().execute(event::DisableMouseCapture);
    ratatui::restore();

    // Solo con el terminal ya restaurado se abre la ventana del juego.
    if let Some(settings) = outcome? {
        display::run_game(&settings)?;
    }
    Ok(())
}

/// Devuelve `Some(settings)` si hay que lanzar el juego, `None` si se sale.
fn run(terminal: &mut DefaultTerminal) -> Result<Option<Settings>> {
    let mut app = App::default();

    loop {
        // 1. Dibujar
        terminal.draw(|frame| ui::draw(frame, &app))?;

        // 2. Esperar un evento (o que termine el efecto de un botón)
        let mut action = app.tick();
        if matches!(action, Action::None) && event::poll(Duration::from_millis(16))? {
            action = match event::read()? {
                Event::Key(key) => app.on_key(key),
                Event::Mouse(mouse) => {
                    let size = terminal.size()?;
                    let layout = ui::menu_layout(Rect::new(0, 0, size.width, size.height));
                    app.on_mouse(mouse, &layout)
                }
                _ => Action::None,
            };
        }

        // 3. Actuar
        match action {
            Action::None => {}
            Action::Quit => return Ok(None),
            Action::StartGame => return Ok(Some(app.settings.clone())),
        }
    }
}
