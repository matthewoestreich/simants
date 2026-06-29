use crate::*;

pub struct Grid {
    /// Grid width in pixels - this is NOT number of cols
    pub cols: u32,
    /// Grid height in pixels - this is NOT number of rows
    pub rows: u32,
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
    pub fn new(cols: u32, rows: u32, cell_size: u32) -> Self {
        let size = (cols * rows) as usize;
        let mut cells = Vec::with_capacity(size);

        for y in 0..rows {
            for x in 0..cols {
                cells.push(Cell::new_with_coords(x, y));
            }
        }

        Self {
            cols,
            rows,
            cell_size,
            cells,
        }
    }

    pub fn len(&self) -> usize {
        self.cells.len()
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
        x < self.cols && y < self.rows
    }

    fn get_grid_index_from_coords(&self, x: u32, y: u32) -> usize {
        (y * self.cols + x) as usize
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
    pub food: Option<f32>,
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
        matches!(self.terrain, Terrain::Food)
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
