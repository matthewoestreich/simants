use crate::*;
use rand::RngExt as _;

pub struct World {
    pub screen_width: i32,
    pub screen_height: i32,
    pub grid: Grid<CellContents>,
    pub colony: AntColony,
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
        let mut grid = Grid::new(
            grid_width,
            grid_height,
            cell_size,
            screen_width,
            screen_height,
        );

        // Calculate number of cells per width & height pixels
        let w = grid_width / cell_size;
        let h = grid_height / cell_size;

        // For drawing vertical line obstacle in mid of screen
        let mid_w = w / 2;
        let line_len = h / 2;
        let line_start_y = (h - line_len) / 2;
        let line_end_y = line_start_y + line_len;
        let x_range = (mid_w - 1)..=(mid_w + 1);
        let y_range = line_start_y..=line_end_y;

        // For drawing food clump
        let food_center_x = (w * 3) / 4;
        let food_center_y = h / 2;
        let food_radius = 6; // Radius measured in number of grid cells

        for x in 0..w {
            for y in 0..h {
                let Some(cell) = grid.get_mut(x, y) else {
                    continue;
                };

                // Draw obstacles around screen border
                if x == 0 || x == w - 1 || y == 0 || y == h - 1 {
                    let kind = Obstacle::Border;
                    let contents = CellContents::new(Terrain::Obstacle { kind });
                    cell.contents = contents;
                    continue;
                }

                // Draws an obstacle, in the form of a line in the middle of the screen,
                // that is half the height and centerted horizontally and vertically
                if x_range.contains(&x) && y_range.contains(&y) {
                    let kind = Obstacle::Normal;
                    let contents = CellContents::new(Terrain::Obstacle { kind });
                    cell.contents = contents;
                    continue;
                }

                // Draw food clump
                let food_dx = x as i32 - food_center_x as i32;
                let food_dy = y as i32 - food_center_y as i32;
                if food_dx * food_dx + food_dy * food_dy <= food_radius * food_radius {
                    // Initialize food with actual consumable values
                    let food = Food::default();
                    cell.contents = CellContents::new(Terrain::Food(food));
                    continue;
                }
            }
        }

        Self {
            colony,
            screen_width,
            screen_height,
            grid,
            current_dt: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.current_dt = dt;
        for cell in &mut self.grid.cells {
            cell.contents.to_food_strength = (cell.contents.to_food_strength - dt).max(0.0);
            cell.contents.to_home_strength = (cell.contents.to_home_strength - dt).max(0.0);
        }
        for ant in &mut self.colony.ants {
            ant.update(dt, &mut self.grid);
        }
    }

    pub fn draw(&mut self, d: &mut RaylibDrawHandle) {
        let cell_size = self.grid.cell_size as i32;

        let grid_pixel_width = (self.grid.width * self.grid.cell_size) as i32;
        let grid_pixel_height = (self.grid.height * self.grid.cell_size) as i32;

        let screen_offset_x = (self.screen_width - grid_pixel_width) / 2;
        let screen_offset_y = (self.screen_height - grid_pixel_height) / 2;

        for cell in &mut self.grid.cells {
            let x = cell.x as i32;
            let y = cell.y as i32;

            match cell.contents.terrain {
                Terrain::Obstacle { kind } => {
                    if !matches!(kind, Obstacle::Border) || SHOW_BORDER {
                        d.draw_rectangle(
                            screen_offset_x + (x * cell_size),
                            screen_offset_y + (y * cell_size),
                            cell_size,
                            cell_size,
                            OBSTACLE_COLOR,
                        );
                    }
                }
                Terrain::Food(f) => {
                    let color = if f.is_harvested {
                        cell.contents.terrain = Terrain::Empty;
                        BACKGROUND_COLOR
                    } else {
                        Color::GOLD
                    };

                    d.draw_rectangle(
                        screen_offset_x + (x * cell_size),
                        screen_offset_y + (y * cell_size),
                        cell_size,
                        cell_size,
                        color,
                    );
                }
                _ => {
                    // Draw standard empty background
                    d.draw_rectangle(
                        screen_offset_x + (x * cell_size),
                        screen_offset_y + (y * cell_size),
                        cell_size,
                        cell_size,
                        BACKGROUND_COLOR,
                    );
                }
            };

            if SHOW_PHEROMONES {
                // If there is searching pheromone here, render it
                if cell.contents.to_food_strength > 0.0 {
                    let mut color = PHEROMONE_RETURNING_FOOD_COLOR;
                    let alpha = cell.contents.to_food_strength / PHEROMONE_MAX_LIFETIME_SECONDS;
                    color.a = (alpha * MAX_RGBA_VALUE as f32) as u8;
                    d.draw_circle(
                        screen_offset_x + (x * cell_size + cell_size / 2),
                        screen_offset_y + (y * cell_size + cell_size / 2),
                        cell_size as f32 / 4.0,
                        color,
                    );
                }
                // If there is a return trail here, render it
                if cell.contents.to_home_strength > 0.0 {
                    let mut color = PHEROMONE_FORAGING_COLOR;
                    let alpha = cell.contents.to_home_strength / PHEROMONE_MAX_LIFETIME_SECONDS;
                    color.a = (alpha * MAX_RGBA_VALUE as f32) as u8;
                    d.draw_circle(
                        screen_offset_x + (x * cell_size + cell_size / 2),
                        screen_offset_y + (y * cell_size + cell_size / 2),
                        cell_size as f32 / 4.0,
                        color,
                    );
                }
            }

            if SHOW_GRID_LINES {
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
}

/* ------------------------------------------------------------------------------ */
/* ---------------- Grid -------------------------------------------------------- */
/* ------------------------------------------------------------------------------ */

#[derive(Debug)]
pub struct Grid<T>
where
    T: Default,
{
    pub width: u32,
    pub height: u32,
    pub cell_size: u32,
    pub cells: Vec<Cell<T>>,
    screen_width: i32,
    screen_height: i32,
}

impl<T> Grid<T>
where
    T: Default,
{
    pub fn new(
        width: u32,
        height: u32,
        cell_size: u32,
        screen_width: i32,
        screen_height: i32,
    ) -> Self {
        let width = width / cell_size;
        let height = height / cell_size;
        let size = (width * height) as usize;

        let mut cells = Vec::with_capacity(size);

        for y in 0..height {
            for x in 0..width {
                let cell = Cell::new_with_coords(T::default(), x, y);
                cells.push(cell);
            }
        }

        Self {
            width,
            height,
            cell_size,
            screen_width,
            screen_height,
            cells,
        }
    }

    pub fn get(&self, x: u32, y: u32) -> Option<&Cell<T>> {
        if !self.is_in_bounds(x, y) {
            return None;
        }
        Some(&self.cells[self.index(x, y)])
    }

    pub fn get_mut(&mut self, x: u32, y: u32) -> Option<&mut Cell<T>> {
        if !self.is_in_bounds(x, y) {
            return None;
        }
        let i = self.index(x, y);
        Some(&mut self.cells[i])
    }

    pub fn get_from_position(&self, position: Vector2) -> Option<&Cell<T>> {
        let (x, y) = self.position_to_grid_coords(position);
        self.get(x, y)
    }

    pub fn get_mut_from_position(&mut self, position: Vector2) -> Option<&mut Cell<T>> {
        let (x, y) = self.position_to_grid_coords(position);
        self.get_mut(x, y)
    }

    // Takes pixel position and turns them into grid coords
    // return is `(x: i32, y: i32)`
    pub fn position_to_grid_coords(&self, position: Vector2) -> (u32, u32) {
        let x = (position.x / self.cell_size as f32).floor() as u32;
        let y = (position.y / self.cell_size as f32).floor() as u32;
        (x, y)
    }

    pub fn screen_to_grid_coords(&self, screen_pos: Vector2) -> Option<(u32, u32)> {
        let cell_size = self.cell_size;

        let grid_pixel_width = self.width * cell_size;
        let grid_pixel_height = self.height * cell_size;

        let offset_x = (self.screen_width as u32 - grid_pixel_width) / 2;
        let offset_y = (self.screen_height as u32 - grid_pixel_height) / 2;

        let local_x = screen_pos.x - offset_x as f32;
        let local_y = screen_pos.y - offset_y as f32;

        let grid_x = (local_x / cell_size as f32).floor() as u32;
        let grid_y = (local_y / cell_size as f32).floor() as u32;

        if grid_x < self.width && grid_y < self.height {
            Some((grid_x, grid_y))
        } else {
            None
        }
    }

    fn index(&self, x: u32, y: u32) -> usize {
        (y * self.width + x) as usize
    }

    fn is_in_bounds(&self, x: u32, y: u32) -> bool {
        x < self.width && y < self.height
    }
}

/* ------------------------------------------------------------------------------ */
/* ---------------- Cell -------------------------------------------------------- */
/* ------------------------------------------------------------------------------ */

#[derive(Debug, Default)]
pub struct Cell<T>
where
    T: Default,
{
    pub contents: T,
    pub x: u32,
    pub y: u32,
}

impl<T> Cell<T>
where
    T: Default,
{
    pub fn new(contents: T) -> Self {
        Self {
            contents,
            ..Self::default()
        }
    }

    pub fn new_with_coords(contents: T, x: u32, y: u32) -> Self {
        let mut this = Self::new(contents);
        this.x = x;
        this.y = y;
        this
    }
}
