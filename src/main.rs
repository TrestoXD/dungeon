use color_eyre::eyre::{Ok, Result};
use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode, MouseButton, MouseEventKind},
};
use ratatui::{layout::{Alignment, Rect}, style::{Color, Modifier, Style}, symbols, widgets::{Block, Borders, Paragraph}, DefaultTerminal, Frame};
use std::io::stdout;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::{Clear, Tabs};

mod display;

#[derive(PartialEq, Clone, Copy)]
enum MenuOption {
    Play,
    Config,
    Exit,
}

fn main() -> Result<()> {
    println!("Welcome to Dungeon, thanks for playing my game!");
    color_eyre::install()?;

    // Enable Mouse Capture
    stdout().execute(event::EnableMouseCapture)?;

    let mut terminal = ratatui::init();

    // Fix the classic terminal size (800x600)
    terminal.resize(Rect::new(0, 0, 80, 24))?;

    let result = run(terminal);

    // Restore when exit
    stdout().execute(event::DisableMouseCapture)?;
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal) -> Result<()> {
    // Default Selection when opening the program
    let mut current_selection = MenuOption::Play;
    let mut buttons_enabled = true;
    // Config
    // is config popup enabled?
    let mut config_popup = false;
    // config selection
    let mut config_selection = 0;

    loop {
        // Get real window size
        let size = terminal.size()?;

        // Button config
        let button_width = 20;
        let button_height = 3;
        let button_x = (size.width.saturating_sub(button_width)) / 2;
        let center_y = size.height.saturating_sub(button_height) / 2;

        let play_area = Rect::new(button_x, center_y + 1, button_width, button_height);
        let config_area = Rect::new(button_x, center_y + 6, button_width, button_height);
        let exit_area = Rect::new(button_x, center_y + 11, button_width, button_height);

        // Title config
        let title_area = Rect::new(0, 2, size.width, 12);

        // Rendering with calculated area
        terminal.draw(|frame| {
            render(frame);
            render_title(frame, title_area);

            // Render PLAY
            let play_focused = current_selection == MenuOption::Play;
            render_button(frame, play_area, " PLAY ", play_focused, false);

            // Render EXIT
            let exit_focused = current_selection == MenuOption::Exit;
            render_button(frame, exit_area, " EXIT ", exit_focused, false);

            // Render CONFIG
            let config_focused = current_selection == MenuOption::Config;
            render_button(frame, config_area, " CONFIG ", config_focused, false);

            render_config(frame, config_popup, config_selection);
            render_config_content(frame, config_popup, config_selection);
        })?;

        // Event Management
        if event::poll(std::time::Duration::from_millis(16))? {
            match event::read()? {
                // Keyboard
                Event::Key(key) => {
                    if key.kind == crossterm::event::KeyEventKind::Press{
                        match key.code {
                            KeyCode::Esc => break,

                            // Open the Config when C is pressed
                            KeyCode::Char('c') => {
                                config_popup = !config_popup;
                                buttons_enabled = !buttons_enabled;
                            },

                            KeyCode::Char('p') => {
                                if buttons_enabled {
                                    let _ = stdout().execute(event::DisableMouseCapture);
                                    ratatui::restore();

                                    display::run_game()?;
                                    return Ok(());
                                }
                            }

                            // Up navigation
                            KeyCode::Up => {
                                if buttons_enabled {
                                    current_selection = match current_selection {
                                        MenuOption::Play => MenuOption::Exit,
                                        MenuOption::Config => MenuOption::Play,
                                        MenuOption::Exit => MenuOption::Config,
                                    };
                                }
                            }

                            // Down navigation
                            KeyCode::Down => {
                                if buttons_enabled {
                                    current_selection = match current_selection {
                                        MenuOption::Play => MenuOption::Config,
                                        MenuOption::Config => MenuOption::Exit,
                                        MenuOption::Exit => MenuOption::Play,
                                    };
                                }
                            }

                            KeyCode::Right => config_selection = (config_selection + 1) % 3,
                            KeyCode::Left => config_selection = (config_selection + 2) % 3,

                            // Enter
                            KeyCode::Enter => {
                                if buttons_enabled{
                                    let _ = terminal.draw(|f| {
                                        let p_active = current_selection == MenuOption::Play;
                                        let e_active = current_selection == MenuOption::Exit;
                                        let c_active = current_selection == MenuOption::Config;
                                        render_button(f, play_area, " PLAY ", p_active, p_active);
                                        render_button(f, exit_area, " EXIT ", e_active, e_active);
                                        render_button(f, config_area, " CONFIG ", c_active, c_active);
                                    });
                                    std::thread::sleep(std::time::Duration::from_millis(80));

                                    if current_selection == MenuOption::Exit {
                                        break;
                                    }else if current_selection == MenuOption::Config{
                                        config_popup = !config_popup;
                                        buttons_enabled = !buttons_enabled;
                                    } else {
                                        let _ = stdout().execute(event::DisableMouseCapture);
                                        ratatui::restore();

                                        display::run_game()?;
                                        return Ok(());
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }

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

                            display::run_game()?;
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
    // Set Background
    let full_background = Block::default().style(Style::default().bg(Color::Blue));
    frame.render_widget(full_background, frame.area());
}

fn render_config(frame: &mut Frame, show_popup: bool, selected_tab: usize) {
    let area = frame.area();

    let layout = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]);
    let [instructions, content] = area.layout(&layout);

    frame.render_widget(
        Line::from("To navigate use: ◀ ▲ ▼ ▶ | play: p | config: c | exit: esc").centered(),
        instructions,
    );

    frame.render_widget(Block::bordered(), content);

    if show_popup{
        let popup_block = Block::bordered().title("Config");
        let centered_area = area.centered(Constraint::Percentage(60), Constraint::Percentage(50));
        // clears out any background in the area before rendering the popup
        frame.render_widget(Clear, centered_area);

        let tabs = Tabs::new(vec!["Video", "Sound", "Controller"])
            .block(popup_block)
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().fg(Color::Yellow).on_black().bold())
            .select(selected_tab)
            .divider(symbols::DOT)
            .padding(" ", " ");
        frame.render_widget(tabs, centered_area);
    }
}
fn render_config_content(frame: &mut Frame, show_popup: bool, selected_tab: usize) {
    let area = frame.area();
    let centered_area = area.centered(Constraint::Percentage(60), Constraint::Percentage(50));

    let text = match selected_tab {
        0 => "Super cool Video settings.".into(),
        1 => "Nah this is cooler sound Settings.".into(),
        2 => "And this too controller Settings!".bold(),
        _ => unreachable!(),
    };
    let block = Paragraph::new(text)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center);

    if show_popup{
        frame.render_widget(block, centered_area);
    }
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

    // We calculate the height and width based on the ascii_title variable
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
    // Not pressed -> Render the shadow
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

    // Create the button
    let button = Paragraph::new(label)
        .style(style)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    // Render for me pawn!
    frame.render_widget(button, area);
}
