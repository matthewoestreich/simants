use crate::{
    ant::{AntColony, AntState},
    settings::{FOOD_CELL_MAX_AMOUNT, FOOD_RADIUS},
    world::World,
};
use raylib::ffi::Vector2;

/* ------------------------------------------------------------------------------ */
/* ---------------- Grid -------------------------------------------------------- */
/* ------------------------------------------------------------------------------ */

pub struct Grid {
    pub cols: u32,
    pub rows: u32,
    cells: Vec<Cell>,
}

impl<'a> IntoIterator for &'a mut Grid {
    type Item = &'a mut Cell;
    type IntoIter = std::slice::IterMut<'a, Cell>;

    fn into_iter(self) -> Self::IntoIter {
        self.cells.iter_mut()
    }
}

impl Grid {
    pub fn new(cols: u32, rows: u32) -> Self {
        let size = (cols * rows) as usize;
        let mut cells = Vec::with_capacity(size);

        for y in 0..rows {
            for x in 0..cols {
                cells.push(Cell::new_with_coords(x, y));
            }
        }

        Self { cols, rows, cells }
    }

    pub fn len(&self) -> usize {
        self.cells.len()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Cell> {
        self.cells.iter_mut()
    }

    pub fn initialize(&mut self, colony: &AntColony) {
        let rows = self.rows;
        let cols = self.cols;

        // For drawing vertical line obstacle in mid of screen
        let line_len = rows / 2;
        let line_start_y = (rows - line_len) / 2;
        let line_end_y = line_start_y + line_len;
        let mid_w = cols / 2;
        let x_range = (mid_w - 1)..=(mid_w + 1);
        let y_range = line_start_y..=line_end_y;

        // For drawing food clump
        let food_center_x = 225; //(cols * 3) / 4; // 900 / 4
        let food_center_y = 100; //rows / 2; // 200 / 2 = 100
        let food_radius = FOOD_RADIUS;

        for cell in self.cells.iter_mut() {
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
            let food_dx = x as i32 - food_center_x;
            let food_dy = y as i32 - food_center_y;
            if food_dx * food_dx + food_dy * food_dy <= food_radius as i32 * food_radius as i32 {
                cell.terrain = Terrain::Food;
                cell.food = FOOD_CELL_MAX_AMOUNT;
                continue;
            }

            // Mark the underlying cells of the colony as such
            //let cell_center = Vector2::new(
            //    (x as f32 * cell_size_f32) + (cell_size_f32 / 2.0),
            //    (y as f32 * cell_size_f32) + (cell_size_f32 / 2.0),
            //);
            let cell_center = Vector2::new(x as f32 + 0.5, y as f32 + 0.5);
            // If distance from cell center to colony center is less than or
            // equal to the colony area, it means we are in the colony.
            if cell_center.distance_sqr(colony.position) <= colony.area {
                cell.terrain = Terrain::Colony;
                continue;
            }
        }
    }

    pub fn world_to_cell(&self, position: Vector2) -> (u32, u32) {
        let x = (position.x.floor() as i32).clamp(0, self.cols as i32 - 1);
        let y = (position.y.floor() as i32).clamp(0, self.rows as i32 - 1);

        (x as u32, y as u32)
    }

    pub fn get_cell(&self, x: u32, y: u32) -> Option<&Cell> {
        if !self.is_within_grid_bounds(x, y) {
            return None;
        }
        Some(&self.cells[self.index(x, y)])
    }

    pub fn cell_center(&self, x: u32, y: u32) -> Vector2 {
        Vector2::new(x as f32 + 0.5, y as f32 + 0.5)
    }

    pub fn is_within_grid_bounds(&self, x: u32, y: u32) -> bool {
        x < self.cols && y < self.rows
    }

    fn index(&self, x: u32, y: u32) -> usize {
        (y * self.cols + x) as usize
    }

    /*
    pub fn position_is_obstruction(&self, position: Vector2) -> bool {
        if let Some(c) = self.get_cell_from_position(position) {
            return c.is_obstruction();
        }
        true
    }
    */

    /*
    pub fn position_is_terrain(&self, position: Vector2, terrain: Terrain) -> bool {
        if let Some(c) = self.get_cell_from_position(position) {
            return c.terrain == terrain;
        }
        // If the user wants the check for Invalid and there is no cell found
        // we can return true since it is technically invalid
        terrain == Terrain::Invalid
    }
    */

    pub fn sample_position(&self, position: Vector2) -> CellSample {
        let (x, y) = self.world_to_cell(position);

        if let Some(c) = self.get_cell(x, y) {
            return CellSample {
                terrain: c.terrain,
                to_food_strength: c.to_food,
                to_home_strength: c.to_home,
                food_amount: c.food,
                ..CellSample::default()
            };
        }

        CellSample {
            terrain: Terrain::Invalid,
            ..CellSample::default()
        }
    }

    /// Samples a position and only fills out pheromone strength for the appropriate
    /// pheromone based upon ant state
    pub fn sample_position_with_pheromone_bias(
        &self,
        position: Vector2,
        ant_state: AntState,
    ) -> CellSample {
        let mut this = self.sample_position(position);
        if this.terrain.is_invalid() {
            return this;
        }

        match ant_state {
            AntState::Foraging => {
                this.target_pheromone = this.to_food_strength;
                if this.terrain.is_food() {
                    this.target_pheromone += 1000.0;
                }
            }
            AntState::ReturningFood => {
                this.target_pheromone = this.to_home_strength;
                if this.terrain.is_colony() {
                    this.target_pheromone += 1000.0;
                }
            }
        };

        this
    }

    /*
    pub fn get_cell_mut(&mut self, x: u32, y: u32) -> Option<&mut Cell> {
        if !self.is_within_grid_bounds(x, y) {
            return None;
        }
        let i = self.index(x, y);
        Some(&mut self.cells[i])
    }
    */

    /*
    pub fn get_cell_from_position(&self, position: Vector2) -> Option<&Cell> {
        let (x, y) = self.position_to_grid_coords(position);
        self.get_cell(x, y)
    }
    */

    /*
    pub fn get_cell_mut_from_position(&mut self, position: Vector2) -> Option<&mut Cell> {
        let (x, y) = self.position_to_grid_coords(position);
        self.get_cell_mut(x, y)
    }
    */

    /*
    pub fn position_to_grid_coords(&self, position: Vector2) -> (u32, u32) {
        let x = (position.x / self.cell_size as f32).floor() as u32;
        let y = (position.y / self.cell_size as f32).floor() as u32;
        (x, y)
    }
    */

    /*
    pub fn grid_coords_to_screen(&self, x: u32, y: u32) -> Option<Vector2> {
        if !self.is_within_grid_bounds(x, y) {
            return None;
        }

        let cell_size = self.cell_size as f32;
        let half_cell = cell_size / 2.0;

        Some(Vector2::new(
            x as f32 * cell_size + half_cell,
            y as f32 * cell_size + half_cell,
        ))
    }
    */

    /*
    #[allow(dead_code)]
    fn get_grid_index_from_position(&self, position: Vector2) -> Option<usize> {
        let (x, y) = self.position_to_grid_coords(position);
        if !self.is_within_grid_bounds(x, y) {
            return None;
        }
        Some(self.get_grid_index_from_coords(x, y))
    }
    */
}

/* ------------------------------------------------------------------------------ */
/* ---------------- CellSample -------------------------------------------------- */
/* ------------------------------------------------------------------------------ */

#[derive(Debug, Default, Clone, Copy)]
pub struct CellSample {
    pub terrain: Terrain,
    pub to_food_strength: f32,
    pub to_home_strength: f32,
    pub target_pheromone: f32,
    pub food_amount: f32,
}

/* ------------------------------------------------------------------------------ */
/* ---------------- Cell -------------------------------------------------------- */
/* ------------------------------------------------------------------------------ */

#[derive(Default, Debug, Clone, Copy)]
pub struct Cell {
    pub terrain: Terrain,
    pub to_food: f32,
    pub to_home: f32,
    pub food: f32,
    pub x: u32,
    pub y: u32,
}

impl Cell {
    pub fn new() -> Self {
        Self { ..Self::default() }
    }

    pub fn new_with_coords(x: u32, y: u32) -> Self {
        Self {
            x,
            y,
            ..Self::default()
        }
    }

    /// Exponentially decreases the trail strength over time.
    /// `decay_rate` controls the speed of the fade (e.g., 0.1 means 10% loss per second).
    pub fn evaporate(&mut self, delta_time: f32, decay_rate: f32) {
        self.to_food = World::calculate_decayed_amount(self.to_food, delta_time, decay_rate);
        self.to_home = World::calculate_decayed_amount(self.to_home, delta_time, decay_rate);
    }

    pub fn is_colony(&self) -> bool {
        self.terrain.is_colony()
    }

    pub fn is_food(&self) -> bool {
        self.terrain.is_food()
    }

    pub fn is_obstruction(&self) -> bool {
        self.terrain.is_obstruction()
    }

    pub fn is_obstacle(&self) -> bool {
        matches!(self.terrain, Terrain::Obstacle)
    }

    pub fn is_border(&self) -> bool {
        matches!(self.terrain, Terrain::Border)
    }

    pub fn allows_pheromones(&self) -> bool {
        !matches!(
            self.terrain,
            Terrain::Obstacle | Terrain::Food | Terrain::Border | Terrain::Colony
        )
    }
}

/* ------------------------------------------------------------------------------ */
/* ---------------- Terrain ----------------------------------------------------- */
/* ------------------------------------------------------------------------------ */

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum Terrain {
    #[default]
    Empty,
    Food,
    Colony,
    Obstacle,
    Border,
    Invalid,
}

impl Terrain {
    // We are obstructed by obstacles and borders
    pub fn is_obstruction(&self) -> bool {
        matches!(self, Terrain::Obstacle | Terrain::Border | Terrain::Invalid)
    }

    pub fn is_food(&self) -> bool {
        matches!(self, Terrain::Food)
    }

    pub fn is_colony(&self) -> bool {
        matches!(self, Terrain::Colony)
    }

    pub fn is_invalid(&self) -> bool {
        matches!(self, Terrain::Invalid)
    }
}
