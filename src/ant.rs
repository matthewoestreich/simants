use crate::*;
use rand::RngExt as _;

#[derive(Default, Debug, Clone, Copy)]
pub enum AntState {
    #[default]
    Foraging,
    ReturningFood,
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
    pub harvested_food: f32,
}

impl AntColony {
    pub fn new(num_ants: usize, radius: f32, position: Vector2) -> Self {
        Self {
            num_ants,
            radius,
            area: radius * radius,
            ants: Vec::with_capacity(num_ants),
            harvested_food: 0.0,
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
            ant.id = i as i32;
            self.ants.insert(i, ant);
        }
    }

    pub fn draw(&self, d: &mut RaylibDrawHandle, offset_x: i32, offset_y: i32) {
        let color = COLONY_COLOR;
        let ox = offset_x as f32;
        let oy = offset_y as f32;
        let position_with_offset = Vector2::new(ox + self.position.x, oy + self.position.y);
        d.draw_circle_v(position_with_offset, self.radius, color);
    }
}

/* ---------------------------------------------------------------- */
/* -------------- SensorReadings ---------------------------------- */
/* ---------------------------------------------------------------- */

#[derive(Default, Debug, Clone)]
pub struct SensorSamples {
    pub left: CellSample,
    pub center: CellSample,
    pub right: CellSample,
}

/* ---------------------------------------------------------------- */
/* -------------- Ant --------------------------------------------- */
/* ---------------------------------------------------------------- */

#[derive(Default, Debug, Clone)]
pub struct Ant {
    pub id: i32,
    pub position: Vector2,
    pub velocity: Vector2,
    pub speed: f32,
    pub state: AntState,
    pub steering_force: Vector2,
    pub food: f32,
    pub total_food_harvested: f32,
    pub paused: f32, // Amount of time left in pause. 0.0 means we are not paused.

    pheromone_tank: f32,
    sensors: Option<(Vector2, Vector2, Vector2)>,
    sensor_samples: SensorSamples,
    rng: rand::rngs::ThreadRng,
}

impl Ant {
    pub fn new(position: Vector2) -> Self {
        let mut rng = rand::rng();
        let forward_direction = rng.random_range(0.0f32..360.0f32).to_radians();

        Self {
            position,
            rng,
            speed: ANT_MAX_SPEED,
            pheromone_tank: ANT_MAX_PHEROMONE_CAPACITY,
            velocity: Vector2::new(forward_direction.cos(), forward_direction.sin())
                * ANT_MAX_SPEED,
            ..Self::default()
        }
    }

    /// Updates current sensor samples and returns current cell
    pub fn sense_environment<'a>(&mut self, grid: &'a mut Grid) -> &'a mut Cell {
        // Rotate forward heading vector to find antenna paths
        let center_dir = if self.velocity.length_sqr() > 0.0 {
            self.velocity.normalize()
        } else {
            Vector2::new(1.0, 0.0) // Fallback heading
        };

        let left_dir = center_dir.rotate(-ANT_SENSOR_ANGLE);
        let right_dir = center_dir.rotate(ANT_SENSOR_ANGLE);

        let sensor_distance = (grid.cell_size * ANT_SENSOR_DISTANCE) as f32;
        let left_position = self.position + (left_dir * sensor_distance);
        let center_position = self.position + (center_dir * sensor_distance);
        let right_position = self.position + (right_dir * sensor_distance);
        self.sensors = Some((left_position, center_position, right_position));

        self.sensor_samples.left =
            grid.sample_position_with_pheromone_bias(left_position, self.state);
        self.sensor_samples.center =
            grid.sample_position_with_pheromone_bias(center_position, self.state);
        self.sensor_samples.right =
            grid.sample_position_with_pheromone_bias(right_position, self.state);

        grid.get_cell_mut_from_position(self.position)
            .expect("current position to always be valid")
    }

    pub fn calculate_next_position(&mut self, delta_time: f32) -> Option<Vector2> {
        let steering_angle = {
            // If we are looking for food and spotted food.
            if self.is_foraging()
                && let Some(angle) = self.steer_towards_terrain(Terrain::Food)
            {
                angle.to_radians()
            }
            // If we are returning food and spotted the colony, steer towards it.
            else if self.is_returning_food()
                && let Some(angle) = self.steer_towards_terrain(Terrain::Colony)
            {
                angle.to_radians()
            }
            // Pheromone based steering, try to sense pheromones to tell us where to go
            // go straight
            else if self.sensor_samples.center.pheromone_bias
                > self.sensor_samples.left.pheromone_bias
                && self.sensor_samples.center.pheromone_bias
                    > self.sensor_samples.right.pheromone_bias
            {
                0.0f32.to_radians()
            }
            // go left
            else if self.sensor_samples.left.pheromone_bias
                > self.sensor_samples.right.pheromone_bias
            {
                -ANT_SENSOR_ANGLE.to_radians()
            }
            // go right
            else if self.sensor_samples.right.pheromone_bias
                > self.sensor_samples.left.pheromone_bias
            {
                ANT_SENSOR_ANGLE.to_radians()
            }
            // wander randomly
            else {
                self.rng
                    .random_range(-ANT_TURN_ANGLE..ANT_TURN_ANGLE)
                    .to_radians()
            }
        };

        self.apply_speed_wobble(ANT_MAX_SPEED, delta_time);
        self.apply_steering(steering_angle, delta_time);

        Some(self.position + self.velocity * delta_time)
    }

    pub fn apply_steering(&mut self, steering_angle: f32, delta_time: f32) {
        let desired_velocity = self.velocity.rotate(steering_angle).normalize() * self.speed;
        let steering_force = desired_velocity - self.velocity;
        self.steering_force = steering_force.scale(ANT_MAX_TURN_FORCE * delta_time);
        self.velocity += self.steering_force;
        if self.velocity.length_sqr() > 0.001 {
            self.velocity = self.velocity.normalize() * self.speed;
        }
    }

    pub fn has_sensed(&self, t: Terrain) -> bool {
        let samples = &self.sensor_samples;
        samples.center.terrain == t || samples.left.terrain == t || samples.right.terrain == t
    }

    pub fn apply_speed_wobble(&mut self, mut target_speed: f32, delta_time: f32) {
        match self.state {
            AntState::Foraging => {
                target_speed += self
                    .rng
                    .random_range(-ANT_SPEED_WOBBLE_PERCENT..=ANT_SPEED_WOBBLE_PERCENT)
            }
            AntState::ReturningFood => target_speed *= ANT_CARRYING_FOOD_SPEED_PENALTY_PERCENT,
        }

        self.speed = self.speed + (target_speed - self.speed) * ANT_ACCELERATION_RATE * delta_time;
    }

    // If the provided terrain is not sensed we return None and thereore do not change steering.
    pub fn steer_towards_terrain(&self, t: Terrain) -> Option<f32> {
        if !self.has_sensed(t) {
            return None;
        }

        let samples = &self.sensor_samples;
        let lb = samples.left.pheromone_bias;
        let cb = samples.center.pheromone_bias;
        let rb = samples.right.pheromone_bias;

        if cb > lb && cb > rb {
            return Some(0.0);
        }
        if lb > cb && lb > rb {
            return Some(-ANT_SENSOR_ANGLE);
        }
        if rb > cb && rb > lb {
            return Some(ANT_SENSOR_ANGLE);
        }

        // In the edge case where all pheromone bias' are the same, prefer going straight
        Some(0.0f32)
    }

    // Gets a random number from ANT_HARVEST_AMOUNT_RANGE and takes it from
    // capacity_to_harvest_from, then returns amount harvested
    pub fn harvest(&mut self, capacity_to_harvest_from: f32) -> f32 {
        let amount = self
            .rng
            .random_range(ANT_HARVEST_AMOUNT_RANGE)
            .min(capacity_to_harvest_from);
        self.harvest_amount(amount)
    }

    // Attempts to harvest the exact amount provided. Returns amount harvested.
    pub fn harvest_amount(&mut self, amount: f32) -> f32 {
        if amount <= 0.0 {
            return 0.0;
        }
        self.food += amount;
        self.state = AntState::ReturningFood;
        amount
    }

    pub fn deliver_food(&mut self) {
        self.total_food_harvested += self.food;
        self.food = 0.0;
        self.state = AntState::Foraging;
    }

    pub fn steer_towards_position(&mut self, target: Vector2, delta_time: f32) {
        let to_target = target - self.position;
        if to_target.length_sqr() <= 0.001 {
            return;
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
        let steering_angle = angle_diff.clamp(-max_turn_rate, max_turn_rate);
        self.apply_steering(steering_angle, delta_time);
        self.position += self.velocity * delta_time;
    }

    pub fn handle_pause(&mut self, decrease_pause_time_by: f32) {
        if self.paused > 0.0 {
            self.pheromone_tank += 0.001;
            self.paused = (self.paused - decrease_pause_time_by).max(0.0);
        } else if self.should_pause(ANT_PAUSE_PROBABILITY) {
            self.paused = self.rng.random_range(ANT_PAUSE_FOR_RANGE_IN_SEC);
        }
    }

    pub fn set_pheromone_tank(&mut self, value: f32) {
        self.pheromone_tank = value.max(0.0);
    }

    pub fn get_pheromones_remaining(&self) -> f32 {
        self.pheromone_tank
    }

    pub fn lose_pheromones(&mut self, value: f32) {
        self.pheromone_tank = (self.pheromone_tank - value).max(0.0);
    }

    #[allow(dead_code)]
    pub fn add_pheromones(&mut self, value: f32) {
        self.pheromone_tank += value;
        if self.is_out_of_pheromones() {
            self.state = AntState::Foraging;
        }
    }

    pub fn is_paused(&self) -> bool {
        self.paused > 0.0
    }

    pub fn is_foraging(&self) -> bool {
        matches!(self.state, AntState::Foraging)
    }

    pub fn is_returning_food(&self) -> bool {
        matches!(self.state, AntState::ReturningFood)
    }

    pub fn is_out_of_pheromones(&self) -> bool {
        self.pheromone_tank <= 0.0
    }

    pub fn place_pheromone(&mut self, cell: &mut Cell, strength: f32) {
        if !self.can_place_pheromone() {
            return;
        }
        if self.is_foraging() && strength > cell.to_home {
            cell.to_home = strength;
            return;
        }
        if self.is_returning_food() && strength > cell.to_food {
            cell.to_food = strength;
        }
    }

    pub fn turn_around(&mut self) {
        self.velocity *= -1.0;
        let panic_angle = self
            .rng
            .random_range(ANT_OBSTACLE_PANIC_ANGLE_RANGE)
            .to_radians();
        self.velocity = self.velocity.rotate(panic_angle);
    }

    pub fn turn_in_any_direction(&mut self) {
        let angle = self
            .rng // Wander randomly
            .random_range(-360.0f32..360.0f32)
            .to_radians();
        self.velocity = self.velocity.rotate(angle);
    }

    pub fn can_place_pheromone(&self) -> bool {
        self.is_foraging() || self.is_returning_food()
    }

    /// `probability_of_pausing` should be >= 0.0 and <= 1.0.
    /// If `probability_of_pausing` === 0.2 then there is a 20% chance o pausing.
    pub fn should_pause(&mut self, probability_of_pausing: f64) -> bool {
        !self.is_paused() && self.rng.random::<f64>() < probability_of_pausing
    }

    pub fn draw(&self, d: &mut RaylibDrawHandle, draw_sensors: bool, offset_x: i32, offset_y: i32) {
        let color = match self.state {
            AntState::Foraging => ANT_FORAGING_COLOR,
            AntState::ReturningFood => ANT_RETURNING_FOOD_COLOR,
        };

        let forward = self.velocity.normalize();
        let right = Vector2::new(-forward.y, forward.x);

        let ant_length = CELL_SIZE as f32 * ANT_SIZE_MULTIPLIER;
        let ant_width = ant_length / 2.0;

        let ox = offset_x as f32;
        let oy = offset_y as f32;

        let position = Vector2::new(ox + self.position.x, oy + self.position.y);
        let spear = position + (forward * (ant_length / 2.0));
        let left_back = position - (forward * (ant_length / 2.0)) - (right * (ant_width / 2.0));
        let right_back = position - (forward * (ant_length / 2.0)) + (right * (ant_width / 2.0));
        d.draw_triangle(spear, left_back, right_back, color);

        if draw_sensors && let Some((l, c, r)) = self.sensors {
            let mut color = match self.state {
                AntState::Foraging => FOOD_COLOR,
                AntState::ReturningFood => COLONY_COLOR,
            };
            color.a = 150;
            let screen_left_sensor = Vector2::new(ox + l.x, oy + l.y);
            let screen_center_sensor = Vector2::new(ox + c.x, oy + c.y);
            let screen_right_sensor = Vector2::new(ox + r.x, oy + r.y);
            // Draw sensor 'whiskers'
            d.draw_line_v(position, screen_left_sensor, color);
            d.draw_line_v(position, screen_center_sensor, color);
            d.draw_line_v(position, screen_right_sensor, color);
            let s = Vector2::new(2.0, 2.0);
            d.draw_rectangle_v(screen_left_sensor, s, color);
            d.draw_rectangle_v(screen_center_sensor, s, color);
            d.draw_rectangle_v(screen_right_sensor, s, color);
        }
    }
}

// For UI stuff

impl Ant {
    pub fn is_clicked(
        &self,
        mouse_screen_pos: Vector2,
        click_radius: f32,
        offset_x: i32,
        offset_y: i32,
    ) -> bool {
        let screen_ant_pos = Vector2::new(
            offset_x as f32 + self.position.x,
            offset_y as f32 + self.position.y,
        );
        let distance_squared = mouse_screen_pos.distance_sqr(screen_ant_pos);
        let click_radius_squared = click_radius * click_radius;
        distance_squared <= click_radius_squared
    }
}

impl std::fmt::Display for Ant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sensors = self.sensors.unwrap_or_default();

        writeln!(f, "{{")?;
        writeln!(f, "  id: {}", self.id)?;
        writeln!(
            f,
            "  position: {{ x: {}, y: {} }}",
            self.position.x, self.position.y
        )?;
        writeln!(
            f,
            "  velocity: {{ x: {}, y: {} }}",
            self.velocity.x, self.velocity.y
        )?;
        writeln!(f, "  speed: {}", self.speed)?;
        writeln!(f, "  pheromone_tank: {}", self.pheromone_tank)?;
        writeln!(
            f,
            "  food: {{ carrying: {}, total_harvested: {} }}",
            self.food, self.total_food_harvested
        )?;
        writeln!(f, "  state: {:?}", self.state)?;
        writeln!(
            f,
            "  steering_force: {{ x: {}, y: {} }}",
            self.steering_force.x, self.steering_force.y
        )?;
        writeln!(f, "  sensors: [")?;
        writeln!(f, "    {{ x: {}, y: {} }}", sensors.0.x, sensors.0.y)?;
        writeln!(f, "    {{ x: {}, y: {} }}", sensors.1.x, sensors.1.y)?;
        writeln!(f, "    {{ x: {}, y: {} }}", sensors.2.x, sensors.2.y)?;
        writeln!(f, "  ]")?;
        writeln!(f, "}}")
    }
}
