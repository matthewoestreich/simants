use crate::*;

pub struct World {
    pub screen_width: i32,
    pub screen_height: i32,
    pub grid_width_pixels: i32,
    pub grid_height_pixels: i32,
    pub screen_offset_x: i32,
    pub screen_offset_y: i32,
    pub colony: AntColony,
    pub grid: Grid,

    pub show_grid: bool,
    pub show_pheromones: bool,
    pub show_border: bool,
}

pub fn is_same_position(position: Vector2, other_position: Vector2, cell_size: u32) -> bool {
    let half_size = cell_size as f32 / 2.0;
    let dx = (other_position.x - position.x).abs();
    let dy = (other_position.y - position.y).abs();
    dx <= half_size && dy <= half_size
}

impl World {
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
                cell.terrain = Terrain::Obstacle;
                continue;
            }

            // Marks a cell as food.
            let food_dx = x as i32 - food_center_x as i32;
            let food_dy = y as i32 - food_center_y as i32;
            if food_dx * food_dx + food_dy * food_dy <= food_radius as i32 * food_radius as i32 {
                cell.terrain = Terrain::Food;
                cell.food = Some(100.0);
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

    pub fn update(&mut self, delta_time: f32) {
        for cell in self.grid.iter_mut() {
            cell.to_food = (cell.to_food - delta_time).max(0.0);
            cell.to_home = (cell.to_home - delta_time).max(0.0);
        }

        let colony_center = self.colony.position;

        for ant in &mut self.colony.ants {
            if ant.is_out_of_energy() {
                continue;
            }

            ant.handle_pause(delta_time);

            if ant.is_paused() {
                continue;
            }

            let current_cell = ant.sense_environment(&mut self.grid);

            if ant.is_foraging() && current_cell.is_food() {
                ant.harvest(current_cell);
                ant.set_energy(ANT_MAX_ENERGY);
                ant.turn_around();
                continue;
            }

            if ant.is_returning_food() && current_cell.is_colony() {
                if !is_same_position(colony_center, ant.position, self.grid.cell_size) {
                    ant.steer_towards_position(colony_center, delta_time);
                    continue;
                }
                ant.deliver_food();
                ant.set_energy(ANT_MAX_ENERGY);
                ant.turn_around();
                continue;
            }

            if current_cell.allows_pheromones() {
                let drop_strength = PHEROMONE_LIFETIME_SECONDS * ant.get_energy();
                ant.place_pheromone(current_cell, drop_strength);
                let energy_loss_amount = delta_time * ANT_PHEROMONE_STRENGTH_DECAY;
                ant.lose_energy(energy_loss_amount);
            }

            if let Some(next_position) = ant.calculate_next_position(current_cell, delta_time) {
                if self.grid.position_is_obstruction(next_position) {
                    ant.turn_around();
                    continue;
                }
                ant.position = next_position;
            }
        }
    }

    pub fn is_colony_position(&self, position: Vector2) -> bool {
        let colony_radius = self.colony.radius * self.colony.radius;
        position.distance_sqr(self.colony.position) < colony_radius
    }

    pub fn is_position_obstacle(&self, position: Vector2) -> bool {
        self.grid
            .get_cell_from_position(position)
            .map(|cell| cell.is_obstacle())
            .unwrap_or(true)
    }

    pub fn screen_to_grid_coords(&self, position: Vector2) -> Option<(u32, u32)> {
        let offset_x = self.screen_offset_x; // (self.screen_width as u32 - self.grid_width_pixels as u32) / 2;
        let offset_y = self.screen_offset_y; // (self.screen_height as u32 - self.grid_height_pixels as u32) / 2;

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

        let grid_pixel_width = self.grid_width_pixels; // (self.grid_width_pixels * self.grid.cell_size as i32);
        let grid_pixel_height = self.grid_height_pixels; //(self.grid_height_pixels * self.grid.cell_size as i32);
        let screen_offset_x = (self.screen_width - grid_pixel_width) / 2;
        let screen_offset_y = (self.screen_height - grid_pixel_height) / 2;

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
                        Color::GOLD,
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
                // If there is searching pheromone here, render it
                if cell.to_home > 0.0 {
                    let mut color = PHEROMONE_FORAGING_COLOR;
                    let intensity = cell.to_home / PHEROMONE_LIFETIME_SECONDS;
                    let alpha = f32::sqrt(intensity) * MAX_RGBA_VALUE;
                    //let alpha = cell.to_home / PHEROMONE_LIFETIME_SECONDS;
                    color.a = (alpha * MAX_RGBA_VALUE) as u8;
                    d.draw_circle(
                        screen_offset_x + (x * cell_size + cell_size / 2),
                        screen_offset_y + (y * cell_size + cell_size / 2),
                        cell_size as f32 / 4.0,
                        color,
                    );
                }
                // If there is a return trail here, render it
                if cell.to_food > 0.0 {
                    let mut color = PHEROMONE_RETURNING_FOOD_COLOR;
                    let intensity = cell.to_food / PHEROMONE_LIFETIME_SECONDS;
                    let alpha = f32::sqrt(intensity) * MAX_RGBA_VALUE;
                    //let alpha = cell.to_food / PHEROMONE_LIFETIME_SECONDS;
                    color.a = (alpha * MAX_RGBA_VALUE as f32) as u8;
                    d.draw_circle(
                        screen_offset_x + (x * cell_size + cell_size / 2),
                        screen_offset_y + (y * cell_size + cell_size / 2),
                        cell_size as f32 / 4.0,
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
        for ant in &self.colony.ants {
            ant.draw(d, screen_offset_x, screen_offset_y);
        }
        // SECOND : Draw colony
        self.colony.draw(d, screen_offset_x, screen_offset_y);
    }

    pub fn get_cell(&self, x: u32, y: u32) -> Option<&Cell> {
        self.grid.get_cell(x, y)
    }

    pub fn get_cell_mut(&mut self, x: u32, y: u32) -> Option<&mut Cell> {
        self.grid.get_cell_mut(x, y)
    }

    pub fn get_cell_from_position(&self, position: Vector2) -> Option<&Cell> {
        self.grid.get_cell_from_position(position)
    }

    pub fn get_cell_mut_from_position(&mut self, position: Vector2) -> Option<&mut Cell> {
        self.grid.get_cell_mut_from_position(position)
    }
}
