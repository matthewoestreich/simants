use crate::*;
use rand::RngExt as _;

pub struct World {
    pub width: i32,
    pub height: i32,
    pub grid: Grid<CellContents>,
    pub colony: AntColony,
    current_dt: f32,
}

impl World {
    pub fn new(width: i32, height: i32, cell_size: i32, colony: AntColony) -> Self {
        let mut grid = Grid::new(width, height, cell_size);
        let mut rng = rand::rng();

        // Calculate number of cells per width & height pixels
        let w = width / cell_size;
        let h = height / cell_size;

        // For drawing vertical line obstacle in mid of screen
        let mid_w = w / 2;
        let line_len = h / 2;
        let line_start_y = (h - line_len) / 2;
        let line_end_y = line_start_y + line_len;
        let x_range = (mid_w - 1)..=(mid_w + 1);
        let y_range = line_start_y..=line_end_y;

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

                // Draw obstacles around screen border
                if x == 0 || x == w - 1 || y == 0 || y == h - 1 {
                    let kind = Obstacle::Normal;
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

                // Randomly sprinkle food
                let rand_x = rng.random_range(0..=w);
                let rand_y = rng.random_range(0..=h);
                let rand_x_range = (rand_x - 10).max(0)..(rand_x + 10).min(w);
                let rand_y_range = (rand_y - 10).max(0)..(rand_y + 10).min(h);

                if rand_x_range.contains(&x) && rand_y_range.contains(&y) {
                    let food = Food::default();
                    let terrain = Terrain::Food(food);
                    let contents = CellContents::new(terrain);
                    cell.contents = contents;
                    continue;
                }
            }
        }

        Self {
            colony,
            width,
            height,
            grid,
            current_dt: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.current_dt = dt;
        for cell in &mut self.grid.cells {
            cell.contents.searching_strength = (cell.contents.searching_strength - dt).max(0.0);
            cell.contents.to_home_strength = (cell.contents.to_home_strength - dt).max(0.0);
        }
        for ant in &mut self.colony.ants {
            ant.update(dt, &mut self.grid);
        }
    }

    pub fn draw(&mut self, d: &mut RaylibDrawHandle) {
        let cell_size = self.grid.cell_size;

        for cell in &mut self.grid.cells {
            let x = cell.x;
            let y = cell.y;

            match cell.contents.terrain {
                Terrain::Obstacle { kind } => {
                    if !matches!(kind, Obstacle::Border) || SHOW_BORDER {
                        d.draw_rectangle(
                            x * cell_size,
                            y * cell_size,
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

                    d.draw_rectangle(x * cell_size, y * cell_size, cell_size, cell_size, color);
                }
                _ => {
                    // Draw standard empty background
                    d.draw_rectangle(
                        x * cell_size,
                        y * cell_size,
                        cell_size,
                        cell_size,
                        BACKGROUND_COLOR,
                    );
                }
            };

            if SHOW_PHEROMONES {
                // If there is searching pheromone here, render it
                if cell.contents.searching_strength > 0.0 {
                    let mut color = PHEROMONE_FORAGING_COLOR;
                    let alpha = cell.contents.searching_strength / PHEROMONE_MAX_LIFETIME_SECONDS;
                    color.a = (alpha * MAX_RGBA_VALUE as f32) as u8;
                    d.draw_circle(
                        x * cell_size + cell_size / 2,
                        y * cell_size + cell_size / 2,
                        cell_size as f32 / 4.0,
                        color,
                    );
                }
                // If there is a return trail here, render it
                if cell.contents.to_home_strength > 0.0 {
                    let mut color = PHEROMONE_RETURNING_FOOD_COLOR;
                    let alpha = cell.contents.to_home_strength / PHEROMONE_MAX_LIFETIME_SECONDS;
                    color.a = (alpha * MAX_RGBA_VALUE as f32) as u8;
                    d.draw_circle(
                        x * cell_size + cell_size / 2,
                        y * cell_size + cell_size / 2,
                        cell_size as f32 / 3.0,
                        color,
                    );
                }
            }
        }

        // FIRST : Draw ants
        for ant in &self.colony.ants {
            ant.draw(d);
        }
        // SECOND : Draw colony
        self.colony.draw(d);
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
    pub width: i32,
    pub height: i32,
    pub cell_size: i32,
    pub cells: Vec<Cell<T>>,
}

impl<T> Grid<T>
where
    T: Default,
{
    pub fn new(width: i32, height: i32, cell_size: i32) -> Self {
        let width = width / cell_size;
        let height = height / cell_size;
        let size = (width * height) as usize;

        let mut cells = Vec::with_capacity(size);

        for y in 0..height {
            for x in 0..width {
                // Pass coordinates straight into your updated cell constructor
                let cell = Cell::new_with_coords(T::default(), x, y);
                cells.push(cell);
            }
        }

        Self {
            width,
            height,
            cell_size,
            cells,
        }
    }

    pub fn get(&self, x: i32, y: i32) -> Option<&Cell<T>> {
        if !self.is_in_bounds(x, y) {
            return None;
        }
        Some(&self.cells[self.index(x, y)])
    }

    pub fn get_mut(&mut self, x: i32, y: i32) -> Option<&mut Cell<T>> {
        if !self.is_in_bounds(x, y) {
            return None;
        }
        let i = self.index(x, y);
        Some(&mut self.cells[i])
    }

    pub fn get_from_position(&self, position: Vector2) -> Option<&Cell<T>> {
        let (x, y) = self.position_to_grid_coords(position);
        if !self.is_in_bounds(x, y) {
            return None;
        }
        Some(&self.cells[self.index(x, y)])
    }

    pub fn get_from_position_mut(&mut self, position: Vector2) -> Option<&mut Cell<T>> {
        let (x, y) = self.position_to_grid_coords(position);
        if !self.is_in_bounds(x, y) {
            return None;
        }
        let i = self.index(x, y);
        Some(&mut self.cells[i])
    }

    pub fn coords_from_index(&self, index: usize) -> Option<(i32, i32)> {
        if index >= self.cells.len() {
            return None;
        }
        let x = (index as i32) % self.width;
        let y = (index as i32) / self.width;
        Some((x, y))
    }

    // Takes grid coords and turns them into pixel position
    pub fn grid_coords_to_position(&self, x: i32, y: i32, get_center: bool) -> Vector2 {
        let cell_size = self.cell_size;
        let mut pixel_x = (x * cell_size) as f32;
        let mut pixel_y = (y * cell_size) as f32;

        if get_center {
            let half_cell = cell_size as f32 / 2.0;
            pixel_x += half_cell;
            pixel_y += half_cell;
        }

        Vector2::new(pixel_x, pixel_y)
    }

    // Takes pixel position and turns them into grid coords
    // return is `(x: i32, y: i32)`
    pub fn position_to_grid_coords(&self, position: Vector2) -> (i32, i32) {
        let x = (position.x / self.cell_size as f32).floor() as i32;
        let y = (position.y / self.cell_size as f32).floor() as i32;
        (x, y)
    }

    fn index(&self, x: i32, y: i32) -> usize {
        (y * self.width + x) as usize
    }

    fn is_in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.width && y < self.height
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
    pub x: i32,
    pub y: i32,
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

    pub fn new_with_coords(contents: T, x: i32, y: i32) -> Self {
        let mut this = Self::new(contents);
        this.x = x;
        this.y = y;
        this
    }
}
