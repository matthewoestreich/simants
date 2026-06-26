mod render;
mod world;

use crate::world::{Grid, World};
use rand::RngExt as _;
use raylib::prelude::*;

const NUM_ANTS: usize = 1000;
const SCREEN_WIDTH: i32 = 800;
const SCREEN_HEIGHT: i32 = 600;
const CELL_SIZE: i32 = 4;

// Pixels per second
const ANT_MAX_SPEED: f32 = 40.0;
// How fast they can turn
// Hihger value creates sharper turns
const ANT_MAX_TURN_FORCE: f32 = 30.0;
// The angle range an ant can turn..
// The range will become `random_rangge(-ANT_TURN_ANGLE..ANT_TURN_ANGLE)`
// Ideally should be less than 180.0
const ANT_TURN_ANGLE: f32 = 30.0;

// For rendering our ant triangle
const ANT_LENGTH: f32 = 10.0;
const ANT_WIDTH: f32 = 6.0;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Ant Simulation")
        .build();

    let mut world = World::new(SCREEN_WIDTH, SCREEN_HEIGHT, CELL_SIZE, NUM_ANTS);

    while !rl.window_should_close() {
        world.update(rl.get_frame_time());

        let mut drawing = rl.begin_drawing(&thread);
        drawing.clear_background(Color::BLACK);

        world.draw(&mut drawing);
    }
}

#[derive(Default, Debug, Clone)]
pub struct Ant {
    pub id: i32,
    pub position: Vector2,
    pub velocity: Vector2,
    pub start_position: Vector2,
    pub state: AntState,
    pub angle: f32,
    pub steering_force: Vector2,
    rng: rand::rngs::ThreadRng,
}

impl Ant {
    pub fn new(position: Vector2) -> Self {
        let mut rng = rand::rng();
        let angle = rng.random_range(0.0f32..360.0f32).to_radians();
        let velocity = Vector2::new(angle.cos(), angle.sin()) * ANT_MAX_SPEED;

        Self {
            position,
            start_position: position,
            rng,
            angle,
            velocity,
            ..Self::default()
        }
    }

    pub fn update(&mut self, dt: f32, grid: &mut Grid<CellContents>) {
        let mut desired_velocity = self.velocity;

        match self.state {
            AntState::Foraging => {
                self.angle = self
                    .rng
                    .random_range(-ANT_TURN_ANGLE..ANT_TURN_ANGLE)
                    .to_radians();
                desired_velocity = desired_velocity.rotate(self.angle);

                /*
                // Check if food is found.
                if self.is_at_food(grid) {
                    // pick up food
                    self.state = AntState::ReturningHome;
                }
                */
            }
            AntState::ReturningHome => unimplemented!(),
        };

        if self.is_at_obstacle(grid) {
            self.turn_around();
        }

        // Steering force using Reynolds steering formula
        desired_velocity = desired_velocity.normalize() * ANT_MAX_SPEED;
        let steering_force = desired_velocity - self.velocity;

        self.steering_force = steering_force.scale(ANT_MAX_TURN_FORCE * dt);
        self.velocity += self.steering_force;
        self.velocity = self.velocity.normalize() * ANT_MAX_SPEED;
        self.position += self.velocity * dt;
    }

    pub fn draw(&self, d: &mut RaylibDrawHandle) {
        let color = match self.state {
            AntState::Foraging => Color::GREEN,
            AntState::ReturningHome => Color::YELLOW,
        };

        let forward = self.velocity.normalize();
        let right = Vector2::new(-forward.y, forward.x);

        let spear = self.position + (forward * (ANT_LENGTH / 2.0));
        let left_back =
            self.position - (forward * (ANT_LENGTH / 2.0)) - (right * (ANT_WIDTH / 2.0));
        let right_back =
            self.position - (forward * (ANT_LENGTH / 2.0)) + (right * (ANT_WIDTH / 2.0));

        d.draw_triangle(spear, left_back, right_back, color);
    }

    fn turn_around(&mut self) {
        self.velocity *= -1.0;
    }

    fn is_at_obstacle(&mut self, grid: &Grid<CellContents>) -> bool {
        let look_ahead_distance = ANT_LENGTH / 2.0;
        let forward_direction = self.velocity.normalize();
        let check_position = self.position + (forward_direction * look_ahead_distance);
        let x = (check_position.x / grid.cell_size as f32).floor() as i32;
        let y = (check_position.y / grid.cell_size as f32).floor() as i32;

        grid.get(x, y)
            .map(|cell| matches!(cell.contents, CellContents::Obstacle))
            .unwrap_or(true)
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub enum AntState {
    #[default]
    Foraging,
    ReturningHome,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Food {
    pub pos_x: i32,
    pub pos_y: i32,
    pub amount: i32,
}

#[derive(Default, Debug, Clone, Copy)]
pub enum Pheromone {
    #[default]
    Foraging,
    Returning,
}

#[derive(Default, Debug, Clone, Copy)]
pub enum CellContents {
    #[default]
    Empty,
    Pheromone(Pheromone),
    Food(Food),
    Obstacle,
}
