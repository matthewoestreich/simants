use crate::{
    ant::AntColony,
    map::{Grid, SpatialGrid, Terrain},
    profiler::Profiler,
    settings::{
        ANT_MAX_PHEROMONE_CAPACITY, ANT_SEPARATION_RADIUS, ANT_SEPARATION_WEIGHT,
        PHEROMONE_EVAPORATION_RATE_IN_ENVIRONMENT,
    },
};
use rand::rngs::SmallRng;
use raylib::ffi::Vector2;
use rayon::iter::{IntoParallelRefIterator as _, ParallelIterator as _};

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

    pub fn update(&mut self, delta_time: f32, rng: &mut SmallRng, profiler: &mut Profiler) {
        let _us = profiler.scope("1. Entire Update");
        self.colony.update_spawning(delta_time, rng);

        let profiler_phero_evap_scope = profiler.scope("2. Grid (pheroEvap):");
        self.grid
            .par_iter_mut()
            .filter(|cell| {
                cell.terrain == Terrain::Empty && (cell.to_food > 0.0 || cell.to_home > 0.0)
            })
            .for_each(|phero_cell| {
                phero_cell.evaporate(delta_time, PHEROMONE_EVAPORATION_RATE_IN_ENVIRONMENT)
            });
        drop(profiler_phero_evap_scope);

        let profiler_ant_loop_scope = profiler.scope("3. EntireAntLoop:");
        for ant in &mut self.colony.ants {
            //ant.handle_pause(delta_time, rng);
            //if ant.is_paused() {
            //    ant.real_speed_cm_s = 0.0;
            //    continue;
            //}

            let profiler_sense_scope = profiler.scope("3.1. Sense Env:");
            let current_cell = ant.sense_environment(&mut self.grid);
            drop(profiler_sense_scope);

            let profiler_explore_scope = profiler.scope("3.2. ExploringCheck:");
            let is_exploring = ant.explore(delta_time, rng);
            drop(profiler_explore_scope);

            if !is_exploring {
                // Gather food
                if ant.is_foraging() && current_cell.has_food() {
                    let harvested_amount = ant.harvest(current_cell.food, rng);
                    current_cell.food = (current_cell.food - harvested_amount).max(0.0);
                    if current_cell.food <= 0.0 {
                        current_cell.terrain = Terrain::Empty;
                    }
                    ant.set_pheromone_tank(ANT_MAX_PHEROMONE_CAPACITY);
                }
                // Deliver food to colony
                else if ant.is_returning_food() && current_cell.is_colony() {
                    if ant.navigator.position.distance_sqr(self.colony.position)
                        <= self.colony.radius
                    {
                        self.colony.harvested_food += ant.deliver_food();
                        ant.set_pheromone_tank(ANT_MAX_PHEROMONE_CAPACITY);
                    } else {
                        ant.navigator.seek(self.colony.position);
                        ant.navigator.calculate_next_position(delta_time);
                        continue;
                    }
                }
            }

            {
                let _s = profiler.scope("3.3. Phero Plcmt:");
                ant.try_place_pheromone(current_cell);
            }

            ant.previous_position = ant.navigator.position;

            {
                let _s = profiler.scope("3.4. Steering Force:");
                ant.update_steering_force(delta_time, rng);
            }

            {
                let _s = profiler.scope("3.5. Calc Next Pos:");
                ant.navigator.calculate_next_position(delta_time);
            }
        }
        drop(profiler_ant_loop_scope);

        {
            let _s = profiler.scope("4. Clear Spatial Grid:");
            self.spatial_grid.clear();
        }

        {
            let _s = profiler.scope("5. Ants->SpatialGrid:");
            for ant in &mut self.colony.ants {
                self.spatial_grid
                    .insert(ant.id as u32, ant.navigator.position, ant.food > 0.0);
            }
        }

        {
            //for _ in 0..pbd_iterations {
            let profiler_pbd_collisions_scope = profiler.scope("6. PBD Collisions:");
            let displacements: Vec<Vector2> = self
                .colony
                .ants
                .par_iter()
                .map(|this_ant| {
                    let mut displacement = Vector2::zero();

                    let this_ant_id = this_ant.id as usize;
                    let this_ant_pos = this_ant.navigator.position;
                    let this_ant_has_food = this_ant.food > 0.0;

                    self.spatial_grid.for_each_neighbor(
                        this_ant_pos,
                        ANT_SEPARATION_RADIUS,
                        |other_id, other_pos, other_has_food| {
                            if other_id == this_ant_id {
                                return;
                            }
                            if this_ant_has_food && !other_has_food {
                                return;
                            }

                            let offset = this_ant_pos - other_pos;
                            let distance = offset.length();

                            if distance > 0.0 && distance < ANT_SEPARATION_RADIUS {
                                let overlap = ANT_SEPARATION_RADIUS - distance;
                                displacement += offset.normalize() * (overlap);
                            }
                        },
                    );

                    displacement * ANT_SEPARATION_WEIGHT // Controls boundary stiffness
                })
                .collect();
            drop(profiler_pbd_collisions_scope);

            {
                let _s = profiler.scope("7. SetPosViaDsplcmt:");
                for (i, ant) in self.colony.ants.iter_mut().enumerate() {
                    ant.navigator.position += displacements[i];
                }
            }
            //}
        }

        {
            let _s = profiler.scope("Final Speeds:");
            for ant in &mut self.colony.ants {
                let actual_displacement = ant.navigator.position - ant.previous_position;
                let distance_moved = actual_displacement.length();
                if delta_time > 0.0 {
                    ant.real_speed_cm_s = distance_moved / delta_time;
                } else {
                    ant.real_speed_cm_s = 0.0;
                }
                ant.total_distance_traveled_cm += distance_moved;
            }
        }
    }

    pub fn calculate_decayed_amount(strength: f32, delta_time: f32, decay_rate: f32) -> f32 {
        if strength <= 0.0 {
            return 0.0;
        }
        let decay_factor = f32::exp(-decay_rate * delta_time);
        let amount = strength * decay_factor;
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
