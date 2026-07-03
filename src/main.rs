#![allow(clippy::assertions_on_constants)]
#![allow(clippy::field_reassign_with_default)]

mod ant;
mod map;
mod render;
mod settings;
mod world;

use crate::{
    ant::AntColony,
    map::Grid,
    render::{Renderer, Viewport},
    settings::{
        BACKGROUND_COLOR, COLONY_RADIUS, GRID_COLS, GRID_ROWS, NUM_ANTS, PERCENT_OF_EXPLORER_ANTS,
        PIXELS_PER_CELL, TITLE, WINDOW_HEIGHT, WINDOW_WIDTH,
    },
    world::World,
};
use raylib::{
    ffi::{Camera2D, Color, Vector2},
    prelude::{RaylibDraw as _, RaylibMode2DExt as _},
};

fn main() {
    assert!(
        WINDOW_WIDTH > 0 && WINDOW_HEIGHT > 0,
        "expected screen dimensions to be > 0 : SCREEN_WIDTH={WINDOW_WIDTH} | SCREEN_HEIGHT={WINDOW_HEIGHT}"
    );
    assert!(
        GRID_COLS <= WINDOW_WIDTH as u32 && GRID_ROWS <= WINDOW_HEIGHT as u32,
        "expected grid dimensions to be <= screen dimensions : GRID_WIDTH={GRID_COLS} | GRID_HEIGHT={GRID_ROWS} | SCREEN_WIDTH={WINDOW_WIDTH} | SCREEN_HEIGHT={WINDOW_HEIGHT}"
    );

    let (mut rl, thread) = raylib::init()
        .title(TITLE)
        .size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .build();

    let viewport = Viewport {
        x: 10,
        y: 10,
        width: WINDOW_WIDTH - 10,
        height: WINDOW_HEIGHT - 10,
    };

    let mut renderer = Renderer::new(viewport, PIXELS_PER_CELL as f32);

    let colony_position = Vector2::new(GRID_COLS as f32 / 8.0, GRID_ROWS as f32 / 2.0);
    let colony_radius = COLONY_RADIUS * PIXELS_PER_CELL as f32;
    let colony = AntColony::new_with_immediate_spawn(
        NUM_ANTS,
        PERCENT_OF_EXPLORER_ANTS,
        colony_radius,
        colony_position,
    );

    let mut grid = Grid::new(GRID_COLS, GRID_ROWS);
    grid.initialize(&colony);

    let mut world = World::new(grid, colony);

    let mut camera = Camera2D::default();
    camera.zoom = 1.0;

    rl.set_target_fps(60);

    //let mut is_paused = false;
    //let mut is_dragging = false;
    //let mut is_pheromone_mode = false;
    //let mut click_start_pos = Vector2::zero();

    /* --------------------------------------- */
    /* ------------ Game Loop ---------------- */
    /* --------------------------------------- */
    while !rl.window_should_close() {
        //if !is_paused {
        //    world.update(rl.get_frame_time());
        //}

        //handle_key_press(&mut rl, &mut world, &mut is_paused, &mut is_pheromone_mode);
        //handle_mouse_wheel(&mut rl, &mut camera);
        //handle_mouse_click(
        //    &mut rl,
        //    &mut camera,
        //    &mut world,
        //    &mut is_dragging,
        //    &mut click_start_pos,
        //);

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(BACKGROUND_COLOR);

        {
            let mut mode2d = d.begin_mode2D(camera);
            renderer.draw_world(&mut world, &mut mode2d);
        }

        //if is_pheromone_mode {
        //    d.draw_text("PHEROMONE MODE ON", 10, 10, 20, Color::WHITE);
        //}
    }
}

/* ---------------------------------------------------------------- */
/* -------------- Helper Functions -------------------------------- */
/* ---------------------------------------------------------------- */

/*
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

fn handle_mouse_wheel(rl: &mut RaylibHandle, camera: &mut Camera2D) {
    let wheel = rl.get_mouse_wheel_move();
    if wheel != 0.0 {
        let mouse_screen_pos = rl.get_mouse_position();
        let mouse_world_pos = rl.get_screen_to_world2D(mouse_screen_pos, *camera);

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
}

fn handle_mouse_click(
    rl: &mut RaylibHandle,
    camera: &mut Camera2D,
    world: &mut World,
    is_dragging: &mut bool,
    click_start_pos: &mut Vector2,
) {
    const DRAG_THRESHOLD: f32 = 5.0;

    if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
        *click_start_pos = rl.get_mouse_position();
        *is_dragging = false;
    }

    if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
        let current_pos = rl.get_mouse_position();

        if !*is_dragging && current_pos.distance(*click_start_pos) > DRAG_THRESHOLD {
            *is_dragging = true;
        }

        if *is_dragging {
            let mouse_delta = rl.get_mouse_delta();
            let drag_vector =
                Vector2::new(mouse_delta.x / camera.zoom, mouse_delta.y / camera.zoom);
            camera.target.x -= drag_vector.x;
            camera.target.y -= drag_vector.y;
        }
    }

    if rl.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT) {
        if !*is_dragging {
            let click_position = rl.get_mouse_position();
            let click_world_position = rl.get_screen_to_world2D(click_position, *camera);

            if let Some(ant) = world.colony.ants.iter().find(|ant| {
                ant.is_clicked(
                    click_world_position,
                    12.0f32,
                    world.screen_offset_x,
                    world.screen_offset_y,
                )
            }) {
                println!("{ant}");
            }
        }
        *is_dragging = false;
    }

    if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_RIGHT) {
        let clicked_screen = rl.get_mouse_position();
        let clicked_world = rl.get_screen_to_world2D(clicked_screen, *camera);

        if let Some((x, y)) = world.screen_to_grid_coords(clicked_world)
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
}
*/
