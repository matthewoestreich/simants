use crate::*;
use std::ops::Range;

/* ----------------------------------- */
/* ------- Simulation Options -------- */
/* ----------------------------------- */

// How many ants to initially render
pub const NUM_ANTS: usize = 12_000;
// We multiply CELL_SIZE by this value, which determines how large ants are.
pub const ANT_SIZE_MULTIPLIER: f32 = 1.0;
// Pixels per second
pub const ANT_MAX_SPEED: f32 = 40.0;
pub const ANT_SPEED_WOBBLE_PERCENT: f32 = 25.0;
pub const ANT_ACCELERATION_RATE: f32 = 5.0;
// Ants carrying food are naturally slower. If this value is 0.70, it will slow the ant down 30%
pub const ANT_CARRYING_FOOD_SPEED_PENALTY_PERCENT: f32 = 0.70;
pub const ANT_MAX_PHEROMONE_CAPACITY: f32 = 200.0;
pub const ANT_HARVEST_AMOUNT_RANGE: Range<f32> = 10.0..50.0;
// How fast can an ant turn. The higher this value, the longer it will take an ant to face a
// difffernt direction.
pub const ANT_MAX_TURN_FORCE: f32 = 15.0;
// In radians
pub const ANT_TURN_ANGLE: f32 = 35.0; //pub const ANT_TURN_ANGLE_RANGE: Range<f32> = -25.0..25.0;
// When an ant is at an obstacle, we furst flip the current direction it is facingg, so we need to
// turn using some degree of random variance as to prevent us from bouncing back and forth in corners.
// We call this the 'panic' angle range. In radians.
pub const ANT_OBSTACLE_PANIC_ANGLE_RANGE: Range<f32> = -20.0..20.0;
// Should be >= 0.0 and <= 1.0. For example, if the value is === 0.2 then there is a 20% chance o pausing.
pub const ANT_PAUSE_PROBABILITY: f64 = 0.001;
// We choose a random number in this range and have the ant pause for that many seconds
pub const ANT_PAUSE_FOR_RANGE_IN_SEC: Range<f32> = 0.5..1.2;
// The longer an ant walks, the weaker the pheromones it drops are.
pub const ANT_PHEROMONE_LOSS_RATE: f32 = 0.5;
// Ants have 3 sensors that can 'read' what is in front of them.
// One directly ahead at some distance, another at some angle to the right of the one directly ahead,
// and one at the negative value of said angle to the left of the one directly ahead.
// This is the angle at which the right sensor will be at.
// The sensor to the left will be the opposite (negative) of this value.
pub const ANT_SENSOR_ANGLE: f32 = 45.0; // In radians
// How far in front of an ant it will read sensors.
// This number will be multiplied by the cell size.
pub const ANT_SENSOR_DISTANCE: u32 = 2;

pub const PHEROMONE_DECAY_RATE: f32 = 0.005;
pub const FOOD_CELL_MAX_AMOUNT: f32 = 1_000_000.0;

/* ----------------------------------- */
/* ------- Hide/Show Entities -------- */
/* ----------------------------------- */

pub const SHOW_ANTS: bool = true;
pub const SHOW_BORDER: bool = false;
pub const SHOW_PHEROMONES: bool = false;
pub const SHOW_ANT_SENSORS: bool = false;
pub const SHOW_GRID_LINES: bool = false;
pub const COLONY_RADIUS: f32 = 8.0;
pub const FOOD_RADIUS: f32 = 4.0;

/* ----------------------------------- */
/* ------- Window/GUI Options -------- */
/* ----------------------------------- */

pub const TITLE: &str = "Ant Simulation";
// If either SCREEN_WIDTH or SCREEN_HEIGHT is <= 0 we use full screen width
pub const SCREEN_WIDTH: i32 = 1200;
// If either SCREEN_WIDTH or SCREEN_HEIGHT is <= 0 we use full screen width
pub const SCREEN_HEIGHT: i32 = 800;
pub const GRID_WIDTH: u32 = 1200;
pub const GRID_HEIGHT: u32 = 800;
// N x N pixels
pub const CELL_SIZE: u32 = 8;
pub const MAX_RGBA_VALUE: f32 = 255.0;

/* ----------------------------------- */
/* ------- Color Options ------------- */
/* ----------------------------------- */

pub const BACKGROUND_COLOR: Color = Color::BLACK;
pub const ANT_FORAGING_COLOR: Color = Color::ORANGERED;
pub const ANT_RETURNING_FOOD_COLOR: Color = Color::LIME;
pub const PHEROMONE_FORAGING_COLOR: Color = Color::RED;
pub const PHEROMONE_RETURNING_FOOD_COLOR: Color = Color::GREEN;
pub const OBSTACLE_COLOR: Color = Color::DARKMAGENTA;
pub const COLONY_COLOR: Color = Color::ROYALBLUE;
pub const FOOD_COLOR: Color = Color::GOLD;
