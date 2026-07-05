use crate::{
    map::{Cell, CellSample, Grid, Terrain},
    reynolds::Navigation,
    settings::{
        ANT_ACCELERATION_RATE, ANT_CARRYING_FOOD_SPEED_PENALTY_PERCENT, ANT_HARVEST_AMOUNT_RANGE,
        ANT_MAX_PHEROMONE_CAPACITY, ANT_MAX_SPEED, ANT_MAX_TURN_FORCE,
        ANT_OBSTACLE_PANIC_ANGLE_MAX, ANT_OBSTACLE_PANIC_ANGLE_MIN, ANT_PAUSE_FOR_RANGE_IN_SEC,
        ANT_PAUSE_PROBABILITY, ANT_SENSOR_ANGLE, ANT_SENSOR_DISTANCE, ANT_SPEED_WOBBLE_PERCENT,
        ANT_TURN_ANGLE, EXPLORER_ANTS_TIME_TO_EXPLORE_RANGE,
    },
};
use rand::RngExt as _;
use raylib::ffi::Vector2;

/* ---------------------------------------------------------------- */
/* -------------- AntColony --------------------------------------- */
/* ---------------------------------------------------------------- */

#[derive(Default, Debug, Clone)]
pub struct AntColony {
    pub ants: Vec<Ant>,
    pub radius: f32,
    pub area: f32,
    pub position: Vector2,
    pub harvested_food: f32,
}

impl AntColony {
    pub fn new(num_ants: usize, radius: f32, position: Vector2) -> Self {
        Self {
            radius,
            area: radius * radius,
            ants: Vec::with_capacity(num_ants),
            harvested_food: 0.0,
            position,
        }
    }

    pub fn new_with_immediate_spawn(
        num_ants: usize,
        percent_of_explorer_ants: f32,
        radius: f32,
        position: Vector2,
    ) -> Self {
        let mut this = Self::new(num_ants, radius, position);
        this.spawn_ants(percent_of_explorer_ants);
        this
    }

    pub fn spawn_ants(&mut self, percent_of_explorer_ants: f32) {
        let len = if !self.ants.is_empty() {
            self.ants.len()
        } else if self.ants.capacity() > 0 {
            self.ants.capacity()
        } else {
            0
        };

        // Calculate number of ants that need to be explorers
        let mut pcnt = ((percent_of_explorer_ants / 100.0) * len as f32).ceil();

        for i in 0..len {
            let mut ant = Ant::new(self.position);

            ant.id = i as i32;
            if pcnt > 0.0 {
                let sec_remaining = ant.rng.random_range(EXPLORER_ANTS_TIME_TO_EXPLORE_RANGE);
                ant.kind = AntKind::Explorer {
                    start: sec_remaining,
                    stop: 0.0,
                };
                pcnt -= 1.0;
            }

            self.ants.insert(i, ant);
        }
    }
}

/* ---------------------------------------------------------------- */
/* -------------- Sensor ------------------------------------------ */
/* ---------------------------------------------------------------- */

#[derive(Default, Debug, Clone)]
pub struct Sensor {
    /// Sensors location in world space
    pub location: Option<Vector2>,
    /// What did this sensor pick up?
    pub reading: CellSample,
}

#[derive(Default, Debug, Clone)]
pub struct Sensors {
    pub left: Sensor,
    pub center: Sensor,
    pub right: Sensor,
}

/* ---------------------------------------------------------------- */
/* -------------- AntState ---------------------------------------- */
/* ---------------------------------------------------------------- */

#[derive(Default, Debug, Clone, Copy)]
pub enum AntState {
    #[default]
    Foraging,
    ReturningFood,
}

/* ---------------------------------------------------------------- */
/* -------------- AntKind ----------------------------------------- */
/* ---------------------------------------------------------------- */

#[derive(Default, Debug, Clone, Copy)]
pub enum AntKind {
    #[default]
    Forager,
    /// `start` = seconds remaining til we start exploring
    /// `stop` = seconds remaining til we stop exploring
    Explorer { start: f32, stop: f32 },
}

/* ---------------------------------------------------------------- */
/* -------------- Ant --------------------------------------------- */
/* ---------------------------------------------------------------- */

#[derive(Default, Debug, Clone)]
pub struct Ant {
    pub id: i32,
    pub navigator: Navigation,
    //pub position: Vector2,
    //pub velocity: Vector2,
    //pub speed: f32,
    pub state: AntState,
    pub kind: AntKind,
    //pub steering_force: Vector2,
    /// Amount of food we are currently carrying
    pub food: f32,
    /// Amount of food 'this' ant has harvested.
    pub harvested_amount: f32,
    /// Amount of time left in pause. 0.0 means we are not paused.
    pub paused: f32,

    pub last_position: Vector2,
    pub real_speed_cm_s: f32,

    pheromone_tank: f32,
    sensors: Sensors,
    rng: rand::rngs::ThreadRng,
}

impl Ant {
    pub fn new(position: Vector2) -> Self {
        let mut rng = rand::rng();
        let forward_direction = rng.random_range(0.0f32.to_radians()..360.0f32.to_radians());
        let vel = Vector2::new(forward_direction.cos(), forward_direction.sin());

        Self {
            //position,
            rng,
            navigator: Navigation::new(
                position,
                vel,
                ANT_TURN_ANGLE,
                ANT_MAX_SPEED,
                ANT_MAX_TURN_FORCE,
            ),
            //speed: ANT_MAX_SPEED,
            pheromone_tank: ANT_MAX_PHEROMONE_CAPACITY,
            //velocity: Vector2::new(forward_direction.cos(), forward_direction.sin())
            //    * ANT_MAX_SPEED,
            ..Self::default()
        }
    }

    pub fn get_sensors(&self) -> &Sensors {
        &self.sensors
    }

    /// Updates current sensor samples and returns current cell
    pub fn sense_environment<'a>(&mut self, grid: &'a mut Grid) -> &'a mut Cell {
        // Rotate forward heading vector to find antenna paths
        let center_dir = if self.navigator.velocity.length_sqr() > 0.0 {
            self.navigator.velocity.normalize()
        } else {
            Vector2::new(1.0, 0.0) // Fallback heading
        };

        let left_dir = center_dir.rotate(-ANT_SENSOR_ANGLE.to_radians());
        let right_dir = center_dir.rotate(ANT_SENSOR_ANGLE.to_radians());

        let sensor_distance = ANT_SENSOR_DISTANCE as f32;
        let left_loc = self.navigator.position + (left_dir * sensor_distance);
        let center_loc = self.navigator.position + (center_dir * sensor_distance);
        let right_loc = self.navigator.position + (right_dir * sensor_distance);

        self.sensors.left = Sensor {
            location: Some(left_loc),
            reading: grid.sample_position_with_pheromone_bias(left_loc, self.state),
        };
        self.sensors.center = Sensor {
            location: Some(center_loc),
            reading: grid.sample_position_with_pheromone_bias(center_loc, self.state),
        };
        self.sensors.right = Sensor {
            location: Some(right_loc),
            reading: grid.sample_position_with_pheromone_bias(right_loc, self.state),
        };

        grid.get_cell_mut(
            self.navigator.position.x.floor() as u32,
            self.navigator.position.y.floor() as u32,
        )
        .expect(&format!(
            "current position to always be valid, ant: {self:?}"
        ))
    }

    pub fn calculate_next_position(&mut self, delta_time: f32) -> Vector2 {
        if let Some(escape_position) = self.avoid_obstruction(delta_time) {
            escape_position
        } else if self.is_exploring() {
            self.navigator.wander(delta_time)
        } else if self.is_foraging()
            && let Some(pos) = self.sense_terrain(Terrain::Food)
        {
            self.navigator.seek(pos, delta_time)
        } else if self.is_returning_food()
            && let Some(pos) = self.sense_terrain(Terrain::Colony)
        {
            self.navigator.seek(pos, delta_time)
        } else if let Some(pos) = self.steer_towards_pheromone() {
            self.navigator.seek(pos, delta_time)
        } else {
            self.navigator.wander(delta_time)
        }
    }

    fn avoid_obstruction(&mut self, delta_time: f32) -> Option<Vector2> {
        if self.sensors.center.reading.terrain.is_obstruction() {
            Some(self.navigator.turn_around(0.0..30.0))
        } else if self.sensors.left.reading.terrain.is_obstruction()
            && let Some(_obs) = self.sensors.left.location
        {
            Some(self.navigator.turn_right(delta_time))
        } else if self.sensors.right.reading.terrain.is_obstruction()
            && let Some(_obs) = self.sensors.right.location
        {
            Some(self.navigator.turn_left(delta_time))
        } else {
            None
        }
    }

    /// Uses ants sensor readings to determine if the provided terrain was even sensed, and if
    /// it was sensed, which sensors sensed it.
    /// For every sensor that sensed provided terrain, we compare their values and return the
    /// strongest value.
    // If the provided terrain is not sensed we return None and thereore do not change steering.
    fn sense_terrain(&self, t: Terrain) -> Option<Vector2> {
        let left = self.sensors.left.reading;
        let center = self.sensors.center.reading;
        let right = self.sensors.right.reading;

        // Use a negative value as a "flag" for if the expected terrain was even found.
        let mut strongest = -1.0;
        //let mut angle = None;
        let mut sensed_location = None;

        if left.terrain == t {
            strongest = left.target_pheromone;
            sensed_location = self.sensors.left.location;
            //angle = Some(-ANT_SENSOR_ANGLE);
        }
        if center.terrain == t && (strongest < 0.0 || center.target_pheromone > strongest) {
            strongest = center.target_pheromone;
            sensed_location = self.sensors.center.location;
            //angle = Some(0.0f32);
        }
        if right.terrain == t {
            #[allow(unused_assignments)]
            if strongest < 0.0 || right.target_pheromone > strongest {
                strongest = right.target_pheromone;
                sensed_location = self.sensors.right.location;
                //angle = Some(ANT_SENSOR_ANGLE);
            }
        }

        sensed_location
        // angle
    }

    /// VALUE ALREADY RETURNED IN RDIANS!!!!
    /// Steers towards strongest target pheromone. If the strongest value is still 0.0,
    /// it means we did not sense the target pheromone, so we return None
    fn steer_towards_pheromone(&self) -> Option<Vector2> {
        let left = self.sensors.left.reading.target_pheromone;
        let center = self.sensors.center.reading.target_pheromone;
        let right = self.sensors.right.reading.target_pheromone;

        // Prefer center (the '>=' comparison)
        if center > 0.0 && center >= left && center >= right {
            return self.sensors.center.location;
        }
        if left > right {
            return self.sensors.left.location;
        }
        if right > left {
            return self.sensors.right.location;
        }
        // If left and right are equal, and at least one of them is > 0.0 (implicitly making both
        // of them > 0.0), randomly pick one.
        if left > 0.0 && left == right {
            if rand::random() {
                return self.sensors.left.location;
            }
            return self.sensors.right.location;
        }

        None // They're all 0s, target pheromone was not sensed
    }

    /*
    pub fn apply_steering(&mut self, steering_angle: f32, delta_time: f32) {
        let sa_rad = steering_angle.to_radians();
        if self.is_returning_food() {
            println!(
                "{} apply_steering | steering_angle= {steering_angle} | steering_angle_radians= {sa_rad} | velocity= {:?}",
                self.id, self.velocity
            );
        }
        let desired_velocity = self.velocity.rotate(steering_angle).normalize() * self.speed;
        if self.is_returning_food() {
            println!(
                "  {} apply_steering | desired_velocity= {desired_velocity:?}",
                self.id
            );
        }
        let steering_force = desired_velocity - self.velocity;
        if self.is_returning_food() {
            println!(
                "  {} apply_steering | steering_force= {steering_force:?}",
                self.id
            );
        }
        self.steering_force = steering_force.scale(ANT_MAX_TURN_FORCE * delta_time);
        if self.is_returning_food() {
            println!(
                "  {} apply_steering | steering_force[scaled]= {:?}",
                self.id, self.steering_force
            );
        }
        self.velocity += self.steering_force;
        if self.is_returning_food() {
            println!(
                "  {} apply_steering | updatd velocity= {:?}",
                self.id, self.velocity
            );
        }
        //if self.velocity.length_sqr() > 0.001 {
        //    self.velocity = self.velocity.normalize() * self.speed;
        //    if self.is_returning_food() {
        //        println!(
        //            "    {} apply_steering | velocity is > 0.001 | normalized velocity * selff.speed= {:?}",
        //            self.id, self.velocity,
        //        );
        //    }
        //}
    }
    */

    /*
    pub fn apply_speed_wobble(&mut self, base_speed: f32, delta_time: f32) {
        match self.state {
            AntState::Foraging => {
                let rolled_percent = self
                    .rng
                    .random_range(-ANT_SPEED_WOBBLE_PERCENT..=ANT_SPEED_WOBBLE_PERCENT);
                let max_variance_ratio = rolled_percent / 100.0;
                let smooth_wobble = max_variance_ratio * ANT_ACCELERATION_RATE * delta_time;
                self.speed += base_speed * smooth_wobble;
                let min_allowed = base_speed * (1.0 - (ANT_SPEED_WOBBLE_PERCENT / 100.0));
                let max_allowed = base_speed * (1.0 + (ANT_SPEED_WOBBLE_PERCENT / 100.0));
                self.speed = self.speed.clamp(min_allowed, max_allowed);
            }
            AntState::ReturningFood => {
                let target_speed = base_speed * ANT_CARRYING_FOOD_SPEED_PENALTY_PERCENT;
                self.speed =
                    self.speed + (target_speed - self.speed) * ANT_ACCELERATION_RATE * delta_time;
            }
        }

        if self.speed < 0.0 {
            self.speed = 0.0;
        }
    }
    */

    /*
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
    */

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

    // Returns amount delivered..
    pub fn deliver_food(&mut self) -> f32 {
        let delivered = self.food;
        self.harvested_amount += delivered;
        self.food = 0.0;
        self.state = AntState::Foraging;
        delivered
    }

    pub fn handle_pause(&mut self, decrease_pause_time_by: f32) {
        if self.paused > 0.0 {
            self.pheromone_tank += 0.001;
            self.paused = (self.paused - decrease_pause_time_by).max(0.0);
        } else if self.should_pause(ANT_PAUSE_PROBABILITY) {
            self.paused = self.rng.random_range(ANT_PAUSE_FOR_RANGE_IN_SEC);
        }
    }

    /// Returns `true` if we are exploring, `false` if not.
    pub fn explore(&mut self, delta_time: f32) -> bool {
        let AntKind::Explorer { start, stop } = &mut self.kind else {
            return false;
        };

        // We are actively exploring
        if *stop > 0.0 {
            *stop -= delta_time;

            if *stop <= 0.0 {
                *start = self.rng.random_range(5.0..10.0);
                *stop = 0.0;
                return false;
            }

            return true;
        }

        // Waiting until next exploration period.
        *start -= delta_time;

        if *start > 0.0 {
            return false;
        }

        *start = 0.0;
        *stop = self.rng.random_range(5.0..10.0);

        _ = self.navigator.wander(delta_time);

        true
    }

    fn get_random_wander_angle(&mut self) -> f32 {
        self.rng.random_range(-ANT_TURN_ANGLE..ANT_TURN_ANGLE)
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
    }

    pub fn is_paused(&self) -> bool {
        self.paused > 0.0
    }

    pub fn is_foraging(&self) -> bool {
        matches!(self.state, AntState::Foraging)
    }

    pub fn is_exploring(&self) -> bool {
        if let AntKind::Explorer { start, stop } = self.kind
            && start <= 0.0
            && stop > 0.0
        {
            return true;
        }
        false
    }

    pub fn is_returning_food(&self) -> bool {
        matches!(self.state, AntState::ReturningFood)
    }

    pub fn is_out_of_pheromones(&self) -> bool {
        self.pheromone_tank <= 0.0
    }

    pub fn place_pheromone(&mut self, cell: &mut Cell, strength: f32) {
        match self.state {
            AntState::Foraging if strength > cell.to_home => cell.to_home = strength,
            AntState::ReturningFood if strength > cell.to_food => cell.to_food = strength,
            _ => {}
        };
    }

    pub fn turn_in_any_direction(&mut self) {
        self.navigator.velocity *= -1.0;
        let angle = self
            .rng
            .random_range(0.0f32.to_radians()..360.0f32.to_radians());
        self.navigator.velocity = self.navigator.velocity.rotate(angle);
    }

    /// `probability_of_pausing` should be >= 0.0 and <= 1.0.
    /// If `probability_of_pausing` === 0.2 then there is a 20% chance o pausing.
    pub fn should_pause(&mut self, probability_of_pausing: f64) -> bool {
        !self.is_paused() && self.rng.random::<f64>() < probability_of_pausing
    }
}

// For UI stuff

impl Ant {
    pub fn is_clicked(&self, mouse_screen_pos: Vector2, click_radius: f32) -> bool {
        let distance_squared = mouse_screen_pos.distance_sqr(self.navigator.position);
        let click_radius_squared = click_radius * click_radius;
        distance_squared <= click_radius_squared
    }
}

impl std::fmt::Display for Ant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{{")?;
        writeln!(f, "  id: {}", self.id)?;
        writeln!(
            f,
            "  position: {{ x: {}, y: {} }}",
            self.navigator.position.x, self.navigator.position.y
        )?;
        writeln!(
            f,
            "  velocity: {{ x: {}, y: {} }}",
            self.navigator.velocity.x, self.navigator.velocity.y
        )?;
        writeln!(
            f,
            "  speed: {{ units: {}, cm/s: {} }}",
            self.navigator.current_speed(),
            self.real_speed_cm_s
        )?;
        writeln!(f, "  pheromone_tank: {}", self.pheromone_tank)?;
        writeln!(
            f,
            "  food: {{ carrying: {}, total_harvested: {} }}",
            self.food, self.harvested_amount
        )?;
        writeln!(f, "  state: {:?}", self.state)?;
        writeln!(f, "  kind: {:?}", self.kind)?;
        writeln!(
            f,
            "  steering_force: {{ x: {}, y: {} }}",
            self.navigator.current_steering_force.x, self.navigator.current_steering_force.y
        )?;

        writeln!(f, "  sensors: [")?;
        writeln!(f, "    {{ left: {:?} }}", self.sensors.left)?;
        writeln!(f, "    {{ center: {:?} }}", self.sensors.center)?;
        writeln!(f, "    {{ right: {:?} }}", self.sensors.right)?;
        writeln!(f, "  ]")?;
        writeln!(f, "}}")
    }
}
