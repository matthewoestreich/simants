use rand::RngExt;
use raylib::ffi::Vector2;

#[derive(Debug, Default, Clone)]
pub struct Navigation {
    pub position: Vector2,
    pub velocity: Vector2,
    pub wander_angle: f32, // Scalar angle used ONLY to calculate the random displacement vector
    pub max_speed: f32,
    pub max_force: f32,

    pub current_steering_force: Vector2,

    rng: rand::rngs::ThreadRng,
}

impl Navigation {
    pub fn new(
        position: Vector2,
        velocity: Vector2,
        wander_angle: f32,
        max_speed: f32,
        max_force: f32,
    ) -> Self {
        Self {
            position,
            velocity,
            wander_angle,
            max_speed,
            max_force,
            rng: rand::rng(),
            current_steering_force: Vector2::default(),
        }
    }

    /// Returns the ant's current scalar speed (magnitude of velocity)
    pub fn current_speed(&self) -> f32 {
        self.velocity.length()
    }

    /// Instantly flips the ant's movement direction 180 degrees
    pub fn turn_around(&mut self) {
        // Reverse the velocity vector to head in the exact opposite direction
        self.velocity = -self.velocity;

        // Mirror the steering force so it doesn't fight the new direction
        self.current_steering_force = -self.current_steering_force;

        // Rotate the wander angle by PI radians (180 degrees)
        // This ensures the next wander step stays aligned with the new heading
        self.wander_angle += std::f32::consts::PI;

        // Keep the angle bounded between -PI and PI to prevent eventual float overflow
        if self.wander_angle > std::f32::consts::PI {
            self.wander_angle -= 2.0 * std::f32::consts::PI;
        } else if self.wander_angle < -std::f32::consts::PI {
            self.wander_angle += 2.0 * std::f32::consts::PI;
        }
    }

    pub fn wander(&mut self, delta_time: f32) -> Vector2 {
        let circle_radius = 20.0;
        let circle_distance = 50.0;
        // Limits how violently the ant changes direction
        let change = 2.0; // Slightly higher value since it is now scaled by delta_time (was 0.5) 

        // Scaled the random angle change by delta_time so wandering speeds up when fast-forwarding
        let random_offset = self.rng.random_range::<f32, _>(-1.0..=1.0);
        self.wander_angle += random_offset * change * delta_time;

        // Calculate the center of the wander circle ahead of the ant
        let mut circle_center = self.velocity;
        if circle_center.length() > 0.0 {
            circle_center = circle_center.normalize() * circle_distance;
        } else {
            circle_center = Vector2::new(0.0, -1.0) * circle_distance; // Default if stopped
        }

        // Calculate displacement vector on the circle circumference using the angle
        let displacement = Vector2::new(
            self.wander_angle.cos() * circle_radius,
            self.wander_angle.sin() * circle_radius,
        );

        // Combine them to get the final target force vector
        let wander_force = circle_center + displacement;

        // Clamp the force vector to max_force
        self.current_steering_force = if wander_force.length() > self.max_force {
            wander_force.normalize() * self.max_force
        } else {
            wander_force
        };

        self.calculate_next_position(delta_time)
    }

    pub fn seek(&mut self, target: Vector2, delta_time: f32) -> Vector2 {
        // Calc desired velocity pointing directly at target
        let mut desired = target - self.position;

        // Scale desired velocity to max speed
        if desired.length() > 0.0 {
            desired = desired.normalize() * self.max_speed;
        }

        let steering_force = desired - self.velocity;

        self.current_steering_force = if steering_force.length() > self.max_force {
            steering_force.normalize() * self.max_force
        } else {
            steering_force
        };

        self.calculate_next_position(delta_time)
    }

    // Was called 'update'
    pub fn calculate_next_position(&mut self, delta_time: f32) -> Vector2 {
        // Apply steering force to velocity over time
        self.velocity += self.current_steering_force * delta_time;

        // Clamp velocity to max speed so the ant doesn't infinitely accelerate
        if self.velocity.length() > self.max_speed {
            self.velocity = self.velocity.normalize() * self.max_speed;
        }

        // Update the tracking position based on the current velocity
        self.position += self.velocity * delta_time;
        self.position
    }
}
