use crate::*;
use rand::RngExt as _;

pub struct World {
    pub width: i32,
    pub height: i32,
    pub screen_offset_x: i32,
    pub screen_offset_y: i32,
    pub colony: AntColony,
    grid: Grid,
    current_dt: f32,
}

impl World {
    pub fn new(
        screen_width: i32,
        screen_height: i32,
        grid_width: u32,
        grid_height: u32,
        cell_size: u32,
        colony: AntColony,
    ) -> Self {
        // Calculate screen position to grid cell offset (needed to map a screen position to a cell)
        let grid_width_in_pixels = (grid_width * cell_size) as i32;
        let grid_height_in_pixels = (grid_height * cell_size) as i32;
        let screen_offset_x = (screen_width - grid_width_in_pixels) / 2;
        let screen_offset_y = (screen_height - grid_height_in_pixels) / 2;

        // Calculate number of cells per width & height pixels
        let w = grid_width / cell_size;
        let h = grid_height / cell_size;

        // For drawing vertical line obstacle in mid of screen
        let line_len = h / 2;
        let line_start_y = (h - line_len) / 2;
        let line_end_y = line_start_y + line_len;
        let mid_w = w / 2;
        let x_range = (mid_w - 1)..=(mid_w + 1);
        let y_range = line_start_y..=line_end_y;

        // For drawing food clump
        let food_center_x = (w * 3) / 4;
        let food_center_y = h / 2;
        let food_radius = 6; // Radius measured in number of grid cells

        let mut grid = Grid::new(grid_width, grid_height, cell_size);

        for cell in &mut grid {
            let x = cell.x;
            let y = cell.y;

            // Create border cells
            if x == 0 || x == w - 1 || y == 0 || y == h - 1 {
                cell.terrain = Terrain::Obstacle {
                    kind: Obstacle::Border,
                };
                continue;
            }
            // Draws an obstacle, in the form of a line in the middle of the screen,
            // that is half the height and centerted horizontally and vertically
            if x_range.contains(&x) && y_range.contains(&y) {
                cell.terrain = Terrain::Obstacle {
                    kind: Obstacle::Normal,
                };
                continue;
            }
            // Draw food clump
            let food_dx = x as i32 - food_center_x as i32;
            let food_dy = y as i32 - food_center_y as i32;
            if food_dx * food_dx + food_dy * food_dy <= food_radius * food_radius {
                cell.terrain = Terrain::Food(Food::default());
                continue;
            }
        }

        Self {
            width: screen_width,
            height: screen_height,
            grid,
            colony,
            screen_offset_x,
            screen_offset_y,
            current_dt: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.current_dt = dt;

        for cell in self.grid.iter_mut() {
            // Pheromone time decay/evaporation
            cell.to_home.weaken(dt);
            cell.to_food.weaken(dt);
        }

        for ant in &mut self.colony.ants {
            ant.handle_pause(dt);

            if !ant.has_energy() || ant.is_paused() {
                continue;
            }

            ant.update_sensor_positions();

            let sensors = ant.get_sensors();
            // Get readings from our sensors posittions.
            let left_reading = self.grid.get_cell_mut_from_position(sensors.left);
            let center_reading = self.grid.get_cell_mut_from_position(sensors.center);
            let right_reading = self.grid.get_cell_mut_from_position(sensors.right);

            // TODO : delete this after refactor
            //ant.update(dt, &mut self.cells);
        }
    }

    pub fn screen_to_grid_coords(&self, position: Vector2) -> Option<(u32, u32)> {
        let local_x = position.x - self.screen_offset_x as f32;
        let local_y = position.y - self.screen_offset_y as f32;
        let grid_x = (local_x / self.grid.cell_size as f32).floor() as i32;
        let grid_y = (local_y / self.grid.cell_size as f32).floor() as i32;

        if grid_x > 0 && grid_y > 0 && grid_x < self.width && grid_y < self.height {
            Some((grid_x as u32, grid_y as u32))
        } else {
            None
        }
    }

    pub fn draw(&mut self, d: &mut RaylibDrawHandle) {
        let cell_size = self.grid.cell_size as i32;

        for cell in &mut self.grid {
            let x = cell.x as i32;
            let y = cell.y as i32;

            match cell.terrain {
                Terrain::Obstacle { kind } => {
                    if !matches!(kind, Obstacle::Border) || SHOW_BORDER {
                        d.draw_rectangle(
                            self.screen_offset_x + (x * cell_size),
                            self.screen_offset_y + (y * cell_size),
                            cell_size,
                            cell_size,
                            OBSTACLE_COLOR,
                        );
                    }
                }
                Terrain::Food(f) => {
                    let color = if f.is_harvested {
                        cell.terrain = Terrain::Empty;
                        BACKGROUND_COLOR
                    } else {
                        Color::GOLD
                    };

                    d.draw_rectangle(
                        self.screen_offset_x + (x * cell_size),
                        self.screen_offset_y + (y * cell_size),
                        cell_size,
                        cell_size,
                        color,
                    );
                }
                _ => {
                    // Draw standard empty background
                    d.draw_rectangle(
                        self.screen_offset_x + (x * cell_size),
                        self.screen_offset_y + (y * cell_size),
                        cell_size,
                        cell_size,
                        BACKGROUND_COLOR,
                    );
                }
            };

            if SHOW_PHEROMONES {
                // If there is searching pheromone here, render it
                if cell.to_home.strength > 0.0 {
                    let mut color = PHEROMONE_FORAGING_COLOR;
                    let alpha = cell.to_home.strength / PHEROMONE_MAX_LIFETIME_SECONDS;
                    color.a = (alpha * MAX_RGBA_VALUE as f32) as u8;
                    d.draw_circle(
                        self.screen_offset_x + (x * cell_size + cell_size / 2),
                        self.screen_offset_y + (y * cell_size + cell_size / 2),
                        cell_size as f32 / 4.0,
                        color,
                    );
                }
                // If there is a return trail here, render it
                if cell.to_food.strength > 0.0 {
                    let mut color = PHEROMONE_RETURNING_FOOD_COLOR;
                    let alpha = cell.to_food.strength / PHEROMONE_MAX_LIFETIME_SECONDS;
                    color.a = (alpha * MAX_RGBA_VALUE as f32) as u8;
                    d.draw_circle(
                        self.screen_offset_x + (x * cell_size + cell_size / 2),
                        self.screen_offset_y + (y * cell_size + cell_size / 2),
                        cell_size as f32 / 4.0,
                        color,
                    );
                }
            }

            if SHOW_GRID_LINES {
                let thickness = 0.5;
                let line_color = Color::new(80, 80, 80, 255);

                let rect = Rectangle::new(
                    (self.screen_offset_x + (x * cell_size)) as f32,
                    (self.screen_offset_y + (y * cell_size)) as f32,
                    cell_size as f32,
                    cell_size as f32,
                );

                d.draw_rectangle_lines_ex(rect, thickness, line_color);
            }
        }

        // FIRST : Draw ants
        for ant in &self.colony.ants {
            ant.draw(d, self.screen_offset_x, self.screen_offset_y);
        }
        // SECOND : Draw colony
        self.colony
            .draw(d, self.screen_offset_x, self.screen_offset_y);
    }

    /// Returns a list of cell coordinates that lie inside the ant's forward vision cone.
    pub fn get_coords_of_cells_in_cone(
        &mut self,
        ant: &Ant,
        max_dist: f32,
        view_angle: f32,
    ) -> Vec<(i32, i32)> {
        let mut visible_cells = Vec::new();

        let cell_size_f32 = self.grid.cell_size as f32;
        let cell_size_i32 = self.grid.cell_size as i32;
        let grid_width_i32 = self.grid.width as i32;
        let grid_height_i32 = self.grid.height as i32;

        // 1. Calculate a tight bounding box around the ant's vision area
        let min_x = ((ant.position.x - max_dist) / cell_size_f32).floor() as i32;
        let max_x = ((ant.position.x + max_dist) / cell_size_f32).ceil() as i32;
        let min_y = ((ant.position.y - max_dist) / cell_size_f32).floor() as i32;
        let max_y = ((ant.position.y + max_dist) / cell_size_f32).ceil() as i32;

        // Clamp bounding box dimensions to absolute grid array dimensions
        let start_x = min_x.clamp(0, grid_width_i32 - 1);
        let end_x = max_x.clamp(0, grid_width_i32 - 1);
        let start_y = min_y.clamp(0, grid_height_i32 - 1);
        let end_y = max_y.clamp(0, grid_height_i32 - 1);

        // 2. Loop through only the cells inside this tiny local square box
        for y in start_y..=end_y {
            for x in start_x..=end_x {
                // Calculate world-space center point of this specific tile
                let cell_world_pos = Vector2::new(
                    (x * cell_size_i32) as f32 + (cell_size_f32 / 2.0),
                    (y * cell_size_i32) as f32 + (cell_size_f32 / 2.0),
                );

                // Check A: Distance Check
                let to_cell = cell_world_pos - ant.position;
                let dist_sqr = to_cell.length_sqr();
                if dist_sqr > max_dist * max_dist {
                    continue; // Too far away!
                }

                // Check B: Angular Vision Check
                // Calculate the angular difference between the ant's face and this tile
                let forward = ant.velocity.normalize();
                let current_heading_angle = forward.y.atan2(forward.x);
                let cell_direction_angle = to_cell.y.atan2(to_cell.x);
                let mut angle_diff = (cell_direction_angle - current_heading_angle).abs();

                // Keep angle difference normalized between 0 and PI
                if angle_diff > std::f32::consts::PI {
                    angle_diff = (2.0 * std::f32::consts::PI) - angle_diff;
                }

                // If the tile falls within our left/right view window, it's inside the cone!
                if angle_diff <= view_angle {
                    // Fetch cell index using your existing method
                    visible_cells.push((x, y));
                }
            }
        }

        visible_cells
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
