use crate::{Ant, CellContents};
use raylib::{
    ffi::{Color, Vector2},
    prelude::{RaylibDraw as _, RaylibDrawHandle},
};

pub struct World {
    pub width: i32,
    pub height: i32,
    pub grid: Grid<CellContents>,
    pub ants: Vec<Ant>,
}

impl World {
    pub fn new(width: i32, height: i32, cell_size: i32, num_ants: usize) -> Self {
        let mut ants = Vec::with_capacity(num_ants);
        ants.resize_with(num_ants, || Ant::new(Vector2::new(100.0, 100.0)));

        let mut grid = Grid::new(width, height, cell_size);

        let w = width / cell_size;
        let h = height / cell_size;
        let mid_x = w / 2;
        let line_length = h / 2;
        let line_start_y = (h - line_length) / 2;
        let line_end_y = line_start_y + line_length;

        for x in 0..w {
            for y in 0..h {
                if (x == 0 || x == w - 1 || y == 0 || y == h - 1)
                    && let Some(cell) = grid.get_mut(x, y)
                {
                    cell.contents = CellContents::Obstacle;
                    continue;
                }

                // Draw vertical line
                if ((line_start_y..=line_end_y).contains(&y) && x == mid_x)
                    && let Some(cell) = grid.get_mut(x, y)
                {
                    cell.contents = CellContents::Obstacle;
                    continue;
                }
            }
        }

        Self {
            ants,
            width,
            height,
            grid,
        }
    }

    pub fn update(&mut self, dt: f32) {
        for ant in &mut self.ants {
            ant.update(dt, &mut self.grid);
        }
    }

    pub fn draw(&mut self, d: &mut RaylibDrawHandle) {
        let cell_size = self.grid.cell_size;

        for y in 0..self.height {
            for x in 0..self.width {
                if let Some(cell) = self.grid.get(x, y)
                    && matches!(cell.contents, CellContents::Obstacle)
                {
                    d.draw_rectangle(
                        x * cell_size,
                        y * cell_size,
                        cell_size,
                        cell_size,
                        Color::RED,
                    );
                }
            }
        }

        for ant in &self.ants {
            ant.draw(d);
        }
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
        cells.resize_with(size, Cell::default);

        Self {
            width,
            height,
            cell_size,
            cells,
        }
    }

    pub fn cells_mut(&mut self) -> &mut Vec<Cell<T>> {
        &mut self.cells
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
}

impl<T> Cell<T>
where
    T: Default,
{
    pub fn new(contents: T) -> Self {
        Self { contents }
    }
}
