mod ant;
mod entities;
mod grid;
mod settings;
mod world;

pub(crate) use ant::*;
pub(crate) use entities::*;
pub(crate) use grid::*;
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
    rl.set_target_fps(60);

    let screen_width = rl.get_screen_width();
    let screen_height = rl.get_screen_height();
    //let grid_width = screen_width.max(0) as u32 - (40 * CELL_SIZE);
    //let grid_height = screen_height.max(0) as u32 - (20 * CELL_SIZE);
    let colony_position_x = GRID_WIDTH as f32 / 8.0;
    let colony_position_y = GRID_HEIGHT as f32 / 2.0;

    let colony = AntColony::new_with_immediate_spawn(
        NUM_ANTS,
        COLONY_RADIUS * CELL_SIZE as f32,
        Vector2::new(colony_position_x, colony_position_y),
    );

    let mut world = World::new(
        screen_width,
        screen_height,
        GRID_WIDTH,
        GRID_HEIGHT,
        CELL_SIZE,
        colony,
        SHOW_GRID_LINES,
        SHOW_PHEROMONES,
        SHOW_BORDER,
        SHOW_ANT_SENSORS,
    );

    let show_pheromones_toggle_key = KeyboardKey::KEY_P;
    let show_border_toggle_key = KeyboardKey::KEY_B;
    let show_grid_toggle_key = KeyboardKey::KEY_G;
    let show_ant_sensors_toggle_key = KeyboardKey::KEY_S;

    while !rl.window_should_close() {
        // delta time can range from ~0.01 - 0.0008..
        // USUALLY it is around 0.0008
        world.update(rl.get_frame_time());

        if rl.is_key_pressed(show_pheromones_toggle_key) {
            world.toggle_show_pheromones();
        } else if rl.is_key_pressed(show_border_toggle_key) {
            world.toggle_show_border();
        } else if rl.is_key_pressed(show_grid_toggle_key) {
            world.toggle_show_grid();
        } else if rl.is_key_pressed(show_ant_sensors_toggle_key) {
            world.toggle_show_ant_sensors();
        }

        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            let click_position = rl.get_mouse_position();
            if let Some(ant) = world.colony.ants.iter().find(|ant| {
                ant.is_clicked(
                    click_position,
                    12.0f32,
                    world.screen_offset_x,
                    world.screen_offset_y,
                )
            }) {
                println!("{ant}");
            } else if let Some((x, y)) = world.screen_to_grid_coords(click_position)
                && let Some(cell) = world.get_cell(x, y)
            {
                println!("{cell:?}");
            }
        }

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(BACKGROUND_COLOR);
        world.draw(&mut d);
    }
}
