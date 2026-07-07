use crate::{
    ant::{Ant, AntColony, AntKind, AntState},
    map::{Grid, Terrain},
    settings::{
        ANT_FORAGING_COLOR, ANT_LENGTH, ANT_PROJECTION_CIRCLE_RADIUS, ANT_RETURNING_FOOD_COLOR,
        ANT_WIDTH, BACKGROUND_COLOR, COLONY_COLOR, FOOD_COLOR, GRID_COLS, GRID_ROWS,
        OBSTACLE_COLOR, PHEROMONE_FORAGING_COLOR, PHEROMONE_RETURNING_FOOD_COLOR, SHOW_ANT_SENSORS,
        SHOW_ANTS, SHOW_BORDER, SHOW_GRID_LINES, SHOW_PHEROMONES,
    },
    world::World,
};
use raylib::{
    ffi::{Color, Vector2},
    prelude::RaylibDraw,
};

#[derive(Default, Debug)]
pub struct Viewport {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub cell_size: Vector2,
}

impl Viewport {
    pub fn new(x: i32, y: i32, width: i32, height: i32, cols: u32, rows: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
            cell_size: Vector2::new(width as f32 / cols as f32, height as f32 / rows as f32),
        }
    }

    pub fn grid_to_world(&self, grid_pos: Vector2) -> Vector2 {
        Vector2::new(grid_pos.x * self.cell_size.x, grid_pos.y * self.cell_size.y)
    }

    pub fn is_within_bounds(&self, pos: Vector2) -> bool {
        pos.x >= self.x as f32
            && pos.x <= (self.x + self.width) as f32
            && pos.y >= self.y as f32
            && pos.y <= (self.y + self.height) as f32
    }
}

#[derive(Default, Debug)]
pub struct Renderer {
    pub viewport: Viewport,
    show_grid: bool,
    show_ants: bool,
    show_to_home_pheromones: bool,
    show_to_food_pheromones: bool,
    show_ant_sensors: bool,
    show_border: bool,
    show_colony: bool,
    show_food: bool,
}

impl Renderer {
    pub fn new(viewport: Viewport) -> Self {
        Self {
            viewport,
            show_ants: SHOW_ANTS,
            show_colony: true,
            show_grid: SHOW_GRID_LINES,
            show_border: SHOW_BORDER,
            show_ant_sensors: SHOW_ANT_SENSORS,
            show_to_home_pheromones: SHOW_PHEROMONES,
            show_to_food_pheromones: SHOW_PHEROMONES,
            show_food: true,
        }
    }

    pub fn toggle_show_border(&mut self) {
        self.show_border = !self.show_border;
    }

    // 'show' should be "ALL", "FOOD", or "HOME"
    pub fn toggle_show_pheromones(&mut self, show: &str) {
        match show {
            "ALL" => {
                self.show_to_home_pheromones = !self.show_to_home_pheromones;
                self.show_to_food_pheromones = !self.show_to_food_pheromones;
            }
            "FOOD" => {
                self.show_to_food_pheromones = !self.show_to_food_pheromones;
            }
            "HOME" => {
                self.show_to_home_pheromones = !self.show_to_home_pheromones;
            }
            _ => {}
        }
    }

    pub fn toggle_show_food(&mut self) {
        self.show_food = !self.show_food;
    }

    pub fn toggle_show_grid(&mut self) {
        self.show_grid = !self.show_grid;
    }

    pub fn toggle_show_ant_sensors(&mut self) {
        self.show_ant_sensors = !self.show_ant_sensors;
    }

    pub fn toggle_show_colony(&mut self) {
        self.show_colony = !self.show_colony;
    }

    pub fn toggle_show_ants(&mut self) {
        self.show_ants = !self.show_ants;
    }

    pub fn draw_ant(&mut self, ant: &Ant, d: &mut impl RaylibDraw) {
        let (mut ant_color, mut sensor_color) = match ant.state {
            AntState::Foraging => (ANT_FORAGING_COLOR, FOOD_COLOR),
            AntState::ReturningFood => (ANT_RETURNING_FOOD_COLOR, COLONY_COLOR),
        };

        if ant.paused > 0.0 {
            ant_color = Color::PURPLE;
        }

        let forward = ant.navigator.velocity.normalize();
        let right = Vector2::new(-forward.y, forward.x);

        let length = ANT_LENGTH;
        let width = ANT_WIDTH;

        let pos = ant.navigator.position;

        let spear_pos = pos + forward * (length * 0.5);
        let left_back_pos = pos - forward * (length * 0.5) - right * (width * 0.5);
        let right_back_pos = pos - forward * (length * 0.5) + right * (width * 0.5);

        let spear = self.viewport.grid_to_world(spear_pos);
        let left_back = self.viewport.grid_to_world(left_back_pos);
        let right_back = self.viewport.grid_to_world(right_back_pos);

        d.draw_triangle(spear, left_back, right_back, ant_color);

        //self.draw_ant_projection_circle(ant, d);

        if matches!(ant.kind, AntKind::Explorer { .. }) {
            d.draw_triangle_lines(spear, left_back, right_back, Color::YELLOW);
        }

        if self.show_ant_sensors {
            sensor_color.a = 150; // semi-transparent
            let size = Vector2::new(2.0, 2.0);

            let sensors = ant.get_sensors();

            if let Some(l) = sensors.left.location {
                let pos = self.viewport.grid_to_world(Vector2::new(l.x, l.y));
                d.draw_line_v(pos, pos, sensor_color);
                d.draw_rectangle_v(pos, size, sensor_color);
            }
            if let Some(c) = sensors.center.location {
                let pos = self.viewport.grid_to_world(Vector2::new(c.x, c.y));
                d.draw_line_v(pos, pos, sensor_color);
                d.draw_rectangle_v(pos, size, sensor_color);
            }
            if let Some(r) = sensors.right.location {
                let pos = self.viewport.grid_to_world(Vector2::new(r.x, r.y));
                d.draw_line_v(pos, pos, sensor_color);
                d.draw_rectangle_v(pos, size, sensor_color);
            }
        }
    }

    #[allow(dead_code, unused_variables)]
    pub fn draw_ant_projection_circle(&mut self, ant: &Ant, d: &mut impl RaylibDraw) {
        // Determine grid positions using your saved fields
        let absolute_circle_grid = ant.navigator.position + ant.navigator.wander_circle;
        let absolute_dot_grid = absolute_circle_grid + ant.navigator.wander_circle_displacement;
        // Map coordinates through your viewport matrix
        let w_circle_center = self.viewport.grid_to_world(absolute_circle_grid);
        let w_target_dot = self.viewport.grid_to_world(absolute_dot_grid);
        let w_ant_center = self.viewport.grid_to_world(ant.navigator.position);
        // Map a secondary point on the circle edge to determine the EXACT scaled pixel radius
        let edge_grid_point = absolute_circle_grid
            + (ant.navigator.velocity.normalize() * ANT_PROJECTION_CIRCLE_RADIUS);
        let w_edge_point = self.viewport.grid_to_world(edge_grid_point);
        let w_circle_radius = (w_edge_point - w_circle_center).length();
        // Draw the perfect hollow border
        d.draw_circle_lines_v(w_circle_center, w_circle_radius, Color::GRAY);
        // Draw the guiding lines
        d.draw_line_v(w_ant_center, w_circle_center, Color::LIGHTGRAY);
        d.draw_line_v(w_circle_center, w_target_dot, Color::DARKGRAY);
        // Draw the solid target point right on the outline
        d.draw_circle_v(w_target_dot, 2.0, Color::DARKGRAY);
    }

    pub fn draw_grid(&mut self, grid: &mut Grid, d: &mut impl RaylibDraw) {
        for cell in grid.iter_mut() {
            let draw = self
                .viewport
                .grid_to_world(Vector2::new(cell.x as f32, cell.y as f32));

            let color = match cell.terrain {
                Terrain::Empty | Terrain::Colony => BACKGROUND_COLOR,
                Terrain::Obstacle | Terrain::Border => OBSTACLE_COLOR,
                Terrain::Food => BACKGROUND_COLOR, // FOOD_COLOR,
                Terrain::Invalid => unreachable!("should never try to draw an invalid cell"),
            };

            if cell.is_border() && !self.show_border {
                continue;
            }
            if cell.has_food() && !self.show_food {
                continue;
            }

            d.draw_rectangle(
                draw.x as i32,
                draw.y as i32,
                self.viewport.cell_size.x as i32 + 1,
                self.viewport.cell_size.y as i32 + 1,
                color,
            );

            if self.show_to_home_pheromones && cell.to_home > 0.0 {
                let brightness = (cell.to_home / 5.0) - 1.5;
                let color = PHEROMONE_FORAGING_COLOR.brightness(brightness);
                d.draw_rectangle(
                    draw.x as i32,
                    draw.y as i32,
                    self.viewport.cell_size.x as i32 + 1,
                    self.viewport.cell_size.y as i32 + 1,
                    color,
                );
            }
            if self.show_to_food_pheromones && cell.to_food > 0.0 {
                let brightness = (cell.to_food / 5.0) - 1.5;
                let color = PHEROMONE_RETURNING_FOOD_COLOR.brightness(brightness);
                d.draw_rectangle(
                    draw.x as i32,
                    draw.y as i32,
                    self.viewport.cell_size.x as i32 + 1,
                    self.viewport.cell_size.y as i32 + 1,
                    color,
                );
            }
        }

        if self.show_food {
            for cell in grid.iter_mut() {
                if cell.has_food() {
                    let draw = self
                        .viewport
                        .grid_to_world(Vector2::new(cell.x as f32, cell.y as f32));
                    d.draw_rectangle(
                        draw.x as i32,
                        draw.y as i32,
                        self.viewport.cell_size.x as i32 + 1,
                        self.viewport.cell_size.y as i32 + 1,
                        FOOD_COLOR,
                    );
                }
            }
        }

        if self.show_grid {
            let thickness = 0.3;
            let line_color = Color::new(80, 80, 80, 255);
            let max_world_x = GRID_COLS as f32 * self.viewport.cell_size.x;
            let max_world_y = GRID_ROWS as f32 * self.viewport.cell_size.y;
            for col in 0..=GRID_COLS {
                let x_pos = col as f32 * self.viewport.cell_size.x;
                let start = Vector2::new(x_pos, 0.0);
                let end = Vector2::new(x_pos, max_world_y);
                d.draw_line_ex(start, end, thickness, line_color);
            }
            for row in 0..=GRID_ROWS {
                let y_pos = row as f32 * self.viewport.cell_size.y;
                let start = Vector2::new(0.0, y_pos);
                let end = Vector2::new(max_world_x, y_pos);
                d.draw_line_ex(start, end, thickness, line_color);
            }
        }
    }

    pub fn draw_colony(&mut self, colony: &mut AntColony, d: &mut impl RaylibDraw) {
        let mut color = COLONY_COLOR;
        color.a = 150;
        let pos = self.viewport.grid_to_world(colony.position);
        let rad = colony.radius * self.viewport.cell_size.x;
        d.draw_circle_v(pos, rad, color);
    }

    pub fn draw_world(&mut self, world: &mut World, d: &mut impl RaylibDraw) {
        self.draw_grid(&mut world.grid, d);

        if self.show_ants {
            for ant in &world.colony.ants {
                self.draw_ant(ant, d);
            }
        }

        if self.show_colony {
            self.draw_colony(&mut world.colony, d);
        }
    }
}
