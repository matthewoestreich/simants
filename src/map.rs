use crate::{
    ant::{AntColony, AntState},
    settings::{FOOD_CELL_MAX_AMOUNT, FOOD_RADIUS},
    world::World,
};
use raylib::ffi::Vector2;
use rayon::iter::IntoParallelRefMutIterator;

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

    pub fn par_iter_mut(&mut self) -> rayon::slice::IterMut<'_, Cell> {
        self.cells.par_iter_mut()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Cell> {
        self.cells.iter()
    }

    pub fn initialize(&mut self, colony: &AntColony) {
        let rows = self.rows;
        let cols = self.cols;

        // For drawing vertical line obstacle in mid of screen
        let line_len = rows / 2;
        let line_start_y = (rows - line_len) / 2;
        let line_end_y = line_start_y + line_len;
        let mid_w = cols / 2;
        #[allow(unused_variables)]
        let x_range = (mid_w - 1)..=(mid_w + 1);
        #[allow(unused_variables)]
        let y_range = line_start_y..=line_end_y;

        // For drawing food clump
        let food_center_x = 225; //(cols * 3) / 4; // 900 / 4
        let food_center_y = 100; //rows / 2; // 200 / 2 = 100
        let food_radius = FOOD_RADIUS;

        for cell in &mut self.cells {
            let cell_x = cell.x;
            let cell_y = cell.y;

            cell.terrain = Terrain::Empty;

            // Mark border cells
            if (cell_x == 0 || cell_x == cols - 1) || (cell_y == 0 || cell_y == rows - 1) {
                cell.terrain = Terrain::Border;
                continue;
            }

            // Marks a cell as an obstacle.
            // This obstacle will eventually be drawn as a vertical line that is centerted horizontally and vertically
            //if x_range.contains(&cell_x) && y_range.contains(&cell_y) {
            //if cell_y % 20 <= 3 {
            //    continue;
            //}
            //     cell.terrain = Terrain::Obstacle;
            //     continue;
            // }

            // Marks a cell as food.
            let food_dx = cell_x as i32 - food_center_x;
            let food_dy = cell_y as i32 - food_center_y;
            if food_dx * food_dx + food_dy * food_dy <= food_radius as i32 * food_radius as i32 {
                cell.terrain = Terrain::Food;
                cell.food = FOOD_CELL_MAX_AMOUNT;
                continue;
            }

            // Mark the underlying cells of the colony as such
            let cell_center = Vector2::new(cell_x as f32 + 0.5, cell_y as f32 + 0.5);
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

    pub fn get_cell_mut(&mut self, x: u32, y: u32) -> Option<&mut Cell> {
        if !self.is_within_grid_bounds(x, y) {
            return None;
        }
        let i = self.index(x, y);
        Some(&mut self.cells[i])
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

    /// Samples a position and only fills out pheromone strength for the appropriate
    /// pheromone based upon ant state. For example, if ant is currently returning food
    /// the target pheromone would be the "to home" pheromone.
    pub fn sample_cell(&self, position: Vector2, ant_state: AntState) -> CellSample {
        let (x, y) = self.world_to_cell(position);

        let Some(cell) = self.get_cell(x, y) else {
            return CellSample {
                terrain: Terrain::Invalid,
                target_pheromone: 0.0,
            };
        };

        CellSample {
            terrain: cell.terrain,
            target_pheromone: match ant_state {
                AntState::Foraging => {
                    let mut food_strength = cell.to_food;
                    if cell.terrain.is_food() {
                        food_strength += 1000.0;
                    }
                    food_strength
                }
                AntState::ReturningFood => {
                    let mut home_strength = cell.to_home;
                    if cell.terrain.is_colony() {
                        home_strength += 1000.0;
                    }
                    home_strength
                }
            },
        }
    }
}

/* ------------------------------------------------------------------------------ */
/* ---------------- CellSample -------------------------------------------------- */
/* ------------------------------------------------------------------------------ */

#[derive(Debug, Default, Clone, Copy)]
pub struct CellSample {
    pub terrain: Terrain,
    pub target_pheromone: f32,
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

    pub fn has_food(&self) -> bool {
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

/* ------------------------------------------------------------------------------ */
/* ---------------- SpatialGrid ------------------------------------------------- */
/* ------------------------------------------------------------------------------ */

#[derive(Debug)]
pub struct SpatialGrid {
    pub cols: usize,
    pub rows: usize,
    pub bucket_size: usize,
    buckets: Vec<Vec<(usize, Vector2, bool)>>,
}

impl SpatialGrid {
    pub fn new(cols: u32, rows: u32, bucket_size: u32) -> Self {
        let bucket_size = bucket_size.max(1) as usize;

        let bucket_cols = (cols as usize).div_ceil(bucket_size);
        let bucket_rows = (rows as usize).div_ceil(bucket_size);

        Self {
            buckets: vec![Vec::new(); bucket_cols * bucket_rows],
            cols: bucket_cols,
            rows: bucket_rows,
            bucket_size,
        }
    }

    fn index(&self, x: usize, y: usize) -> usize {
        y * self.cols + x
    }

    pub fn bucket(&self, x: usize, y: usize) -> &Vec<(usize, Vector2, bool)> {
        let idx = self.index(x, y);
        &self.buckets[idx]
    }

    pub fn clear(&mut self) {
        for bucket in &mut self.buckets {
            bucket.clear();
        }
    }

    pub fn insert(&mut self, ant_id: u32, position: Vector2, has_food: bool) {
        let x = (position.x / self.bucket_size as f32).floor() as isize;
        let y = (position.y / self.bucket_size as f32).floor() as isize;

        if x < 0 || y < 0 || x >= self.cols as isize || y >= self.rows as isize {
            return;
        }

        let index = self.index(x as usize, y as usize);
        self.buckets[index].push((ant_id as usize, position, has_food));
    }

    pub fn for_each_neighbor<F>(&self, position: Vector2, search_radius: f32, mut callback: F)
    where
        F: FnMut(usize, Vector2, bool),
    {
        let bucket_x = (position.x / self.bucket_size as f32).floor() as isize;
        let bucket_y = (position.y / self.bucket_size as f32).floor() as isize;

        let bucket_radius = (search_radius / self.bucket_size as f32).ceil() as isize;

        for y in (bucket_y - bucket_radius)..=(bucket_y + bucket_radius) {
            for x in (bucket_x - bucket_radius)..=(bucket_x + bucket_radius) {
                if x < 0 || y < 0 {
                    continue;
                }

                if x >= self.cols as isize || y >= self.rows as isize {
                    continue;
                }

                for &(id, pos, has_food) in self.bucket(x as usize, y as usize) {
                    callback(id, pos, has_food);
                }
            }
        }
    }
}
