pub const GRID_WIDTH: usize = 16;
pub const GRID_HEIGHT: usize = 16;
pub const MARGIN: f32 = 16.0;
pub const PLAYFIELD_WIDTH: f32 = 1024.0;
pub const PLAYFIELD_HEIGHT: f32 = 1024.0;
pub const SCREEN_WIDTH: f32 = PLAYFIELD_WIDTH + MARGIN * 2.0;
pub const SCREEN_HEIGHT: f32 = PLAYFIELD_HEIGHT + MARGIN * 2.0;
pub const PIXEL_WIDTH: f32 = GRID_WIDTH as f32 / PLAYFIELD_WIDTH;
pub const PIXEL_HEIGHT: f32 = GRID_HEIGHT as f32 / PLAYFIELD_HEIGHT;
pub const PLAYER_WIDTH: f32 = 0.75;
pub const PLAYER_HEIGHT: f32 = 0.75;
pub const PLAYER_SPEED: f32 = GRID_WIDTH as f32 / 4.0;

pub const NUM_CURSORS: i32 = 1;
pub const MAZE_GEN_TIME_MS: u64 = 100;
