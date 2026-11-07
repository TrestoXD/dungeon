use crate::display::player::{Input, Player};
use crate::display::rng::Rng;
use crate::display::sprite::{Sprite, SpriteKind};
use crate::display::world::{CELL, World};

pub struct Game {
    pub time: i32,
    pub player: Player,
    pub world: World,
    pub sprites: Vec<Sprite>,   // <- nuevo
}
impl Game {
    pub fn new(rng: &mut Rng) -> Self {
        let world = World::generate(128, 128, rng);
        let player = Player::new(world.spawn.0, world.spawn.1);

        // El rng del mundo ya se usó, así que no cambia los mundos de cada semilla.
        let (cx, cz) = world.random_open_cell(rng, 4.0);
        println!("Sprite en la celda {cx} {cz}");
        let sprites = vec![Sprite::new(
            SpriteKind::Waifu,
            cx as f64 * CELL + CELL / 2.0,
            cz as f64 * CELL + CELL / 2.0,
        )];

        Game { time: 0, player, world, sprites }
    }

    // Tick Function
    pub fn tick(&mut self, input: &Input) {
        self.time = self.time.wrapping_add(1);
        self.player.tick(input, &self.world);
    }
}
