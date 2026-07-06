use crate::settings::{ANT_PROJECTION_CIRCLE_DISTANCE, ANT_PROJECTION_CIRCLE_RADIUS};
use rand::{RngExt as _, rngs::SmallRng};
use raylib::prelude::*;
use std::ops::Range;

#[derive(Debug, Default, Clone)]
pub struct Navigation {
    pub position: Vector2,
    pub velocity: Vector2,
    pub wander_angle: f32,
    pub max_speed: f32,
    pub max_force: f32,
    pub current_steering_force: Vector2,
    pub wander_circle: Vector2,
    pub wander_circle_displacement: Vector2,
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
            wander_circle: Vector2::default(),
            wander_circle_displacement: Vector2::default(),
            current_steering_force: Vector2::default(),
        }
    }

    /// Returns the ant's current scalar speed (magnitude of velocity)
    pub fn current_speed(&self) -> f32 {
        self.velocity.length()
    }

    /// Calculates wander force using delta_time for angle drift,
    /// executes physics mutation, and returns the updated position.
    pub fn wander(&mut self, delta_time: f32, rng: &mut SmallRng) -> Vector2 {
        let circle_radius = ANT_PROJECTION_CIRCLE_RADIUS;
        let circle_distance = ANT_PROJECTION_CIRCLE_DISTANCE;
        let change = 8.0;

        let random_offset = rng.random_range::<f32, _>(-1.0..=1.0);
        self.wander_angle += random_offset * change * delta_time;

        let mut circle_center = self.velocity;
        if circle_center.length() > 0.0 {
            circle_center = circle_center.normalize() * circle_distance;
        } else {
            circle_center = Vector2::new(0.0, -1.0) * circle_distance;
        }

        self.wander_circle = circle_center;

        let displacement = Vector2::new(
            self.wander_angle.cos() * circle_radius,
            self.wander_angle.sin() * circle_radius,
        );

        self.wander_circle_displacement = displacement;
        let wander_force = circle_center + displacement;

        self.current_steering_force = if wander_force.length() > self.max_force {
            wander_force.normalize() * self.max_force
        } else {
            wander_force
        };

        self.calculate_next_position(delta_time)
    }

    /// Calculates force pointing at a target, updates physics,
    /// and returns the updated position.
    pub fn seek(&mut self, target: Vector2, delta_time: f32) -> Vector2 {
        let mut desired = target - self.position;

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

    /// Directly forces a lateral banking turn, scales the
    /// internal wander angle to match, updates physics, and returns position.
    pub fn turn_right(&mut self, delta_time: f32) -> Vector2 {
        let forward = if self.velocity.length_sqr() > 0.0 {
            self.velocity.normalize()
        } else {
            Vector2::new(1.0, 0.0)
        };

        let turn_right_dir = Vector2::new(-forward.y, forward.x);
        self.velocity += turn_right_dir * self.max_force * 3.0 * delta_time;

        if self.velocity.length() > self.max_speed {
            self.velocity = self.velocity.normalize() * self.max_speed;
        }

        self.wander_angle = self.velocity.y.atan2(self.velocity.x);
        self.position += self.velocity * delta_time;
        self.position
    }

    /// Directly forces a lateral banking turn to the left,
    /// scales the internal wander angle counter-clockwise, updates physics, and returns position.
    pub fn turn_left(&mut self, delta_time: f32) -> Vector2 {
        let forward = if self.velocity.length_sqr() > 0.0 {
            self.velocity.normalize()
        } else {
            Vector2::new(1.0, 0.0)
        };

        let turn_left_dir = Vector2::new(forward.y, -forward.x);
        self.velocity += turn_left_dir * self.max_force * 3.0 * delta_time;

        if self.velocity.length() > self.max_speed {
            self.velocity = self.velocity.normalize() * self.max_speed;
        }

        self.wander_angle = self.velocity.y.atan2(self.velocity.x);
        self.position += self.velocity * delta_time;
        self.position
    }

    /// Instantly flips velocity and wander vectors 180 degrees,
    /// processes the physical position step, and returns the new position.
    pub fn turn_around(
        &mut self,
        panic_angle: Range<f32>,
        delta_time: f32,
        rng: &mut SmallRng,
    ) -> Vector2 {
        self.velocity *= -1.0;
        let panic_angle = rng.random_range(panic_angle).to_radians();
        self.velocity = self.velocity.rotate(panic_angle);
        self.wander_angle += std::f32::consts::PI + panic_angle;

        if self.wander_angle > std::f32::consts::PI {
            self.wander_angle -= 2.0 * std::f32::consts::PI;
        } else if self.wander_angle < -std::f32::consts::PI {
            self.wander_angle += 2.0 * std::f32::consts::PI;
        }

        self.current_steering_force = Vector2::zero();
        self.position += self.velocity * delta_time;
        self.position
    }

    /// Private internal engine method to cleanly consolidate movement code
    fn calculate_next_position(&mut self, delta_time: f32) -> Vector2 {
        self.velocity += self.current_steering_force * delta_time;

        if self.velocity.length() > self.max_speed {
            self.velocity = self.velocity.normalize() * self.max_speed;
        }

        self.position += self.velocity * delta_time;
        self.position
    }
}
