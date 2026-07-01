#![allow(clippy::assertions_on_constants)]

mod ant;
mod map;
mod settings;
mod world;

pub(crate) use ant::*;
pub(crate) use map::*;
pub(crate) use raylib::prelude::*;
pub(crate) use settings::*;

use crate::world::World;

fn main() {
    assert!(
        SCREEN_WIDTH > 0 && SCREEN_HEIGHT > 0,
        "expected screen dimensions to be > 0 : SCREEN_WIDTH={SCREEN_WIDTH} | SCREEN_HEIGHT={SCREEN_HEIGHT}"
    );
    assert!(
        GRID_WIDTH <= SCREEN_WIDTH as u32 && GRID_HEIGHT <= SCREEN_HEIGHT as u32,
        "expected grid dimensions to be <= screen dimensions : GRID_WIDTH={GRID_WIDTH} | GRID_HEIGHT={GRID_HEIGHT} | SCREEN_WIDTH={SCREEN_WIDTH} | SCREEN_HEIGHT={SCREEN_HEIGHT}"
    );

    let (mut rl, thread) = raylib::init()
        .title(TITLE)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .build();

    rl.set_target_fps(60);

    let colony_position = Vector2::new(GRID_WIDTH as f32 / 8.0, GRID_HEIGHT as f32 / 2.0);

    let colony = AntColony::new_with_immediate_spawn(
        NUM_ANTS,
        COLONY_RADIUS * CELL_SIZE as f32,
        colony_position,
    );

    let mut world = World::new(
        rl.get_screen_width(),
        rl.get_screen_height(),
        GRID_WIDTH,
        GRID_HEIGHT,
        CELL_SIZE,
        colony,
        SHOW_GRID_LINES,
        SHOW_PHEROMONES,
        SHOW_BORDER,
        SHOW_ANT_SENSORS,
        SHOW_ANTS,
    );

    let mut is_paused = false;

    while !rl.window_should_close() {
        if rl.is_key_pressed(KeyboardKey::KEY_A) {
            world.toggle_show_ants();
        } else if rl.is_key_pressed(KeyboardKey::KEY_P) {
            world.toggle_show_pheromones();
        } else if rl.is_key_pressed(KeyboardKey::KEY_B) {
            world.toggle_show_border();
        } else if rl.is_key_pressed(KeyboardKey::KEY_G) {
            world.toggle_show_grid();
        } else if rl.is_key_pressed(KeyboardKey::KEY_S) {
            world.toggle_show_ant_sensors();
        } else if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            is_paused = !is_paused;
        }

        if !is_paused {
            world.update(rl.get_frame_time());
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
            }
        } else if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_RIGHT) {
            let clicked = rl.get_mouse_position();
            if let Some((x, y)) = world.screen_to_grid_coords(clicked)
                && let Some(cell) = world.get_cell(x, y)
            {
                println!("{cell:?}");
                if cell.is_colony() {
                    println!(
                        "Colony has harvested : '{}' food",
                        world.colony.harvested_food
                    );
                }
            }
        }

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(BACKGROUND_COLOR);
        world.draw(&mut d);
    }
}
