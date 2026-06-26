use crate::*;
use std::ops::Range;

/* ----------------------------------- */
/* ------- Simulation Options -------- */
/* ----------------------------------- */

// How many ants to initially render
pub const NUM_ANTS: usize = 50;
// We multiply CELL_SIZE by this value, which determines how large ants are.
pub const ANT_SIZE_MULTIPLIER: i32 = 4;
// Pixels per second
pub const ANT_MAX_SPEED: f32 = 50.0;
// How fast can an ant turn. The higher this value, the longer it will take an ant to face a
// difffernt direction.
pub const ANT_MAX_TURN_FORCE: f32 = 0.0;
// In radians
pub const ANT_TURN_ANGLE_RANGE: Range<f32> = -45.0..45.0;
// When an ant is at an obstacle, we furst flip the current direction it is facingg, so we need to
// turn using some degree of random variance as to prevent us from bouncing back and forth in corners.
// We call this the 'panic' angle range. In radians.
pub const ANT_OBSTACLE_PANIC_ANGLE_RANGE: Range<f32> = -20.0..20.0;
// Should be >= 0.0 and <= 1.0. For example, if the value is === 0.2 then there is a 20% chance o pausing.
pub const ANT_PAUSE_PROBABILITY: f64 = 0.0019;
// We choose a random number in this range and have the ant pause for that many seconds
pub const ANT_PAUSE_FOR_RANGE_IN_SEC: Range<f32> = 0.5..1.2;
// Pheromones slowly evaporate over time.
pub const PHEROMONE_MAX_LIFETIME_SECONDS: f32 = 10.0;

/* ----------------------------------- */
/* ------- Hide/Show Entities -------- */
/* ----------------------------------- */

pub const SHOW_BORDER: bool = false;
pub const SHOW_PHEROMONES: bool = true;

/* ----------------------------------- */
/* ------- Window/GUI Options -------- */
/* ----------------------------------- */

pub const TITLE: &str = "Ant Simulation";
// If either SCREEN_WIDTH or SCREEN_HEIGHT is <= 0 we use full screen width
pub const SCREEN_WIDTH: i32 = 1200;
// If either SCREEN_WIDTH or SCREEN_HEIGHT is <= 0 we use full screen width
pub const SCREEN_HEIGHT: i32 = 800;
// N x N pixels
pub const CELL_SIZE: i32 = 4;
pub const MAX_RGBA_VALUE: u8 = 255;

/* ----------------------------------- */
/* ------- Color Options ------------- */
/* ----------------------------------- */

pub const BACKGROUND_COLOR: Color = Color::BLACK;
pub const ANT_FORAGING_COLOR: Color = Color::GREEN;
pub const ANT_RETURNING_FOOD_COLOR: Color = Color::YELLOW;
pub const PHEROMONE_FORAGING_COLOR: Color = Color::WHITE;
pub const PHEROMONE_RETURNING_FOOD_COLOR: Color = Color::ROYALBLUE;
pub const OBSTACLE_COLOR: Color = Color::DARKMAGENTA;
