//! Sprites: imágenes planas que siempre miran a la cámara (billboards).

use crate::display::texture::Bitmap;

#[derive(Clone, Copy, PartialEq)]
pub enum SpriteKind {
    Waifu,
}

pub struct Sprite {
    pub kind: SpriteKind,
    pub x: f64,       // unidades de mundo (1 celda = 16)
    pub z: f64,
    pub height: f32,  // en celdas; el ancho sale de las proporciones de la imagen
}

impl Sprite {
    pub fn new(kind: SpriteKind, x: f64, z: f64) -> Self {
        Sprite { kind, x, z, height: 1.5 }
    }
}

/// Tamaño de waifu.rgba. Si la imagen es cuadrada se deduce sola; si no lo es,
/// pon aquí su tamaño real, por ejemplo `Some((32, 48))`.
const WAIFU_SIZE: Option<(usize, usize)> = Some((49,128));

/// Imágenes de los sprites. El orden debe coincidir con el de `SpriteKind`.
pub fn load_images() -> Vec<Bitmap> {
    vec![load("waifu.rgba", include_bytes!("../../assets/waifu.rgba"), WAIFU_SIZE)]
}

fn load(name: &str, bytes: &[u8], size: Option<(usize, usize)>) -> Bitmap {
    let (w, h) = size.unwrap_or_else(|| {
        let side = ((bytes.len() / 4) as f64).sqrt().round() as usize;
        assert!(
            side * side * 4 == bytes.len(),
            "{name} no es cuadrada ({} bytes): indica su tamaño en WAIFU_SIZE",
            bytes.len()
        );
        (side, side)
    });
    Bitmap::from_rgba_any(bytes, w, h)
}