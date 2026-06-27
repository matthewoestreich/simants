mod ant;
mod settings;
mod world;
mod world_entities;

pub(crate) use ant::*;
pub(crate) use raylib::prelude::*;
pub(crate) use settings::*;
pub(crate) use world_entities::*;

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
    let sw = rl.get_screen_width();
    let sh = rl.get_screen_height();

    let colony = AntColony::new_with_immediate_spawn(
        NUM_ANTS,
        5.0 * CELL_SIZE as f32,
        Vector2::new(sw as f32 / 4.0, sh as f32 / 3.0),
    );

    let mut world = World::new(sw, sh, CELL_SIZE, colony);

    while !rl.window_should_close() {
        world.update(rl.get_frame_time());
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(BACKGROUND_COLOR);
        world.draw(&mut d);
    }
}
