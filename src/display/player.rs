// SUBSTITUTE OF CONTROLLER IN JAVA

use crate::display::world::World;

#[derive(Default, Clone, Copy)]
/// All player Inputs
pub struct Input {
    pub forward: bool,
    pub back: bool,
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub crouch: bool,
    pub sprint: bool,
    pub turn_left: bool,
    pub turn_right: bool,
}

/// Tick movement turn
const KEY_TURN: f64 = 0.016;
/// Impulse radiant for the mouse movement
const MOUSE_TURN: f64 = 0.004;
const MOUSE_PITCH: f64 = 2.0;
const PLAYER_RADIUS: f64 = 2.5;

pub struct Player {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub rotation: f64,
    pub ylook: f64,
    xa: f64,
    za: f64,
    rotationa: f64,
    ylooka: f64,
    pub walk: bool,
    pub sprint_walk: bool,
    pub crouch_walk: bool,
}

impl Player {
    pub fn new(x: f64, z: f64) -> Self {
        Player {
            x,
            y: 0.0,
            z,
            rotation: 0.0,
            ylook: 0.0,
            xa: 0.0,
            za: 0.0,
            rotationa: 0.0,
            ylooka: 0.0,
            walk: false,
            sprint_walk: false,
            crouch_walk: false,
        }
    }

    /// Accumulated mouse movement through last tick.
    pub fn look(&mut self, dx: f64, dy: f64) {
        self.rotationa += MOUSE_TURN * dx;
        self.ylooka -= dy * MOUSE_PITCH;
    }

    pub fn tick(&mut self, input: &Input, world: &World) {
        let mut walk_speed = 0.5;
        let jump_height = 0.8;
        let crouch_height = 0.3;
        let mut x_move = 0.0;
        let mut z_move = 0.0;

        if input.forward {
            z_move += 1.0;
            self.walk = true;
        }
        if input.back {
            z_move -= 1.0;
            self.walk = true;
        }
        if input.left {
            x_move -= 1.0;
            self.walk = true;
        }
        if input.right {
            x_move += 1.0;
            self.walk = true;
        }

        if input.turn_left {
            self.rotationa -= KEY_TURN;
        }
        if input.turn_right {
            self.rotationa += KEY_TURN;
        }

        let mut sprint = input.sprint;
        if input.jump {
            self.y += jump_height;
            sprint = false;
        }
        if input.crouch {
            walk_speed = 0.2;
            self.y -= crouch_height;
            sprint = false;
            self.crouch_walk = true;
        }
        if sprint {
            walk_speed = 1.25;
            self.sprint_walk = true;
        }

        if !input.forward && !input.back && !input.left && !input.right {
            self.walk = false;
        }
        if !input.crouch {
            self.crouch_walk = false;
        }
        if !sprint {
            self.sprint_walk = false;
        }

        let (sin, cos) = self.rotation.sin_cos();
        self.xa += (x_move * cos + z_move * sin) * walk_speed;
        self.za += (z_move * cos - x_move * sin) * walk_speed;

        let r = PLAYER_RADIUS;

        let next_x = self.x + self.xa;
        if self.xa > 0.0 {
            if !world.is_solid_at(next_x + r, self.z - r) && !world.is_solid_at(next_x + r, self.z + r) {
                self.x = next_x;
            } else {
                self.xa = 0.0;
            }
        } else if self.xa < 0.0 {
            if !world.is_solid_at(next_x - r, self.z - r) && !world.is_solid_at(next_x - r, self.z + r) {
                self.x = next_x;
            } else {
                self.xa = 0.0;
            }
        }

        let next_z = self.z + self.za;
        if self.za > 0.0 {
            if !world.is_solid_at(self.x - r, next_z + r) && !world.is_solid_at(self.x + r, next_z + r) {
                self.z = next_z;
            } else {
                self.za = 0.0;
            }
        } else if self.za < 0.0 {
            if !world.is_solid_at(self.x - r, next_z - r) && !world.is_solid_at(self.x + r, next_z - r) {
                self.z = next_z;
            } else {
                self.za = 0.0;
            }
        }

        self.xa *= 0.1;
        self.y *= 0.96;
        self.za *= 0.1;
        self.rotation += self.rotationa;
        self.rotationa *= 0.5;

        self.ylook += self.ylooka;
        self.ylooka *= 0.7;
        self.ylook = self.ylook.clamp(-300.0, 360.0);
    }
}
