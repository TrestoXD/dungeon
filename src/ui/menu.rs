//! Fondo, título y los tres botones.

use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::Text,
    widgets::{Block, Paragraph},
};

use super::{MenuLayout, widgets::render_button};
use crate::app::{App, MenuItem};

pub fn draw(frame: &mut Frame, app: &App, layout: &MenuLayout) {
    let area = frame.area();

    // Fondo
    frame.render_widget(
        Block::default().style(Style::default().bg(Color::Blue)),
        area,
    );

    // Línea de ayuda arriba
    let help = Paragraph::new("◀ ▲ ▼ ▶ navigate | p: play | c: config | Esc: exit")
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center);
    frame.render_widget(help, Rect::new(0, 0, area.width, 1));

    draw_title(frame, layout.title);

    let buttons = [
        (MenuItem::Play, " PLAY ", layout.play),
        (MenuItem::Config, " CONFIG ", layout.config),
        (MenuItem::Exit, " EXIT ", layout.exit),
    ];
    for (item, label, rect) in buttons {
        let hovered = app.selected == item;
        let pressed = matches!(app.pressed, Some((p, _)) if p == item);
        render_button(frame, rect, label, hovered, pressed);
    }
}

fn draw_title(frame: &mut Frame, area: Rect) {
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
    let title_width = ascii_title
        .lines()
        .map(|l| l.chars().count())
        .max()
        .unwrap_or(0) as u16;

    let centered = Rect::new(
        area.x + area.width.saturating_sub(title_width) / 2,
        area.y,
        title_width.min(area.width),
        title_height.min(area.height),
    );

    let title = Paragraph::new(Text::from(ascii_title)).style(
        Style::default()
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );
    frame.render_widget(title, centered);
}
