mod render;
mod world;

use crate::world::{Grid, World};
use rand::RngExt as _;
use raylib::prelude::*;

const SCREEN_WIDTH: i32 = 800;
const SCREEN_HEIGHT: i32 = 600;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Ant Simulation")
        .build();

    let mut world = World::new(SCREEN_WIDTH, SCREEN_HEIGHT, 4, 100);

    while !rl.window_should_close() {
        world.update(rl.get_frame_time());

        let mut drawing = rl.begin_drawing(&thread);
        drawing.clear_background(Color::BLACK);

        println!("{:?}", world.ants[0]);

        draw_world(&world, &mut drawing);
    }
}

fn draw_world(world: &World, d: &mut RaylibDrawHandle) {
    let cell_size = world.grid.cell_size;

    for y in 0..world.height {
        for x in 0..world.width {
            if let Some(cell) = world.grid.get(x, y)
                && matches!(cell.contents, CellContents::Obstacle)
            {
                d.draw_rectangle(
                    x * cell_size,
                    y * cell_size,
                    cell_size,
                    cell_size,
                    Color::RED,
                );
            }
        }
    }

    for ant in &world.ants {
        d.draw_circle_v(ant.position, 4.0, Color::GREEN);
        let head_pos = ant.position
            + Vector2::new(
                ant.forward_direction.x.cos() * 6.0,
                ant.forward_direction.y.sin() * 6.0,
            );
        d.draw_circle_v(head_pos, 2.0, Color::YELLOW);
    }
}

pub struct AntSettings;

#[allow(dead_code)]
impl AntSettings {
    const SPEED: f32 = 30.0;
    const ACCELERATION: f32 = 40.0;
    const RANDOM_STEER_MAX_DURATION: f32 = 10.0;
    const RANDOM_STEER_STRENGTH: f32 = 10.0;
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Ant {
    pub id: i32,
    pub position: Vector2,
    pub forward_direction: Vector2,
    pub velocity: Vector2,
    pub start_position: Vector2,
    pub state: AntState,
    pub random_steer_timer: f32,
    pub random_steer_force: Vector2,
}

impl Ant {
    pub fn new(position: Vector2) -> Self {
        // Facing right
        let forward_direction = Vector2::new(1.0, 0.0);
        let velocity = forward_direction * AntSettings::SPEED;

        Self {
            position,
            start_position: position,
            velocity,
            ..Self::default()
        }
    }

    pub fn update(&mut self, dt: f32, grid: &mut Grid<CellContents>) {
        self.random_steer_timer -= dt;
        self.handle_random_steering();
        self.handle_movement(dt);

        let x = (self.position.x / grid.cell_size as f32).floor() as i32;
        let y = (self.position.y / grid.cell_size as f32).floor() as i32;

        let hit_obstacle = grid
            .get_mut(x, y)
            .map(|cell| matches!(cell.contents, CellContents::Obstacle))
            .unwrap_or(true);

        if hit_obstacle {
            println!("Ant at {:?} is over grid cell at ({x},{y})", self.position);
            self.velocity = -self.velocity;
            self.forward_direction = self.velocity.normalize();
        }
    }

    fn handle_random_steering(&mut self) {
        if self.random_steer_timer <= 0.0 {
            let mut rng = rand::rng();
            let rsmd = AntSettings::RANDOM_STEER_MAX_DURATION;
            self.random_steer_timer = rng.random_range(rsmd / 3.0..rsmd);
            self.random_steer_force = self.get_random_direction(self.forward_direction)
                * AntSettings::RANDOM_STEER_STRENGTH;
        }
    }

    fn handle_movement(&mut self, dt: f32) {
        let steer_force = self.random_steer_force;

        let desired_velocity = if steer_force.length() > 0.0001 {
            steer_force.normalize() * AntSettings::SPEED
        } else {
            Vector2::zero()
        };

        // SteerTowards
        let mut steering = desired_velocity - self.velocity;
        let steering_len = steering.length();
        if steering_len > AntSettings::ACCELERATION {
            steering *= AntSettings::ACCELERATION / steering_len;
        }

        self.velocity += steering * dt;

        let speed = self.velocity.length();
        if speed > AntSettings::SPEED {
            self.velocity *= AntSettings::SPEED / speed;
        }
        // 4. forward dir (Unity line)
        let forward = if self.velocity.length() > 0.0001 {
            self.velocity.normalize()
        } else {
            Vector2::new(1.0, 0.0)
        };

        // 5. movement integration (Unity line)
        self.position += self.velocity * dt;
        self.forward_direction = forward;
    }

    fn get_random_direction(&self, reference: Vector2) -> Vector2 {
        let reference = reference.normalize();

        let mut best = Vector2::zero();
        let mut best_dot = -1.0;

        for _ in 0..4 {
            let rand = self.random_unit_vector(); // already unit length
            let dot = reference.x * rand.x + reference.y * rand.y;

            if dot > best_dot {
                best_dot = dot;
                best = rand;
            }
        }

        best
    }

    fn random_unit_vector(&self) -> Vector2 {
        let angle = rand::random::<f32>() * std::f32::consts::TAU;

        Vector2 {
            x: angle.cos(),
            y: angle.sin(),
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub enum AntState {
    #[default]
    Foraging,
    ReturningFood,
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
