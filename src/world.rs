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
        if !self.colony.is_at_max_population() {
            self.colony.update_spawning(delta_time, rng);
        }

        {
            let _s = profiler.scope("Environment                 ");

            for cell in self.grid.iter_mut() {
                match cell.terrain {
                    Terrain::Food if cell.food <= 0.0 => cell.terrain = Terrain::Empty,
                    Terrain::Empty => {
                        cell.evaporate(delta_time, PHEROMONE_EVAPORATION_RATE_IN_ENVIRONMENT)
                    }
                    _ => {}
                }
            }
        }

        let colony_center = self.colony.position;

        {
            for ant in &mut self.colony.ants {
                //ant.handle_pause(delta_time, rng);
                //if ant.is_paused() {
                //    ant.real_speed_cm_s = 0.0;
                //    continue;
                //}

                let current_cell = {
                    let _s = profiler.scope("Sense Environment           ");
                    ant.sense_environment(&mut self.grid)
                };

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
                        if ant.navigator.position.distance_sqr(colony_center)
                            <= (self.colony.radius / 2.0)
                        {
                            self.colony.harvested_food += ant.deliver_food();
                            ant.set_pheromone_tank(ANT_MAX_PHEROMONE_CAPACITY);
                        } else {
                            ant.navigator.seek(colony_center);
                            ant.navigator.calculate_next_position(delta_time);
                            continue;
                        }
                    }
                }

                {
                    let _s = profiler.scope("Pheromone Placement         ");
                    ant.try_place_pheromone(current_cell);
                }
                ant.previous_position = ant.navigator.position;
                {
                    let _s = profiler.scope("Update Steering Force       ");
                    ant.update_steering_force(delta_time, rng);
                }

                // Ants with food don't move out of another ants way.. Ants without food yield to ants
                // with food
                /*
                if !current_cell.is_colony() {
                    let _s = profiler.scope("Spatial Grid");
                    let mut separation_force = Vector2::zero();
                    let separation_radius = ANT_SEPARATION_RADIUS;
                    let separation_weight = ANT_SEPARATION_WEIGHT;
                    let this_ant_has_food = ant.food > 0.0;
                    self.spatial_grid.for_each_neighbor(
                        ant.navigator.position,
                        separation_radius,
                        |other_id, other_pos, other_has_food| {
                            if other_id == ant.id as usize {
                                return;
                            }
                            if this_ant_has_food && !other_has_food {
                                return;
                            }

                            let offset = ant.navigator.position - other_pos;
                            let distance = offset.length();

                            if distance > 0.0 && distance < separation_radius {
                                //let add_to_sep_force = offset.normalize() * (1.0 / distance);
                                let strength = (separation_radius - distance) / separation_radius;
                                //separation_force += add_to_sep_force;
                                separation_force += offset.normalize() * strength;
                            }
                        },
                    );
                    ant.navigator.current_steering_force += separation_force * separation_weight;
                }
                */

                {
                    let _s = profiler.scope("Movement                    ");
                    ant.navigator.calculate_next_position(delta_time);
                }
                //{
                //    let _s = profiler.scope("Update Speed\t");
                //    ant.update_speed(delta_time);
                //}
                //{
                //    let _s = profiler.scope("Update Distance Traveled\t");
                //    ant.update_distance_traveled();
                //}
            }
        }

        {
            let _s = profiler.scope("Clear Spatial Grid          ");
            self.spatial_grid.clear();
        }

        {
            for ant in &mut self.colony.ants {
                let _s = profiler.scope("Insert Ant Into Spatial Grid");
                self.spatial_grid
                    .insert(ant.id as u32, ant.navigator.position, ant.food > 0.0);
            }
        }

        {
            let pbd_iterations = 1;
            let separation_radius = ANT_SEPARATION_RADIUS;
            let separation_weight = ANT_SEPARATION_WEIGHT; // Controls boundary stiffness

            for _ in 0..pbd_iterations {
                let _s = profiler.scope("PBD Collision Resolution    ");

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
                            separation_radius,
                            |other_id, other_pos, other_has_food| {
                                if other_id == this_ant_id {
                                    return;
                                }
                                if this_ant_has_food && !other_has_food {
                                    return;
                                }

                                let offset = this_ant_pos - other_pos;
                                let distance = offset.length();

                                if distance > 0.0 && distance < separation_radius {
                                    let overlap = separation_radius - distance;
                                    displacement += offset.normalize() * (overlap);
                                }
                            },
                        );

                        displacement * separation_weight
                    })
                    .collect();

                for (i, ant) in self.colony.ants.iter_mut().enumerate() {
                    ant.navigator.position += displacements[i];
                }
            }
        }

        /*
        {
            let pbd_iterations = 1;
            let separation_radius = ANT_SEPARATION_RADIUS;
            let separation_weight = ANT_SEPARATION_WEIGHT;

            for _ in 0..pbd_iterations {
                let _s = profiler.scope("PBD Collision Resolution    ");
                self.spatial_grid.clear();
                for ant in &self.colony.ants {
                    self.spatial_grid
                        .insert(ant.id as u32, ant.navigator.position, ant.food > 0.0);
                }

                for i in 0..self.colony.ants.len() {
                    let mut displacement = Vector2::zero();

                    let this_ant_id = self.colony.ants[i].id as usize;
                    let this_ant_pos = self.colony.ants[i].navigator.position;
                    let this_ant_has_food = self.colony.ants[i].food > 0.0;

                    self.spatial_grid.for_each_neighbor(
                        this_ant_pos,
                        separation_radius, // Using your custom radius constant
                        |other_id, other_pos, other_has_food| {
                            if other_id == this_ant_id {
                                return;
                            }
                            if this_ant_has_food && !other_has_food {
                                return;
                            }

                            let offset = this_ant_pos - other_pos;
                            let distance = offset.length();

                            if distance > 0.0 && distance < separation_radius {
                                // Calculate how deeply they intersected your separation radius
                                let overlap = separation_radius - distance;

                                //let push_weight = if !this_ant_has_food && other_has_food {
                                //    1.0
                                //} else {
                                //    0.5
                                //};

                                // Accumulate direct position adjustments
                                displacement += offset.normalize() * (overlap); // * push_weight);
                            }
                        },
                    );
                    self.colony.ants[i].navigator.position += displacement * separation_weight;
                }
            }

            //for i in 0..self.colony.ants.len() {
            //    let ant_a_pos = self.colony.ants[i].navigator.position;
            //    let ant_a_id = self.colony.ants[i].id as usize;
            //    let mut displacement = Vector2::zero();
            //    // Query using the combined diameter distance
            //    self.spatial_grid.for_each_neighbor(
            //        ant_a_pos,
            //        min_separation, // Look far enough out to catch touching edges
            //        |other_id, other_pos, _other_has_food| {
            //            if other_id == ant_a_id {
            //                return;
            //            }
            //            let offset = ant_a_pos - other_pos;
            //            let distance = offset.length();
            //            // Trigger physics the microsecond the circles intersect
            //            if distance > 0.0 && distance < min_separation {
            //                let overlap = min_separation - distance;
            //                // Split the push 50/50 so they perfectly glide edge-to-edge
            //                displacement += offset.normalize() * (overlap * 0.5);
            //            }
            //        },
            //    );
            //    // Push position out of intersection space instantly
            //    self.colony.ants[i].navigator.position += displacement;
            //}
        }
        */

        {
            let _s = profiler.scope("Update Final Speeds         ");
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
