use crate::map::{Map, Cell};

pub const SCREEN_W: usize = 640;
pub const SCREEN_H: usize = 480;
pub const NUM_RAYS: usize = SCREEN_W;
pub const FOV: f32 = std::f32::consts::PI / 3.0;
pub const HALF_FOV: f32 = FOV / 2.0;

#[allow(dead_code)]
pub struct RayHit {
    pub distance: f32,
    pub cell: Cell,
    pub side: Side,
    pub wall_x: f32,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Side {
    Vertical,
    Horizontal,
}

pub fn cast_rays(px: f32, py: f32, angle: f32, map: &Map) -> Vec<RayHit> {
    let mut hits = Vec::with_capacity(NUM_RAYS);

    for i in 0..NUM_RAYS {
        let ray_angle = angle - HALF_FOV + (i as f32 / NUM_RAYS as f32) * FOV;
        let hit = dda(px, py, ray_angle, map);
        hits.push(hit);
    }

    hits
}

fn dda(px: f32, py: f32, angle: f32, map: &Map) -> RayHit {
    let dir_x = angle.cos();
    let dir_y = angle.sin();

    let mut map_x = px as i32;
    let mut map_y = py as i32;

    let delta_dist_x = if dir_x == 0.0 { f32::INFINITY } else { (1.0 / dir_x).abs() };
    let delta_dist_y = if dir_y == 0.0 { f32::INFINITY } else { (1.0 / dir_y).abs() };

    let (step_x, mut side_dist_x) = if dir_x < 0.0 {
        (-1i32, (px - map_x as f32) * delta_dist_x)
    } else {
        (1i32, (map_x as f32 + 1.0 - px) * delta_dist_x)
    };

    let (step_y, mut side_dist_y) = if dir_y < 0.0 {
        (-1i32, (py - map_y as f32) * delta_dist_y)
    } else {
        (1i32, (map_y as f32 + 1.0 - py) * delta_dist_y)
    };

    let mut side;
    let mut cell;

    loop {
        if side_dist_x < side_dist_y {
            side_dist_x += delta_dist_x;
            map_x += step_x;
            side = Side::Vertical;
        } else {
            side_dist_y += delta_dist_y;
            map_y += step_y;
            side = Side::Horizontal;
        }

        cell = map.cell_at(map_x, map_y);
        if cell != Cell::Empty {
            break;
        }
    }

    let perp_wall_dist = match side {
        Side::Vertical => side_dist_x - delta_dist_x,
        Side::Horizontal => side_dist_y - delta_dist_y,
    };

    let wall_x = match side {
        Side::Vertical => py + perp_wall_dist * dir_y,
        Side::Horizontal => px + perp_wall_dist * dir_x,
    };
    let wall_x = wall_x - wall_x.floor();

    RayHit {
        distance: perp_wall_dist.max(0.001),
        cell,
        side,
        wall_x,
    }
}

pub fn compute_column_height(distance: f32) -> u32 {
    let h = (SCREEN_H as f32 / distance) as u32;
    h.min(SCREEN_H as u32)
}

pub fn wall_color(cell: &Cell, side: &Side) -> [f32; 4] {
    let base = match cell {
        Cell::Wall(1) => [0.6, 0.6, 0.6, 1.0],
        Cell::Wall(2) => [0.7, 0.4, 0.2, 1.0],
        Cell::Wall(3) => [0.4, 0.4, 0.6, 1.0],
        Cell::Door    => [0.6, 0.5, 0.1, 1.0],
        _             => [0.5, 0.5, 0.5, 1.0],
    };
    if *side == Side::Horizontal {
        [base[0] * 0.6, base[1] * 0.6, base[2] * 0.6, 1.0]
    } else {
        base
    }
}
