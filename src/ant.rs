use crate::{world::Grid, *};
use rand::RngExt as _;

#[derive(Default, Debug, Clone, Copy)]
pub enum AntState {
    #[default]
    Foraging,
    ReturningHome,
    Paused {
        remaining: f32,
    },
}

#[derive(Default, Debug, Clone)]
pub struct AntColony {
    pub num_ants: usize,
    pub ants: Vec<Ant>,
    pub radius: f32,
    pub position: Vector2,
}

impl AntColony {
    pub fn new(num_ants: usize, radius: f32, position: Vector2) -> Self {
        Self {
            num_ants,
            radius,
            ants: Vec::with_capacity(num_ants),
            position,
        }
    }

    pub fn new_with_immediate_spawn(num_ants: usize, radius: f32, position: Vector2) -> Self {
        let mut this = Self::new(num_ants, radius, position);
        this.spawn_ants();
        this
    }

    pub fn spawn_ants(&mut self) {
        for i in 0..self.num_ants {
            self.ants.insert(i, Ant::new(self.position));
        }
    }

    pub fn draw(&self, d: &mut RaylibDrawHandle) {
        let mut color = Color::ORANGERED;
        color.a = 150; // Semi-transparent tint
        d.draw_circle_v(self.position, self.radius, color);
    }
}

#[derive(Default, Debug, Clone)]
pub struct Ant {
    pub id: i32,
    pub position: Vector2,
    pub velocity: Vector2,
    pub start_position: Vector2,
    pub state: AntState,
    pub state_before_pause: Option<AntState>,
    pub steering_force: Vector2,
    pub food: Option<Food>,
    rng: rand::rngs::ThreadRng,
}

impl Ant {
    pub fn new(position: Vector2) -> Self {
        let mut rng = rand::rng();
        let forward_direction = rng.random_range(0.0f32..360.0f32).to_radians();

        Self {
            position,
            rng,
            start_position: position,
            velocity: Vector2::new(forward_direction.cos(), forward_direction.sin())
                * ANT_MAX_SPEED,
            ..Self::default()
        }
    }

    pub fn update(&mut self, dt: f32, grid: &mut Grid<CellContents>) {
        if let AntState::Paused { ref mut remaining } = self.state {
            *remaining -= dt;
            if *remaining <= 0.0 {
                self.state = self
                    .state_before_pause
                    .expect("this should never be None when coming out of a pause");
                self.state_before_pause = None;
            }
            return;
        } else if self.should_pause(ANT_PAUSE_PROBABILITY) {
            self.state_before_pause = Some(self.state);
            self.state = AntState::Paused {
                remaining: self.rng.random_range(ANT_PAUSE_FOR_RANGE_IN_SEC),
            };
            return;
        }

        let mut desired_velocity = self.velocity;
        // Calculate left and right sensory antenna positions
        let forward = self.velocity.normalize();
        // Define biological properties
        let sensor_angle: f32 = 35.0f32.to_radians();
        let sensor_length: f32 = (grid.cell_size * 3) as f32;
        // Rotate forward heading vector to find antenna paths
        let left_dir = forward.rotate(sensor_angle);
        let right_dir = forward.rotate(-sensor_angle);
        let left_sensor_pos = self.position + left_dir * sensor_length;
        let right_sensor_pos = self.position + right_dir * sensor_length;

        match self.state {
            AntState::Foraging => {
                // Sample the environment using the antennas
                let left_smell = self.sample_pheromone(grid, left_sensor_pos);
                let right_smell = self.sample_pheromone(grid, right_sensor_pos);

                let steering_angle = if left_smell > right_smell {
                    15.0f32.to_radians()
                } else if right_smell > left_smell {
                    -15.0f32.to_radians()
                } else {
                    // No pheromone found: fall back to a random search walk
                    self.rng.random_range(ANT_TURN_ANGLE_RANGE).to_radians()
                };

                desired_velocity = desired_velocity.rotate(steering_angle);

                /*
                if self.is_at_food(grid) {
                    self.state = AntState::ReturningHome;
                }
                */

                self.place_pheromone_at_curr_pos(grid);
            }
            AntState::ReturningHome => {
                // Implement tracking using the alternative pheromone strength type
                let left_smell = self.sample_pheromone(grid, left_sensor_pos);
                let right_smell = self.sample_pheromone(grid, right_sensor_pos);

                let steering_angle = if left_smell > right_smell {
                    15.0f32.to_radians()
                } else if right_smell > left_smell {
                    -15.0f32.to_radians()
                } else {
                    self.rng.random_range(ANT_TURN_ANGLE_RANGE).to_radians()
                };

                desired_velocity = desired_velocity.rotate(steering_angle);
                self.place_pheromone_at_curr_pos(grid);
            }
            AntState::Paused { .. } => unreachable!("handled"),
        };

        desired_velocity = desired_velocity.normalize() * ANT_MAX_SPEED;
        let steering_force = desired_velocity - self.velocity;

        self.steering_force = steering_force.scale(ANT_MAX_TURN_FORCE * dt);
        self.velocity += self.steering_force;
        self.velocity = self.velocity.normalize() * ANT_MAX_SPEED;

        let next_position = self.position + self.velocity * dt;

        if self.is_position_obstacle(next_position, grid) {
            self.turn_around();
            return;
        }

        self.pick_up_food_from_position(next_position, grid);

        self.position = next_position;
    }

    fn turn_around(&mut self) {
        self.velocity *= -1.0;
        let panic_angle = self
            .rng
            .random_range(ANT_OBSTACLE_PANIC_ANGLE_RANGE)
            .to_radians();
        self.velocity = self.velocity.rotate(panic_angle);
    }

    pub fn place_pheromone_at_curr_pos(&mut self, grid: &mut Grid<CellContents>) {
        if let Some(cell) = grid.get_from_position_mut(self.position)
            && !matches!(cell.contents.terrain, Terrain::Obstacle { .. })
        {
            match self.state {
                AntState::Foraging => {
                    cell.contents.searching_strength = PHEROMONE_MAX_LIFETIME_SECONDS;
                }
                AntState::ReturningHome => {
                    cell.contents.to_home_strength = PHEROMONE_MAX_LIFETIME_SECONDS;
                }
                AntState::Paused { .. } => unreachable!(),
            }
        }
    }

    pub fn is_position_obstacle(&mut self, position: Vector2, grid: &Grid<CellContents>) -> bool {
        grid.get_from_position(position + self.velocity.normalize())
            .map(|cell| matches!(cell.contents.terrain, Terrain::Obstacle { .. }))
            .unwrap_or(true)
    }

    pub fn pick_up_food_from_position(&mut self, position: Vector2, grid: &mut Grid<CellContents>) {
        if self.food.is_none()
            && let Some(cell) = grid.get_from_position_mut(position)
            && let Terrain::Food(ref mut f) = cell.contents.terrain
        {
            f.is_harvested = true;
            self.food = Some(*f);
            self.state = AntState::ReturningHome;
        }
    }

    /// `probability_of_pausing` should be >= 0.0 and <= 1.0.
    /// If `probability_of_pausing` === 0.2 then there is a 20% chance o pausing.
    pub fn should_pause(&mut self, probability_of_pausing: f64) -> bool {
        self.rng.random::<f64>() < probability_of_pausing
    }

    pub fn draw(&self, d: &mut RaylibDrawHandle) {
        let color = match self.state {
            AntState::Foraging | AntState::Paused { .. } => ANT_FORAGING_COLOR,
            AntState::ReturningHome => ANT_RETURNING_FOOD_COLOR,
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

    // Add helper to fetch smell strength safely from grid coords
    fn sample_pheromone(&self, grid: &Grid<CellContents>, world_pos: Vector2) -> f32 {
        let x = (world_pos.x / grid.cell_size as f32).floor() as i32;
        let y = (world_pos.y / grid.cell_size as f32).floor() as i32;

        let Some(cell) = grid.get(x, y) else {
            return 0.0;
        };

        match self.state {
            AntState::Foraging => {
                // Foraging ants are looking for food paths or tracking home back paths
                cell.contents.to_home_strength
            }
            AntState::ReturningHome => {
                // Ants heading home look for trails left by searchers leading outward
                cell.contents.searching_strength
            }
            _ => 0.0,
        }
    }
}
