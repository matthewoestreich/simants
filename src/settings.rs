#![allow(unused_variables, dead_code)]

use raylib::ffi::Color;
use std::ops::Range;

/* ----------------------------------- */
/* ------- Simulation Options -------- */
/* ----------------------------------- */

// How many ants to initially render
pub const NUM_ANTS: usize = 12_000;
pub const PERCENT_OF_EXPLORER_ANTS: f32 = 1.0;
// When an explorer ants time to explore runs out, it goes exploring.
// We pick a random number in this range so not all ants go exploring at the same time.
pub const EXPLORER_ANTS_TIME_TO_EXPLORE_RANGE: Range<f32> = 5.0..10.0;
pub const ANT_LENGTH: f32 = 1.2; // in cm. 1.2 = 1.2cm
pub const ANT_WIDTH: f32 = 0.4; // in cm. 0.4 = 0.4cm
pub const ANT_MAX_SPEED: f32 = 2.5; // (2.5 seems stable) cm/s with bursts that go higher

pub const ANT_HARVEST_AMOUNT_RANGE: Range<f32> = 10.0..50.0;

// The longer an ant walks, the weaker the pheromones it drops are.
pub const ANT_PHEROMONE_LOSS_RATE: f32 = 0.5; // 0.3 - 0.5 seems to be stable
pub const ANT_PHEROMONE_GAIN_WHILE_PAUSED: f32 = 0.001;
pub const ANT_MAX_PHEROMONE_CAPACITY: f32 = 200.0;

pub const ANT_MAX_TURN_FORCE: f32 = 15.0;
pub const ANT_TURN_ANGLE: f32 = 30.0;

pub const ANT_PROJECTION_CIRCLE_RADIUS: f32 = 10.0; // 20.0 seems stable
pub const ANT_PROJECTION_CIRCLE_DISTANCE: f32 = 20.0; // 50.0 was OG value

pub const ANT_SPEED_WOBBLE_PERCENT: f32 = 50.0; // Should be btwn 0.1 and 100.0
pub const ANT_ACCELERATION_RATE: f32 = 5.0; // 5.0 seems stable
pub const ANT_CARRYING_FOOD_SPEED_PENALTY_PERCENT: f32 = 0.70; // If this value is 0.70, it will slow the ant down 30%

// When an ant is at an obstacle, we furst flip the current direction it is facingg, so we need to
// turn using some degree of random variance as to prevent us from bouncing back and forth in corners.
// We call this the 'panic' angle range. In radians.
pub const ANT_OBSTACLE_PANIC_ANGLE_MIN: f32 = -20.0;
pub const ANT_OBSTACLE_PANIC_ANGLE_MAX: f32 = 20.0;
// Should be >= 0.0 and <= 1.0. For example, if the value is === 0.2 then there is a 20% chance o pausing.
pub const ANT_PAUSE_PROBABILITY: f64 = 0.001;
// We choose a random number in this range and have the ant pause for that many seconds
pub const ANT_PAUSE_FOR_RANGE_IN_SEC: Range<f32> = 0.5..1.2;
// Ants have 3 sensors that can 'read' what is in front of them.
// One directly ahead at some distance, another at some angle to the right of the one directly ahead,
// and one at the negative value of said angle to the left of the one directly ahead.
// This is the angle at which the right sensor will be at.
// The sensor to the left will be the opposite (negative) of this value.
pub const ANT_SENSOR_ANGLE: f32 = 45.0;
// How far in front of an ant it will read sensors.
// This number will be multiplied by the cell size.
pub const ANT_SENSOR_DISTANCE: u32 = 2;

pub const PHEROMONE_DECAY_RATE: f32 = 0.005; // 0.005 seems to be stable
pub const FOOD_CELL_MAX_AMOUNT: f32 = 100_000.0;
pub const MAX_RGBA_VALUE: f32 = 255.0;

/* ----------------------------------- */
/* ------- Hide/Show Entities -------- */
/* ----------------------------------- */

pub const SHOW_ANTS: bool = true;
pub const SHOW_BORDER: bool = false;
pub const SHOW_PHEROMONES: bool = false;
pub const SHOW_ANT_SENSORS: bool = false;
pub const SHOW_GRID_LINES: bool = false;
pub const COLONY_RADIUS: f32 = 16.0;
pub const FOOD_RADIUS: f32 = 16.0;

/* ----------------------------------- */
/* ------- Window/GUI Options -------- */
/* ----------------------------------- */

pub const TITLE: &str = "Ant Simulation";

pub const WINDOW_WIDTH: i32 = 1200;
pub const WINDOW_HEIGHT: i32 = 800;

pub const WORLD_WIDTH: i32 = 1000;
pub const WORLD_HEIGHT: i32 = 600;

pub const GRID_COLS: u32 = 300; //300;
pub const GRID_ROWS: u32 = 200; //200;

/* ----------------------------------- */
/* ------- Color Options ------------- */
/* ----------------------------------- */

pub const BACKGROUND_COLOR: Color = Color::BLACK;
pub const ANT_FORAGING_COLOR: Color = Color::RED;
pub const ANT_RETURNING_FOOD_COLOR: Color = Color::LIME;
pub const PHEROMONE_FORAGING_COLOR: Color = Color::MAROON;
pub const PHEROMONE_RETURNING_FOOD_COLOR: Color = Color::GREEN;
pub const OBSTACLE_COLOR: Color = Color::DARKMAGENTA;
pub const COLONY_COLOR: Color = Color::ROYALBLUE;
pub const FOOD_COLOR: Color = Color::GOLD;
