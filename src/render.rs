use crate::{
    ant::{Ant, AntColony, AntKind, AntState},
    map::{Grid, Terrain},
    settings::{
        ANT_FORAGING_COLOR, ANT_RETURNING_FOOD_COLOR, BACKGROUND_COLOR, COLONY_COLOR, FOOD_COLOR,
        FOOD_RADIUS, OBSTACLE_COLOR,
    },
    world::World,
};
use raylib::{
    ffi::{Color, Rectangle, Vector2},
    prelude::{RaylibDraw as _, RaylibDrawHandle, RaylibMode2D},
};

pub struct WorldPanel {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub cell_size: Vector2,
}

impl WorldPanel {
    pub fn new(x: i32, y: i32, width: i32, height: i32, cols: u32, rows: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
            cell_size: Vector2::new(width as f32 / cols as f32, height as f32 / rows as f32),
        }
    }

    pub fn world_to_screen(&self, world: Vector2) -> Vector2 {
        Vector2::new(
            self.x as f32 + world.x * self.cell_size.x,
            self.y as f32 + world.y * self.cell_size.y,
        )
    }
}

pub struct Renderer {
    world_panel: WorldPanel,
    show_grid: bool,
    show_ants: bool,
    show_pheromones: bool,
    show_ant_sensors: bool,
    show_border: bool,
}

impl Renderer {
    pub fn new(world_panel: WorldPanel) -> Self {
        Self {
            world_panel,
            show_grid: false,
            show_ants: true,
            show_ant_sensors: false,
            show_border: true,
            show_pheromones: true,
        }
    }

    pub fn toggle_show_border(&mut self) {
        self.show_border = !self.show_border;
    }

    // 'show' should be "ALL", "FOOD", or "HOME"
    //pub fn toggle_show_pheromones(&mut self, show: &str) {
    //    match show {
    //        "ALL" => {
    //            self.show_to_home_pheromones = !self.show_to_home_pheromones;
    //            self.show_to_food_pheromones = !self.show_to_food_pheromones;
    //        }
    //        "FOOD" => {
    //            self.show_to_food_pheromones = !self.show_to_food_pheromones;
    //        }
    //        "HOME" => {
    //            self.show_to_home_pheromones = !self.show_to_home_pheromones;
    //        }
    //        _ => {}
    //    }
    //}

    pub fn toggle_show_grid(&mut self) {
        self.show_grid = !self.show_grid;
    }

    pub fn toggle_show_ant_sensors(&mut self) {
        self.show_ant_sensors = !self.show_ant_sensors;
    }

    pub fn toggle_show_ants(&mut self) {
        self.show_ants = !self.show_ants;
    }

    pub fn draw_ant(
        &mut self,
        ant: &Ant,
        d: &mut RaylibMode2D<RaylibDrawHandle>,
        draw_sensors: bool,
    ) {
        let (ant_color, mut sensor_color) = match ant.state {
            AntState::Foraging => (ANT_FORAGING_COLOR, FOOD_COLOR),
            AntState::ReturningFood => (ANT_RETURNING_FOOD_COLOR, COLONY_COLOR),
        };

        let forward = ant.velocity.normalize();
        let right = Vector2::new(-forward.y, forward.x);

        let length = 1.0; // * ANT_SIZE_MULTIPLIER;
        let width = length * 1.0;

        let pos = ant.position;

        let spear_pos = pos + forward * (length * 0.5);
        let left_back_pos = pos - forward * (length * 0.5) - right * (width * 0.5);
        let right_back_pos = pos - forward * (length * 0.5) + right * (width * 0.5);

        let spear = self.world_panel.world_to_screen(spear_pos);
        let left_back = self.world_panel.world_to_screen(left_back_pos);
        let right_back = self.world_panel.world_to_screen(right_back_pos);

        d.draw_triangle(spear, left_back, right_back, ant_color);

        if matches!(ant.kind, AntKind::Explorer { .. }) {
            d.draw_triangle_lines(spear, left_back, right_back, Color::YELLOW);
        }

        /*
        if draw_sensors {
            sensor_color.a = 150; // semi-transparent
            let size = Vector2::new(2.0, 2.0);

            let sensors = ant.get_sensors();

            if let Some(l) = sensors.left.location {
                let pos = Vector2::new(ox + l.x, oy + l.y);
                d.draw_line_v(pos, pos, sensor_color);
                d.draw_rectangle_v(pos, size, sensor_color);
            }
            if let Some(c) = sensors.center.location {
                let pos = Vector2::new(ox + c.x, oy + c.y);
                d.draw_line_v(pos, pos, sensor_color);
                d.draw_rectangle_v(pos, size, sensor_color);
            }
            if let Some(r) = sensors.right.location {
                let pos = Vector2::new(ox + r.x, oy + r.y);
                d.draw_line_v(pos, pos, sensor_color);
                d.draw_rectangle_v(pos, size, sensor_color);
            }
        }
        */
    }

    pub fn draw_grid(&mut self, grid: &mut Grid, d: &mut RaylibMode2D<RaylibDrawHandle>) {
        for cell in grid.iter_mut() {
            let world_pos = Vector2::new(cell.x as f32, cell.y as f32);
            let screen = self.world_panel.world_to_screen(world_pos);

            let color = match cell.terrain {
                Terrain::Empty | Terrain::Colony => BACKGROUND_COLOR,
                Terrain::Obstacle | Terrain::Border => OBSTACLE_COLOR,
                Terrain::Food => FOOD_COLOR,
                Terrain::Invalid => unreachable!("should never try to draw an invalid cell"),
            };

            if self.show_grid {
                let thickness = 0.5;
                let line_color = Color::new(80, 80, 80, 255);
                let rect = Rectangle::new(
                    screen.x,
                    screen.y,
                    self.world_panel.cell_size.x,
                    self.world_panel.cell_size.y,
                );
                d.draw_rectangle_lines_ex(rect, thickness, line_color);
            }

            if cell.is_border() && !self.show_border {
                continue;
            }

            d.draw_rectangle(
                screen.x as i32,
                screen.y as i32,
                self.world_panel.cell_size.x as i32 + 1,
                self.world_panel.cell_size.y as i32 + 1,
                color,
            );
        }

        //let color = FOOD_COLOR;
        //let pos = self.world_panel.world_to_screen(Vector2::new(225.0, 100.0));
        //let rad = FOOD_RADIUS * self.world_panel.cell_size.x;
        //d.draw_circle_v(pos, rad, color);
    }

    pub fn draw_colony(&mut self, colony: &mut AntColony, d: &mut RaylibMode2D<RaylibDrawHandle>) {
        let color = COLONY_COLOR;
        let pos = self.world_panel.world_to_screen(colony.position);
        let rad = colony.radius * self.world_panel.cell_size.x;
        d.draw_circle_v(pos, rad, color);
    }

    pub fn draw_world(&mut self, world: &mut World, d: &mut RaylibMode2D<RaylibDrawHandle>) {
        self.draw_grid(&mut world.grid, d);
        if self.show_ants {
            for ant in &world.colony.ants {
                self.draw_ant(ant, d, false);
            }
        }
        self.draw_colony(&mut world.colony, d);
    }
}
