mod ant;
mod settings;
mod world;

pub(crate) use ant::*;
pub(crate) use raylib::prelude::*;
pub(crate) use settings::*;

use crate::world::World;

fn main() {
    let mut rl_builder = raylib::init();
    rl_builder.title(TITLE);

    if SCREEN_WIDTH <= 0 || SCREEN_HEIGHT <= 0 {
        rl_builder.fullscreen();
    } else {
        rl_builder.size(SCREEN_WIDTH, SCREEN_HEIGHT);
    }

    let (mut rl, thread) = rl_builder.build();

    let mut colony = AntColony::new(NUM_ANTS, 5, 10, 10);
    colony.spawn_ants();

    let mut world = World::new(
        rl.get_screen_width(),
        rl.get_screen_height(),
        CELL_SIZE,
        colony,
    );

    println!("{}", world.width * world.height);

    while !rl.window_should_close() {
        world.update(rl.get_frame_time());

        let mut drawing = rl.begin_drawing(&thread);
        drawing.clear_background(BACKGROUND_COLOR);

        world.draw(&mut drawing);
    }
}

pub fn grid_to_pixel(grid_x: i32, grid_y: i32, cell_size: i32, get_center: bool) -> Vector2 {
    let mut pixel_x = (grid_x * cell_size) as f32;
    let mut pixel_y = (grid_y * cell_size) as f32;

    if get_center {
        let half_cell = cell_size as f32 / 2.0;
        pixel_x += half_cell;
        pixel_y += half_cell;
    }

    Vector2::new(pixel_x, pixel_y)
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Food {
    pub pos_x: i32,
    pub pos_y: i32,
    pub amount: i32,
}

#[derive(Default, Debug, Clone, Copy)]
pub enum Pheromone {
    #[default]
    Searching,
    ToFood,
    ToHome,
}

#[derive(Default, Debug, Clone, Copy)]
pub enum Obstacle {
    #[default]
    Normal,
    Border,
}

#[derive(Default, Debug, Clone, Copy)]
pub enum CellContents {
    #[default]
    Empty,
    Colony,
    Pheromone {
        kind: Pheromone,
        // strength can be viewed as 'real world seconds to live for'
        strength: f32,
    },
    Food(Food),
    Obstacle {
        kind: Obstacle,
    },
}
