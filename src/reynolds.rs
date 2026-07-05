use std::ops::Range;

use rand::RngExt;
use raylib::ffi::Vector2;

use crate::settings::{ANT_PROJECTION_CIRCLE_DISTANCE, ANT_PROJECTION_CIRCLE_RADIUS};

#[derive(Debug, Default, Clone)]
pub struct Navigation {
    pub position: Vector2,
    pub velocity: Vector2,
    pub wander_angle: f32, // Scalar angle used ONLY to calculate the random displacement vector
    pub max_speed: f32,
    pub max_force: f32,

    pub current_steering_force: Vector2,

    pub wander_circle: Vector2,
    pub wander_circle_displacement: Vector2,

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
            wander_circle: Vector2::default(),
            wander_circle_displacement: Vector2::default(),
            current_steering_force: Vector2::default(),
        }
    }

    /// Returns the ant's current scalar speed (magnitude of velocity)
    pub fn current_speed(&self) -> f32 {
        self.velocity.length()
    }

    /// Instantly flips the ant's movement direction 180 degrees
    pub fn turn_around(&mut self, panic_angle_range: Range<f32>) -> Vector2 {
        self.velocity *= -1.0;
        let panic_angle = self.rng.random_range(panic_angle_range).to_radians();
        self.velocity = self.velocity.rotate(panic_angle);
        // Mirror and rotate the wander angle so the wander target
        // stays directly in front of the ant's new heading
        self.wander_angle += std::f32::consts::PI + panic_angle;
        // Keep the angle bounded between -PI and PI to prevent float overflow
        if self.wander_angle > std::f32::consts::PI {
            self.wander_angle -= 2.0 * std::f32::consts::PI;
        } else if self.wander_angle < -std::f32::consts::PI {
            self.wander_angle += 2.0 * std::f32::consts::PI;
        }
        // A small, safe pixel nudge to push the ant's mouth away
        // from the wall tile so it doesn't re-trigger the bumper next frame.
        if self.velocity.length_sqr() > 0.0 {
            self.position += self.velocity.normalize() * 2.0;
        }
        self.position
    }

    pub fn turn_right(&mut self, delta_time: f32) -> Vector2 {
        // Get the ant's current forward heading
        let forward = if self.velocity.length_sqr() > 0.0 {
            self.velocity.normalize()
        } else {
            Vector2::new(1.0, 0.0)
        };

        // Derive a 90-degree vector pointing RIGHT
        let turn_right_dir = Vector2::new(-forward.y, forward.x);
        // Inject the lateral steering force directly into velocity
        self.velocity += turn_right_dir * self.max_force * 3.0 * delta_time;

        // Re-clamp velocity to max_speed so physics stay stable
        if self.velocity.length() > self.max_speed {
            self.velocity = self.velocity.normalize() * self.max_speed;
        }

        // Update wander_angle so its internal wander target rotates
        // with the new heading, preventing a rubber-band snap next frame
        self.wander_angle = self.velocity.y.atan2(self.velocity.x);

        self.position += self.velocity * delta_time;
        self.position
    }

    pub fn turn_left(&mut self, delta_time: f32) -> Vector2 {
        // Get the ant's current forward heading
        let forward = if self.velocity.length_sqr() > 0.0 {
            self.velocity.normalize()
        } else {
            Vector2::new(1.0, 0.0)
        };

        // Derive a 90-degree vector pointing LEFT
        let turn_left_dir = Vector2::new(forward.y, -forward.x);
        // Inject the lateral steering force directly into velocity
        self.velocity += turn_left_dir * self.max_force * 3.0 * delta_time;

        // Re-clamp velocity to max_speed
        if self.velocity.length() > self.max_speed {
            self.velocity = self.velocity.normalize() * self.max_speed;
        }

        // Update wander_angle to match the new heading
        self.wander_angle = self.velocity.y.atan2(self.velocity.x);

        self.position += self.velocity * delta_time;
        self.position
    }

    pub fn wander(&mut self, delta_time: f32) -> Vector2 {
        // ORIGINAL VALUE : 20.0;
        let circle_radius = ANT_PROJECTION_CIRCLE_RADIUS;
        // ORIGINAL VALUE : 50.0;
        let circle_distance = ANT_PROJECTION_CIRCLE_DISTANCE;
        // Limits how violently the ant changes direction
        let change = 5.0; //2.0; // Slightly higher value since it is now scaled by delta_time (was 0.5)

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

        self.wander_circle = circle_center;
        self.wander_circle_displacement = displacement;

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

    fn calculate_next_position(&mut self, delta_time: f32) -> Vector2 {
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
