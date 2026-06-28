use crate::*;

pub struct Grid {
    /// Grid width in pixels - this is NOT number of cols
    pub width: u32,
    /// Grid height in pixels - this is NOT number of rows
    pub height: u32,
    /// Size of a cells width and height in pixels
    pub cell_size: u32,
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
    pub fn new(width: u32, height: u32, cell_size: u32) -> Self {
        let w = width / cell_size;
        let h = height / cell_size;
        let size = (w * h) as usize;

        let mut cells = Vec::with_capacity(size);

        for y in 0..h {
            for x in 0..w {
                cells.push(Cell::new_with_coords(x, y));
            }
        }

        Self {
            width,
            height,
            cell_size,
            cells,
        }
    }

    pub fn get_sensed_food_position(
        &self,
        left_smell: Option<&Cell>,
        center_smell: Option<&Cell>,
        right_smell: Option<&Cell>,
    ) -> Option<Vector2> {
        [left_smell, center_smell, right_smell]
            .into_iter()
            .flatten()
            .find(|cell| cell.is_food())
            .and_then(|cell| self.grid_coords_to_screen(cell.x, cell.y))
    }

    pub fn get_sensed_colony_position(
        &self,
        left_smell: Option<&Cell>,
        center_smell: Option<&Cell>,
        right_smell: Option<&Cell>,
        colony_radius: f32,
        colony_position: Vector2,
    ) -> Option<Vector2> {
        [left_smell, center_smell, right_smell]
            .into_iter()
            .flatten()
            .filter_map(|cell| self.grid_coords_to_screen(cell.x, cell.y))
            .find(|position| position.distance_sqr(colony_position) < colony_radius)
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Cell> {
        self.cells.iter_mut()
    }

    pub fn get_cell(&self, x: u32, y: u32) -> Option<&Cell> {
        if !self.is_within_grid_bounds(x, y) {
            return None;
        }
        Some(&self.cells[self.get_grid_index_from_coords(x, y)])
    }

    pub fn get_cell_mut(&mut self, x: u32, y: u32) -> Option<&mut Cell> {
        if !self.is_within_grid_bounds(x, y) {
            return None;
        }
        let i = self.get_grid_index_from_coords(x, y);
        Some(&mut self.cells[i])
    }

    pub fn get_cell_from_position(&self, position: Vector2) -> Option<&Cell> {
        let (x, y) = self.position_to_grid_coords(position);
        self.get_cell(x, y)
    }

    pub fn get_cell_mut_from_position(&mut self, position: Vector2) -> Option<&mut Cell> {
        let (x, y) = self.position_to_grid_coords(position);
        self.get_cell_mut(x, y)
    }

    pub fn position_to_grid_coords(&self, position: Vector2) -> (u32, u32) {
        let x = (position.x / self.cell_size as f32).floor() as u32;
        let y = (position.y / self.cell_size as f32).floor() as u32;
        (x, y)
    }

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

    pub fn is_within_grid_bounds(&self, x: u32, y: u32) -> bool {
        x < self.width && y < self.height
    }

    fn get_grid_index_from_coords(&self, x: u32, y: u32) -> usize {
        (y * self.width + x) as usize
    }
}

/* ------------------------------------------------------------------------------ */
/* ---------------- Cell -------------------------------------------------------- */
/* ------------------------------------------------------------------------------ */

#[derive(Debug, Default, Clone, Copy)]
pub struct Cell {
    pub terrain: Terrain,
    pub to_food: Pheromone,
    pub to_home: Pheromone,
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

    pub fn is_food(&self) -> bool {
        matches!(self.terrain, Terrain::Food(_))
    }

    fn terrain_allows_pheromone(&self) -> bool {
        !matches!(self.terrain, Terrain::Obstacle { .. } | Terrain::Food(_))
    }
}
