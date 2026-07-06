use raylib::{ffi::Vector2, prelude::RaylibDrawHandle};

mod slide_panel;

pub mod controls {
    pub use super::slide_panel::*;
}

pub trait GuiComponent {
    fn update(&mut self, delta_time: f32);

    fn draw(&mut self, d: &mut RaylibDrawHandle);

    #[allow(unused_variables)]
    fn wants_mouse(&self, mouse_pos: Vector2) -> bool {
        false
    }
}

pub struct Gui<T>
where
    T: GuiComponent,
{
    components: Vec<T>,
}

impl<T> Gui<T>
where
    T: GuiComponent,
{
    pub fn new() -> Self {
        Self { components: vec![] }
    }

    pub fn register(&mut self, component: T) {
        self.components.push(component);
    }

    pub fn update(&mut self, delta_time: f32) {
        for component in &mut self.components {
            component.update(delta_time);
        }
    }

    pub fn draw(&mut self, d: &mut RaylibDrawHandle) {
        for component in &mut self.components {
            component.draw(d);
        }
    }

    pub fn blocks_mouse(&self, mouse_pos: Vector2) -> bool {
        self.components
            .iter()
            .any(|component| component.wants_mouse(mouse_pos))
    }
}
