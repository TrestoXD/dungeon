//! Luz de antorcha por vértice, con línea de visión contra la rejilla y parpadeo.
//! Solo se recalcula cuando cambia algo (jugador, paso de parpadeo, mundo, parámetros).

use crate::display::texture::FOG_LEVELS;
use crate::display::world::{AIR, World};

pub const FIX_SHIFT: u32 = 8;
const FIX_ONE: f32 = 256.0;
const MAX_FIX: i32 = ((FOG_LEVELS as i32) - 1) << FIX_SHIFT;
const LOS_STEP: f32 = 0.25;

pub struct Lighting {
    pub ambient: f32,
    pub radius: f32,
    pub intensity: f32,
    pub flicker_amount: f32,

    pub vertex_light: Vec<i32>,
    pub stride: usize,

    vw: usize,
    vh: usize,
    last_version: Option<u32>,
    last_step: i32,
    last_px: f32,
    last_pz: f32,
    last_radius: f32,
    last_intensity: f32,
    last_ambient: f32,
}

impl Lighting {
    pub fn new() -> Self {
        Lighting {
            ambient: 1.5,
            radius: 10.0,
            intensity: 30.0,
            flicker_amount: 0.3,
            vertex_light: Vec::new(),
            stride: 0,
            vw: 0,
            vh: 0,
            last_version: None,
            last_step: i32::MIN,
            last_px: f32::NAN,
            last_pz: f32::NAN,
            last_radius: f32::NAN,
            last_intensity: f32::NAN,
            last_ambient: f32::NAN,
        }
    }

    pub fn ambient_fix(&self) -> i32 {
        ((self.ambient * FIX_ONE) as i32).clamp(0, MAX_FIX)
    }

    /// `px`, `pz` en unidades de celda.
    pub fn update(&mut self, px: f32, pz: f32, world: &World, time: i32) {
        let (nvw, nvh) = (world.width + 1, world.height + 1);
        let mut world_changed = self.last_version != Some(world.version);
        if nvw != self.vw || nvh != self.vh {
            self.vw = nvw;
            self.vh = nvh;
            self.stride = nvw;
            self.vertex_light = vec![0; nvw * nvh];
            world_changed = true;
        }

        let step = time >> 2;
        let unchanged = !world_changed
            && step == self.last_step
            && px == self.last_px
            && pz == self.last_pz
            && self.radius == self.last_radius
            && self.intensity == self.last_intensity
            && self.ambient == self.last_ambient;
        if unchanged {
            return;
        }

        self.last_version = Some(world.version);
        self.last_step = step;
        self.last_px = px;
        self.last_pz = pz;
        self.last_radius = self.radius;
        self.last_intensity = self.intensity;
        self.last_ambient = self.ambient;

        let amb = self.ambient_fix();
        self.vertex_light.fill(amb);

        let peak = self.intensity * self.flicker(step);
        let radius = self.radius;
        let (vw, vh) = (self.vw as i32, self.vh as i32);
        let x0 = ((px - radius).floor() as i32).max(0);
        let x1 = ((px + radius).ceil() as i32).min(vw - 1);
        let z0 = ((pz - radius).floor() as i32).max(0);
        let z1 = ((pz + radius).ceil() as i32).min(vh - 1);

        let occ = Occluders { grid: world.grid(), stride: world.grid_stride(), vw, vh };
        for vz in z0..=z1 {
            let row = vz as usize * self.vw;
            for vx in x0..=x1 {
                let l = light_at(&occ, radius, vx, vz, px, pz, peak);
                if l > 0.0 {
                    let v = amb + (l * FIX_ONE) as i32;
                    self.vertex_light[row + vx as usize] = v.min(MAX_FIX);
                }
            }
        }
    }

    fn flicker(&self, step: i32) -> f32 {
        if self.flicker_amount <= 0.0 {
            return 1.0;
        }
        let t = step as f32 * 0.9;
        1.0 + self.flicker_amount * (0.06 * t.sin() + 0.04 * (t * 2.7 + 1.3).sin())
    }
}

impl Default for Lighting {
    fn default() -> Self {
        Self::new()
    }
}

struct Occluders<'a> {
    grid: &'a [u8],
    stride: usize,
    vw: i32,
    vh: i32,
}

impl Occluders<'_> {
    fn solid_at(&self, cx: i32, cz: i32) -> bool {
        if cx < -1 || cz < -1 || cx > self.vw - 1 || cz > self.vh - 1 {
            return true;
        }
        self.grid[(cx + 1) as usize + (cz + 1) as usize * self.stride] != AIR
    }

    fn blocked(&self, lx: f32, lz: f32, dx: f32, dz: f32, dist: f32) -> bool {
        let limit = dist - 0.5;
        if limit <= LOS_STEP {
            return false;
        }
        let inv = LOS_STEP / dist;
        let (sx, sz) = (dx * inv, dz * inv);
        let (mut px, mut pz) = (lx, lz);
        let mut t = LOS_STEP;
        while t < limit {
            px += sx;
            pz += sz;
            if self.solid_at(px.floor() as i32, pz.floor() as i32) {
                return true;
            }
            t += LOS_STEP;
        }
        false
    }
}

fn light_at(occ: &Occluders<'_>, radius: f32, vx: i32, vz: i32, lx: f32, lz: f32, peak: f32) -> f32 {
    let dx = vx as f32 - lx;
    let dz = vz as f32 - lz;
    let d2 = dx * dx + dz * dz;
    if d2 >= radius * radius {
        return 0.0;
    }
    let dist = d2.sqrt();
    if occ.blocked(lx, lz, dx, dz, dist) {
        return 0.0;
    }
    let s = 1.0 - dist / radius;
    peak * s * s
}
