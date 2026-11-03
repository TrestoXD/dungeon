//! Fuente 3x5 mínima, solo lo necesario para el overlay de depuración (F3).

fn glyph(c: char) -> [u8; 5] {
    match c {
        '0' | 'O' => [0b111, 0b101, 0b101, 0b101, 0b111],
        '1' => [0b010, 0b110, 0b010, 0b010, 0b111],
        '2' => [0b111, 0b001, 0b111, 0b100, 0b111],
        '3' => [0b111, 0b001, 0b111, 0b001, 0b111],
        '4' => [0b101, 0b101, 0b111, 0b001, 0b001],
        '5' | 'S' => [0b111, 0b100, 0b111, 0b001, 0b111],
        '6' => [0b111, 0b100, 0b111, 0b101, 0b111],
        '7' => [0b111, 0b001, 0b001, 0b010, 0b010],
        '8' => [0b111, 0b101, 0b111, 0b101, 0b111],
        '9' => [0b111, 0b101, 0b111, 0b001, 0b111],
        'F' => [0b111, 0b100, 0b110, 0b100, 0b100],
        'P' => [0b111, 0b101, 0b111, 0b100, 0b100],
        'B' => [0b110, 0b101, 0b110, 0b101, 0b110],
        'L' => [0b100, 0b100, 0b100, 0b100, 0b111],
        'C' => [0b111, 0b100, 0b100, 0b100, 0b111],
        'K' => [0b101, 0b101, 0b110, 0b101, 0b101],
        ':' => [0b000, 0b010, 0b000, 0b010, 0b000],
        '-' => [0b000, 0b000, 0b111, 0b000, 0b000],
        _ => [0; 5],
    }
}

/// Ancho en píxeles de pantalla de `text` con píxeles de fuente de `px`x`px`.
pub fn text_width(text: &str, px: usize) -> usize {
    text.chars().count() * 4 * px
}

#[allow(clippy::too_many_arguments)]
pub fn draw_text(
    buf: &mut [u32],
    stride: usize,
    rows: usize,
    x: usize,
    y: usize,
    px: usize,
    color: u32,
    text: &str,
) {
    let mut cx = x;
    for ch in text.chars() {
        let g = glyph(ch.to_ascii_uppercase());
        for (gy, &row) in g.iter().enumerate() {
            for gx in 0..3usize {
                if (row >> (2 - gx)) & 1 == 0 {
                    continue;
                }
                let x0 = cx + gx * px;
                let y0 = y + gy * px;
                for dy in 0..px {
                    let yy = y0 + dy;
                    if yy >= rows {
                        break;
                    }
                    for dx in 0..px {
                        let xx = x0 + dx;
                        if xx >= stride {
                            break;
                        }
                        buf[yy * stride + xx] = color;
                    }
                }
            }
        }
        cx += 4 * px;
    }
}
