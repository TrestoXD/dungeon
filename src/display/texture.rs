//! Texturas embebidas en el binario (sin decodificar PNG en runtime) y tablas de niebla/sombreado.

use crate::display::lighting::MAX_FIX;

pub const FOG_LEVELS: usize = 32;
pub const DEFAULT_COLOR_MASK: u32 = 0xF8;
pub const DEFAULT_FOG_SHIFT: u32 = 8;

const G_BASE: f32 = 0.50;
const G_GAIN: f32 = 0.35;
const B_BASE: f32 = 0.10;
const B_GAIN: f32 = 0.25;

pub struct Bitmap {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u32>,
}

impl Bitmap {
    pub fn from_rgb(bytes: &[u8], width: usize, height: usize) -> Bitmap {
        assert!(
            width.is_power_of_two() && height.is_power_of_two(),
            "las texturas deben ser potencia de 2"
        );
        assert_eq!(bytes.len(), width * height * 3);
        let pixels = bytes
            .chunks_exact(3)
            .map(|c| ((c[0] as u32) << 16) | ((c[1] as u32) << 8) | (c[2] as u32))
            .collect();
        Bitmap {
            width,
            height,
            pixels,
        }
    }
}

/// Textura con FOG_LEVELS variantes pre-sombreadas, una detrás de otra.
pub struct ShadedTexture {
    pub table: Vec<u32>,
    pub width: usize,
    pub height: usize,
    pub mask_u: i32,
    pub mask_v: i32,
    pub shift_v: u32,
    pub tile_shift: u32,
    pub fog_shift: u32,
    pub max_level: i32,
}

impl ShadedTexture {
    pub fn new(bmp: &Bitmap, mask: u32, fog_shift: u32) -> Self {
        let n = bmp.pixels.len();
        let rows = ((MAX_FIX >> fog_shift) + 1) as usize;
        let mut table = vec![0u32; rows * n];
        for level in 0..rows {
            let k = level as f32 / (rows - 1) as f32;
            let fr = k;
            let fg = k * (G_BASE + G_GAIN * k);
            let fb = k * (B_BASE + B_GAIN * k);
            for (i, &c) in bmp.pixels.iter().enumerate() {
                let r = (((c >> 16) & 0xff) as f32 * fr) as u32;
                let g = (((c >> 8) & 0xff) as f32 * fg) as u32;
                let b = ((c & 0xff) as f32 * fb) as u32;
                table[level * n + i] = ((r & mask) << 16) | ((g & mask) << 8) | (b & mask);
            }
        }
        ShadedTexture {
            table,
            width: bmp.width,
            height: bmp.height,
            mask_u: bmp.width as i32 - 1,
            mask_v: bmp.height as i32 - 1,
            shift_v: bmp.width.trailing_zeros(),
            tile_shift: n.trailing_zeros(),
            fog_shift,
            max_level: rows as i32 - 1,
        }
    }
}

pub struct Textures {
    /// Suelo y techo.
    pub block: ShadedTexture,
    /// Paredes.
    pub stone: ShadedTexture,
}

impl Textures {
    pub fn load(mask: u32, fog_shift: u32) -> Self {
        let block = Bitmap::from_rgb(include_bytes!("../../assets/cobblestone.rgb"), 16, 16);
        let stone = Bitmap::from_rgb(include_bytes!("../../assets/stone.rgb"), 16, 16);
        Textures {
            block: ShadedTexture::new(&block, mask, fog_shift),
            stone: ShadedTexture::new(&stone, mask, fog_shift),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fog_rows_follow_the_shift() {
        let bmp = Bitmap::from_rgb(&[255u8; 16 * 16 * 3], 16, 16);
        for (shift, rows) in [(10, 8), (9, 16), (8, 32), (7, 63), (6, 125), (5, 249)] {
            let t = ShadedTexture::new(&bmp, 0xFF, shift);
            assert_eq!(t.table.len(), rows * 256);
            assert_eq!(t.max_level, rows as i32 - 1);
            // la fila más clara conserva el rojo al 100 %
            assert_eq!((t.table[(rows - 1) * 256] >> 16) & 0xFF, 255);
        }
    }
}
