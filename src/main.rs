use color_eyre::eyre::{Ok, Result};
use crossterm::{
    event::{self, Event, KeyCode, MouseButton, MouseEventKind},
    ExecutableCommand,
};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
    DefaultTerminal, Frame,
};
use std::io::stdout;

mod game;

#[derive(PartialEq, Clone, Copy)]
enum MenuOption {
    Play,
    Exit,
}

fn main() -> Result<()> {
    println!("Welcome to Dungeon, thanks for playing my game!");
    color_eyre::install()?;

    // Habilitar el ratón
    stdout().execute(event::EnableMouseCapture)?;

    let mut terminal = ratatui::init();

    // Fijamos el tamaño típico de una terminal clásica (80 columnas x 24 filas)
    terminal.resize(Rect::new(0, 0, 80, 24))?;

    let result = run(terminal);

    // Restaurar al salir
    stdout().execute(event::DisableMouseCapture)?;
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal) -> Result<()> {
    let mut current_selection = MenuOption::Play;

    loop {
        // Get real window size
        let size = terminal.size()?;

        // button config (CORRECCIÓN: Quitamos el mut redundante)
        let button_width = 20;
        let button_height = 3;
        let button_x = (size.width.saturating_sub(button_width)) / 2;
        let center_y = size.height.saturating_sub(button_height) / 2;

        let play_area = Rect::new(button_x, center_y + 1, button_width, button_height);
        let exit_area = Rect::new(button_x, center_y + 6, button_width, button_height);

        // TITLE title config (CORRECCIÓN: Quitamos el mut redundante)
        let title_area = Rect::new(0, 2, size.width, 12);

        // Rendering pasándole el área calculada
        terminal.draw(|frame| {
            render(frame);
            render_title(frame, title_area);

            // Render de PLAY (CORRECCIÓN: Pasamos los estados dinámicos limpios)
            let play_focused = current_selection == MenuOption::Play;
            render_button(frame, play_area, " PLAY ", play_focused, false);

            // Render de EXIT (CORRECCIÓN: Pasamos los estados dinámicos limpios)
            let exit_focused = current_selection == MenuOption::Exit;
            render_button(frame, exit_area, " EXIT ", exit_focused, false);
        })?;

        // Event Management
        if event::poll(std::time::Duration::from_millis(16))? {
            match event::read()? {
                // Keyboard
                Event::Key(key) => match key.code {
                    KeyCode::Esc => break,

                    // ARRIBA te lleva siempre a PLAY de forma fija
                    KeyCode::Up => {
                        current_selection = MenuOption::Play;
                    }

                    // ABAJO te lleva siempre a EXIT de forma fija
                    KeyCode::Down => {
                        current_selection = MenuOption::Exit;
                    }

                    KeyCode::Enter => {
                        let _ = terminal.draw(|f| {
                            let p_active = current_selection == MenuOption::Play;
                            let e_active = current_selection == MenuOption::Exit;
                            render_button(f, play_area, " PLAY ", p_active, p_active);
                            render_button(f, exit_area, " EXIT ", e_active, e_active);
                        });
                        std::thread::sleep(std::time::Duration::from_millis(80));

                        if current_selection == MenuOption::Exit { break; }
                        else{
                            let _ = stdout().execute(event::DisableMouseCapture);
                            ratatui::restore();

                            game::run_game()?;
                            return Ok(());
                        }
                    }
                    _ => {}
                },

                // Mouse
                Event::Mouse(mouse_event) => {
                    let inside_play = mouse_event.column >= play_area.left() && mouse_event.column < play_area.right()
                        && mouse_event.row >= play_area.top() && mouse_event.row < play_area.bottom();

                    let inside_exit = mouse_event.column >= exit_area.left() && mouse_event.column < exit_area.right()
                        && mouse_event.row >= exit_area.top() && mouse_event.row < exit_area.bottom();

                    if inside_play {
                        current_selection = MenuOption::Play;
                        if mouse_event.kind == MouseEventKind::Down(MouseButton::Left) {
                            let _ = terminal.draw(|f| render_button(f, play_area, " PLAY ", true, true));
                            std::thread::sleep(std::time::Duration::from_millis(80));
                            let _ = stdout().execute(event::DisableMouseCapture);
                            ratatui::restore();

                            game::run_game()?;
                            return Ok(());
                        }
                    } else if inside_exit {
                        current_selection = MenuOption::Exit;
                        if mouse_event.kind == MouseEventKind::Down(MouseButton::Left) {
                            let _ = terminal.draw(|f| render_button(f, exit_area, " EXIT ", true, true));
                            std::thread::sleep(std::time::Duration::from_millis(80));
                            break;
                        }
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn render(frame: &mut Frame) {
    // Set Background (CORRECCIÓN: Cambiado a Cyan para que combine con tus botones)
    let full_background = Block::default().style(Style::default().bg(Color::Blue));
    frame.render_widget(full_background, frame.area());
}

fn render_title(frame: &mut Frame, area: Rect) {
    let ascii_title = r#"
 ▄▄▄▄▄▄
█▀██▀▀██
  ██   ██       ▄        ▄▄             ▄
  ██   ██ ██ ██ ████▄ ▄████ ▄█▀█▄ ▄███▄ ████▄
▄ ██   ██ ██ ██ ██ ██ ██ ██ ██▄█▀ ██ ██ ██ ██
▀██▀███▀ ▄▀██▀█▄██ ▀█▄▀████▄▀█▄▄▄▄▀███▀▄██ ▀█
                         ██
                       ▀▀▀
    "#;

    let title_height = ascii_title.lines().filter(|l| !l.is_empty()).count() as u16;
    let title_width = ascii_title.lines().map(|l| l.chars().count()).max().unwrap_or(0) as u16;

    let centered_title_area = Rect::new(
        area.x + (area.width.saturating_sub(title_width)) / 2,
        area.y,
        title_width.min(area.width),
        title_height.min(area.height),
    );

    let title = Paragraph::new(ratatui::text::Text::from(ascii_title))
        .style(Style::default().fg(Color::Black).add_modifier(Modifier::BOLD));
    frame.render_widget(title, centered_title_area);
}

fn render_button(frame: &mut Frame, area: Rect, label: &str, is_hovered: bool, is_pressed: bool) {
    if !is_pressed {
        let shadow_area = Rect::new(
            area.x + 1,
            area.y + 1,
            area.width,
            area.height,
        );

        let shadow = Block::default().style(Style::default().bg(Color::Black));
        frame.render_widget(shadow, shadow_area);
    }

    let style = match (is_pressed, is_hovered) {
        (true, _) => {
            // When pressed: text cyan and background black
            Style::default().fg(Color::Blue).bg(Color::Black).add_modifier(Modifier::BOLD)
        }
        (false, true) => {
            // On hovered: text black and background yellow
            Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD)
        }
        (false, false) => {
            // Normal State: Black Text and gray Background
            Style::default().fg(Color::Black).bg(Color::Gray)
        }
    };

    let button = Paragraph::new(label)
        .style(style)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(button, area);
}
