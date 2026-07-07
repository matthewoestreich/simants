#![allow(clippy::assertions_on_constants)]
#![allow(clippy::field_reassign_with_default)]

mod ant;
mod app;
mod gui;
mod map;
mod render;
mod reynolds;
mod settings;
mod world;

use crate::{
    ant::AntColony,
    app::{App, InputState, Simulation, SimulationState, Stats},
    gui::{
        Gui,
        controls::{DockSide, SlidePanel, TextBox},
    },
    map::Grid,
    render::{Renderer, Viewport},
    settings::{
        BACKGROUND_COLOR, COLONY_RADIUS, GRID_COLS, GRID_ROWS, NUM_ANTS, PERCENT_OF_EXPLORER_ANTS,
        TITLE, WINDOW_HEIGHT, WINDOW_WIDTH, WORLD_HEIGHT, WORLD_WIDTH,
    },
    world::World,
};
use rand::rngs::SmallRng;
use raylib::{
    RaylibHandle,
    ffi::{Camera2D, Color, KeyboardKey, MouseButton, Rectangle, Vector2},
    prelude::{RaylibDraw as _, RaylibDrawHandle, RaylibMode2DExt as _, RaylibScissorModeExt},
    rgui::RaylibGuiControls as _,
};
use std::time::{Duration, Instant};

fn main() {
    let (mut rl, thread) = raylib::init()
        .title(TITLE)
        .size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .build();

    rl.set_target_fps(60);

    let viewport = Viewport::new(
        (WINDOW_WIDTH - WORLD_WIDTH) / 2,
        (WINDOW_HEIGHT - WORLD_HEIGHT) / 2,
        WORLD_WIDTH,
        WORLD_HEIGHT,
        GRID_COLS,
        GRID_ROWS,
    );

    let simulation = Simulation {
        elapsed_time: 0.0,
        rng: rand::make_rng(),
        state: SimulationState {
            paused: false,
            fast_forward: false,
        },
        camera: Camera2D {
            target: Vector2::new(
                (GRID_COLS as f32 * viewport.cell_size.x) / 2.0,
                (GRID_ROWS as f32 * viewport.cell_size.y) / 2.0,
            ),
            offset: Vector2::new(
                viewport.x as f32 + (WORLD_WIDTH as f32 / 2.0),
                viewport.y as f32 + (WORLD_HEIGHT as f32 / 2.0),
            ),
            rotation: 0.0,
            zoom: 1.0,
        },
        stats: Stats {
            update_interval: 0.5,
            update_time: Duration::ZERO,
            update_timer: 0.0,
            render_timer: 0.0,
            render_time: Duration::ZERO,
        },
        world: World {
            grid: Grid::new(GRID_COLS, GRID_ROWS),
            colony: AntColony {
                ants: vec![],
                radius: COLONY_RADIUS,
                area: COLONY_RADIUS * COLONY_RADIUS,
                position: Vector2::new(
                    (GRID_COLS as f32 / 8.0).floor(),
                    (GRID_ROWS as f32 / 2.0).floor(),
                ),
                harvested_food: 0.0,
            },
        },
    };

    let mut app = App {
        simulation,
        gui: Gui::new(),
        renderer: Renderer::new(viewport),
        input_state: app::InputState {
            dragging: false,
            click_start: Vector2::ZERO,
            pheromone_mode: false,
        },
    };

    let sim_render_time_stat = TextBox {
        position: Vector2::new((WORLD_WIDTH / 2) as f32, 10.0),
        font_size: 20,
        color: Color::WHITE,
        render_text: || "foo".into(),
    };

    let debug_panel = SlidePanel {
        side: DockSide::Left,
        open: false,
        current_size: 0.0,
        speed: 800.0,
        title: "Debug".into(),
        tab_position: Vector2::new(0.0, 150.0),
        tab_size: Vector2::new(32.0, 120.0),
        panel_size: Vector2::new(350.0, 300.0),
        render_contents: |d: &mut RaylibDrawHandle, panel: Rectangle| {
            d.gui_label(
                Rectangle {
                    x: panel.x + 10.0,
                    y: panel.y + 10.0,
                    width: 100.0,
                    height: 20.0,
                },
                "FPS",
            );
        },
    };

    let colony_panel = SlidePanel {
        side: DockSide::Right,
        open: false,
        current_size: 0.0,
        speed: 800.0,
        title: "Debug".into(),
        tab_position: Vector2::new(0.0, 150.0),
        tab_size: Vector2::new(32.0, 120.0),
        panel_size: Vector2::new(350.0, 300.0),
        render_contents: |d: &mut RaylibDrawHandle, panel: Rectangle| {
            d.gui_label(
                Rectangle {
                    x: panel.x + 10.0,
                    y: panel.y + 10.0,
                    width: 100.0,
                    height: 20.0,
                },
                "FPS",
            );
        },
    };

    app.gui.register(debug_panel);
    app.gui.register(colony_panel);
    app.gui.register(sim_render_time_stat);

    // ///////////////////////////////////////////////////////
    // OLD
    // ///////////////////////////////////////////////////////

    let viewport = Viewport::new(
        (WINDOW_WIDTH - WORLD_WIDTH) / 2,
        (WINDOW_HEIGHT - WORLD_HEIGHT) / 2,
        WORLD_WIDTH,
        WORLD_HEIGHT,
        GRID_COLS,
        GRID_ROWS,
    );

    println!("{GRID_COLS} x {GRID_ROWS}");

    let mut rng: SmallRng = rand::make_rng();
    let mut renderer = Renderer::new(viewport);
    let colony = AntColony::new_with_immediate_spawn(
        NUM_ANTS,
        PERCENT_OF_EXPLORER_ANTS,
        COLONY_RADIUS,
        Vector2::new(
            (GRID_COLS as f32 / 8.0).floor(),
            (GRID_ROWS as f32 / 2.0).floor(),
        ),
        &mut rng,
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

    let mut is_paused = false;
    let mut is_dragging = false;
    let mut is_pheromone_mode = false;
    let mut click_start_pos = Vector2::zero();
    let mut is_fast_forwarding = false;

    let stats_update_interval_seconds = 0.5f32; // 1.0 = 1second
    let mut stats_update_timer = 0.0f32;
    let mut world_update_time = Duration::ZERO;
    let mut world_render_time = Duration::ZERO;

    let mut simulation_time = 0.0;

    let debug_panel = SlidePanel {
        side: DockSide::Left,
        open: false,
        current_size: 0.0,
        speed: 800.0,
        title: "Debug".into(),
        tab_position: Vector2::new(0.0, 150.0),
        tab_size: Vector2::new(32.0, 120.0),
        panel_size: Vector2::new(350.0, 300.0),
        render_contents: |d: &mut RaylibDrawHandle, panel: Rectangle| {
            d.gui_label(
                Rectangle {
                    x: panel.x + 10.0,
                    y: panel.y + 10.0,
                    width: 100.0,
                    height: 20.0,
                },
                "FPS",
            );
        },
    };

    let mut gui = Gui::new();
    gui.register(debug_panel);

    app.on_key_press(
        &rl,
        |rl: &RaylibHandle,
         renderer: &mut Renderer,
         simulation_state: &mut SimulationState,
         input_state: &mut InputState| {
            if !renderer.viewport.is_within_bounds(rl.get_mouse_position()) {
                return;
            }

            if input_state.pheromone_mode {
                if rl.is_key_pressed(KeyboardKey::KEY_P) {
                    input_state.pheromone_mode = false;
                } else if rl.is_key_pressed(KeyboardKey::KEY_F) {
                    renderer.toggle_show_pheromones("FOOD");
                } else if rl.is_key_pressed(KeyboardKey::KEY_H) {
                    renderer.toggle_show_pheromones("HOME");
                } else if rl.is_key_pressed(KeyboardKey::KEY_A) {
                    renderer.toggle_show_pheromones("ALL");
                }
            } else if rl.is_key_pressed(KeyboardKey::KEY_F) {
                simulation_state.fast_forward = !simulation_state.fast_forward;
            } else if rl.is_key_pressed(KeyboardKey::KEY_A) {
                renderer.toggle_show_ants();
            } else if rl.is_key_pressed(KeyboardKey::KEY_P) {
                input_state.pheromone_mode = true;
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
                simulation_state.paused = !simulation_state.paused;
            }
        },
    );

    app.on_mouse_wheel(
        &rl,
        |rl: &RaylibHandle, cam: &mut Camera2D, renderer: &mut Renderer| {
            let wheel = rl.get_mouse_wheel_move();
            if wheel != 0.0 {
                let raw_mouse_pos = rl.get_mouse_position();
                let world_mouse_before = rl.get_screen_to_world2D(raw_mouse_pos, *cam);

                let scale_factor = 1.0 + (0.15 * wheel.abs());
                let mut next_zoom = if wheel > 0.0 {
                    cam.zoom * scale_factor
                } else {
                    cam.zoom / scale_factor
                };

                next_zoom = next_zoom.clamp(1.0, 100.0);

                if next_zoom > 1.0 {
                    cam.zoom = next_zoom;
                    let world_mouse_after = rl.get_screen_to_world2D(raw_mouse_pos, *cam);
                    cam.target.x += world_mouse_before.x - world_mouse_after.x;
                    cam.target.y += world_mouse_before.y - world_mouse_after.y;
                } else {
                    let wp = &renderer.viewport;
                    // Restore clean home alignment
                    cam.zoom = 1.0;
                    cam.target = Vector2::new(
                        (GRID_COLS as f32 * wp.cell_size.x) / 2.0,
                        (GRID_ROWS as f32 * wp.cell_size.y) / 2.0,
                    );
                    cam.offset = Vector2::new(
                        wp.x as f32 + (WORLD_WIDTH as f32 / 2.0),
                        wp.y as f32 + (WORLD_HEIGHT as f32 / 2.0),
                    );
                }
            }
        },
    );

    /* --------------------------------------- */
    /* ------------ Game Loop ---------------- */
    /* --------------------------------------- */
    while !rl.window_should_close() {
        app.update(rl.get_frame_time());

        //let mut d = rl.begin_drawing(&thread);
        //d.clear_background(BACKGROUND_COLOR);

        // //////////////////
        // OLD
        // //////////////////

        if !gui.blocks_mouse(rl.get_mouse_position()) {
            handle_world_click(
                &mut world,
                &mut renderer,
                &mut rl,
                &mut camera,
                &mut is_paused,
                &mut is_pheromone_mode,
                &mut is_fast_forwarding,
                &mut is_dragging,
                &mut click_start_pos,
            );
        }

        let dt = rl.get_frame_time();
        stats_update_timer -= dt;

        gui.update(dt);

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(BACKGROUND_COLOR);

        if !is_paused {
            if is_fast_forwarding {
                let steps = 5;
                let stable_dt = 0.01666667;
                for _ in 0..steps {
                    simulation_time += stable_dt;
                    let t = calc_game_time(simulation_time);
                    d.draw_text(
                        &format!("Time: {}:{}:{}", t.0, t.1, t.2),
                        WORLD_WIDTH / 2,
                        10,
                        20,
                        Color::WHITE,
                    );
                    let start_t = Instant::now();
                    world.update(stable_dt, &mut rng);
                    if stats_update_timer <= 0.0 {
                        world_update_time = start_t.elapsed();
                    }
                }
            } else {
                simulation_time += dt;
                let world_update_start_t = Instant::now();
                world.update(dt, &mut rng);
                if stats_update_timer <= 0.0 {
                    world_update_time = world_update_start_t.elapsed();
                }
            }
        }

        let t = calc_game_time(simulation_time);
        d.draw_text(
            &format!("Time: {}:{}:{}", t.0, t.1, t.2),
            WORLD_WIDTH / 2,
            10,
            20,
            Color::WHITE,
        );

        d.draw_text(
            &format!("FPS: {}", d.get_fps()),
            WORLD_WIDTH - 20,
            10,
            20,
            Color::WHITE,
        );
        d.draw_text(
            &format!("Update: {world_update_time:?}"),
            WORLD_WIDTH - 20,
            50,
            20,
            Color::WHITE,
        );

        let render_time_start = Instant::now();

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

        if stats_update_timer <= 0.0 {
            world_render_time = render_time_start.elapsed();
        }

        // Draw render time stats
        d.draw_text(
            &format!("Render: {world_render_time:?}"),
            WORLD_WIDTH - 20,
            30,
            20,
            Color::WHITE,
        );
        // Draw num ants
        d.draw_text(
            &format!("Ants: {NUM_ANTS}"),
            WORLD_WIDTH - 20,
            70,
            20,
            Color::WHITE,
        );

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
        if is_fast_forwarding {
            d.draw_text(">> x5 >>", 10, 30, 10, Color::WHITE);
        }

        if stats_update_timer <= 0.0 {
            stats_update_timer = stats_update_interval_seconds;
        }

        gui.draw(&mut d);
    }
}

/* ---------------------------------------------------------------- */
/* -------------- Helper Functions -------------------------------- */
/* ---------------------------------------------------------------- */

#[allow(clippy::too_many_arguments)]
fn handle_world_click(
    world: &mut World,
    renderer: &mut Renderer,
    rl: &mut RaylibHandle,
    camera: &mut Camera2D,
    is_paused: &mut bool,
    is_pheromone_mode: &mut bool,
    is_fast_forwarding: &mut bool,
    is_dragging: &mut bool,
    click_start_pos: &mut Vector2,
) {
    if renderer.viewport.is_within_bounds(rl.get_mouse_position()) {
        handle_key_press(
            rl,
            renderer,
            is_paused,
            is_pheromone_mode,
            is_fast_forwarding,
        );
        handle_mouse_wheel(rl, camera, renderer);
        handle_mouse_click(
            rl,
            camera,
            world,
            &renderer.viewport,
            is_dragging,
            click_start_pos,
        );
    }
}

// Returns tuple of (i32, i32, i32) representng (hours, min, sec)
fn calc_game_time(simulation_time: f32) -> (i32, i32, i32) {
    let total_seconds = simulation_time as i32;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds / 60) % 60;
    let seconds = total_seconds % 60;
    (hours, minutes, seconds)
}

fn handle_key_press(
    rl: &mut RaylibHandle,
    renderer: &mut Renderer,
    is_paused: &mut bool,
    is_pheromone_mode: &mut bool,
    is_fast_forwarding: &mut bool,
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
        *is_fast_forwarding = !*is_fast_forwarding;
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
                .find(|ant| ant.is_clicked(click_grid_position, 1.0f32))
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
