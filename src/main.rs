#![allow(clippy::assertions_on_constants)]
#![allow(clippy::field_reassign_with_default)]

mod ant;
mod gui;
mod map;
mod profiler;
mod render;
mod reynolds;
mod settings;
mod world;

use crate::{
    ant::AntColony,
    gui::{
        Gui,
        controls::{DockSide, SlidePanel},
    },
    map::{Grid, SpatialGrid},
    profiler::Profiler,
    render::{Renderer, Viewport},
    settings::{
        BACKGROUND_COLOR, COLONY_RADIUS, GRID_COLS, GRID_ROWS, NUM_ANTS, PERCENT_OF_EXPLORER_ANTS,
        SPATIAL_GRID_BUCKET_SIZE, TITLE, WINDOW_HEIGHT, WINDOW_WIDTH, WORLD_HEIGHT, WORLD_WIDTH,
    },
    world::World,
};
use rand::rngs::SmallRng;
use raylib::{
    RaylibHandle,
    ffi::{Camera2D, Color, KeyboardKey, MouseButton, Rectangle, Vector2},
    prelude::{
        Font, RaylibDraw as _, RaylibDrawHandle, RaylibMode2DExt as _, RaylibScissorModeExt,
    },
};

struct AppState {
    /// Is the simulation paused.
    pub is_paused: bool,
    /// Are we dragging within the simulation?
    pub is_dragging: bool,
    /// Are we in pheromone mode?
    pub is_pheromone_mode: bool,
    /// To help us determine if we are dragging or not
    pub click_start_pos: Vector2,
    pub is_fast_forwarding: bool,
    /// 5 means full speed is 5x normal speed.
    pub fast_forward_speed: usize,
    /// Time elapsed in simulation
    pub simulation_time: f32,
    pub fps: u32,
    pub profiler: Profiler,
    pub font: Font,
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .title(TITLE)
        .size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .build();

    rl.set_target_fps(60);

    let font = rl
        .load_font(
            &thread,
            "assets/playfair-display-font/PlayfairDisplayBold-nRv8g.ttf",
        )
        .expect("failed to load font");

    let ant_texture = rl
        .load_texture(&thread, "assets/ant.png")
        .expect("something went wrong loading ant texture");

    let profiler = Profiler::new(0.2);

    let mut app_state = AppState {
        is_paused: false,
        is_dragging: false,
        is_pheromone_mode: false,
        click_start_pos: Vector2::ZERO,
        is_fast_forwarding: false,
        fast_forward_speed: 5,
        simulation_time: 0.0,
        fps: 0,
        profiler,
        font,
    };

    let viewport = Viewport::new(
        (WINDOW_WIDTH - WORLD_WIDTH) / 2,
        (WINDOW_HEIGHT - WORLD_HEIGHT) / 2,
        WORLD_WIDTH,
        WORLD_HEIGHT,
        GRID_COLS,
        GRID_ROWS,
    );

    let mut renderer = Renderer::new(viewport, ant_texture);

    let mut rng: SmallRng = rand::make_rng();

    let mut world = World::new(
        Grid::new(GRID_COLS, GRID_ROWS),
        SpatialGrid::new(GRID_COLS, GRID_ROWS, SPATIAL_GRID_BUCKET_SIZE),
        AntColony::new(
            NUM_ANTS,
            PERCENT_OF_EXPLORER_ANTS,
            COLONY_RADIUS,
            Vector2::new(
                (GRID_COLS as f32 / 8.0).floor(),
                (GRID_ROWS as f32 / 2.0).floor(),
            ),
        ),
    );

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

    let debug_panel_width = 360;
    let debug_panel_height = 510.0;
    let debug_panel = SlidePanel {
        side: DockSide::Top,
        open: false,
        enabled: true,
        current_size: 0.0,
        speed: 900.0,
        title: "Debug".into(),
        tab_position: Vector2::new((WINDOW_WIDTH - debug_panel_width) as f32, 0.0),
        tab_size: Vector2::new(80.0, 22.0),
        panel_size: Vector2::new(debug_panel_width as f32, debug_panel_height),
        render_contents: |d: &mut RaylibDrawHandle, panel: Rectangle, state: &mut AppState| {
            let font_size = 21.0;
            let color = Color::BLACK;
            let spacing = 1.0;
            let mut text_pos = Vector2::new(panel.x + 5.0, panel.y + 10.0);

            for (name, section) in state.profiler.sections().iter() {
                let t = format!(
                    "{:<20} {:>8.3} ms  calls: {}",
                    name,
                    section.accumulated.as_secs_f64() * 1000.0,
                    section.accumulated_calls
                );
                d.draw_text_ex(&state.font, &t, text_pos, font_size, spacing, color);
                text_pos.y += font_size;
            }

            let customs = [
                &format!("FPS: {}", state.fps),
                &format!(
                    "Rows='{GRID_ROWS}' Columns='{GRID_COLS}' Cells='{}'",
                    GRID_ROWS * GRID_COLS
                ),
                &format!("Number of Ants: {NUM_ANTS}"),
            ];

            for custom in customs {
                d.draw_text_ex(&state.font, custom, text_pos, font_size, spacing, color);
                text_pos.y += font_size;
            }
        },
    };

    let mut gui = Gui::new();
    gui.register(debug_panel);

    println!("Ant        {}", size_of::<ant::Ant>());
    println!("Navigator  {}", size_of::<crate::reynolds::Navigation>());
    println!("Sensors    {}", size_of::<ant::Sensors>());
    println!("Sensor     {}", size_of::<ant::Sensor>());
    println!("Vector2    {}", size_of::<Vector2>());

    /* --------------------------------------- */
    /* ------------ Game Loop ---------------- */
    /* --------------------------------------- */
    while !rl.window_should_close() {
        app_state.profiler.begin_frame();
        if !gui.blocks_mouse(rl.get_mouse_position()) {
            handle_world_click(
                &mut world,
                &mut renderer,
                &mut rl,
                &mut camera,
                &mut app_state,
            );
        }

        let mut dt = rl.get_frame_time();

        gui.update(dt);

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(BACKGROUND_COLOR);

        app_state.fps = d.get_fps();

        if !app_state.is_paused {
            if app_state.is_fast_forwarding {
                dt = 0.01666667; // stabilize dt
                for _ in 0..app_state.fast_forward_speed {
                    app_state.simulation_time += dt;
                    world.update(dt, &mut rng, &mut app_state.profiler);
                }
            } else {
                app_state.simulation_time += dt;
                world.update(dt, &mut rng, &mut app_state.profiler);
            }
        }

        let t = calc_game_time(app_state.simulation_time);
        d.draw_text(
            &format!("Time: {}:{}:{}", t.0, t.1, t.2),
            WORLD_WIDTH / 2,
            10,
            20,
            Color::WHITE,
        );

        {
            let mut scissor = d.begin_scissor_mode(
                renderer.viewport.x,
                renderer.viewport.y,
                renderer.viewport.width,
                renderer.viewport.height,
            );
            let mut mode2d = scissor.begin_mode2D(camera);
            renderer.draw_world(&mut world, &mut mode2d, &mut app_state.profiler);
        }

        // Bordr around viewport
        d.draw_rectangle_lines(
            renderer.viewport.x,
            renderer.viewport.y,
            renderer.viewport.width,
            renderer.viewport.height,
            Color::RED,
        );

        if app_state.is_pheromone_mode {
            d.draw_text("PHEROMONE MODE ON", 10, 10, 20, Color::WHITE);
        }
        if app_state.is_fast_forwarding {
            d.draw_text(">> x5 >>", 10, 30, 10, Color::WHITE);
        }

        gui.draw(&mut d, &mut app_state);
        app_state.profiler.end_frame();
        app_state.profiler.print();
    }
}

/* ---------------------------------------------------------------- */
/* -------------- Helper Functions -------------------------------- */
/* ---------------------------------------------------------------- */

fn handle_world_click(
    world: &mut World,
    renderer: &mut Renderer,
    rl: &mut RaylibHandle,
    camera: &mut Camera2D,
    app_state: &mut AppState,
) {
    if renderer.viewport.is_within_bounds(rl.get_mouse_position()) {
        handle_key_press(rl, renderer, app_state);
        handle_mouse_wheel(rl, camera, renderer);
        handle_mouse_click(rl, camera, world, &renderer.viewport, app_state);
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

fn handle_key_press(rl: &mut RaylibHandle, renderer: &mut Renderer, app_state: &mut AppState) {
    if app_state.is_pheromone_mode {
        if rl.is_key_pressed(KeyboardKey::KEY_P) {
            app_state.is_pheromone_mode = false;
        } else if rl.is_key_pressed(KeyboardKey::KEY_F) {
            renderer.toggle_show_pheromones("FOOD");
        } else if rl.is_key_pressed(KeyboardKey::KEY_H) {
            renderer.toggle_show_pheromones("HOME");
        } else if rl.is_key_pressed(KeyboardKey::KEY_A) {
            renderer.toggle_show_pheromones("ALL");
        }
    } else if rl.is_key_pressed(KeyboardKey::KEY_F) {
        app_state.is_fast_forwarding = !app_state.is_fast_forwarding;
    } else if rl.is_key_pressed(KeyboardKey::KEY_A) {
        renderer.toggle_show_ants();
    } else if rl.is_key_pressed(KeyboardKey::KEY_P) {
        app_state.is_pheromone_mode = true;
    } else if rl.is_key_pressed(KeyboardKey::KEY_B) {
        renderer.toggle_show_border();
    } else if rl.is_key_pressed(KeyboardKey::KEY_G) {
        renderer.toggle_show_grid();
    } else if rl.is_key_pressed(KeyboardKey::KEY_S) {
        renderer.toggle_show_ant_sensors();
    } else if rl.is_key_pressed(KeyboardKey::KEY_C) {
        renderer.toggle_show_colony();
    } else if rl.is_key_pressed(KeyboardKey::KEY_R) {
        renderer.toggle_ant_projection_circle();
    } else if rl.is_key_pressed(KeyboardKey::KEY_O) {
        renderer.toggle_show_food();
    } else if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
        app_state.is_paused = !app_state.is_paused;
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
    app_state: &mut AppState,
) {
    const DRAG_THRESHOLD: f32 = 5.0;
    // Always use the absolute, raw window mouse coordinates for camera calculations
    let raw_mouse_pos = rl.get_mouse_position();

    if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
        app_state.click_start_pos = raw_mouse_pos;
        app_state.is_dragging = false;
    }

    if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
        if !app_state.is_dragging
            && raw_mouse_pos.distance(app_state.click_start_pos) > DRAG_THRESHOLD
        {
            app_state.is_dragging = true;
        }

        if app_state.is_dragging {
            let mouse_delta = rl.get_mouse_delta();
            let drag_vector =
                Vector2::new(mouse_delta.x / camera.zoom, mouse_delta.y / camera.zoom);
            camera.target.x -= drag_vector.x;
            camera.target.y -= drag_vector.y;
        }
    }

    if rl.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT) {
        if !app_state.is_dragging {
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
        app_state.is_dragging = false;
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
