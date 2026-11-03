//! Todo lo que dibuja. Solo LEE `App`, nunca la modifica.

mod config;
mod menu;
mod widgets;

use ratatui::{layout::Rect, Frame};

use crate::app::{App, Screen};

/// Dónde está cada cosa del menú. La usan el dibujo Y el ratón, así nunca se desincronizan.
pub struct MenuLayout {
    pub title: Rect,
    pub play: Rect,
    pub config: Rect,
    pub exit: Rect,
}

pub fn menu_layout(area: Rect) -> MenuLayout {
    let (w, h) = (20, 3);
    let x = area.width.saturating_sub(w) / 2;
    let y = area.height.saturating_sub(h) / 2;
    MenuLayout {
        title: Rect::new(0, 2, area.width, 12),
        play: Rect::new(x, y + 1, w, h),
        config: Rect::new(x, y + 6, w, h),
        exit: Rect::new(x, y + 11, w, h),
    }
}

pub fn draw(frame: &mut Frame, app: &App) {
    let layout = menu_layout(frame.area());
    menu::draw(frame, app, &layout);
    // El popup se dibuja encima, solo si estamos en la pantalla de config.
    if let Screen::Config(cfg) = &app.screen {
        config::draw(frame, cfg, &app.settings);
    }
}
