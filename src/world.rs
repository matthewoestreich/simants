use crate::*;

pub struct World {
    #[allow(dead_code)]
    pub screen_width: i32,
    #[allow(dead_code)]
    pub screen_height: i32,
    #[allow(dead_code)]
    pub grid_width_pixels: i32,
    #[allow(dead_code)]
    pub grid_height_pixels: i32,
    pub screen_offset_x: i32,
    pub screen_offset_y: i32,
    pub colony: AntColony,
    pub grid: Grid,

    pub show_grid: bool,
    pub show_pheromones: bool,
    pub show_border: bool,
    pub show_ant_sensors: bool,
    pub show_ants: bool,
}

pub fn is_same_position(position: Vector2, other_position: Vector2, cell_size: u32) -> bool {
    let half_size = cell_size as f32 / 2.0;
    let dx = (other_position.x - position.x).abs();
    let dy = (other_position.y - position.y).abs();
    dx <= half_size && dy <= half_size
}

impl World {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        screen_width: i32,
        screen_height: i32,
        grid_width: u32,
        grid_height: u32,
        cell_size: u32,
        colony: AntColony,
        show_grid: bool,
        show_pheromones: bool,
        show_border: bool,
        show_ant_sensors: bool,
        show_ants: bool,
    ) -> Self {
        // Calculate screen position to grid cell offset (needed to map a screen position to a cell)
        let screen_offset_x = (screen_width - grid_width as i32) / 2;
        let screen_offset_y = (screen_height - grid_height as i32) / 2;

        // Calculate number of cells per width & height pixels
        let cols = grid_width / cell_size;
        let rows = grid_height / cell_size;

        // For drawing vertical line obstacle in mid of screen
        let line_len = rows / 2;
        let line_start_y = (rows - line_len) / 2;
        let line_end_y = line_start_y + line_len;
        let mid_w = cols / 2;
        let x_range = (mid_w - 1)..=(mid_w + 1);
        let y_range = line_start_y..=line_end_y;

        // For drawing food clump
        let food_center_x = (cols * 3) / 4;
        let food_center_y = rows / 2;
        let food_radius = FOOD_RADIUS; // Radius measured in number of grid cells

        let mut grid = Grid::new(cols, rows, cell_size);
        let cell_size_f32 = cell_size as f32;

        for cell in &mut grid {
            let x = cell.x;
            let y = cell.y;

            // Mark border cells
            if x == 0 || x == cols - 1 || y == 0 || y == rows - 1 {
                cell.terrain = Terrain::Border;
                continue;
            }

            // Marks a cell as an obstacle.
            // This obstacle will eventually be drawn as a vertical line that is centerted horizontally and vertically
            if x_range.contains(&x) && y_range.contains(&y) {
                //if y % 20 <= 3 {
                //    continue;
                //}
                cell.terrain = Terrain::Obstacle;
                continue;
            }

            // Marks a cell as food.
            let food_dx = x as i32 - food_center_x as i32;
            let food_dy = y as i32 - food_center_y as i32;
            if food_dx * food_dx + food_dy * food_dy <= food_radius as i32 * food_radius as i32 {
                cell.terrain = Terrain::Food;
                cell.food = FOOD_CELL_MAX_AMOUNT;
                continue;
            }

            // Mark the underlying cells of the colony as such
            let cell_center = Vector2::new(
                (x as f32 * cell_size_f32) + (cell_size_f32 / 2.0),
                (y as f32 * cell_size_f32) + (cell_size_f32 / 2.0),
            );
            // If distance from cell center to colony center is less than or
            // equal to the colony area, it means we are in the colony.
            if cell_center.distance_sqr(colony.position) <= colony.area {
                cell.terrain = Terrain::Colony;
                continue;
            }
        }

        Self {
            screen_width,
            screen_height,
            grid_width_pixels: grid_width as i32,
            grid_height_pixels: grid_height as i32,
            screen_offset_x,
            screen_offset_y,
            grid,
            colony,
            show_grid,
            show_pheromones,
            show_border,
            show_ant_sensors,
            show_ants,
        }
    }

    pub fn toggle_show_border(&mut self) {
        self.show_border = !self.show_border;
    }

    pub fn toggle_show_pheromones(&mut self) {
        self.show_pheromones = !self.show_pheromones;
    }

    pub fn toggle_show_grid(&mut self) {
        self.show_grid = !self.show_grid;
    }

    pub fn toggle_show_ant_sensors(&mut self) {
        self.show_ant_sensors = !self.show_ant_sensors;
    }

    pub fn toggle_show_ants(&mut self) {
        self.show_ants = !self.show_ants;
    }

    pub fn update(&mut self, delta_time: f32) {
        for cell in self.grid.iter_mut() {
            if cell.is_food() && cell.food <= 0.0 {
                cell.terrain = Terrain::Empty;
                continue;
            }
            if cell.is_colony() || cell.is_obstruction() {
                continue;
            }
            cell.evaporate(delta_time, PHEROMONE_DECAY_RATE);
        }

        let colony_center = self.colony.position;

        for ant in &mut self.colony.ants {
            ant.handle_pause(delta_time);

            if ant.is_paused() {
                continue;
            }

            let current_cell = ant.sense_environment(&mut self.grid);

            if ant.is_foraging() && current_cell.is_food() {
                let harvested_amount = ant.harvest(current_cell.food);
                current_cell.food = (current_cell.food - harvested_amount).max(0.0);
                self.colony.harvested_food += harvested_amount;
                ant.set_pheromone_tank(ANT_MAX_PHEROMONE_CAPACITY);
                //ant.turn_around();
                ant.turn_in_any_direction();
                continue;
            }

            if ant.is_returning_food() && current_cell.is_colony() {
                if is_same_position(colony_center, ant.position, self.grid.cell_size) {
                    ant.deliver_food();
                    ant.set_pheromone_tank(ANT_MAX_PHEROMONE_CAPACITY);
                    ant.turn_in_any_direction();
                    continue;
                }
                ant.steer_towards_position(colony_center, delta_time);
                continue;
            }

            if !ant.is_out_of_pheromones() && current_cell.allows_pheromones() {
                let remaining_pheromones = ant.get_pheromones_remaining();
                let drop_strength = World::calculate_decayed_amount(
                    remaining_pheromones,
                    delta_time,
                    ANT_PHEROMONE_LOSS_RATE,
                );
                ant.place_pheromone(current_cell, drop_strength);
                let pheromone_loss =
                    (remaining_pheromones - drop_strength) * ANT_PHEROMONE_LOSS_RATE;
                ant.lose_pheromones(pheromone_loss);
            }

            if let Some(next_position) = ant.calculate_next_position(delta_time) {
                if self.grid.position_is_obstruction(next_position) {
                    ant.turn_around();
                    continue;
                }
                ant.position = next_position;
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

    pub fn draw(&mut self, d: &mut RaylibDrawHandle) {
        let cell_size = self.grid.cell_size as i32;
        let screen_offset_x = self.screen_offset_x;
        let screen_offset_y = self.screen_offset_y;

        for cell in &mut self.grid {
            let x = cell.x as i32;
            let y = cell.y as i32;

            match cell.terrain {
                Terrain::Obstacle => {
                    d.draw_rectangle(
                        screen_offset_x + (x * cell_size),
                        screen_offset_y + (y * cell_size),
                        cell_size,
                        cell_size,
                        OBSTACLE_COLOR,
                    );
                }
                Terrain::Border => {
                    if self.show_border {
                        d.draw_rectangle(
                            screen_offset_x + (x * cell_size),
                            screen_offset_y + (y * cell_size),
                            cell_size,
                            cell_size,
                            OBSTACLE_COLOR,
                        );
                    }
                }
                Terrain::Food => {
                    d.draw_rectangle(
                        screen_offset_x + (x * cell_size),
                        screen_offset_y + (y * cell_size),
                        cell_size,
                        cell_size,
                        FOOD_COLOR,
                    );
                }
                Terrain::Empty | Terrain::Colony => {
                    // Draw standard empty background
                    // Colonies are drawn directly on screen.
                    d.draw_rectangle(
                        screen_offset_x + (x * cell_size),
                        screen_offset_y + (y * cell_size),
                        cell_size,
                        cell_size,
                        BACKGROUND_COLOR,
                    );
                }
                Terrain::Invalid => unreachable!("we should never try to draw an invalid cell"),
            };

            if self.show_pheromones {
                if cell.to_home > 0.0 {
                    let brightness = ((cell.to_home / MAX_RGBA_VALUE) * 2.0) - 1.0;
                    let color = PHEROMONE_FORAGING_COLOR.brightness(brightness + 0.2);
                    d.draw_rectangle(
                        screen_offset_x + (x * cell_size + cell_size / 2),
                        screen_offset_y + (y * cell_size + cell_size / 2),
                        cell_size,
                        cell_size,
                        color,
                    );
                }
                if cell.to_food > 0.0 {
                    let brightness = ((cell.to_food / MAX_RGBA_VALUE) * 2.0) - 1.0;
                    let color = PHEROMONE_RETURNING_FOOD_COLOR.brightness(brightness + 0.2);
                    d.draw_rectangle(
                        screen_offset_x + (x * cell_size + cell_size / 2),
                        screen_offset_y + (y * cell_size + cell_size / 2),
                        cell_size,
                        cell_size,
                        color,
                    );
                }
            }

            if self.show_grid {
                if cell.is_border() {
                    continue;
                }
                let thickness = 0.5;
                let line_color = Color::new(80, 80, 80, 255);
                let rect = Rectangle::new(
                    (screen_offset_x + (x * cell_size)) as f32,
                    (screen_offset_y + (y * cell_size)) as f32,
                    cell_size as f32,
                    cell_size as f32,
                );
                d.draw_rectangle_lines_ex(rect, thickness, line_color);
            }
        }

        // FIRST : Draw ants
        if self.show_ants {
            for ant in &self.colony.ants {
                ant.draw(d, self.show_ant_sensors, screen_offset_x, screen_offset_y);
            }
        }
        // SECOND : Draw colony
        self.colony.draw(d, screen_offset_x, screen_offset_y);
    }

    pub fn get_cell(&self, x: u32, y: u32) -> Option<&Cell> {
        self.grid.get_cell(x, y)
    }

    #[allow(dead_code)]
    pub fn get_cell_mut(&mut self, x: u32, y: u32) -> Option<&mut Cell> {
        self.grid.get_cell_mut(x, y)
    }

    #[allow(dead_code)]
    pub fn get_cell_from_position(&self, position: Vector2) -> Option<&Cell> {
        self.grid.get_cell_from_position(position)
    }

    #[allow(dead_code)]
    pub fn get_cell_mut_from_position(&mut self, position: Vector2) -> Option<&mut Cell> {
        self.grid.get_cell_mut_from_position(position)
    }
}

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
