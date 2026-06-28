use crate::{world::Grid, *};
use rand::RngExt as _;

#[derive(Default, Debug, Clone, Copy)]
pub enum AntState {
    #[default]
    Foraging,
    ReturningFood,
    NoEnergy,
    Paused {
        remaining: f32,
    },
}

/* ---------------------------------------------------------------- */
/* -------------- AntColony --------------------------------------- */
/* ---------------------------------------------------------------- */

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
            let mut ant = Ant::new(self.position);
            ant.colony_radius = Some(self.radius);
            self.ants.insert(i, ant);
        }
    }

    pub fn draw(&self, d: &mut RaylibDrawHandle, offset_x: i32, offset_y: i32) {
        let mut color = Color::ORANGERED;
        color.a = 150; // Semi-transparent tint
        let ox = offset_x as f32;
        let oy = offset_y as f32;
        let position_with_offset = Vector2::new(ox + self.position.x, oy + self.position.y);
        d.draw_circle_v(position_with_offset, self.radius, color);
    }
}

/* ---------------------------------------------------------------- */
/* -------------- Ant --------------------------------------------- */
/* ---------------------------------------------------------------- */

#[derive(Default, Debug, Clone)]
pub struct Ant {
    pub id: i32,
    pub position: Vector2,
    pub energy: f32,
    pub velocity: Vector2,
    pub colony_center: Vector2,
    pub colony_radius: Option<f32>,
    pub state: AntState,
    pub state_before_pause: Option<AntState>,
    pub steering_force: Vector2,
    pub food: Option<Food>,
    pub sensors: Option<(Vector2, Vector2, Vector2)>,
    rng: rand::rngs::ThreadRng,
}

impl Ant {
    pub fn new(position: Vector2) -> Self {
        let mut rng = rand::rng();
        let forward_direction = rng.random_range(0.0f32..360.0f32).to_radians();

        Self {
            position,
            rng,
            colony_center: position,
            energy: 1.0,
            velocity: Vector2::new(forward_direction.cos(), forward_direction.sin())
                * ANT_MAX_SPEED,
            ..Self::default()
        }
    }

    pub fn update(&mut self, dt: f32, grid: &mut Grid<CellContents>) {
        if self.energy <= 0.0 {
            self.state = AntState::NoEnergy;
            return;
        }

        if self.is_paused(dt) {
            return;
        }

        if self.food.is_some() && self.sensed_colony() {
            self.food = None;
            self.state = AntState::Foraging;
            self.energy = 1.0;
            self.turn_around();
            return;
        }

        let sensor_distance = (grid.cell_size * ANT_SENSOR_DISTANCE) as f32;

        // Rotate forward heading vector to find antenna paths
        let forward = self.velocity.normalize();
        let left = forward.rotate(ANT_SENSOR_ANGLE);
        let right = forward.rotate(-ANT_SENSOR_ANGLE);

        let left_sensor_pos = self.position + left * sensor_distance;
        let forward_sensor_pos = self.position + forward * sensor_distance;
        let right_sensor_pos = self.position + right * sensor_distance;

        self.sensors = Some((left_sensor_pos, forward_sensor_pos, right_sensor_pos));

        let mut left_reading = self.sample_grid(grid, left_sensor_pos);
        let forward_reading = self.sample_grid(grid, forward_sensor_pos);
        let right_reading = self.sample_grid(grid, right_sensor_pos);

        /*
                let food = match self.state {
                    AntState::Foraging => {
                        if let Some(ref mut lr) = left_reading
                            && let Terrain::Food(ref mut f) = lr.terrain
                        {
                            Some(f)
                        } else if let Some(fr) = forward_reading
                            && let Terrain::Food(f) = &mut fr.terrain
                        {
                            Some(f)
                        } else if let Some(rr) = right_reading
                            && let Terrain::Food(f) = &mut rr.terrain
                        {
                            Some(f)
                        } else {
                            None
                        }
                    }
                    AntState::ReturningFood => None,
                    AntState::NoEnergy | AntState::Paused { .. } => unreachable!(),
                };

                if let Some(f) = food {
                    if self.food.is_none() {
                        //f.is_harvested = true;
                        self.food = Some(*f);
                        self.state = AntState::ReturningFood;
                        self.energy = 1.0;
                    }
                }
        */

        let (left_smell, center_smell, right_smell) = match self.state {
            AntState::Foraging => (
                left_reading.map_or(0.0, |s| s.to_home_strength),
                forward_reading.map_or(0.0, |s| s.to_home_strength),
                right_reading.map_or(0.0, |s| s.to_home_strength),
            ),
            AntState::ReturningFood => (
                left_reading.map_or(0.0, |s| s.foraging_strength),
                forward_reading.map_or(0.0, |s| s.foraging_strength),
                right_reading.map_or(0.0, |s| s.foraging_strength),
            ),
            AntState::NoEnergy | AntState::Paused { .. } => {
                unreachable!("you removed guard clause didnt you")
            }
        };

        let steering_angle = if center_smell > left_smell && center_smell > right_smell {
            0.0f32.to_radians()
        } else if left_smell > right_smell {
            -15.0f32.to_radians()
        } else if right_smell > left_smell {
            15.0f32.to_radians()
        } else {
            // No pheromone found: fall back to a random search walk
            self.rng
                .random_range(-ANT_TURN_ANGLE..ANT_TURN_ANGLE)
                .to_radians()
        };

        self.try_place_pheromone(grid, dt * ANT_PHEROMONE_STRENGTH_DECAY);

        let desired_velocity = self.velocity.rotate(steering_angle).normalize() * ANT_MAX_SPEED;
        let steering_force = desired_velocity - self.velocity;

        self.steering_force = steering_force.scale(ANT_MAX_TURN_FORCE * dt);
        self.velocity += self.steering_force;
        self.velocity = self.velocity.normalize() * ANT_MAX_SPEED;

        let next_position = self.position + self.velocity * dt;

        if self.is_position_obstacle(next_position, grid) {
            self.turn_around();
            return;
        }

        if self.pick_up_food_from_position(next_position, grid) {
            self.turn_around();
        }

        self.position = next_position;
    }

    // Returns true if we are already paused or need to pause
    fn is_paused(&mut self, dt: f32) -> bool {
        if let AntState::Paused { ref mut remaining } = self.state {
            self.energy += 0.001;
            *remaining -= dt;
            if *remaining <= 0.0 {
                self.state = self
                    .state_before_pause
                    .expect("this should never be None when coming out of a pause");
                self.state_before_pause = None;
            }
            return true;
        }
        if self.should_pause(ANT_PAUSE_PROBABILITY) {
            self.state_before_pause = Some(self.state);
            self.state = AntState::Paused {
                remaining: self.rng.random_range(ANT_PAUSE_FOR_RANGE_IN_SEC),
            };
            return true;
        }
        false
    }

    fn turn_around(&mut self) {
        self.velocity *= -1.0;
        let panic_angle = self
            .rng
            .random_range(ANT_OBSTACLE_PANIC_ANGLE_RANGE)
            .to_radians();
        self.velocity = self.velocity.rotate(panic_angle);
    }

    fn terrain_allows_pheromone(&self, terrain: Terrain) -> bool {
        !matches!(terrain, Terrain::Obstacle { .. } | Terrain::Food(_))
    }

    fn can_place_pheromone(&self) -> bool {
        matches!(self.state, AntState::Foraging | AntState::ReturningFood)
            && !self.is_within_colony()
    }

    // Try to place a pheromone at current position.
    // If we were able to place a pheromone, we subtract specified energy loss amount from ants energy.
    pub fn try_place_pheromone(&mut self, grid: &mut Grid<CellContents>, energy_loss_amount: f32) {
        if !self.can_place_pheromone() {
            return;
        }

        if let Some(cell) = grid.get_mut_from_position(self.position)
            && self.terrain_allows_pheromone(cell.contents.terrain)
        {
            let pheromone_strength = PHEROMONE_MAX_LIFETIME_SECONDS * self.energy;

            match self.state {
                AntState::Foraging => {
                    // Only overwrite if pheromone strength is stronger than what exists
                    if pheromone_strength > cell.contents.foraging_strength {
                        cell.contents.foraging_strength = pheromone_strength;
                        self.energy = (self.energy - energy_loss_amount).max(0.0);
                    }
                }
                AntState::ReturningFood => {
                    // Only overwrite if pheromone strength is stronger than what exists
                    if pheromone_strength > cell.contents.to_home_strength {
                        cell.contents.to_home_strength = pheromone_strength;
                        self.energy = (self.energy - energy_loss_amount).max(0.0);
                    }
                }
                AntState::NoEnergy => unreachable!("you removed the guard clause didnt you"),
                AntState::Paused { .. } => unreachable!(),
            }
        };
    }

    pub fn is_position_obstacle(&mut self, position: Vector2, grid: &Grid<CellContents>) -> bool {
        grid.get_from_position(position + self.velocity.normalize())
            .map(|cell| matches!(cell.contents.terrain, Terrain::Obstacle { .. }))
            .unwrap_or(true)
    }

    pub fn is_within_colony(&self) -> bool {
        debug_assert!(self.colony_radius.is_some());
        if let Some(colony_radius) = self.colony_radius {
            let distance_squared = self.position.distance_sqr(self.colony_center);
            let radius_squared = colony_radius * colony_radius;
            return distance_squared <= radius_squared;
        }
        false
    }

    pub fn sensed_colony(&self) -> bool {
        if let Some((l, c, r)) = self.sensors
            && let Some(radius) = self.colony_radius
        {
            let colony_radius = radius * radius;
            let lds = l.distance_sqr(self.colony_center);
            let cds = c.distance_sqr(self.colony_center);
            let rds = r.distance_sqr(self.colony_center);
            return lds <= colony_radius || cds <= colony_radius || rds <= colony_radius;
        }
        false
    }

    // Returns true if food was picked up, false if not.
    pub fn pick_up_food_from_position(
        &mut self,
        position: Vector2,
        grid: &mut Grid<CellContents>,
    ) -> bool {
        if self.food.is_none()
            && let Some(cell) = grid.get_mut_from_position(position)
            && let Terrain::Food(ref mut f) = cell.contents.terrain
        {
            //f.is_harvested = true;
            self.food = Some(*f);
            self.state = AntState::ReturningFood;
            self.energy = 1.0;
            return true;
        }
        false
    }

    /// `probability_of_pausing` should be >= 0.0 and <= 1.0.
    /// If `probability_of_pausing` === 0.2 then there is a 20% chance o pausing.
    pub fn should_pause(&mut self, probability_of_pausing: f64) -> bool {
        self.rng.random::<f64>() < probability_of_pausing
    }

    pub fn draw(&self, d: &mut RaylibDrawHandle, offset_x: i32, offset_y: i32) {
        let color = match self.state {
            AntState::Foraging => ANT_FORAGING_COLOR,
            AntState::ReturningFood => ANT_RETURNING_FOOD_COLOR,
            AntState::Paused { .. } => Color::YELLOW,
            AntState::NoEnergy => Color::RED,
        };

        let forward = self.velocity.normalize();
        let right = Vector2::new(-forward.y, forward.x);

        let ant_length = (CELL_SIZE * ANT_SIZE_MULTIPLIER) as f32;
        let ant_width = ant_length / 2.0;

        let ox = offset_x as f32;
        let oy = offset_y as f32;

        let position = Vector2::new(ox + self.position.x, oy + self.position.y);
        let spear = position + (forward * (ant_length / 2.0));
        let left_back = position - (forward * (ant_length / 2.0)) - (right * (ant_width / 2.0));
        let right_back = position - (forward * (ant_length / 2.0)) + (right * (ant_width / 2.0));
        d.draw_triangle(spear, left_back, right_back, color);

        if SHOW_ANT_SENSORS && let Some((l, c, r)) = self.sensors {
            let indicator_color = Color::PINK;
            let screen_left_sensor = Vector2::new(ox + l.x, oy + l.y);
            let screen_center_sensor = Vector2::new(ox + c.x, oy + c.y);
            let screen_right_sensor = Vector2::new(ox + r.x, oy + r.y);
            // Draw sensor 'whiskers'
            d.draw_line_v(position, screen_left_sensor, indicator_color);
            d.draw_line_v(position, screen_center_sensor, indicator_color);
            d.draw_line_v(position, screen_right_sensor, indicator_color);
            d.draw_circle_v(screen_left_sensor, 1.5, indicator_color);
            d.draw_circle_v(screen_center_sensor, 1.5, indicator_color);
            d.draw_circle_v(screen_right_sensor, 1.5, indicator_color);
        }
    }

    fn sample_grid<'grid>(
        &self,
        grid: &'grid Grid<CellContents>,
        position: Vector2,
    ) -> Option<&'grid CellContents> {
        if let Some(cell) = grid.get_from_position(position) {
            return Some(&cell.contents);
        }
        None
    }

    // Add helper to fetch smell strength safely from grid coords
    fn sample_pheromone(&self, grid: &Grid<CellContents>, world_pos: Vector2) -> f32 {
        let Some(cell) = grid.get_from_position(world_pos) else {
            return 0.0;
        };

        match self.state {
            // Foraging ants are looking for food paths or tracking home back paths
            AntState::Foraging => cell.contents.to_home_strength,
            // Ants heading home look for trails left by searchers leading outward
            AntState::ReturningFood => cell.contents.foraging_strength,
            _ => 0.0,
        }
    }
}
