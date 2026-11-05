//! Renderer por software (320x240), port directo de Render3D.java.
//! Suelo/techo con texturizado en punto fijo e iluminación bilineal por vértice;
//! paredes con interpolación afín (estilo PSX a propósito) y z-buffer por columna.

use crate::display::game::Game;
use crate::display::lighting::Lighting;
use crate::display::texture::{DEFAULT_COLOR_MASK, DEFAULT_FOG_SHIFT, Textures};
use crate::display::world::{AIR, SOLID, World};

pub const RENDER_WIDTH: usize = 320;
pub const RENDER_HEIGHT: usize = 240;
pub const TARGET_HEIGHT: usize = 240;

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

pub fn internal_size(win_w: usize, win_h: usize) -> (usize, usize, usize) {
    let scale = win_h.div_ceil(TARGET_HEIGHT).max(1);
    (
        win_w.div_ceil(scale).max(1),
        win_h.div_ceil(scale).max(1),
        scale,
    )
}

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

#[derive(Clone, Copy)]
struct Face {
    dist: f32,
    x0: usize,
    z0: usize,
    x1: usize,
    z1: usize,
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
    faces: Vec<Face>,
}



impl Renderer {
    pub fn new(width: usize, height: usize) -> Self {
        Renderer {
            width,
            height,
            pixels: vec![0; width * height],
            fov: 70.0,
            lighting: Lighting::new(),
            textures: Textures::load(DEFAULT_COLOR_MASK, DEFAULT_FOG_SHIFT),
            z_buffer: vec![0.0; width],
            cam: Camera::default(),
            faces: Vec::with_capacity(512),
        }
    }


    pub fn render(&mut self, game: &Game) {
        self.render_interpolated(game, 1.0);
    }
    pub fn render_interpolated(&mut self, game: &Game, alpha: f32) {
        self.setup_camera(game, alpha);
        self.z_buffer.fill(0.0);
        // La luz usa la posición REAL del tick, no la interpolada: así solo se
        // recalcula 60 veces por segundo aunque se dibuje a 120 o sin límite.
        let (lx, lz) = (game.player.x as f32 / 16.0, game.player.z as f32 / 16.0);
        self.lighting.update(lx, lz, &game.world, game.time);
        self.render_floor_and_ceiling(&game.world);
        self.render_walls(&game.world);
    }

    fn setup_camera(&mut self, game: &Game, alpha: f32) {
        let p = &game.player;
        let v = p.view(alpha as f64);
        let forward = v.z as f32;
        let right = v.x as f32;
        let up = v.y as f32 + 16.0;
        let y_look = v.ylook as f32 * self.height as f32 / REF_HEIGHT;

        let cosine = v.rotation.cos() as f32;
        let sine = v.rotation.sin() as f32;

        let half_w = self.width as f32 * 0.5;
        let focal = (self.height as f32 * (2.0 / 3.0)) / (self.fov.to_radians() * 0.5).tan();

        let t = game.time as f64 - 1.0 + alpha as f64;
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

    pub fn set_color_mask(&mut self, mask: u32, fog_shift: u32) {
        self.textures = Textures::load(mask, fog_shift);
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        if width == self.width && height == self.height {
            return;
        }
        self.width = width;
        self.height = height;
        self.pixels.resize(width * height, 0);
        self.z_buffer.resize(width, 0.0);
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
        let fog = tex.fog_shift;
        let max_level = tex.max_level;
        let (mu, mv, sv) = (tex.mask_u, tex.mask_v, tex.shift_v);

        let lv = &self.lighting.vertex_light[..];
        let ls = self.lighting.stride;
        let (cw, ch) = (world.width as i32, world.height as i32);
        let outside = ((self.lighting.ambient_fix() >> fog) << shift) as usize;

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
                    ((light >> fog) << shift) as usize
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
        let (cx, cz) = (self.cam.cam_x, self.cam.cam_z);

        // Se saca el Vec de self para poder llamar a render_wall (&mut self) mientras se recorre.
        let mut faces = std::mem::take(&mut self.faces);
        faces.clear();
        let mut push = |x0: usize, z0: usize, x1: usize, z1: usize| {
            let dx = (x0 + x1) as f32 * 0.5 - cx;
            let dz = (z0 + z1) as f32 * 0.5 - cz;
            faces.push(Face { dist: dx * dx + dz * dz, x0, z0, x1, z1 });
        };

        for z in 0..h {
            for x in 0..w {
                let i = (x + 1) + (z + 1) * s;
                if grid[i] != SOLID {
                    continue;
                }
                if grid[i - s] == AIR {
                    push(x, z, x + 1, z); // norte
                }
                if grid[i + s] == AIR {
                    push(x + 1, z + 1, x, z + 1); // sur
                }
                if grid[i - 1] == AIR {
                    push(x, z + 1, x, z); // oeste
                }
                if grid[i + 1] == AIR {
                    push(x + 1, z, x + 1, z + 1); // este
                }
            }
        }

        // De la más cercana a la más lejana.
        faces.sort_unstable_by(|a, b| a.dist.total_cmp(&b.dist));
        for f in &faces {
            self.render_wall(f.x0, f.z0, f.x1, f.z1);
        }
        self.faces = faces;
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
        let fog = tex.fog_shift;
        let max_level = tex.max_level;
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

            let level = (((light_l + d_light * t) as i32) >> fog).clamp(0, max_level);
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
pub fn blit_scaled(
    src: &[u32],
    sw: usize,
    sh: usize,
    dst: &mut [u32],
    dw: usize,
    dh: usize,
    scale: usize,
) {
    debug_assert_eq!(src.len(), sw * sh);
    debug_assert_eq!(dst.len(), dw * dh);
    let scale = scale.max(1);
    let visible_w = (sw * scale).min(dw);

    // Con `internal_size` esto no ocurre; es solo una red de seguridad.
    if visible_w < dw || sh * scale < dh {
        dst.fill(0);
    }

    for sy in 0..sh {
        let dy0 = sy * scale;
        if dy0 >= dh {
            break;
        }
        let src_row = &src[sy * sw..(sy + 1) * sw];
        let first = dy0 * dw;
        {
            let dst_row = &mut dst[first..first + visible_w];
            for (sx, &p) in src_row.iter().enumerate() {
                let start = sx * scale;
                if start >= visible_w {
                    break;
                }
                dst_row[start..(start + scale).min(visible_w)].fill(p);
            }
        }
        for k in 1..scale {
            let dy = dy0 + k;
            if dy >= dh {
                break;
            }
            dst.copy_within(first..first + visible_w, dy * dw);
        }
    }
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
    fn blit_scales_and_crops() {
        let src = [1u32, 2, 3, 4]; // 2x2 a escala 3 = 6x6, ventana 5x5: recorta
        let mut dst = vec![9u32; 25];
        blit_scaled(&src, 2, 2, &mut dst, 5, 5, 3);
        assert_eq!(&dst[0..5], &[1, 1, 1, 2, 2]);
        assert_eq!(&dst[15..20], &[3, 3, 3, 4, 4]);
        assert!(dst.iter().all(|&p| p != 9)); // no queda ningún píxel sin cubrir
    }

    #[test]
    fn internal_size_covers_window() {
        for (w, h) in [(960, 720), (1920, 1080), (2560, 1440), (1366, 768), (800, 600), (321, 241)] {
            let (iw, ih, s) = internal_size(w, h);
            assert!(iw * s >= w && ih * s >= h);
            assert!(iw * s - w < s && ih * s - h < s); // recorte menor que un píxel interno
            assert!(ih <= TARGET_HEIGHT);
        }
        assert_eq!(internal_size(960, 720), (320, 240, 3));
    }

    #[test]
    #[ignore = "benchmark: cargo test --release bench_stages -- --ignored --nocapture"]
    fn bench_stages() {
        use std::time::{Duration, Instant};

        const WARMUP: u32 = 300;
        const FRAMES: u32 = 3000;
        const WIN_W: usize = 1920;
        const WIN_H: usize = 1080;

        let mut g = game(1234);
        let mut r = Renderer::new(RENDER_WIDTH, RENDER_HEIGHT);
        let mut window = vec![0u32; WIN_W * WIN_H];
        let mut input = Input::default();
        let (iw, ih, scale) = internal_size(WIN_W, WIN_H);
        let mut r = Renderer::new(iw, ih);
        input.turn_right = true;

        let (mut t_light, mut t_floor, mut t_walls, mut t_blit) =
            (Duration::ZERO, Duration::ZERO, Duration::ZERO, Duration::ZERO);

        for i in 0..(WARMUP + FRAMES) {
            input.forward = (i / 120) % 2 == 0;
            g.tick(&input);

            let a = Instant::now();
            r.setup_camera(&g, 1.0);
            r.z_buffer.fill(0.0);
            let (lx, lz) = (g.player.x as f32 / 16.0, g.player.z as f32 / 16.0);
            r.lighting.update(lx, lz, &g.world, g.time);
            let b = Instant::now();
            r.render_floor_and_ceiling(&g.world);
            let c = Instant::now();
            r.render_walls(&g.world);
            let d = Instant::now();
            blit_scaled(&r.pixels, iw, ih, &mut window, WIN_W, WIN_H, scale);
            let e = Instant::now();

            if i >= WARMUP {
                t_light += b - a;
                t_floor += c - b;
                t_walls += d - c;
                t_blit += e - d;
            }
        }
        std::hint::black_box(&window);

        let total = t_light + t_floor + t_walls + t_blit;
        let ms = |t: Duration| t.as_secs_f64() * 1000.0 / FRAMES as f64;
        let pct = |t: Duration| 100.0 * t.as_secs_f64() / total.as_secs_f64();
        println!("\n--- ms por frame (media de {FRAMES}) ---");
        println!("camara + luz     {:7.3} ms  {:5.1} %", ms(t_light), pct(t_light));
        println!("suelo y techo    {:7.3} ms  {:5.1} %", ms(t_floor), pct(t_floor));
        println!("paredes          {:7.3} ms  {:5.1} %", ms(t_walls), pct(t_walls));
        println!("escalado ventana {:7.3} ms  {:5.1} %", ms(t_blit), pct(t_blit));
        println!("TOTAL            {:7.3} ms  ({:.0} FPS teóricos)", ms(total), 1000.0 / ms(total));
    }
}
