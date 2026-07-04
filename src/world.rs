use crate::{
    ant::AntColony,
    map::{Grid, Terrain},
    settings::{ANT_MAX_PHEROMONE_CAPACITY, ANT_PHEROMONE_LOSS_RATE, PHEROMONE_DECAY_RATE},
};

pub struct World {
    pub colony: AntColony,
    pub grid: Grid,
}

impl World {
    pub fn new(grid: Grid, colony: AntColony) -> Self {
        Self { grid, colony }
    }

    pub fn update(&mut self, delta_time: f32) {
        for cell in self.grid.iter_mut() {
            match cell.terrain {
                Terrain::Food if cell.food <= 0.0 => cell.terrain = Terrain::Empty,
                Terrain::Empty => cell.evaporate(delta_time, PHEROMONE_DECAY_RATE),
                _ => {}
            }
        }

        let colony_center = self.colony.position;

        for ant in &mut self.colony.ants {
            ant.handle_pause(delta_time);
            if ant.is_paused() {
                ant.real_speed_cm_s = 0.0;
                continue;
            }

            let current_cell = ant.sense_environment(&mut self.grid);
            let is_exploring = ant.explore(delta_time);

            if !is_exploring {
                // Gather food
                if ant.is_foraging() && current_cell.is_food() {
                    // TODO : clean this up
                    let harvested_amount = ant.harvest(current_cell.food);
                    current_cell.food = (current_cell.food - harvested_amount).max(0.0);
                    ant.set_pheromone_tank(ANT_MAX_PHEROMONE_CAPACITY);
                    //ant.turn_in_any_direction();
                    ant.navigator.turn_around();
                    continue;
                }
                // Deliver food to colony
                if ant.is_returning_food() && current_cell.is_colony() {
                    // If ant is at colony center
                    if ant.navigator.position.distance_sqr(colony_center) <= 0.1 {
                        self.colony.harvested_food += ant.deliver_food();
                        ant.set_pheromone_tank(ANT_MAX_PHEROMONE_CAPACITY);
                        //ant.turn_in_any_direction();
                        ant.navigator.turn_around();
                    } else {
                        _ = ant.navigator.seek(colony_center, delta_time);
                        //ant.steer_towards_position(colony_center, delta_time);
                    }
                    continue;
                }
            }

            // Place pheromone
            if !ant.is_out_of_pheromones() && current_cell.allows_pheromones() {
                let loss_rate = ANT_PHEROMONE_LOSS_RATE;
                let remaining = ant.get_pheromones_remaining();
                let strength = World::calculate_decayed_amount(remaining, delta_time, loss_rate);
                let lost = (remaining - strength) * loss_rate;
                ant.place_pheromone(current_cell, strength);
                ant.lose_pheromones(lost);
            }

            // Handle movement
            if let Some(next_position) = ant.calculate_next_position(delta_time) {
                let cell_pos = self.grid.world_to_cell(next_position);
                if let Some(c) = self.grid.get_cell(cell_pos.0, cell_pos.1)
                    && c.is_obstruction()
                {
                    ant.navigator.turn_around();
                } else {
                    let distance_traveled_cm = ant.navigator.position.distance(ant.last_position);
                    if delta_time > 0.0 {
                        ant.real_speed_cm_s = distance_traveled_cm / delta_time;
                    } else {
                        ant.real_speed_cm_s = 0.0;
                    }
                    ant.last_position = ant.navigator.position;
                    ant.navigator.position = next_position;
                }
            }
        }
    }

    pub fn calculate_decayed_amount(strength: f32, delta_time: f32, decay_rate: f32) -> f32 {
        if strength <= 0.0 {
            return 0.0;
        }
        let factor = f32::exp(-decay_rate * delta_time);
        let amount = strength * factor;
        if amount < 0.1 {
            return 0.0;
        }
        amount
    }

    //pub fn is_same_position(position: Vector2, other_position: Vector2, cell_size: u32) -> bool {
    //    let half_size = cell_size as f32 / 2.0;
    //    let dx = (other_position.x - position.x).abs();
    //    let dy = (other_position.y - position.y).abs();
    //    dx <= half_size && dy <= half_size
    //}

    /*
    pub fn screen_to_grid_coords(&self, position: Vector2) -> Option<(u32, u32)> {
        let offset_x = self.screen_offset_x;
        let offset_y = self.screen_offset_y;

        let local_x = position.x - offset_x as f32;
        let local_y = position.y - offset_y as f32;
        let grid_x = (local_x / self.grid.cell_size as f32).floor() as i32;
        let grid_y = (local_y / self.grid.cell_size as f32).floor() as i32;

        if grid_x > 0
            && grid_y > 0
            && grid_x < self.grid.cols as i32
            && grid_y < self.grid.rows as i32
        {
            Some((grid_x as u32, grid_y as u32))
        } else {
            None
        }
    }
    */
}

/*
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exponential_pheromone_decay() {
        let initial_strength = 100.0;
        let decay_rate = 0.5;
        let delta_time = 2.0;

        let mut cell = Cell {
            to_food: initial_strength,
            to_home: initial_strength,
            terrain: Terrain::Empty,
            food: 1.0,
            x: 5,
            y: 5,
        };

        cell.evaporate(delta_time, decay_rate);
        let expected_factor = (-decay_rate * delta_time).exp();
        let expected_strength = initial_strength * expected_factor;

        println!(
            "cell.to_food= {} | expected_strength= {expected_strength}",
            cell.to_food
        );

        //let epsilon = f32::EPSILON * 100.0; // Small tolerance threshold
        assert!(
            //(cell.to_food - expected_strength).abs() < epsilon,
            cell.to_food - expected_strength == 0.0,
            "Expected {}, got {}",
            expected_strength,
            cell.to_food
        );
        assert!(
            //(cell.to_home - expected_strength).abs() < epsilon,
            cell.to_home - expected_strength == 0.0,
            "Expected {}, got {}",
            expected_strength,
            cell.to_home
        );
    }

    #[test]
    fn test_time_step_independence() {
        let initial_strength = 50.0;
        let decay_rate = 0.3;

        let mut cell_single_step = Cell {
            to_food: initial_strength,
            to_home: initial_strength,
            terrain: Terrain::Empty,
            food: 1.0,
            x: 5,
            y: 5,
        };

        let single_step_delta_time = 2.0;
        // Process 2.0 seconds in one frame
        cell_single_step.evaporate(single_step_delta_time, decay_rate);
        let single_expected_factor = (-decay_rate * single_step_delta_time).exp();
        let single_expected_strength = initial_strength * single_expected_factor;

        assert!(
            cell_single_step.to_food - single_expected_strength == 0.0,
            "Single Expected {}, got {}",
            single_expected_strength,
            cell_single_step.to_food
        );
        assert!(
            cell_single_step.to_home - single_expected_strength == 0.0,
            "Single Expected {}, got {}",
            single_expected_strength,
            cell_single_step.to_home
        );

        // MULTI STEP

        let mut cell_multi_step = Cell {
            to_food: initial_strength,
            to_home: initial_strength,
            terrain: Terrain::Empty,
            food: 1.0,
            x: 5,
            y: 5,
        };

        let multi_step_delta_time = 1.0;
        let num_multi_step_steps = 2;
        let mut multi_expected_strength = initial_strength;

        for _ in 0..num_multi_step_steps {
            cell_multi_step.evaporate(multi_step_delta_time, decay_rate);
            let step_factor = (-decay_rate * multi_step_delta_time).exp();
            multi_expected_strength *= step_factor;
        }

        println!(
            "sing_step= {{ to_food: {}, to_home: {} }} | multi_step= {{ to_food: {}, to_home: {} }}",
            cell_single_step.to_food,
            cell_single_step.to_home,
            cell_multi_step.to_food,
            cell_multi_step.to_home
        );

        assert!(
            cell_multi_step.to_food - multi_expected_strength == 0.0,
            "Multi Expected {}, got {}",
            multi_expected_strength,
            cell_multi_step.to_food
        );
        assert!(
            cell_multi_step.to_home - multi_expected_strength == 0.0,
            "Multi Expected {}, got {}",
            multi_expected_strength,
            cell_multi_step.to_home
        );

        // let epsilon = 1e-5;
        //assert!((cell_single_step.to_food - cell_multi_step.to_food).abs() < epsilon);
    }
}
*/
