pub const GRID_WIDTH: usize = 8;
pub const GRID_HEIGHT: usize = 8;
pub const MARGIN: f32 = 16.0;
pub const PLAYFIELD_WIDTH: f32 = 64.0 * 16.0;
pub const PLAYFIELD_HEIGHT: f32 = 64.0 * 16.0;
pub const SCREEN_WIDTH: f32 = PLAYFIELD_WIDTH + MARGIN * 2.0;
pub const SCREEN_HEIGHT: f32 = PLAYFIELD_HEIGHT + MARGIN * 2.0;
pub const PIXEL_WIDTH: f32 = GRID_WIDTH as f32 / PLAYFIELD_WIDTH;
pub const PIXEL_HEIGHT: f32 = GRID_HEIGHT as f32 / PLAYFIELD_HEIGHT;
pub const TILE_WIDTH: usize = PLAYFIELD_WIDTH as usize / GRID_WIDTH / 2;
pub const TILE_HEIGHT: usize = PLAYFIELD_HEIGHT as usize / GRID_HEIGHT / 2;

pub const PLAYER_WIDTH: f32 = 0.75;
pub const PLAYER_HEIGHT: f32 = 0.75;
pub const PLAYER_SPEED: f32 = GRID_WIDTH as f32 / 4.0;

pub const NUM_CURSORS: i32 = 4;

pub const BG_COLOR: [u8; 3] = [0x31, 0x99, 0x6f];
