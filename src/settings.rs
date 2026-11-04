//! Opciones del juego. El menú las modifica; `display::run_game(&settings)` las lee.

use crate::app::Tab;

#[derive(Clone)]
pub struct Settings {
    pub fullscreen: bool,
    pub fps_limit: u32, // 0 = sin límite
    pub fov: u32,
    pub color_aspect: u32,
    pub volume: u8,     // 0..=100
}

impl Default for Settings {
    fn default() -> Self {
        Settings { fullscreen: false, fps_limit: 60, fov: 70, color_aspect: 32, volume: 80 }
    }
}

const FPS_OPTIONS: [u32; 4] = [30, 60, 120, 0];
const COLOR_OPTIONS: [(u32, u32); 4] = [(256, 0xFF), (64, 0xFC), (32, 0xF8), (16, 0xF0)];

impl Settings {
    /// Máscara que se aplica a cada canal RGB en texture.rs
    pub fn color_mask(&self) -> u32 {
        COLOR_OPTIONS
            .iter()
            .find(|&&(levels, _)| levels == self.color_aspect)
            .map(|&(_, mask)| mask)
            .unwrap_or(0xF8)
    }

    /// Las filas que se muestran en cada pestaña. Es la ÚNICA fuente de verdad:
    /// la UI las dibuja y `app.rs` usa `.len()` para saber cuántas hay.
    pub fn rows(&self, tab: Tab) -> Vec<String> {
        match tab {
            Tab::Video => {
                let fs = if self.fullscreen { "ON" } else { "OFF" };
                let fps = if self.fps_limit == 0 {
                    "Unlimited".to_string()
                } else {
                    self.fps_limit.to_string()
                };
                vec![
                    format!("FullScreen         < {fs} >"),
                    format!("Frames Per Second  < {fps} >"),
                    format!("FOV                < {} >", self.fov),
                    format!("Color Aspect       < {} >", self.color_aspect),
                ]
            }
            Tab::Sound => vec![format!("Volume             < {} >", self.volume)],
            Tab::Controls => vec![],
        }
    }

    /// Cambia la opción de la fila `row` de la pestaña `tab`. `dir` es -1 (izquierda) o +1 (derecha).
    /// El orden de filas debe coincidir con `rows()`.
    pub fn adjust(&mut self, tab: Tab, row: usize, dir: i32) {
        match (tab, row) {
            (Tab::Video, 0) => self.fullscreen = !self.fullscreen,
            (Tab::Video, 1) => {
                let i = FPS_OPTIONS.iter().position(|&f| f == self.fps_limit).unwrap_or(1) as i32;
                self.fps_limit = FPS_OPTIONS[(i + dir).rem_euclid(FPS_OPTIONS.len() as i32) as usize];
            }
            (Tab::Video, 2) => { self.fov = (self.fov as i32 + dir * 5).clamp(50, 110) as u32; }
            (Tab::Video, 3) => {
                let i = COLOR_OPTIONS
                    .iter()
                    .position(|&(levels, _)| levels == self.color_aspect)
                    .unwrap_or(2) as i32;
                let n = COLOR_OPTIONS.len() as i32;
                self.color_aspect = COLOR_OPTIONS[(i + dir).rem_euclid(n) as usize].0;
            }
            (Tab::Sound, 0) => {
                self.volume = (self.volume as i32 + dir * 5).clamp(0, 100) as u8;
            }
            _ => {}
        }
    }
}
