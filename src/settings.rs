//! Opciones del juego. El menú las modifica; `display::run_game(&settings)` las lee.

use crate::app::Tab;
use serde::{Deserialize, Serialize};
use std::{fs, io::ErrorKind};

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub fullscreen: bool,
    pub fps_limit: u32,
    pub fov: u32,
    pub color_aspect: u32,
    pub volume: u8,
}

impl Default for Settings {
    fn default() -> Self {
        Settings { fullscreen: false, fps_limit: 60, fov: 70, color_aspect: 32, volume: 80 }
    }
}

const FPS_OPTIONS: [u32; 4] = [30, 60, 120, 0];
const FOV_MIN: u32 = 50;
const FOV_MAX: u32 = 110;
const SETTINGS_FILE: &str = "settings.toml";
const COLOR_OPTIONS: [u32; 5] = [16, 32, 64, 128, 256];

impl Settings {
    /// Máscara que se aplica a cada canal RGB en texture.rs
    pub fn color_mask(&self) -> u32 {
        if !self.color_aspect.is_power_of_two() {
            return 0xF8;
        }
        let bits = self.color_aspect.trailing_zeros().min(8);   // 16 → 4 bits, 128 → 7, 256 → 8
        (0xFFu32 << (8 - bits)) & 0xFF
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
            (Tab::Video, 2) => {
                self.fov = (self.fov as i32 + dir * 5).clamp(FOV_MIN as i32, FOV_MAX as i32) as u32;
            }
            (Tab::Video, 3) => {
                let i = COLOR_OPTIONS.iter().position(|&l| l == self.color_aspect).unwrap_or(1) as i32;
                let n = COLOR_OPTIONS.len() as i32;
                self.color_aspect = COLOR_OPTIONS[(i + dir).rem_euclid(n) as usize];
            }
            (Tab::Sound, 0) => {
                self.volume = (self.volume as i32 + dir * 5).clamp(0, 100) as u8;
            }
            _ => {}
        }
    }

    /// Corrige cualquier valor fuera de lo permitido (archivo editado a mano o de otra versión).
    pub fn sanitized(mut self) -> Self {
        if !FPS_OPTIONS.contains(&self.fps_limit) {
            self.fps_limit = 60;
        }
        self.fov = self.fov.clamp(FOV_MIN, FOV_MAX);
        if !COLOR_OPTIONS.contains(&self.color_aspect) {
            self.color_aspect = 32;
        }
        self.volume = self.volume.min(100);
        self
    }

    /// Lee `settings.toml`. Nunca falla: si no existe o está roto, devuelve los valores por defecto.
    pub fn load() -> Self {
        match fs::read_to_string(SETTINGS_FILE) {
            Ok(text) => match toml::from_str::<Settings>(&text) {
                Ok(settings) => settings.sanitized(),
                Err(e) => {
                    eprintln!("{SETTINGS_FILE} no es válido ({e}); se usan los valores por defecto.");
                    Settings::default()
                }
            },
            Err(e) if e.kind() == ErrorKind::NotFound => Settings::default(),
            Err(e) => {
                eprintln!("No se pudo leer {SETTINGS_FILE}: {e}");
                Settings::default()
            }
        }
    }

    /// Escribe `settings.toml`. Un fallo al guardar no debe tumbar el juego: solo se avisa.
    pub fn save(&self) {
        match toml::to_string_pretty(self) {
            Ok(body) => {
                let text = format!("# Dungeon - opciones (se sobrescribe al salir del menu)\n{body}");
                if let Err(e) = fs::write(SETTINGS_FILE, text) {
                    eprintln!("No se pudo guardar {SETTINGS_FILE}: {e}");
                }
            }
            Err(e) => eprintln!("No se pudo serializar las opciones: {e}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let s = Settings { fov: 90, color_aspect: 16, ..Settings::default() };
        let text = toml::to_string_pretty(&s).unwrap();
        let back: Settings = toml::from_str(&text).unwrap();
        assert!(s == back);
    }

    #[test]
    fn missing_fields_use_defaults() {
        let s: Settings = toml::from_str("fov = 80").unwrap();
        assert_eq!(s.fov, 80);
        assert_eq!(s.fps_limit, 60);
    }

    #[test]
    fn invalid_values_are_sanitized() {
        let s: Settings =
            toml::from_str("fov = 500\nfps_limit = 45\ncolor_aspect = 17\nvolume = 200").unwrap();
        let s = s.sanitized();
        assert_eq!((s.fov, s.fps_limit, s.color_aspect, s.volume), (110, 60, 32, 100));
    }
}
