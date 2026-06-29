use crate::*;
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
    pub area: f32,
    pub position: Vector2,
}

impl AntColony {
    pub fn new(num_ants: usize, radius: f32, position: Vector2) -> Self {
        Self {
            num_ants,
            radius,
            area: radius * radius,
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
/* -------------- SensorReadings ---------------------------------- */
/* ---------------------------------------------------------------- */

#[derive(Default, Debug, Clone, Copy)]
pub struct SensorReadings<'a> {
    pub left: Option<&'a Cell>,
    pub center: Option<&'a Cell>,
    pub right: Option<&'a Cell>,
    pub left_pheromone: f32,
    pub center_pheromone: f32,
    pub right_pheromone: f32,
}

/* ---------------------------------------------------------------- */
/* -------------- Ant --------------------------------------------- */
/* ---------------------------------------------------------------- */

#[derive(Default, Debug, Clone)]
pub struct Ant {
    pub id: i32,
    pub position: Vector2,
    pub velocity: Vector2,
    pub state: AntState,
    pub state_before_pause: Option<AntState>,
    pub steering_force: Vector2,
    pub food: Option<Food>,

    // Should really be private
    pub energy: f32,
    sensors: Option<(Vector2, Vector2, Vector2)>,
    rng: rand::rngs::ThreadRng,
}

impl Ant {
    pub fn new(position: Vector2) -> Self {
        let mut rng = rand::rng();
        let forward_direction = rng.random_range(0.0f32..360.0f32).to_radians();

        let velocity =
            Vector2::new(forward_direction.cos(), forward_direction.sin()) * ANT_MAX_SPEED;

        Self {
            position,
            rng,
            energy: 1.0,
            velocity,
            ..Self::default()
        }
    }

    // Depending upon the ants state, the pharomone kind will differ.
    pub fn sense_environment<'a>(&mut self, grid: &'a Grid) -> SensorReadings<'a> {
        // Rotate forward heading vector to find antenna paths
        let center_dir = if self.velocity.length_sqr() > 0.0 {
            self.velocity.normalize()
        } else {
            Vector2::new(1.0, 0.0) // Fallback heading
        };

        let left_dir = center_dir.rotate(ANT_SENSOR_ANGLE);
        let right_dir = center_dir.rotate(-ANT_SENSOR_ANGLE);

        let sensor_distance = (grid.cell_size * ANT_SENSOR_DISTANCE) as f32;

        let left_position = self.position + (left_dir * sensor_distance);
        let center_position = self.position + (center_dir * sensor_distance);
        let right_position = self.position + (right_dir * sensor_distance);

        self.sensors = Some((left_position, center_position, right_position));

        let left = grid.get_cell_from_position(left_position);
        let center = grid.get_cell_from_position(center_position);
        let right = grid.get_cell_from_position(right_position);

        let (left_pheromone, center_pheromone, right_pheromone) = match self.state {
            AntState::Foraging => (
                left.map_or(0.0, |r| r.to_food.strength()),
                center.map_or(0.0, |r| r.to_food.strength()),
                right.map_or(0.0, |r| r.to_food.strength()),
            ),
            AntState::ReturningFood => (
                left.map_or(0.0, |r| r.to_home.strength()),
                center.map_or(0.0, |r| r.to_home.strength()),
                right.map_or(0.0, |r| r.to_home.strength()),
            ),
            _ => unreachable!(
                "\n\nExpected state to be Foraging or ReturningFood!\n\n{:?}",
                self
            ),
        };

        SensorReadings {
            left,
            center,
            right,
            left_pheromone,
            center_pheromone,
            right_pheromone,
        }
    }

    pub fn calculate_next_position(
        &mut self,
        sensor_reading: &SensorReadings<'_>,
        delta_time: f32,
    ) -> Option<Vector2> {
        let steering_angle = if self.is_foraging() && self.has_sensed(Terrain::Food, sensor_reading)
        {
            self.steer_towards_sensor(Terrain::Food, sensor_reading)
        } else if self.is_returning_food() && self.has_sensed(Terrain::Colony, sensor_reading) {
            println!("ant sensed colony!");
            self.steer_towards_sensor(Terrain::Colony, sensor_reading)
        } else if sensor_reading.center_pheromone > sensor_reading.left_pheromone
            && sensor_reading.center_pheromone > sensor_reading.right_pheromone
        {
            0.0f32.to_radians() // Go straight
        } else if sensor_reading.left_pheromone > sensor_reading.right_pheromone {
            -15.0f32.to_radians() // Go left
        } else if sensor_reading.right_pheromone > sensor_reading.left_pheromone {
            15.0f32.to_radians() // Go right
        } else {
            self.rng // Wander randomly
                .random_range(-ANT_TURN_ANGLE..ANT_TURN_ANGLE)
                .to_radians()
        };

        let desired_velocity = self.velocity.rotate(steering_angle).normalize() * ANT_MAX_SPEED;
        let steering_force = desired_velocity - self.velocity;
        self.steering_force = steering_force.scale(ANT_MAX_TURN_FORCE * delta_time);
        self.velocity += self.steering_force;
        self.velocity += self.steering_force;

        Some(self.position + self.velocity * delta_time)
    }

    pub fn has_sensed(&self, t: Terrain, sensor_reading: &SensorReadings<'_>) -> bool {
        // Prefer going straight
        sensor_reading.center.is_some_and(|r| r.terrain == t)
            || sensor_reading.left.is_some_and(|s| s.terrain == t)
            || sensor_reading.right.is_some_and(|s| s.terrain == t)
    }

    pub fn steer_towards_sensor(&self, t: Terrain, sensor_reading: &SensorReadings<'_>) -> f32 {
        // Prefer going straight
        if sensor_reading.center.is_some_and(|r| r.terrain == t) {
            0.0
        } else if sensor_reading.left.is_some_and(|r| r.terrain == t) {
            -ANT_SENSOR_ANGLE
        } else if sensor_reading.right.is_some_and(|r| r.terrain == t) {
            ANT_SENSOR_ANGLE
        } else {
            0.0
        }
    }

    pub fn steer_towards_position(&self, target: Vector2) -> f32 {
        let to_target = target - self.position;

        if to_target.length_sqr() <= 0.001 {
            return 0.0;
        }

        let current_angle = self.velocity.y.atan2(self.velocity.x);
        let target_angle = to_target.y.atan2(to_target.x);

        let mut angle_diff = target_angle - current_angle;

        while angle_diff > std::f32::consts::PI {
            angle_diff -= 2.0 * std::f32::consts::PI;
        }
        while angle_diff < -std::f32::consts::PI {
            angle_diff += 2.0 * std::f32::consts::PI;
        }

        let max_turn_rate = ANT_TURN_ANGLE.to_radians();

        angle_diff.clamp(-max_turn_rate, max_turn_rate)
    }

    /*
    pub fn update(&mut self, dt: f32, grid: &mut Grid) {
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

        let sensor_distance = (CELL_SIZE * ANT_SENSOR_DISTANCE) as f32;

        // Rotate forward heading vector to find antenna paths
        let forward = self.velocity.normalize();
        let left = forward.rotate(ANT_SENSOR_ANGLE);
        let right = forward.rotate(-ANT_SENSOR_ANGLE);

        let left_sensor_pos = self.position + left * sensor_distance;
        let forward_sensor_pos = self.position + forward * sensor_distance;
        let right_sensor_pos = self.position + right * sensor_distance;

        self.sensors = Some((left_sensor_pos, forward_sensor_pos, right_sensor_pos));

        let left_reading = self.sample_grid(grid, left_sensor_pos);
        let forward_reading = self.sample_grid(grid, forward_sensor_pos);
        let right_reading = self.sample_grid(grid, right_sensor_pos);

        let (left_smell, center_smell, right_smell) = match self.state {
            AntState::Foraging => (
                left_reading.map_or(0.0, |s| s.to_food_strength),
                forward_reading.map_or(0.0, |s| s.to_food_strength),
                right_reading.map_or(0.0, |s| s.to_food_strength),
            ),
            AntState::ReturningFood => (
                left_reading.map_or(0.0, |s| s.to_home_strength),
                forward_reading.map_or(0.0, |s| s.to_home_strength),
                right_reading.map_or(0.0, |s| s.to_home_strength),
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
    */

    pub fn handle_pause(&mut self, decrease_pause_time_by: f32) {
        if let AntState::Paused { ref mut remaining } = self.state {
            self.energy += 0.001;
            *remaining -= decrease_pause_time_by;
            if *remaining <= 0.0 {
                self.state = self
                    .state_before_pause
                    .expect("this should never be None when coming out of a pause");
                self.state_before_pause = None;
            }
        } else if self.should_pause(ANT_PAUSE_PROBABILITY) {
            self.state_before_pause = Some(self.state);
            self.state = AntState::Paused {
                remaining: self.rng.random_range(ANT_PAUSE_FOR_RANGE_IN_SEC),
            };
        }
    }

    pub fn lose_energy(&mut self, value: f32) {
        self.energy -= value;
        if self.energy <= 0.0 {
            self.state = AntState::NoEnergy;
        }
    }

    pub fn gain_energy(&mut self, value: f32) {
        self.energy += value;
        if self.energy > 0.0 && matches!(self.state, AntState::NoEnergy) {
            self.state = AntState::Foraging;
        }
    }

    pub fn is_paused(&self) -> bool {
        matches!(self.state, AntState::Paused { .. })
    }

    pub fn is_foraging(&self) -> bool {
        matches!(self.state, AntState::Foraging)
    }

    pub fn is_returning_food(&self) -> bool {
        matches!(self.state, AntState::ReturningFood)
    }

    pub fn is_out_of_energy(&self) -> bool {
        self.energy <= 0.0 || matches!(self.state, AntState::NoEnergy)
    }

    pub fn turn_around(&mut self) {
        self.velocity *= -1.0;
        let panic_angle = self
            .rng
            .random_range(ANT_OBSTACLE_PANIC_ANGLE_RANGE)
            .to_radians();
        self.velocity = self.velocity.rotate(panic_angle);
    }

    pub fn place_pheromone(&mut self, cell: &mut Cell) {
        if !self.can_place_pheromone() {
            return;
        }

        if self.is_foraging() {
            let strength = PHEROMONE_MAX_LIFETIME_SECONDS * self.energy;
            if strength > cell.to_home.strength() {
                cell.to_home.strengthen(strength);
                self.lose_energy(0.25);
            }
        } else if self.is_returning_food() {
            let strength = PHEROMONE_MAX_LIFETIME_SECONDS * self.energy;
            if strength > cell.to_food.strength() {
                cell.to_food.strengthen(strength);
                self.lose_energy(0.25);
            }
        }
    }

    pub fn can_place_pheromone(&self) -> bool {
        self.is_foraging() || self.is_returning_food()
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
}
