//! Estado del menú + reacción a teclado y ratón. Aquí NO se dibuja nada.

use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;

use crate::settings::Settings;
use crate::ui::MenuLayout;

/// Lo que dura el botón "hundido" antes de ejecutar su acción (antes era un sleep que congelaba todo).
const PRESS_TIME: Duration = Duration::from_millis(80);

// ---------- Botones del menú principal ----------

#[derive(Clone, Copy, PartialEq)]
pub enum MenuItem {
    Play,
    Config,
    Exit,
}

impl MenuItem {
    const ALL: [MenuItem; 3] = [MenuItem::Play, MenuItem::Config, MenuItem::Exit];

    fn index(self) -> usize {
        Self::ALL.iter().position(|&m| m == self).unwrap_or(0)
    }
    pub fn next(self) -> Self {
        Self::ALL[(self.index() + 1) % Self::ALL.len()]
    }
    pub fn prev(self) -> Self {
        Self::ALL[(self.index() + Self::ALL.len() - 1) % Self::ALL.len()]
    }
}

// ---------- Pestañas de configuración ----------

#[derive(Clone, Copy, PartialEq)]
pub enum Tab {
    Video,
    Sound,
    Controls,
}

impl Tab {
    pub const ALL: [Tab; 3] = [Tab::Video, Tab::Sound, Tab::Controls];

    pub fn title(self) -> &'static str {
        match self {
            Tab::Video => "Video",
            Tab::Sound => "Sound",
            Tab::Controls => "Controller",
        }
    }
    pub fn index(self) -> usize {
        Self::ALL.iter().position(|&t| t == self).unwrap_or(0)
    }
    pub fn next(self) -> Self {
        Self::ALL[(self.index() + 1) % Self::ALL.len()]
    }
    pub fn prev(self) -> Self {
        Self::ALL[(self.index() + Self::ALL.len() - 1) % Self::ALL.len()]
    }
}

pub struct ConfigState {
    pub tab: Tab,
    pub row: usize,
}

/// Un solo estado en vez de `config_popup` + `buttons_enabled`.
pub enum Screen {
    Menu,
    Config(ConfigState),
}

/// Lo que `app` le pide a `main`.
pub enum Action {
    None,
    Quit,
    StartGame,
}

// ---------- App ----------

pub struct App {
    pub selected: MenuItem,
    pub screen: Screen,
    pub settings: Settings,
    /// Botón recién pulsado y desde cuándo (para el efecto "hundido").
    pub pressed: Option<(MenuItem, Instant)>,
}

impl Default for App {
    fn default() -> Self {
        App {
            selected: MenuItem::Play,
            screen: Screen::Menu,
            settings: Settings::default(),
            pressed: None,
        }
    }
}

impl App {
    /// Se llama una vez por vuelta del bucle: ejecuta la acción cuando termina el efecto del botón.
    pub fn tick(&mut self) -> Action {
        let Some((item, since)) = self.pressed else { return Action::None };
        if since.elapsed() < PRESS_TIME {
            return Action::None;
        }
        self.pressed = None;
        match item {
            MenuItem::Play => Action::StartGame,
            MenuItem::Config => {
                self.open_config();
                Action::None
            }
            MenuItem::Exit => Action::Quit,
        }
    }

    /// Menú nuevo que conserva las opciones de la partida anterior.
    pub fn with_settings(settings: Settings) -> Self {
        App { settings, ..App::default() }
    }

    pub fn on_key(&mut self, key: KeyEvent) -> Action {
        if key.kind != KeyEventKind::Press {
            return Action::None;
        }
        if matches!(self.screen, Screen::Config(_)) {
            self.config_key(key.code);
            Action::None
        } else {
            self.menu_key(key.code)
        }
    }

    pub fn on_mouse(&mut self, m: MouseEvent, layout: &MenuLayout) -> Action {
        // Con el popup abierto o mientras un botón se hunde, el ratón no hace nada.
        if !matches!(self.screen, Screen::Menu) || self.pressed.is_some() {
            return Action::None;
        }
        let buttons = [
            (MenuItem::Play, layout.play),
            (MenuItem::Config, layout.config),
            (MenuItem::Exit, layout.exit),
        ];
        let hovered = buttons
            .into_iter()
            .find(|&(_, rect)| hit(rect, m.column, m.row))
            .map(|(item, _)| item);

        if let Some(item) = hovered {
            self.selected = item;
            if m.kind == MouseEventKind::Down(MouseButton::Left) {
                self.press(item);
            }
        }
        Action::None
    }

    fn press(&mut self, item: MenuItem) {
        self.pressed = Some((item, Instant::now()));
    }

    fn menu_key(&mut self, code: KeyCode) -> Action {
        if self.pressed.is_some() {
            return Action::None;
        }
        match code {
            KeyCode::Esc => return Action::Quit,
            KeyCode::Char('p') => return Action::StartGame,
            KeyCode::Char('c') => self.open_config(),
            KeyCode::Up => self.selected = self.selected.prev(),
            KeyCode::Down => self.selected = self.selected.next(),
            KeyCode::Enter => self.press(self.selected),
            _ => {}
        }
        Action::None
    }

    fn config_key(&mut self, code: KeyCode) {
        if matches!(code, KeyCode::Esc | KeyCode::Char('c')) {
            self.screen = Screen::Menu;
            return;
        }
        let Screen::Config(cfg) = &mut self.screen else { return };
        match code {
            KeyCode::Tab => {
                cfg.tab = cfg.tab.next();
                cfg.row = 0;
            }
            KeyCode::BackTab => {
                cfg.tab = cfg.tab.prev();
                cfg.row = 0;
            }
            KeyCode::Up => cfg.row = cfg.row.saturating_sub(1),
            KeyCode::Down => {
                let n = self.settings.rows(cfg.tab).len();
                if n > 0 {
                    cfg.row = (cfg.row + 1).min(n - 1);
                }
            }
            KeyCode::Left => self.settings.adjust(cfg.tab, cfg.row, -1),
            KeyCode::Right => self.settings.adjust(cfg.tab, cfg.row, 1),
            _ => {}
        }
    }

    fn open_config(&mut self) {
        self.screen = Screen::Config(ConfigState { tab: Tab::Video, row: 0 });
    }
}

fn hit(r: Rect, col: u16, row: u16) -> bool {
    col >= r.left() && col < r.right() && row >= r.top() && row < r.bottom()
}
