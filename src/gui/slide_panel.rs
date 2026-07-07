use crate::gui::GuiComponent;
use raylib::{
    ffi::{Color, MouseButton, Rectangle, Vector2},
    prelude::{RaylibDraw as _, RaylibDrawHandle},
};

#[allow(dead_code)]
pub enum DockSide {
    Left,
    Right,
    Top,
    Bottom,
}

pub struct SlidePanel<RenderFn> {
    pub side: DockSide,
    pub open: bool,
    // animation
    pub current_size: f32,
    pub speed: f32,
    // tab
    pub title: String,
    pub tab_position: Vector2,
    pub tab_size: Vector2,
    // panel
    pub panel_size: Vector2,
    pub render_contents: RenderFn,
    pub enabled: bool,
}

impl<State, RenderFn> GuiComponent<State> for SlidePanel<RenderFn>
where
    RenderFn: FnMut(&mut RaylibDrawHandle, Rectangle, &mut State),
{
    fn is_disabled(&self) -> bool {
        !self.enabled
    }

    fn update(&mut self, delta_time: f32) {
        let target = match self.side {
            DockSide::Left | DockSide::Right => self.panel_size.x,
            DockSide::Top | DockSide::Bottom => self.panel_size.y,
        };

        let target = if self.open { target } else { 0.0 };
        let delta = self.speed * delta_time;

        if self.current_size < target {
            self.current_size = (self.current_size + delta).min(target);
        } else {
            self.current_size = (self.current_size - delta).max(target);
        }
    }

    fn draw(&mut self, d: &mut RaylibDrawHandle, state: &mut State) {
        let panel = self.panel_rect();
        let tab = self.tab_rect();

        if self.current_size > 0.0 {
            d.draw_rectangle_rec(panel, Color::DARKGRAY);
            d.draw_rectangle_lines_ex(panel, 1.0, Color::BLACK);

            (self.render_contents)(d, panel, state);
        }

        // Tab
        let hovered = tab.check_collision_point_rec(d.get_mouse_position());

        let bg = if hovered {
            Color::GRAY
        } else {
            Color::DARKGRAY
        };

        d.draw_rectangle_rec(tab, bg);
        d.draw_rectangle_lines_ex(tab, 1.0, Color::BLACK);

        // Toggle panel
        if hovered && d.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            self.open = !self.open;
        }

        // Draw title
        let font_size = 20;

        match self.side {
            DockSide::Left | DockSide::Right => {
                let text_width = d.measure_text(&self.title, font_size) as f32;
                let text_height = font_size as f32;
                let center = Vector2::new(tab.x + tab.width * 0.5, tab.y + tab.height * 0.5);

                d.draw_text_pro(
                    d.get_font_default(),
                    &self.title,
                    center,
                    Vector2::new(text_width * 0.5, text_height * 0.5),
                    90.0,
                    font_size as f32,
                    1.0,
                    Color::WHITE,
                );
            }
            DockSide::Top | DockSide::Bottom => {
                let text_width = d.measure_text(&self.title, font_size) as f32;
                let x = tab.x + (tab.width - text_width) * 0.5;
                let y = tab.y + (tab.height - font_size as f32) * 0.5;
                d.draw_text(&self.title, x as i32, y as i32, font_size, Color::WHITE);
            }
        }
    }

    fn wants_mouse(&self, mouse_pos: Vector2) -> bool {
        if self.current_size <= 0.0 {
            return false;
        }
        self.panel_rect().check_collision_point_rec(mouse_pos)
            || self.tab_rect().check_collision_point_rec(mouse_pos)
    }
}

impl<RenderFn> SlidePanel<RenderFn> {
    fn panel_rect(&self) -> Rectangle {
        match self.side {
            DockSide::Left => Rectangle {
                x: 0.0,
                y: self.tab_position.y,
                width: self.current_size,
                height: self.panel_size.y,
            },
            DockSide::Right => Rectangle {
                x: self.tab_position.x - self.current_size,
                y: self.tab_position.y,
                width: self.current_size,
                height: self.panel_size.y,
            },
            DockSide::Top => Rectangle {
                x: self.tab_position.x,
                y: 0.0,
                width: self.panel_size.x,
                height: self.current_size,
            },
            DockSide::Bottom => Rectangle {
                x: self.tab_position.x,
                y: self.tab_position.y - self.current_size,
                width: self.panel_size.x,
                height: self.current_size,
            },
        }
    }

    fn tab_rect(&self) -> Rectangle {
        match self.side {
            DockSide::Left => Rectangle {
                x: self.current_size,
                y: self.tab_position.y,
                width: self.tab_size.x,
                height: self.tab_size.y,
            },
            DockSide::Right => Rectangle {
                x: self.tab_position.x,
                y: self.tab_position.y,
                width: self.tab_size.x,
                height: self.tab_size.y,
            },
            DockSide::Top => Rectangle {
                x: self.tab_position.x,
                y: self.current_size,
                width: self.tab_size.x,
                height: self.tab_size.y,
            },
            DockSide::Bottom => Rectangle {
                x: self.tab_position.x,
                y: self.tab_position.y,
                width: self.tab_size.x,
                height: self.tab_size.y,
            },
        }
    }
}
