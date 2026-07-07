use raylib::{ffi::Vector2, prelude::RaylibDrawHandle};

mod slide_panel;
mod text_box;

pub mod controls {
    pub use super::slide_panel::*;
    pub use super::text_box::*;
}

pub trait GuiComponent {
    fn update(&mut self, delta_time: f32);

    fn draw(&mut self, d: &mut RaylibDrawHandle);

    #[allow(unused_variables)]
    fn wants_mouse(&self, mouse_pos: Vector2) -> bool {
        false
    }
}

pub struct Gui {
    components: Vec<Box<dyn GuiComponent>>,
}

impl Gui {
    pub fn new() -> Self {
        Self { components: vec![] }
    }

    pub fn register<T>(&mut self, component: T)
    where
        T: GuiComponent + 'static,
    {
        self.components.push(Box::new(component));
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
