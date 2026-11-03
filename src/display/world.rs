//! Mundo: rejilla de celdas sólidas/vacías. Sustituye a World/Block/SolidBlock/NonSolidWall
//! (cuatro clases con un `boolean`) por un `Vec<bool>` plano y una rejilla con borde precalculada.

use crate::display::rng::Rng;

pub const AIR: u8 = 0;
pub const SOLID: u8 = 1;

/// Units of the world by cell
pub const CELL: f64 = 16.0;

pub struct World {
    pub width: usize,
    pub height: usize,
    /// Change the content if the world changes (NOW STATIC)
    pub version: u32,
    /// Position of Player Spawn in World units
    pub spawn: (f64, f64),
    cells: Vec<bool>,
    /// (width+2) x (height+2) a SOLID ring around, builds only once.
    grid: Vec<u8>,
}

impl World {
    /// Generate the World
    pub fn generate(width: usize, height: usize, rng: &mut Rng) -> World {
        assert!(width >= 3 && height >= 3, "The world needs a minimum of 3x3");
        let idx = |x: usize, y: usize| x + y * width;
        let mut cells = vec![true; width * height];

        let (mut ex, mut ey) = (width / 2, height / 2);
        cells[idx(ex, ey)] = false;

        let capacity = (width - 2) * (height - 2) - 1;
        let target = (((width * height) as f64 * 0.40) as usize).min(capacity);
        let mut opened = 0;
        const DIRS: [(isize, isize); 4] = [(0, -1), (0, 1), (-1, 0), (1, 0)];

        while opened < target {
            let (dx, dy) = DIRS[rng.below(4)];
            let nx = ex as isize + dx;
            let ny = ey as isize + dy;
            if nx > 0 && nx < width as isize - 1 && ny > 0 && ny < height as isize - 1 {
                ex = nx as usize;
                ey = ny as usize;
                if cells[idx(ex, ey)] {
                    cells[idx(ex, ey)] = false;
                    opened += 1;
                }
            } else {
                loop {
                    ex = rng.below(width - 2) + 1;
                    ey = rng.below(height - 2) + 1;
                    if !cells[idx(ex, ey)] {
                        break;
                    }
                }
            }
        }
        // perfect cell
        let spawn_cell = find_open(&cells, width, width / 2..width - 1, height / 2..height - 1)
            .or_else(|| find_open(&cells, width, 1..width - 1, 1..height - 1))
            .unwrap_or((width / 2, height / 2));
        let spawn = (
            spawn_cell.0 as f64 * CELL + CELL / 2.0,
            spawn_cell.1 as f64 * CELL + CELL / 2.0,
        );

        let stride = width + 2;
        let mut grid = vec![SOLID; stride * (height + 2)];
        for z in 0..height {
            for x in 0..width {
                if !cells[idx(x, z)] {
                    grid[(x + 1) + (z + 1) * stride] = AIR;
                }
            }
        }

        World { width, height, version: 0, spawn, cells, grid }
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

    /// Out of the map is Solid.
    pub fn is_solid_at(&self, x: f64, z: f64) -> bool {
        let bx = (x / CELL).floor() as i64;
        let bz = (z / CELL).floor() as i64;
        if bx < 0 || bz < 0 || bx >= self.width as i64 || bz >= self.height as i64 {
            return true;
        }
        self.cells[bx as usize + bz as usize * self.width]
    }
}

// Find an open space for the player
fn find_open(
    cells: &[bool],
    width: usize,
    xs: std::ops::Range<usize>,
    zs: std::ops::Range<usize>,
) -> Option<(usize, usize)> {
    for x in xs {
        for z in zs.clone() {
            if !cells[x + z * width] {
                return Some((x, z));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generation_is_sane() {
        for seed in 0..50 {
            let mut rng = Rng::new(seed);
            let w = World::generate(16, 16, &mut rng);
            // centro + 102 celdas excavadas
            assert_eq!(w.open_cells(), 1 + 102);
            assert!(!w.is_solid_at(w.spawn.0, w.spawn.1));
            // el borde exterior nunca se excava
            for i in 0..16 {
                assert!(w.is_solid_at(i as f64 * CELL + 1.0, 1.0));
                assert!(w.is_solid_at(1.0, i as f64 * CELL + 1.0));
                assert!(w.is_solid_at(i as f64 * CELL + 1.0, 15.0 * CELL + 1.0));
                assert!(w.is_solid_at(15.0 * CELL + 1.0, i as f64 * CELL + 1.0));
            }
        }
    }

    #[test]
    fn tiny_world_terminates() {
        let mut rng = Rng::new(1);
        let w = World::generate(3, 3, &mut rng);
        assert_eq!(w.open_cells(), 1);
    }

    #[test]
    fn out_of_bounds_is_solid() {
        let mut rng = Rng::new(7);
        let w = World::generate(16, 16, &mut rng);
        assert!(w.is_solid_at(-5.0, 20.0));
        assert!(w.is_solid_at(1000.0, 20.0));
    }
}
