use crate::{
    gui::{Gui, GuiComponent},
    render::Renderer,
    world::{self, World},
};
use rand::rngs::SmallRng;
use raylib::{
    RaylibHandle,
    ffi::{Camera2D, KeyboardKey, Vector2},
    prelude::RaylibDrawHandle,
};
use std::time::Duration;

pub struct Stats {
    pub update_interval: f32,
    pub update_timer: f32,
    pub update_time: Duration,
    pub render_timer: f32,
    pub render_time: Duration,
}

pub struct InputState {
    pub dragging: bool,
    pub click_start: Vector2,
    pub pheromone_mode: bool,
}

pub struct SimulationState {
    pub paused: bool,
    pub fast_forward: bool,
}

pub struct Simulation {
    pub world: World,
    pub rng: SmallRng,
    pub state: SimulationState,
    pub elapsed_time: f32,
    pub stats: Stats,
    pub camera: Camera2D,
}

impl Simulation {
    pub fn update(&mut self, delta_time: f32) {
        self.elapsed_time += delta_time;
    }

    pub fn draw(&mut self, draw: &mut RaylibDrawHandle) {
        //
    }

    fn calc_game_time(&self) -> (i32, i32, i32) {
        let total_seconds = self.elapsed_time as i32;
        let hours = total_seconds / 3600;
        let minutes = (total_seconds / 60) % 60;
        let seconds = total_seconds % 60;
        (hours, minutes, seconds)
    }
}

pub struct App {
    pub simulation: Simulation,
    pub renderer: Renderer,
    pub gui: Gui,
    pub input_state: InputState,
}

impl App {
    pub fn update(&mut self, delta_time: f32) {
        self.simulation.stats.update_timer -= delta_time;
        self.gui.update(delta_time);
        if !self.simulation.state.paused {
            self.simulation.update(delta_time);
        }
    }

    pub fn draw(&mut self, draw: &mut RaylibDrawHandle) {
        self.renderer.draw_world(&mut self.simulation.world, draw);
        self.gui.draw(draw);
    }

    pub fn on_mouse_left_click<F>(&mut self, mut handler: F)
    where
        F: FnMut(&mut InputState),
    {
        (handler)(&mut self.input_state);
    }

    pub fn on_key_press<F>(&mut self, rl: &RaylibHandle, mut handler: F)
    where
        F: FnMut(&RaylibHandle, &mut Renderer, &mut SimulationState, &mut InputState),
    {
        let mouse = rl.get_mouse_position();
        if self.gui.blocks_mouse(mouse) {
            return;
        }

        (handler)(
            rl,
            &mut self.renderer,
            &mut self.simulation.state,
            &mut self.input_state,
        );
    }

    pub fn on_mouse_wheel<F>(&mut self, rl: &RaylibHandle, mut handler: F)
    where
        F: FnMut(&RaylibHandle, &mut Camera2D, &mut Renderer),
    {
        (handler)(rl, &mut self.simulation.camera, &mut self.renderer);
    }
}
