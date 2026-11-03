use crate::display::player::{Input, Player};
use crate::display::rng::Rng;
use crate::display::world::World;

pub struct Game {
    pub time: i32,
    pub player: Player,
    pub world: World,
}

impl Game {
    pub fn new(rng: &mut Rng) -> Self {
        let world = World::generate(16, 16, rng);
        let player = Player::new(world.spawn.0, world.spawn.1);
        Game { time: 0, player, world }
    }

    pub fn tick(&mut self, input: &Input) {
        self.time = self.time.wrapping_add(1);
        self.player.tick(input, &self.world);
    }
}
