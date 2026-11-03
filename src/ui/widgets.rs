//! Piezas reutilizables.

use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render_button(frame: &mut Frame, area: Rect, label: &str, is_hovered: bool, is_pressed: bool) {
    // Sin pulsar -> sombra
    if !is_pressed {
        let shadow_area = Rect::new(area.x + 1, area.y + 1, area.width, area.height);
        let shadow = Block::default().style(Style::default().bg(Color::Black));
        frame.render_widget(shadow, shadow_area);
    }

    let style = match (is_pressed, is_hovered) {
        (true, _) => Style::default().fg(Color::Blue).bg(Color::Black).add_modifier(Modifier::BOLD),
        (false, true) => Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD),
        (false, false) => Style::default().fg(Color::Black).bg(Color::Gray),
    };

    let button = Paragraph::new(label)
        .style(style)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(button, area);
}
