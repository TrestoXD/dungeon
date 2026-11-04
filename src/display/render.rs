//! Renderer por software (320x240), port directo de Render3D.java.
//! Suelo/techo con texturizado en punto fijo e iluminación bilineal por vértice;
//! paredes con interpolación afín (estilo PSX a propósito) y z-buffer por columna.

use crate::display::game::Game;
use crate::display::lighting::{self, Lighting};
use crate::display::texture::{DEFAULT_COLOR_MASK, FOG_LEVELS, Textures};
use crate::display::world::{AIR, SOLID, World};

pub const RENDER_WIDTH: usize = 320;
pub const RENDER_HEIGHT: usize = 240;

const FLOOR_POS: f32 = 8.0;
const CEILING_POS: f32 = 40.0;
const Z_CUTOFF: f32 = 500.0;
const REF_HEIGHT: f32 = 600.0;
const CLIP: f32 = 0.1;
const WALL_LEVELS: f32 = 3.0;
/// 1 = vértices al píxel entero (temblor PSX puro).
const SNAP: f32 = 1.0;

const FIX_SHIFT: u32 = 16;
const FIX_ONE: f32 = 65536.0;
const MAX_LEVEL: i32 = FOG_LEVELS as i32 - 1;
const LIGHT_SHIFT: u32 = lighting::FIX_SHIFT;

#[derive(Clone, Copy, Default)]
struct Camera {
    forward: f32,
    right: f32,
    up: f32,
    cosine: f32,
    sine: f32,
    walking: f32,
    y_look: f32,
    cam_x: f32,
    cam_z: f32,
    cam_y_correction: f32,
    half_w: f32,
    half_h: f32,
    focal: f32,
}

pub struct Renderer {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u32>,
    pub fov: f32,
    lighting: Lighting,
    textures: Textures,
    z_buffer: Vec<f32>,
    cam: Camera,
}

impl Renderer {
    pub fn new(width: usize, height: usize) -> Self {
        Renderer {
            width,
            height,
            pixels: vec![0; width * height],
            fov: 70.0,
            lighting: Lighting::new(),
            textures: Textures::load(DEFAULT_COLOR_MASK),
            z_buffer: vec![0.0; width],
            cam: Camera::default(),
        }
    }

    pub fn render(&mut self, game: &Game) {
        self.setup_camera(game);
        self.z_buffer.fill(0.0);
        let (lx, lz) = (self.cam.right / 16.0, self.cam.forward / 16.0);
        self.lighting.update(lx, lz, &game.world, game.time);
        self.render_floor_and_ceiling(&game.world);
        self.render_walls(&game.world);
    }

    fn setup_camera(&mut self, game: &Game) {
        let p = &game.player;
        let forward = p.z as f32;
        let right = p.x as f32;
        let up = p.y as f32 + 16.0;
        let y_look = p.ylook as f32 * self.height as f32 / REF_HEIGHT;

        let cosine = p.rotation.cos() as f32;
        let sine = p.rotation.sin() as f32;

        let half_w = self.width as f32 * 0.5;
        let focal = half_w / (self.fov.to_radians() * 0.5).tan();

        let t = game.time as f64;
        let walking = if p.walk {
            if p.crouch_walk {
                ((t / 6.0).sin() * 0.2) as f32
            } else if p.sprint_walk {
                ((t / 4.5).sin() * 2.0) as f32
            } else {
                ((t / 6.0).sin() * 0.5) as f32
            }
        } else {
            0.0
        };

        self.cam = Camera {
            forward,
            right,
            up,
            cosine,
            sine,
            walking,
            y_look,
            cam_x: right / 16.0,
            cam_z: forward / 16.0,
            cam_y_correction: -(up + walking) / 16.0,
            half_w,
            half_h: self.height as f32 * 0.5,
            focal,
        };
    }

    pub fn set_color_mask(&mut self, mask: u32) {
        self.textures = Textures::load(mask);
    }

    fn render_floor_and_ceiling(&mut self, world: &World) {
        let width = self.width;
        let height = self.height;
        let half_height = (height / 2) as i32;
        let half_width = width as f32 * 0.5;
        let cam = self.cam;
        let inv_focal = 1.0 / cam.focal;

        let tex = &self.textures.block;
        let table = &tex.table[..];
        let shift = tex.tile_shift;
        let (mu, mv, sv) = (tex.mask_u, tex.mask_v, tex.shift_v);

        let lv = &self.lighting.vertex_light[..];
        let ls = self.lighting.stride;
        let (cw, ch) = (world.width as i32, world.height as i32);
        let outside = ((self.lighting.ambient_fix() >> LIGHT_SHIFT) << shift) as usize;

        for y in 0..height {
            let mut c = ((y as i32 - half_height) as f32 - cam.y_look) * inv_focal;
            if c == 0.0 {
                c = 0.0001;
            }
            let z = if c < 0.0 {
                (CEILING_POS - cam.up - cam.walking) / -c
            } else {
                (FLOOR_POS + cam.up + cam.walking) / c
            };

            let row = &mut self.pixels[y * width..(y + 1) * width];
            if !(z > 0.0 && z <= Z_CUTOFF) {
                row.fill(0);
                continue;
            }

            let depth_base = z * inv_focal;
            let depth0 = -half_width * depth_base;
            let mut u = ((depth0 * cam.cosine + z * cam.sine + cam.right) * FIX_ONE) as i32;
            let mut v = ((z * cam.cosine - depth0 * cam.sine + cam.forward) * FIX_ONE) as i32;
            let du = (depth_base * cam.cosine * FIX_ONE) as i32;
            let dv = (-depth_base * cam.sine * FIX_ONE) as i32;

            for px in row.iter_mut() {
                let cx = u >> 20;
                let cz = v >> 20;
                let base = if cx >= 0 && cx < cw && cz >= 0 && cz < ch {
                    let i = cx as usize + cz as usize * ls;
                    let fx = (u >> 12) & 255;
                    let fz = (v >> 12) & 255;
                    let a = lv[i];
                    let b = lv[i + 1];
                    let cc = lv[i + ls];
                    let d = lv[i + ls + 1];
                    let top = a + (((b - a) * fx) >> 8);
                    let bot = cc + (((d - cc) * fx) >> 8);
                    let light = top + (((bot - top) * fz) >> 8);
                    ((light >> LIGHT_SHIFT) << shift) as usize
                } else {
                    outside
                };
                let ti = base + ((u >> FIX_SHIFT) & mu) as usize + ((((v >> FIX_SHIFT) & mv) << sv) as usize);
                *px = table[ti];
                u = u.wrapping_add(du);
                v = v.wrapping_add(dv);
            }
        }
    }

    fn render_walls(&mut self, world: &World) {
        let (w, h) = (world.width, world.height);
        let s = world.grid_stride();
        let grid = world.grid();

        for z in 0..h {
            for x in 0..w {
                let i = (x + 1) + (z + 1) * s;
                if grid[i] != SOLID {
                    continue;
                }
                if grid[i - s] == AIR {
                    self.render_wall(x, z, x + 1, z); // norte
                }
                if grid[i + s] == AIR {
                    self.render_wall(x + 1, z + 1, x, z + 1); // sur
                }
                if grid[i - 1] == AIR {
                    self.render_wall(x, z + 1, x, z); // oeste
                }
                if grid[i + 1] == AIR {
                    self.render_wall(x + 1, z, x + 1, z + 1); // este
                }
            }
        }
    }

    /// Cara de celda de (xl0, zl0) a (xr0, zr0), en coordenadas enteras de rejilla.
    fn render_wall(&mut self, xl0: usize, zl0: usize, xr0: usize, zr0: usize) {
        let cam = self.cam;
        let width = self.width;
        let height = self.height;
        let height_f = height as f32;
        let ls = self.lighting.stride;
        let light_a = self.lighting.vertex_light[xl0 + zl0 * ls] as f32;
        let light_b = self.lighting.vertex_light[xr0 + zr0 * ls] as f32;

        let xc_l = (xl0 as f32 - cam.cam_x) * 2.0;
        let zc_l = (zl0 as f32 - cam.cam_z) * 2.0;
        let xc_r = (xr0 as f32 - cam.cam_x) * 2.0;
        let zc_r = (zr0 as f32 - cam.cam_z) * 2.0;

        let mut lx = xc_l * cam.cosine - zc_l * cam.sine;
        let mut lz = zc_l * cam.cosine + xc_l * cam.sine;
        let mut rx = xc_r * cam.cosine - zc_r * cam.sine;
        let mut rz = zc_r * cam.cosine + xc_r * cam.sine;

        if lz < CLIP && rz < CLIP {
            return;
        }

        let mut t_l = 0.0f32;
        let mut t_r = 1.0f32;
        if lz < CLIP {
            let t = (CLIP - lz) / (rz - lz);
            lx += (rx - lx) * t;
            lz = CLIP;
            t_l = t;
        } else if rz < CLIP {
            let t = (CLIP - lz) / (rz - lz);
            rx = lx + (rx - lx) * t;
            rz = CLIP;
            t_r = t;
        }

        let xl = ((lx / lz * cam.focal + cam.half_w) * SNAP).floor() / SNAP;
        let xr = ((rx / rz * cam.focal + cam.half_w) * SNAP).floor() / SNAP;
        if xl >= xr {
            return;
        }

        let x0 = (xl.ceil() as i32).clamp(0, width as i32) as usize;
        let x1 = (xr.ceil() as i32).clamp(0, width as i32) as usize;
        if x0 >= x1 {
            return;
        }

        let y_top_corner = ((-0.5 - (WALL_LEVELS - 1.0)) - cam.cam_y_correction) * 2.0;
        let y_bot_corner = (0.5 - cam.cam_y_correction) * 2.0;

        let inv_lz = 1.0 / lz;
        let inv_rz = 1.0 / rz;
        let y_look_off = cam.half_h + cam.y_look;

        let tl_y = ((y_top_corner * inv_lz * cam.focal + y_look_off) * SNAP).floor() / SNAP;
        let tr_y = ((y_top_corner * inv_rz * cam.focal + y_look_off) * SNAP).floor() / SNAP;
        let bl_y = ((y_bot_corner * inv_lz * cam.focal + y_look_off) * SNAP).floor() / SNAP;
        let br_y = ((y_bot_corner * inv_rz * cam.focal + y_look_off) * SNAP).floor() / SNAP;

        let tex = &self.textures.stone;
        let table = &tex.table[..];
        let tex_w = tex.width as f32;
        let tex_h = tex.height as f32 * WALL_LEVELS;
        let u_l = t_l * tex_w;
        let d_u = (t_r - t_l) * tex_w;
        let light_l = light_a + (light_b - light_a) * t_l;
        let d_light = (light_b - light_a) * (t_r - t_l);

        let inv_dx = 1.0 / (xr - xl);
        let d_inv_z = inv_rz - inv_lz;

        let shift = tex.tile_shift;
        let (mu, mv, sv) = (tex.mask_u, tex.mask_v, tex.shift_v);

        for x in x0..x1 {
            let t = (x as f32 - xl) * inv_dx;

            let inv_z = inv_lz + d_inv_z * t;
            if self.z_buffer[x] > inv_z {
                continue;
            }
            self.z_buffer[x] = inv_z;

            let y_top = tl_y + (tr_y - tl_y) * t;
            let y_bot = bl_y + (br_y - bl_y) * t;
            if y_bot <= y_top {
                continue;
            }

            let y_start = if y_top < 0.0 { 0 } else { y_top as usize };
            let y_end = if y_bot > height_f { height } else { y_bot as usize };
            if y_start >= y_end {
                continue;
            }

            let level = (((light_l + d_light * t) as i32) >> LIGHT_SHIFT).clamp(0, MAX_LEVEL);
            let base = ((level as usize) << shift) + (((u_l + d_u * t) as i32) & mu) as usize;

            let v_step = tex_h / (y_bot - y_top);
            let dv = (v_step * FIX_ONE) as i32;
            let mut v = (((y_start as f32 - y_top) * v_step * FIX_ONE) as i32).max(0);

            let mut idx = y_start * width + x;
            for _ in y_start..y_end {
                self.pixels[idx] = table[base + ((((v >> FIX_SHIFT) & mv) << sv) as usize)];
                idx += width;
                v = v.wrapping_add(dv);
            }
        }
    }
}

/// Escala `src` a `dst` por un factor entero (vecino más cercano), centrado y con barras negras.
/// Devuelve el factor usado. Cada fila se expande una vez y se copia las demás veces.
pub fn blit_scaled(src: &[u32], sw: usize, sh: usize, dst: &mut [u32], dw: usize, dh: usize) -> usize {
    debug_assert_eq!(src.len(), sw * sh);
    debug_assert_eq!(dst.len(), dw * dh);
    let scale = (dw / sw).min(dh / sh).max(1);
    let out_w = sw * scale;
    let out_h = sh * scale;
    let ox = dw.saturating_sub(out_w) / 2;
    let oy = dh.saturating_sub(out_h) / 2;
    if out_w != dw || out_h != dh {
        dst.fill(0);
    }
    let visible_w = out_w.min(dw);

    for sy in 0..sh {
        let dy0 = oy + sy * scale;
        if dy0 >= dh {
            break;
        }
        let src_row = &src[sy * sw..(sy + 1) * sw];
        let first = dy0 * dw + ox;
        {
            let dst_row = &mut dst[first..first + visible_w];
            for (sx, &p) in src_row.iter().enumerate() {
                let start = sx * scale;
                if start >= visible_w {
                    break;
                }
                let end = (start + scale).min(visible_w);
                dst_row[start..end].fill(p);
            }
        }
        for k in 1..scale {
            let dy = dy0 + k;
            if dy >= dh {
                break;
            }
            dst.copy_within(first..first + visible_w, dy * dw + ox);
        }
    }
    scale
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::display::player::Input;
    use crate::display::rng::Rng;

    fn game(seed: u64) -> Game {
        Game::new(&mut Rng::new(seed))
    }

    #[test]
    fn renders_something() {
        let g = game(42);
        let mut r = Renderer::new(RENDER_WIDTH, RENDER_HEIGHT);
        r.render(&g);
        assert!(r.pixels.iter().any(|&p| p != 0));
    }

    #[test]
    fn wandering_never_panics_and_stays_in_open_space() {
        for seed in 0..8 {
            let mut g = game(seed);
            let mut r = Renderer::new(RENDER_WIDTH, RENDER_HEIGHT);
            let mut input = Input::default();
            for tick in 0..1500 {
                input.forward = true;
                input.turn_right = (tick / 40) % 2 == 0;
                input.turn_left = !input.turn_right;
                input.sprint = tick % 300 < 100;
                input.jump = tick % 97 == 0;
                input.crouch = tick % 211 == 0;
                g.tick(&input);
                if tick % 5 == 0 {
                    r.render(&g);
                }
                assert!(!g.world.is_solid_at(g.player.x, g.player.z));
            }
        }
    }

    #[test]
    fn looking_extremes_do_not_panic() {
        let mut g = game(3);
        let mut r = Renderer::new(RENDER_WIDTH, RENDER_HEIGHT);
        for dy in [-1.0e6, 1.0e6] {
            g.player.look(0.0, dy);
            for _ in 0..30 {
                g.tick(&Input::default());
            }
            r.render(&g);
        }
    }

    #[test]
    fn blit_scales_and_centers() {
        let src = [1u32, 2, 3, 4]; // 2x2
        let mut dst = vec![9u32; 6 * 5];
        let scale = blit_scaled(&src, 2, 2, &mut dst, 6, 5);
        assert_eq!(scale, 2);
        // área 4x4 centrada: ox = 1, oy = 0 (5-4)/2 = 0
        assert_eq!(&dst[1..5], &[1, 1, 2, 2]);
        assert_eq!(&dst[6 + 1..6 + 5], &[1, 1, 2, 2]);
        assert_eq!(&dst[2 * 6 + 1..2 * 6 + 5], &[3, 3, 4, 4]);
        assert_eq!(dst[0], 0); // barra negra
    }
}
