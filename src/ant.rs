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

pub struct SensorSamples<'a> {
    pub current: &'a mut Cell,
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
    pub state: AntState,
    pub state_before_pause: Option<AntState>,
    pub steering_force: Vector2,
    pub food: Option<Food>,

    // Should really be private
    energy: f32,
    sensors: Option<(Vector2, Vector2, Vector2)>,
    rng: rand::rngs::ThreadRng,
}

impl Ant {
    pub fn new(position: Vector2) -> Self {
        let mut rng = rand::rng();
        let forward_direction = rng.random_range(0.0f32..360.0f32).to_radians();

        Self {
            position,
            rng,
            energy: ANT_MAX_ENERGY,
            velocity: Vector2::new(forward_direction.cos(), forward_direction.sin())
                * ANT_MAX_SPEED,
            ..Self::default()
        }
    }

    // Depending upon the ants state, the pharomone kind will differ.
    pub fn sense_environment<'a>(&mut self, grid: &'a mut Grid) -> SensorSamples<'a> {
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

        let left = grid.sample_position_with_pheromone_bias(left_position, self.state);
        let center = grid.sample_position_with_pheromone_bias(center_position, self.state);
        let right = grid.sample_position_with_pheromone_bias(right_position, self.state);

        let current = grid
            .get_cell_mut_from_position(self.position)
            .expect("current position to always be valid");

        SensorSamples {
            current,
            left,
            center,
            right,
        }
    }

    pub fn calculate_next_position(
        &mut self,
        sensor_reading: &SensorSamples,
        delta_time: f32,
    ) -> Option<Vector2> {
        let steering_angle = {
            // If we hit an obstruction, turn around.
            if sensor_reading.current.terrain.is_obstruction() {
                self.velocity *= -1.0;
                // Get angle that lets us turn around behind us anywher within 90 deg
                self.get_turn_around_angle()
            }
            // If we are looking for food and sensed food, steer towards it
            else if self.is_foraging() && self.has_sensed(Terrain::Food, sensor_reading) {
                println!("found food");
                self.steer_towards_sensor(Terrain::Food, sensor_reading)
                    .to_radians()
            }
            // If we are returning food and spotted the colony, steer towards it.
            else if self.is_returning_food() && self.has_sensed(Terrain::Colony, sensor_reading) {
                self.steer_towards_sensor(Terrain::Colony, sensor_reading)
                    .to_radians()
            }
            // Pheromone based steering, try to sense pheromones to tell us where to go
            else if sensor_reading.center.pheromone_bias > sensor_reading.left.pheromone_bias
                && sensor_reading.center.pheromone_bias > sensor_reading.right.pheromone_bias
            {
                0.0f32.to_radians() // Go straight
            } else if sensor_reading.left.pheromone_bias > sensor_reading.right.pheromone_bias {
                -15.0f32.to_radians() // Go left
            } else if sensor_reading.right.pheromone_bias > sensor_reading.left.pheromone_bias {
                15.0f32.to_radians() // Go right
            } else {
                self.rng // Wander randomly
                    .random_range(-ANT_TURN_ANGLE..ANT_TURN_ANGLE)
                    .to_radians()
            }
        };

        let desired_velocity = self.velocity.rotate(steering_angle).normalize() * ANT_MAX_SPEED;
        let steering_force = desired_velocity - self.velocity;
        self.steering_force = steering_force.scale(ANT_MAX_TURN_FORCE * delta_time);
        self.velocity += self.steering_force;
        self.velocity += self.steering_force;

        Some(self.position + self.velocity * delta_time)
    }

    pub fn has_sensed(&self, t: Terrain, sensor_reading: &SensorSamples) -> bool {
        // Prefer going straight
        sensor_reading.center.terrain == t
            || sensor_reading.left.terrain == t
            || sensor_reading.right.terrain == t
    }

    pub fn steer_towards_sensor(&self, t: Terrain, sensor_reading: &SensorSamples) -> f32 {
        // Prefer going straight
        if sensor_reading.center.terrain == t {
            0.0
        } else if sensor_reading.left.terrain == t {
            -ANT_SENSOR_ANGLE
        } else if sensor_reading.right.terrain == t {
            ANT_SENSOR_ANGLE
        } else {
            0.0
        }
    }

    pub fn harvest(&mut self, from: &mut Cell) {
        let amount = self.rng.random_range(ANT_HARVEST_AMOUNT_RANGE);
        self.harvest_amount(from, amount);
    }

    pub fn harvest_amount(&mut self, from: &mut Cell, amount: f32) {
        if !self.is_foraging() {
            return;
        }
        if let Some(ref mut food) = from.food {
            *food -= amount;
            self.state = AntState::ReturningFood;
        }
    }

    pub fn deliver_food(&mut self) {
        if !self.is_returning_food() {
            return;
        }
        self.food = None;
        self.state = AntState::Foraging;
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
        }
        if self.should_pause(ANT_PAUSE_PROBABILITY) {
            self.state_before_pause = Some(self.state);
            self.state = AntState::Paused {
                remaining: self.rng.random_range(ANT_PAUSE_FOR_RANGE_IN_SEC),
            };
        }
    }

    pub fn set_energy(&mut self, value: f32) {
        self.energy = value.max(0.0);
        if self.is_out_of_energy() {
            self.state = AntState::NoEnergy;
        }
    }

    pub fn lose_energy(&mut self, value: f32) {
        self.energy = (self.energy - value).max(0.0);
        if self.is_out_of_energy() {
            self.state = AntState::NoEnergy;
        }
    }

    pub fn gain_energy(&mut self, value: f32) {
        self.energy += value;
        if self.is_out_of_energy() {
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

    pub fn place_pheromone(&mut self, cell: &mut Cell) {
        if !self.can_place_pheromone() {
            return;
        }

        if self.is_foraging() {
            let strength = ANT_PHEROMONE_STRENGTH_DECAY * self.energy;
            if strength > cell.to_home.strength() {
                cell.to_home.strengthen(strength);
            }
        } else if self.is_returning_food() {
            let strength = ANT_PHEROMONE_STRENGTH_DECAY * self.energy;
            if strength > cell.to_food.strength() {
                cell.to_food.strengthen(strength);
            }
        }
    }

    fn get_turn_around_angle(&mut self) -> f32 {
        let half_wedge = 90.0f32.to_radians();
        self.rng.random_range(-half_wedge..=half_wedge)
    }

    pub fn turn_around(&mut self) {
        self.velocity *= -1.0;
        self.velocity = self.velocity.rotate(self.get_turn_around_angle());

        // ORIGINAL

        //let panic_angle = self
        //    .rng
        //    .random_range(ANT_OBSTACLE_PANIC_ANGLE_RANGE)
        //    .to_radians();
        //self.velocity = self.velocity.rotate(panic_angle);
    }

    pub fn can_place_pheromone(&self) -> bool {
        self.is_foraging() || self.is_returning_food()
    }

    /// `probability_of_pausing` should be >= 0.0 and <= 1.0.
    /// If `probability_of_pausing` === 0.2 then there is a 20% chance o pausing.
    pub fn should_pause(&mut self, probability_of_pausing: f64) -> bool {
        !self.is_paused() && self.rng.random::<f64>() < probability_of_pausing
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
        writeln!(f, "{{")?;

        writeln!(
            f,
            "  position: {{ x: {}, y: {} }}",
            self.position.x, self.position.y
        )?;

        writeln!(f, "  energy: {}", self.energy)?;
        writeln!(f, "  food: {}", self.food.map_or(0, |f| f.amount))?;

        writeln!(
            f,
            "  velocity: {{ x: {}, y: {} }}",
            self.velocity.x, self.velocity.y
        )?;

        writeln!(
            f,
            "  state: {{ current: {:?}, before_pause: {:?}  }}",
            self.state, self.state_before_pause
        )?;

        writeln!(
            f,
            "  steering_force: {{ x: {}, y: {} }}",
            self.steering_force.x, self.steering_force.y
        )?;

        let sensors =
            self.sensors
                .unwrap_or((Vector2::default(), Vector2::default(), Vector2::default()));

        writeln!(f, "  sensors: [")?;
        writeln!(f, "    {{ x: {}, y: {} }}", sensors.0.x, sensors.0.y)?;
        writeln!(f, "    {{ x: {}, y: {} }}", sensors.1.x, sensors.1.y)?;
        writeln!(f, "    {{ x: {}, y: {} }}", sensors.2.x, sensors.2.y)?;
        writeln!(f, "  ]")?;

        writeln!(f, "}}")
    }
}
