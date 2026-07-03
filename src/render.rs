use crate::{
    ant::{Ant, AntColony, AntKind, AntState},
    map::{Grid, Terrain},
    settings::{
        ANT_FORAGING_COLOR, ANT_RETURNING_FOOD_COLOR, ANT_SIZE_MULTIPLIER, BACKGROUND_COLOR,
        COLONY_COLOR, FOOD_COLOR, OBSTACLE_COLOR, PIXELS_PER_CELL,
    },
    world::World,
};
use raylib::{
    ffi::{Color, Vector2},
    prelude::{RaylibDraw as _, RaylibDrawHandle},
};

pub struct Viewport {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

pub struct Renderer {
    viewport: Viewport,
    pixels_per_unit: f32,

    show_grid: bool,
    show_ants: bool,
    show_pheromones: bool,
    show_ant_sensors: bool,
    show_border: bool,
}

impl Renderer {
    pub fn new(viewport: Viewport, pixels_per_unit: f32) -> Self {
        Self {
            viewport,
            pixels_per_unit,
            show_grid: true,
            show_ants: true,
            show_ant_sensors: true,
            show_border: true,
            show_pheromones: true,
        }
    }

    pub fn world_to_screen(&self, p: Vector2) -> Vector2 {
        raylib::prelude::Vector2::new(
            self.viewport.x as f32 + p.x * self.pixels_per_unit,
            self.viewport.y as f32 + p.y * self.pixels_per_unit,
        )
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

    pub fn draw_ant(&mut self, ant: &Ant, d: &mut RaylibDrawHandle, draw_sensors: bool) {
        let (ant_color, mut sensor_color) = match ant.state {
            AntState::Foraging => (ANT_FORAGING_COLOR, FOOD_COLOR),
            AntState::ReturningFood => (ANT_RETURNING_FOOD_COLOR, COLONY_COLOR),
        };

        let forward = ant.velocity.normalize();
        let right = Vector2::new(-forward.y, forward.x);

        let length = self.pixels_per_unit * 0.8; // * ANT_SIZE_MULTIPLIER;
        let width = length * 0.5;

        let pos = self.world_to_screen(ant.position);

        let spear = pos + forward * (length * 0.5);
        let left_back = pos - forward * (length * 0.5) - right * (width * 0.5);
        let right_back = pos - forward * (length * 0.5) + right * (width * 0.5);

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

    pub fn draw_grid(&mut self, grid: &mut Grid, d: &mut RaylibDrawHandle) {
        for cell in grid.iter_mut() {
            let x = cell.x as i32;
            let y = cell.y as i32;

            let world = Vector2::new(x as f32, y as f32);
            let screen = self.world_to_screen(world);
            let screen_x = screen.x as i32;
            let screen_y = screen.y as i32;

            let color = match cell.terrain {
                Terrain::Empty | Terrain::Colony => BACKGROUND_COLOR,
                Terrain::Food => FOOD_COLOR,
                Terrain::Obstacle | Terrain::Border => OBSTACLE_COLOR,
                Terrain::Invalid => unreachable!("should never try to draw an invalid cell"),
            };

            let size = (self.pixels_per_unit * 0.9) as i32;
            d.draw_rectangle(screen_x, screen_y, size, size, color);
        }
    }

    pub fn draw_colony(&mut self, colony: &mut AntColony, d: &mut RaylibDrawHandle) {
        let color = COLONY_COLOR;
        let pos = self.world_to_screen(colony.position);
        d.draw_circle_v(pos, colony.radius, color);
    }

    pub fn draw_world(&mut self, world: &mut World, d: &mut RaylibDrawHandle) {
        self.draw_grid(&mut world.grid, d);
        if self.show_ants {
            for ant in &world.colony.ants {
                self.draw_ant(ant, d, false);
            }
        }
        self.draw_colony(&mut world.colony, d);
    }
}
