use raylib::{ffi::Vector2, prelude::RaylibDrawHandle};

mod slide_panel;

pub mod controls {
    pub use super::slide_panel::*;
}

pub trait GuiComponent<State> {
    fn update(&mut self, delta_time: f32);

    fn draw(&mut self, d: &mut RaylibDrawHandle, state: &mut State);

    fn is_disabled(&self) -> bool;

    #[allow(unused_variables)]
    fn wants_mouse(&self, mouse_pos: Vector2) -> bool {
        false
    }
}

pub struct Gui<State> {
    components: Vec<Box<dyn GuiComponent<State>>>,
}

impl<State> Gui<State> {
    pub fn new() -> Self {
        Self { components: vec![] }
    }

    pub fn register<T>(&mut self, component: T)
    where
        T: GuiComponent<State> + 'static,
    {
        self.components.push(Box::new(component));
    }

    pub fn update(&mut self, delta_time: f32) {
        for component in &mut self.components {
            component.update(delta_time);
        }
    }

    pub fn draw(&mut self, d: &mut RaylibDrawHandle, state: &mut State) {
        for component in &mut self.components {
            if !component.is_disabled() {
                component.draw(d, state);
            }
        }
    }

    pub fn blocks_mouse(&self, mouse_pos: Vector2) -> bool {
        self.components
            .iter()
            .any(|component| component.wants_mouse(mouse_pos))
    }
}
