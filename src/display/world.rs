//! Mundo: rejilla de celdas sólidas/vacías, generada como una mazmorra de salas unidas por pasillos.

use std::sync::atomic::{AtomicU32, Ordering};

use crate::display::rng::Rng;

pub const AIR: u8 = 0;
pub const SOLID: u8 = 1;

/// Unidades de mundo por celda.
pub const CELL: f64 = 16.0;

/// Lado mínimo y máximo de una sala, en celdas.
pub const MIN_ROOM: usize = 3;
pub const MAX_ROOM: usize = 16;
/// Celdas de muro que deben quedar, como mínimo, entre dos salas.
const ROOM_MARGIN: usize = 2;

/// Cada mundo generado recibe un número distinto. Los cachés (luz, caras de pared)
/// lo usan para saber cuándo tienen que reconstruirse.
static NEXT_VERSION: AtomicU32 = AtomicU32::new(1);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Room {
    pub x: usize,
    pub z: usize,
    pub w: usize,
    pub h: usize,
}

impl Room {
    pub fn center(&self) -> (usize, usize) {
        (self.x + self.w / 2, self.z + self.h / 2)
    }

    /// Distancia al cuadrado entre centros.
    pub fn dist2(&self, other: &Room) -> u64 {
        let (ax, az) = self.center();
        let (bx, bz) = other.center();
        let (dx, dz) = (ax.abs_diff(bx) as u64, az.abs_diff(bz) as u64);
        dx * dx + dz * dz
    }

    fn too_close(&self, o: &Room) -> bool {
        self.x < o.x + o.w + ROOM_MARGIN
            && o.x < self.x + self.w + ROOM_MARGIN
            && self.z < o.z + o.h + ROOM_MARGIN
            && o.z < self.z + self.h + ROOM_MARGIN
    }
}

pub struct World {
    pub width: usize,
    pub height: usize,
    /// Distinto en cada mundo generado; habría que cambiarlo si el mundo se modificara en juego.
    pub version: u32,
    /// Posición de aparición en unidades de mundo (centro de la primera sala).
    pub spawn: (f64, f64),
    pub rooms: Vec<Room>,
    cells: Vec<bool>,
    /// (width+2) x (height+2) con un anillo de SOLID alrededor; se construye una sola vez.
    grid: Vec<u8>,
}

impl World {
    pub fn generate(width: usize, height: usize, rng: &mut Rng) -> World {
        assert!(width >= 3 && height >= 3, "el mundo necesita al menos 3x3");
        let mut cells = vec![true; width * height];

        let rooms = place_rooms(width, height, rng);
        for r in &rooms {
            for z in r.z..r.z + r.h {
                for x in r.x..r.x + r.w {
                    cells[x + z * width] = false;
                }
            }
        }
        for (a, b) in connections(&rooms, rng) {
            let horizontal_first = rng.below(2) == 0;
            carve_l(&mut cells, width, height, rooms[a].center(), rooms[b].center(), horizontal_first);
        }

        let (sx, sz) = rooms[0].center();
        let spawn = (sx as f64 * CELL + CELL / 2.0, sz as f64 * CELL + CELL / 2.0);

        let stride = width + 2;
        let mut grid = vec![SOLID; stride * (height + 2)];
        for z in 0..height {
            for x in 0..width {
                if !cells[x + z * width] {
                    grid[(x + 1) + (z + 1) * stride] = AIR;
                }
            }
        }

        World {
            width,
            height,
            version: NEXT_VERSION.fetch_add(1, Ordering::Relaxed),
            spawn,
            rooms,
            cells,
            grid,
        }
    }

    pub fn grid(&self) -> &[u8] {
        &self.grid
    }

    pub fn grid_stride(&self) -> usize {
        self.width + 2
    }

    pub fn open_cells(&self) -> usize {
        self.cells.iter().filter(|&&s| !s).count()
    }

    /// Fuera del mapa cuenta como sólido.
    pub fn is_solid_at(&self, x: f64, z: f64) -> bool {
        let bx = (x / CELL).floor() as i64;
        let bz = (z / CELL).floor() as i64;
        if bx < 0 || bz < 0 || bx >= self.width as i64 || bz >= self.height as i64 {
            return true;
        }
        self.cells[bx as usize + bz as usize * self.width]
    }

    /// Centro de la sala más cercana a la de aparición (distinta de ella).
    pub fn nearest_other_room(&self) -> Option<(usize, usize)> {
        let first = self.rooms.first()?;
        self.rooms
            .iter()
            .skip(1)
            .min_by_key(|r| first.dist2(r))
            .map(|r| r.center())
    }

    /// Una celda libre al azar, a `min_dist` celdas o más de la aparición del jugador.
    pub fn random_open_cell(&self, rng: &mut Rng, min_dist: f64) -> (usize, usize) {
        let (sx, sz) = (self.spawn.0 / CELL, self.spawn.1 / CELL);
        let far: Vec<(usize, usize)> = (0..self.height)
            .flat_map(|z| (0..self.width).map(move |x| (x, z)))
            .filter(|&(x, z)| !self.cells[x + z * self.width])
            .filter(|&(x, z)| {
                let (dx, dz) = (x as f64 + 0.5 - sx, z as f64 + 0.5 - sz);
                (dx * dx + dz * dz).sqrt() >= min_dist
            })
            .collect();
        if far.is_empty() {
            return (sx as usize, sz as usize);
        }
        far[rng.below(far.len())]
    }
}

// ---------------------------------------------------------------- generación

/// Intenta colocar salas al azar; descarta las que quedan demasiado cerca de otra.
/// Siempre devuelve al menos una (el primer intento nunca puede chocar).
fn place_rooms(width: usize, height: usize, rng: &mut Rng) -> Vec<Room> {
    let max_w = MAX_ROOM.min(width - 2);
    let max_h = MAX_ROOM.min(height - 2);
    let min_w = MIN_ROOM.min(max_w);
    let min_h = MIN_ROOM.min(max_h);

    let mut rooms: Vec<Room> = Vec::new();
    for _ in 0..(width * height / 16).max(8) {
        let w = min_w + rng.below(max_w - min_w + 1);
        let h = min_h + rng.below(max_h - min_h + 1);
        // el borde exterior (celdas 0 y width-1) nunca se excava
        let x = 1 + rng.below(width - 1 - w);
        let z = 1 + rng.below(height - 1 - h);
        let room = Room { x, z, w, h };
        if rooms.iter().all(|o| !room.too_close(o)) {
            rooms.push(room);
        }
    }
    rooms
}

fn has_edge(edges: &[(usize, usize)], a: usize, b: usize) -> bool {
    edges.iter().any(|&(p, q)| (p == a && q == b) || (p == b && q == a))
}

/// Qué salas se unen: un árbol de expansión mínima (todo queda alcanzable)
/// más algunos enlaces extra entre salas cercanas (bucles).
fn connections(rooms: &[Room], rng: &mut Rng) -> Vec<(usize, usize)> {
    let n = rooms.len();
    let mut edges: Vec<(usize, usize)> = Vec::new();

    let mut in_tree = vec![false; n];
    in_tree[0] = true;
    for _ in 1..n {
        let mut best = (0usize, 0usize, u64::MAX);
        for a in (0..n).filter(|&i| in_tree[i]) {
            for b in (0..n).filter(|&i| !in_tree[i]) {
                let d = rooms[a].dist2(&rooms[b]);
                if d < best.2 {
                    best = (a, b, d);
                }
            }
        }
        in_tree[best.1] = true;
        edges.push((best.0, best.1));
    }

    for _ in 0..n / 3 {
        let a = rng.below(n);
        let nearest = (0..n)
            .filter(|&b| b != a && !has_edge(&edges, a, b))
            .min_by_key(|&b| rooms[a].dist2(&rooms[b]));
        if let Some(b) = nearest {
            edges.push((a, b));
        }
    }
    edges
}

/// Abre una celda, respetando el borde exterior.
fn open(cells: &mut [bool], w: usize, h: usize, x: usize, z: usize) {
    if x >= 1 && z >= 1 && x + 1 < w && z + 1 < h {
        cells[x + z * w] = false;
    }
}

/// Tramo recto de 2 de ancho: horizontal (filas z y z+1) o vertical (columnas x y x+1).
fn carve_line(cells: &mut [bool], w: usize, h: usize, from: (usize, usize), to: (usize, usize)) {
    if from.1 == to.1 {
        for x in from.0.min(to.0)..=from.0.max(to.0) {
            open(cells, w, h, x, from.1);
            open(cells, w, h, x, from.1 + 1);
        }
    } else {
        for z in from.1.min(to.1)..=from.1.max(to.1) {
            open(cells, w, h, from.0, z);
            open(cells, w, h, from.0 + 1, z);
        }
    }
}

/// Pasillo en L de 2 de ancho entre dos puntos.
fn carve_l(
    cells: &mut [bool],
    w: usize,
    h: usize,
    a: (usize, usize),
    b: (usize, usize),
    horizontal_first: bool,
) {
    let corner = if horizontal_first { (b.0, a.1) } else { (a.0, b.1) };
    carve_line(cells, w, h, a, corner);
    carve_line(cells, w, h, corner, b);
    // el codo se rellena entero (2x2) para que el giro no quede achaflanado
    for dx in 0..2 {
        for dz in 0..2 {
            open(cells, w, h, corner.0 + dx, corner.1 + dz);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reachable(w: &World) -> usize {
        let (sx, sz) = w.rooms[0].center();
        let mut seen = vec![false; w.width * w.height];
        let mut stack = vec![(sx, sz)];
        seen[sx + sz * w.width] = true;
        let mut count = 0;
        while let Some((x, z)) = stack.pop() {
            count += 1;
            let around = [(x.wrapping_sub(1), z), (x + 1, z), (x, z.wrapping_sub(1)), (x, z + 1)];
            for (nx, nz) in around {
                if nx < w.width && nz < w.height && !w.cells[nx + nz * w.width] && !seen[nx + nz * w.width] {
                    seen[nx + nz * w.width] = true;
                    stack.push((nx, nz));
                }
            }
        }
        count
    }

    #[test]
    fn rooms_are_valid() {
        for seed in 0..30 {
            let w = World::generate(96, 96, &mut Rng::new(seed));
            assert!(w.rooms.len() >= 10);
            for (i, r) in w.rooms.iter().enumerate() {
                assert!((MIN_ROOM..=MAX_ROOM).contains(&r.w) && (MIN_ROOM..=MAX_ROOM).contains(&r.h));
                assert!(r.x >= 1 && r.z >= 1 && r.x + r.w <= 95 && r.z + r.h <= 95);
                for o in &w.rooms[i + 1..] {
                    assert!(!r.too_close(o));
                }
            }
            assert!(!w.is_solid_at(w.spawn.0, w.spawn.1));
        }
    }

    #[test]
    fn everything_is_connected() {
        for seed in 0..30 {
            let w = World::generate(96, 96, &mut Rng::new(seed));
            assert_eq!(reachable(&w), w.open_cells());
        }
    }

    #[test]
    fn border_stays_solid() {
        for seed in 0..30 {
            let w = World::generate(64, 48, &mut Rng::new(seed));
            for x in 0..w.width {
                assert!(w.cells[x] && w.cells[x + (w.height - 1) * w.width]);
            }
            for z in 0..w.height {
                assert!(w.cells[z * w.width] && w.cells[w.width - 1 + z * w.width]);
            }
        }
    }

    #[test]
    fn corridors_are_two_wide() {
        let (w, h) = (24, 24);
        let mut cells = vec![true; w * h];
        carve_l(&mut cells, w, h, (3, 3), (12, 10), true);
        let is_open = |x: usize, z: usize| !cells[x + z * w];
        for x in 3..=12 {
            assert!(is_open(x, 3) && is_open(x, 4)); // tramo horizontal
        }
        for z in 3..=10 {
            assert!(is_open(12, z) && is_open(13, z)); // tramo vertical
        }
        assert!(!is_open(11, 5) && !is_open(14, 6)); // alrededor sigue habiendo muro
    }
}