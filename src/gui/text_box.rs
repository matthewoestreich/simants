use crate::{app::SimulationState, gui::GuiComponent};
use raylib::{
    ffi::{Color, Rectangle, Vector2},
    prelude::{RaylibDraw as _, RaylibDrawHandle},
};

pub struct TextBox<RenderFn>
where
    RenderFn: FnMut() -> String,
{
    pub font_size: i32,
    pub position: Vector2,
    pub color: Color,
    pub render_text: RenderFn,
}

impl<RenderFn> GuiComponent for TextBox<RenderFn>
where
    RenderFn: FnMut() -> String,
{
    fn update(&mut self, _delta_time: f32) {
        //
    }

    fn draw(&mut self, d: &mut RaylibDrawHandle) {
        d.draw_text(
            &(self.render_text)(),
            self.position.x as i32,
            self.position.y as i32,
            self.font_size,
            self.color,
        );
    }
}
