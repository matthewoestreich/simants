#![allow(clippy::assertions_on_constants)]
#![allow(clippy::field_reassign_with_default)]

mod ant;
mod map;
mod render;
mod settings;
mod world;

use std::collections::HashSet;

use crate::{
    ant::AntColony,
    map::Grid,
    render::{Renderer, Viewport},
    settings::{
        BACKGROUND_COLOR, COLONY_RADIUS, GRID_COLS, GRID_ROWS, NUM_ANTS, PERCENT_OF_EXPLORER_ANTS,
        TITLE, WINDOW_HEIGHT, WINDOW_WIDTH, WORLD_HEIGHT, WORLD_WIDTH,
    },
    world::World,
};
use raylib::{
    RaylibHandle,
    ffi::{Camera2D, Color, KeyboardKey, MouseButton, Vector2},
    prelude::{RaylibDraw as _, RaylibMode2DExt as _, RaylibScissorModeExt},
};

fn main() {
    let (mut rl, thread) = raylib::init()
        .title(TITLE)
        .size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .build();

    let viewport = Viewport::new(
        (WINDOW_WIDTH - WORLD_WIDTH) / 2,
        (WINDOW_HEIGHT - WORLD_HEIGHT) / 2,
        WORLD_WIDTH,
        WORLD_HEIGHT,
        GRID_COLS,
        GRID_ROWS,
    );

    let mut renderer = Renderer::new(viewport);

    let colony = AntColony::new_with_immediate_spawn(
        NUM_ANTS,
        PERCENT_OF_EXPLORER_ANTS,
        COLONY_RADIUS,
        Vector2::new(
            (GRID_COLS as f32 / 8.0).floor(),
            (GRID_ROWS as f32 / 2.0).floor(),
        ),
    );

    let mut grid = Grid::new(GRID_COLS, GRID_ROWS);
    grid.initialize(&colony);

    let mut world = World::new(grid, colony);

    let mut camera = Camera2D {
        target: Vector2::new(
            (GRID_COLS as f32 * renderer.viewport.cell_size.x) / 2.0,
            (GRID_ROWS as f32 * renderer.viewport.cell_size.y) / 2.0,
        ),
        offset: Vector2::new(
            renderer.viewport.x as f32 + (WORLD_WIDTH as f32 / 2.0),
            renderer.viewport.y as f32 + (WORLD_HEIGHT as f32 / 2.0),
        ),
        rotation: 0.0,
        zoom: 1.0,
    };

    rl.set_target_fps(60);

    let mut is_paused = false;
    let mut is_dragging = false;
    let mut is_pheromone_mode = false;
    let mut click_start_pos = Vector2::zero();
    let mut fast_forward_multiplier = 1.0; // 1 is normal speed

    /* --------------------------------------- */
    /* ------------ Game Loop ---------------- */
    /* --------------------------------------- */
    while !rl.window_should_close() {
        if !is_paused {
            let dt = rl.get_frame_time() * fast_forward_multiplier;

            if fast_forward_multiplier > 1.0 {
                // Fast-forward splits the work into stable, tiny slices!
                // Example: at 10x speed, we loop 10 times, passing a safe 1x delta_time each loop
                let steps = fast_forward_multiplier.floor() as i32;
                let step_dt = 0.01666667;
                for _ in 0..steps {
                    world.update(step_dt);
                }
            }

            world.update(dt);
        }

        if renderer.viewport.is_within_bounds(rl.get_mouse_position()) {
            handle_key_press(
                &mut rl,
                &mut renderer,
                &mut is_paused,
                &mut is_pheromone_mode,
                &mut fast_forward_multiplier,
            );
            handle_mouse_wheel(&mut rl, &mut camera, &mut renderer);
            handle_mouse_click(
                &mut rl,
                &mut camera,
                &mut world,
                &renderer.viewport,
                &mut is_dragging,
                &mut click_start_pos,
            );
        }

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(BACKGROUND_COLOR);

        {
            let mut scissor = d.begin_scissor_mode(
                renderer.viewport.x,
                renderer.viewport.y,
                renderer.viewport.width,
                renderer.viewport.height,
            );
            let mut mode2d = scissor.begin_mode2D(camera);
            renderer.draw_world(&mut world, &mut mode2d);
        }

        // Bordr around viewport
        d.draw_rectangle_lines(
            renderer.viewport.x,
            renderer.viewport.y,
            renderer.viewport.width,
            renderer.viewport.height,
            Color::RED,
        );

        if is_pheromone_mode {
            d.draw_text("PHEROMONE MODE ON", 10, 10, 20, Color::WHITE);
        }
        if fast_forward_multiplier > 1.0 {
            d.draw_text(
                &format!(">> x{fast_forward_multiplier}"),
                10,
                30,
                10,
                Color::WHITE,
            );
        }
    }
}

/* ---------------------------------------------------------------- */
/* -------------- Helper Functions -------------------------------- */
/* ---------------------------------------------------------------- */

fn handle_key_press(
    rl: &mut RaylibHandle,
    renderer: &mut Renderer,
    is_paused: &mut bool,
    is_pheromone_mode: &mut bool,
    fast_forward: &mut f32,
) {
    if *is_pheromone_mode {
        if rl.is_key_pressed(KeyboardKey::KEY_P) {
            *is_pheromone_mode = false;
        } else if rl.is_key_pressed(KeyboardKey::KEY_F) {
            renderer.toggle_show_pheromones("FOOD");
        } else if rl.is_key_pressed(KeyboardKey::KEY_H) {
            renderer.toggle_show_pheromones("HOME");
        } else if rl.is_key_pressed(KeyboardKey::KEY_A) {
            renderer.toggle_show_pheromones("ALL");
        }
    } else if rl.is_key_pressed(KeyboardKey::KEY_F) {
        *fast_forward += 1.0;
        if *fast_forward == 6.0 {
            *fast_forward = 10.0;
        } else if *fast_forward >= 10.0 {
            *fast_forward = 1.0;
        }
    } else if rl.is_key_pressed(KeyboardKey::KEY_A) {
        renderer.toggle_show_ants();
    } else if rl.is_key_pressed(KeyboardKey::KEY_P) {
        *is_pheromone_mode = true;
    } else if rl.is_key_pressed(KeyboardKey::KEY_B) {
        renderer.toggle_show_border();
    } else if rl.is_key_pressed(KeyboardKey::KEY_G) {
        renderer.toggle_show_grid();
    } else if rl.is_key_pressed(KeyboardKey::KEY_S) {
        renderer.toggle_show_ant_sensors();
    } else if rl.is_key_pressed(KeyboardKey::KEY_C) {
        renderer.toggle_show_colony();
    } else if rl.is_key_pressed(KeyboardKey::KEY_O) {
        renderer.toggle_show_food();
    } else if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
        *is_paused = !*is_paused;
    }
}

fn handle_mouse_wheel(rl: &mut RaylibHandle, camera: &mut Camera2D, renderer: &mut Renderer) {
    let wheel = rl.get_mouse_wheel_move();
    if wheel != 0.0 {
        let raw_mouse_pos = rl.get_mouse_position();
        let world_mouse_before = rl.get_screen_to_world2D(raw_mouse_pos, *camera);

        let scale_factor = 1.0 + (0.15 * wheel.abs());
        let mut next_zoom = if wheel > 0.0 {
            camera.zoom * scale_factor
        } else {
            camera.zoom / scale_factor
        };

        next_zoom = next_zoom.clamp(1.0, 100.0);

        if next_zoom > 1.0 {
            camera.zoom = next_zoom;
            let world_mouse_after = rl.get_screen_to_world2D(raw_mouse_pos, *camera);
            camera.target.x += world_mouse_before.x - world_mouse_after.x;
            camera.target.y += world_mouse_before.y - world_mouse_after.y;
        } else {
            let wp = &renderer.viewport;
            // Restore clean home alignment
            camera.zoom = 1.0;
            camera.target = Vector2::new(
                (GRID_COLS as f32 * wp.cell_size.x) / 2.0,
                (GRID_ROWS as f32 * wp.cell_size.y) / 2.0,
            );
            camera.offset = Vector2::new(
                wp.x as f32 + (WORLD_WIDTH as f32 / 2.0),
                wp.y as f32 + (WORLD_HEIGHT as f32 / 2.0),
            );
        }
    }
}

fn handle_mouse_click(
    rl: &mut RaylibHandle,
    camera: &mut Camera2D,
    world: &mut World,
    world_panel: &Viewport,
    is_dragging: &mut bool,
    click_start_pos: &mut Vector2,
) {
    const DRAG_THRESHOLD: f32 = 5.0;
    // Always use the absolute, raw window mouse coordinates for camera calculations
    let raw_mouse_pos = rl.get_mouse_position();

    if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
        *click_start_pos = raw_mouse_pos;
        *is_dragging = false;
    }

    if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
        if !*is_dragging && raw_mouse_pos.distance(*click_start_pos) > DRAG_THRESHOLD {
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
            let click_world_position = rl.get_screen_to_world2D(raw_mouse_pos, *camera);
            let click_grid_position = Vector2::new(
                (click_world_position.x / world_panel.cell_size.x).floor(),
                (click_world_position.y / world_panel.cell_size.y).floor(),
            );

            println!(
                "Screen click (raw window pixel): {raw_mouse_pos:?}\n\
                 World Pixel coordinate (Simulation space): {click_world_position:?}\n\
                 Grid block target (Cell row/col): {click_grid_position:?}"
            );

            if let Some(ant) = world
                .colony
                .ants
                .iter()
                .find(|ant| ant.is_clicked(click_grid_position, 3.0f32))
            {
                println!("{ant}");
            }
            println!();
        }
        *is_dragging = false;
    }

    if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_RIGHT) {
        let raw_mouse_pos = rl.get_mouse_position();
        let clicked_world = rl.get_screen_to_world2D(raw_mouse_pos, *camera);

        // Convert continuous world pixel parameters straight down to array cells
        let x = (clicked_world.x / world_panel.cell_size.x).floor() as u32;
        let y = (clicked_world.y / world_panel.cell_size.y).floor() as u32;

        println!("Right clicked World Pixels = {:?}", clicked_world);
        println!("Calculated Grid Slots = X: {}, Y: {}", x, y);

        if let Some(cell) = world.grid.get_cell(x, y) {
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
