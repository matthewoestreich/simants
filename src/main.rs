mod settings;
mod world;

pub(crate) use raylib::prelude::*;
pub(crate) use settings::*;

use crate::world::{Grid, World};
use rand::RngExt as _;

fn main() {
    let mut rl_builder = raylib::init();
    rl_builder.title(TITLE);

    if SCREEN_WIDTH <= 0 || SCREEN_HEIGHT <= 0 {
        rl_builder.fullscreen();
    } else {
        rl_builder.size(SCREEN_WIDTH, SCREEN_HEIGHT);
    }

    let (mut rl, thread) = rl_builder.build();
    rl.set_target_fps(60);

    let mut world = World::new(
        rl.get_screen_width(),
        rl.get_screen_height(),
        CELL_SIZE,
        NUM_ANTS,
    );

    while !rl.window_should_close() {
        world.update(rl.get_frame_time());

        let mut drawing = rl.begin_drawing(&thread);
        drawing.clear_background(BACKGROUND_COLOR);

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
    pub paused_for: Option<f32>,
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
        if let Some(ref mut paused_for) = self.paused_for {
            *paused_for -= dt;
            if *paused_for <= 0.0 {
                self.paused_for = None;
            }
            return;
        }
        if self.should_pause(ANT_PAUSE_PROBABILITY) {
            self.paused_for = Some(self.rng.random_range(ANT_PAUSE_FOR_RANGE_IN_SEC));
            return;
        }

        let mut desired_velocity = self.velocity;

        match self.state {
            AntState::Foraging => {
                self.angle = self.rng.random_range(ANT_TURN_ANGLE_RANGE).to_radians();
                desired_velocity = desired_velocity.rotate(self.angle);
                self.drop_pheromone(grid);

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

        // Steering force using Reynolds steering formula
        desired_velocity = desired_velocity.normalize() * ANT_MAX_SPEED;
        let steering_force = desired_velocity - self.velocity;

        self.steering_force = steering_force.scale(ANT_MAX_TURN_FORCE * dt);
        self.velocity += self.steering_force;
        self.velocity = self.velocity.normalize() * ANT_MAX_SPEED;

        let next_position = self.position + self.velocity * dt;

        if self.is_at_obstacle(next_position, 0.0, grid) {
            self.turn_around();
        } else {
            self.position = next_position;
        }
    }

    fn turn_around(&mut self) {
        self.velocity *= -1.0;
        let panic_angle = self
            .rng
            .random_range(ANT_OBSTACLE_PANIC_ANGLE_RANGE)
            .to_radians();
        self.velocity = self.velocity.rotate(panic_angle);
    }

    pub fn drop_pheromone(&mut self, grid: &mut Grid<CellContents>) {
        let x = (self.position.x / grid.cell_size as f32).floor() as i32;
        let y = (self.position.y / grid.cell_size as f32).floor() as i32;

        if let Some(cell) = grid.get_mut(x, y)
            && !matches!(cell.contents, CellContents::Obstacle { .. })
        {
            cell.contents = CellContents::Pheromone {
                kind: match self.state {
                    AntState::Foraging => Pheromone::Searching,
                    AntState::ReturningHome => Pheromone::ToHome,
                },
                strength: PHEROMONE_MAX_LIFETIME_SECONDS,
            };
        }
    }

    pub fn is_at_obstacle(
        &mut self,
        position: Vector2,
        look_ahead_distance: f32,
        grid: &Grid<CellContents>,
    ) -> bool {
        let forward_direction = self.velocity.normalize();
        let check_position = position + (forward_direction * look_ahead_distance);
        let x = (check_position.x / grid.cell_size as f32).floor() as i32;
        let y = (check_position.y / grid.cell_size as f32).floor() as i32;

        grid.get(x, y)
            .map(|cell| matches!(cell.contents, CellContents::Obstacle { .. }))
            .unwrap_or(true)
    }

    /// `probability_of_pausing` should be >= 0.0 and <= 1.0.
    /// If `probability_of_pausing` === 0.2 then there is a 20% chance o pausing.
    pub fn should_pause(&mut self, probability_of_pausing: f64) -> bool {
        let roll: f64 = self.rng.random();
        roll < probability_of_pausing
    }

    pub fn draw(&self, d: &mut RaylibDrawHandle) {
        let color = if self.paused_for.is_some() {
            Color::YELLOWGREEN
        } else {
            match self.state {
                AntState::Foraging => ANT_FORAGING_COLOR,
                AntState::ReturningHome => ANT_RETURNING_FOOD_COLOR,
            }
        };

        let forward = self.velocity.normalize();
        let right = Vector2::new(-forward.y, forward.x);

        let ant_length = (CELL_SIZE * ANT_SIZE_MULTIPLIER) as f32;
        let ant_width = ant_length / 2.0;

        let spear = self.position + (forward * (ant_length / 2.0));
        let left_back =
            self.position - (forward * (ant_length / 2.0)) - (right * (ant_width / 2.0));
        let right_back =
            self.position - (forward * (ant_length / 2.0)) + (right * (ant_width / 2.0));

        d.draw_triangle(spear, left_back, right_back, color);
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
    Searching,
    ToFood,
    ToHome,
}

#[derive(Default, Debug, Clone, Copy)]
pub enum Obstacle {
    #[default]
    Normal,
    Border,
}

#[derive(Default, Debug, Clone, Copy)]
pub enum CellContents {
    #[default]
    Empty,
    Pheromone {
        kind: Pheromone,
        // strength can be viewed as 'real world seconds to live for'
        strength: f32,
    },
    Food(Food),
    Obstacle {
        kind: Obstacle,
    },
}
