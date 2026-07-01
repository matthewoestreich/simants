#![allow(clippy::assertions_on_constants)]
#![allow(clippy::field_reassign_with_default)]

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

    let colony_position = Vector2::new(GRID_WIDTH as f32 / 8.0, GRID_HEIGHT as f32 / 2.0);
    let colony_radius = COLONY_RADIUS * CELL_SIZE as f32;

    let colony = AntColony::new_with_immediate_spawn(NUM_ANTS, colony_radius, colony_position);

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

    let mut camera = Camera2D::default();
    camera.zoom = 1.0;

    rl.set_target_fps(60);

    let is_paused = &mut false;
    let is_pheromone_mode = &mut false;

    while !rl.window_should_close() {
        let wheel = rl.get_mouse_wheel_move();
        if wheel != 0.0 {
            let mouse_screen_pos = rl.get_mouse_position();
            let mouse_world_pos = rl.get_screen_to_world2D(mouse_screen_pos, camera);

            let scale_factor = 1.0 + (0.15 * wheel.abs());
            let mut next_zoom = if wheel > 0.0 {
                camera.zoom * scale_factor
            } else {
                camera.zoom / scale_factor
            };

            next_zoom = next_zoom.clamp(1.0, 10.0);

            if next_zoom > 1.0 {
                camera.offset = mouse_screen_pos;
                camera.target = mouse_world_pos;
                camera.zoom = next_zoom;
            } else {
                camera.zoom = 1.0;
                camera.offset = Vector2::new(0.0, 0.0);
                camera.target = Vector2::new(0.0, 0.0);
            }
        }

        if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
            let mouse_delta = rl.get_mouse_delta();
            let drag_vector =
                Vector2::new(mouse_delta.x / camera.zoom, mouse_delta.y / camera.zoom);
            camera.target.x -= drag_vector.x;
            camera.target.y -= drag_vector.y;
        }

        handle_key_press(&mut rl, &mut world, is_paused, is_pheromone_mode);

        if !*is_paused {
            world.update(rl.get_frame_time());
        }

        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            let click_position = rl.get_mouse_position();
            let click_position = rl.get_screen_to_world2D(click_position, camera);
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

        {
            let mut mode2d = d.begin_mode2D(camera);
            world.draw(&mut mode2d, *is_pheromone_mode);
        }
    }
}

fn handle_key_press(
    rl: &mut RaylibHandle,
    world: &mut World,
    is_paused: &mut bool,
    is_pheromone_mode: &mut bool,
) {
    if *is_pheromone_mode {
        if rl.is_key_pressed(KeyboardKey::KEY_P) {
            *is_pheromone_mode = false;
        } else if rl.is_key_pressed(KeyboardKey::KEY_F) {
            world.toggle_show_pheromones("FOOD");
        } else if rl.is_key_pressed(KeyboardKey::KEY_H) {
            world.toggle_show_pheromones("HOME");
        } else if rl.is_key_pressed(KeyboardKey::KEY_A) {
            world.toggle_show_pheromones("ALL");
        }
    } else if rl.is_key_pressed(KeyboardKey::KEY_A) {
        world.toggle_show_ants();
    } else if rl.is_key_pressed(KeyboardKey::KEY_P) {
        *is_pheromone_mode = true;
    } else if rl.is_key_pressed(KeyboardKey::KEY_B) {
        world.toggle_show_border();
    } else if rl.is_key_pressed(KeyboardKey::KEY_G) {
        world.toggle_show_grid();
    } else if rl.is_key_pressed(KeyboardKey::KEY_S) {
        world.toggle_show_ant_sensors();
    } else if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
        *is_paused = !*is_paused;
    }
}
