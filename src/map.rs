pub const MAP_WIDTH: usize = 16;
pub const MAP_HEIGHT: usize = 16;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Cell {
    Empty,
    Wall(u8),
    Door,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum EnemyKind {
    Guard,
    Ss,
    Officer,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct Enemy {
    pub kind: EnemyKind,
    pub x: f32,
    pub y: f32,
    pub alive: bool,
}

#[allow(dead_code)]
pub struct Map {
    pub walls: [[Cell; MAP_WIDTH]; MAP_HEIGHT],
    pub enemies: Vec<Enemy>,
    pub player_start: (f32, f32, f32),
}

impl Map {
    pub fn load() -> Self {
        let walls_src   = include_str!("../maps/map1_walls.txt");
        let enemies_src = include_str!("../maps/map1_enemies.txt");
        let spawn_src   = include_str!("../maps/map1_spawn.txt");
        let walls = parse_walls(walls_src);
        let (enemies, player_start) = parse_actors(enemies_src, spawn_src);
        Map { walls, enemies, player_start }
    }

    pub fn is_solid(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 || x >= MAP_WIDTH as i32 || y >= MAP_HEIGHT as i32 {
            return true;
        }
        matches!(self.walls[y as usize][x as usize], Cell::Wall(_) | Cell::Door)
    }

    #[allow(dead_code)]
    pub fn is_door(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 || x >= MAP_WIDTH as i32 || y >= MAP_HEIGHT as i32 {
            return false;
        }
        self.walls[y as usize][x as usize] == Cell::Door
    }

    pub fn cell_at(&self, x: i32, y: i32) -> Cell {
        if x < 0 || y < 0 || x >= MAP_WIDTH as i32 || y >= MAP_HEIGHT as i32 {
            return Cell::Wall(1);
        }
        self.walls[y as usize][x as usize]
    }
}

fn parse_walls(content: &str) -> [[Cell; MAP_WIDTH]; MAP_HEIGHT] {
    let mut grid = [[Cell::Empty; MAP_WIDTH]; MAP_HEIGHT];
    let mut row = 0;
    for line in content.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        if row >= MAP_HEIGHT {
            break;
        }
        for (col, token) in line.split_whitespace().enumerate() {
            if col >= MAP_WIDTH {
                break;
            }
            grid[row][col] = match token {
                "0" => Cell::Empty,
                "4" => Cell::Door,
                v => Cell::Wall(v.parse::<u8>().unwrap_or(1)),
            };
        }
        row += 1;
    }
    grid
}

fn parse_actors(enemy_content: &str, spawn_content: &str) -> (Vec<Enemy>, (f32, f32, f32)) {
    let mut enemies = Vec::new();
    let mut row = 0usize;
    for line in enemy_content.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        if row >= MAP_HEIGHT {
            break;
        }
        for (col, token) in line.split_whitespace().enumerate() {
            if col >= MAP_WIDTH {
                break;
            }
            let kind = match token {
                "1" => Some(EnemyKind::Guard),
                "2" => Some(EnemyKind::Ss),
                "3" => Some(EnemyKind::Officer),
                _ => None,
            };
            if let Some(k) = kind {
                enemies.push(Enemy {
                    kind: k,
                    x: col as f32 + 0.5,
                    y: row as f32 + 0.5,
                    alive: true,
                });
            }
        }
        row += 1;
    }

    let mut player_start = (1.5f32, 1.5f32, 0.0f32);
    let mut srow = 0usize;
    'outer: for line in spawn_content.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        if srow >= MAP_HEIGHT {
            break;
        }
        for (col, token) in line.split_whitespace().enumerate() {
            if token == "P" {
                player_start = (col as f32 + 0.5, srow as f32 + 0.5, 0.0);
                break 'outer;
            }
        }
        srow += 1;
    }

    (enemies, player_start)
}
