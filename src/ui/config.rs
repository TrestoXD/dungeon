//! Popup de configuración: pestañas arriba, contenido en medio, ayuda abajo.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Margin},
    style::{Color, Modifier, Style},
    symbols,
    widgets::{Block, Clear, List, ListState, Paragraph, Tabs},
};
use ratatui::widgets::{ LineGauge};
use crate::app::{ConfigState, Tab};
use crate::settings::Settings;

pub fn draw(frame: &mut Frame, cfg: &ConfigState, settings: &Settings) {
    let area = frame
        .area()
        .centered(Constraint::Percentage(60), Constraint::Percentage(50));

    // Limpiar lo que hubiera debajo y dibujar el marco
    frame.render_widget(Clear, area);
    let block = Block::bordered()
        .title(" CONFIGURATION ")
        .style(Style::default().fg(Color::White).bg(Color::Black));
    let inner = block.inner(area).inner(Margin::new(1, 1));
    frame.render_widget(block, area);

    let [tabs_area, content_area, help_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(inner);

    // Pestañas
    let titles: Vec<&str> = Tab::ALL.iter().map(|t| t.title()).collect();
    let tabs = Tabs::new(titles)
        .select(cfg.tab.index())
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .divider(symbols::DOT)
        .padding(" ", " ");
    frame.render_widget(tabs, tabs_area);

    // Contenido de la pestaña activa
    match cfg.tab {
        Tab::Controls => {
            let text = "W A S D  move\nShift    crouch\nCtrl     sprint\nSpace    jump\nMouse    look\nF3       debug";
            frame.render_widget(Paragraph::new(text), content_area);
        }
        Tab::Sound => {
            let volumegauge = LineGauge::default()
                .label( format!("Master Volume: {:.0}", settings.volume))
                .filled_symbol(symbols::block::FULL)
                .ratio((settings.volume) as f64 / 100.0);
            frame.render_widget(volumegauge, content_area);
        }
        tab => {
            let mut state = ListState::default().with_selected(Some(cfg.row));
            let list = List::new(settings.rows(tab))
                .highlight_style(
                    Style::default()
                        .bg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol("> ");
            frame.render_stateful_widget(list, content_area, &mut state);
        }
    }

    // Ayuda
    let help = Paragraph::new("Tab: tab | ▲ ▼: option | ◀ ▶: change | Esc: back")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    frame.render_widget(help, help_area);
}
