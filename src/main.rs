use std::{io::stdout, time::Duration};

use color_eyre::Result;
use crossterm::{
    event::{self, Event},
    ExecutableCommand,
};
use ratatui::{layout::Rect, DefaultTerminal};
use winit::event_loop::EventLoop;

mod app;
mod display;
mod settings;
mod ui;

use app::{Action, App};
use settings::Settings;

/// Lo que devuelve el menú al terminar.
struct MenuResult {
    settings: Settings,
    start_game: bool,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    // winit solo permite UN EventLoop por proceso: se crea aquí y se reutiliza en cada partida.
    let mut event_loop = EventLoop::new()?;
    let mut settings = Settings::load();

    loop {
        let before = settings.clone();
        let result = run_menu(settings)?;
        settings = result.settings;

        // Solo se escribe en disco si algo cambió.
        if settings != before {
            settings.save();
        }
        if !result.start_game {
            break;
        }
        display::run_game(&settings, &mut event_loop)?;
    }
    Ok(())
}

/// Abre el terminal, ejecuta el menú y lo restaura SIEMPRE (aunque `run` falle).
fn run_menu(settings: Settings) -> Result<MenuResult> {
    stdout().execute(event::EnableMouseCapture)?;
    let mut terminal = ratatui::init();

    let outcome = run(&mut terminal, settings);

    let _ = stdout().execute(event::DisableMouseCapture);
    ratatui::restore();
    outcome
}

fn run(terminal: &mut DefaultTerminal, settings: Settings) -> Result<MenuResult> {
    terminal.resize(Rect::new(0, 0, 80, 24))?;
    let mut app = App::with_settings(settings);

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
            Action::Quit => return Ok(MenuResult { settings: app.settings, start_game: false }),
            Action::StartGame => return Ok(MenuResult { settings: app.settings, start_game: true }),
        }
    }
}