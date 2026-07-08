use crate::{
    ant::AntColony,
    map::{Grid, SpatialGrid, Terrain},
    settings::{ANT_MAX_PHEROMONE_CAPACITY, PHEROMONE_EVAPORATION_RATE_IN_ENVIRONMENT},
};
use rand::rngs::SmallRng;
use raylib::ffi::Vector2;

pub struct World {
    pub colony: AntColony,
    pub grid: Grid,
    pub spatial_grid: SpatialGrid,
}

impl World {
    pub fn new(mut grid: Grid, spatial_grid: SpatialGrid, colony: AntColony) -> Self {
        grid.initialize(&colony);
        Self {
            grid,
            colony,
            spatial_grid,
        }
    }

    pub fn update(&mut self, delta_time: f32, rng: &mut SmallRng) {
        for cell in self.grid.iter_mut() {
            match cell.terrain {
                Terrain::Food if cell.food <= 0.0 => cell.terrain = Terrain::Empty,
                Terrain::Empty => {
                    cell.evaporate(delta_time, PHEROMONE_EVAPORATION_RATE_IN_ENVIRONMENT)
                }
                _ => {}
            }
        }

        self.spatial_grid.clear();

        for ant in &mut self.colony.ants {
            self.spatial_grid
                .insert(ant.id as u32, ant.navigator.position, ant.food > 0.0);
        }

        // println!("spatial_grid= {:?}", self.spatial_grid);

        let colony_center = self.colony.position;

        for ant in &mut self.colony.ants {
            ant.handle_pause(delta_time, rng);
            if ant.is_paused() {
                ant.real_speed_cm_s = 0.0;
                continue;
            }

            let current_cell = ant.sense_environment(&mut self.grid);
            let is_exploring = ant.explore(delta_time, rng);

            if !is_exploring {
                // Gather food
                if ant.is_foraging() && current_cell.has_food() {
                    let harvested_amount = ant.harvest(current_cell.food, rng);
                    current_cell.food = (current_cell.food - harvested_amount).max(0.0);
                    ant.set_pheromone_tank(ANT_MAX_PHEROMONE_CAPACITY);
                }
                // Deliver food to colony
                else if ant.is_returning_food() && current_cell.is_colony() {
                    if ant.navigator.position.distance_sqr(colony_center) <= 0.1 {
                        self.colony.harvested_food += ant.deliver_food();
                        ant.set_pheromone_tank(ANT_MAX_PHEROMONE_CAPACITY);
                    } else {
                        ant.navigator.seek(colony_center, delta_time);
                        ant.navigator.calculate_next_position(delta_time);
                        continue;
                    }
                }
            }

            ant.try_place_pheromone(current_cell);
            ant.update_speed(delta_time);
            ant.update_distance_traveled();
            ant.previous_position = ant.navigator.position;
            ant.update_steering_force(delta_time, rng);

            // Ants with food don't move out of another ants way.. Ants without food yield to ants
            // with food
            let mut separation_force = Vector2::zero();
            let separation_radius = 2.0;
            let separation_weight = 0.5;

            let this_ant_has_food = ant.food > 0.0;

            self.spatial_grid.for_each_neighbor(
                ant.navigator.position,
                separation_radius,
                |other_id, other_pos, other_has_food| {
                    if other_id == ant.id as usize {
                        return;
                    }
                    //if this_ant_has_food && !other_has_food {
                    //    return;
                    //}

                    let offset = ant.navigator.position - other_pos;
                    let distance = offset.length();

                    if distance > 0.0 && distance < separation_radius {
                        let add_to_sep_force = offset.normalize() * (1.0 / distance);
                        separation_force += add_to_sep_force;
                    }
                },
            );
            ant.navigator.current_steering_force += separation_force * separation_weight;

            ant.navigator.calculate_next_position(delta_time);
            ant.update_speed(delta_time);
            ant.update_distance_traveled();
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
