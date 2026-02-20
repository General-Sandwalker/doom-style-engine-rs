use std::collections::HashSet;
use winit::keyboard::KeyCode;

pub const MOVE_SPEED: f32 = 0.05;
pub const ROT_SPEED: f32 = 0.04;

pub struct Player {
    pub x: f32,
    pub y: f32,
    pub angle: f32,
}

impl Player {
    pub fn new(x: f32, y: f32, angle: f32) -> Self {
        Self { x, y, angle }
    }

    pub fn update(&mut self, keys: &HashSet<KeyCode>, map: &crate::map::Map) {
        let dx = self.angle.cos();
        let dy = self.angle.sin();

        if keys.contains(&KeyCode::KeyW) || keys.contains(&KeyCode::ArrowUp) {
            self.try_move(dx * MOVE_SPEED, dy * MOVE_SPEED, map);
        }
        if keys.contains(&KeyCode::KeyS) || keys.contains(&KeyCode::ArrowDown) {
            self.try_move(-dx * MOVE_SPEED, -dy * MOVE_SPEED, map);
        }
        if keys.contains(&KeyCode::ArrowLeft) || keys.contains(&KeyCode::KeyA) {
            self.angle -= ROT_SPEED;
        }
        if keys.contains(&KeyCode::ArrowRight) || keys.contains(&KeyCode::KeyD) {
            self.angle += ROT_SPEED;
        }
    }

    fn try_move(&mut self, dx: f32, dy: f32, map: &crate::map::Map) {
        let nx = self.x + dx;
        let ny = self.y + dy;
        let margin = 0.25;
        if !map.is_solid((nx + margin * dx.signum()) as i32, self.y as i32) {
            self.x = nx;
        }
        if !map.is_solid(self.x as i32, (ny + margin * dy.signum()) as i32) {
            self.y = ny;
        }
    }
}
