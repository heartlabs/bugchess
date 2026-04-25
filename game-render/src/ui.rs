use macroquad::{
    math::Rect,
    prelude::{DARKGREEN, MouseButton, WHITE, is_mouse_button_pressed},
    shapes::draw_rectangle,
    text::draw_text,
};
use macroquad_canvas::Canvas2D;

#[derive(Debug, Clone)]
pub struct Button {
    rect: Rect,
    text: String,
}

impl Button {
    pub fn new(rect: Rect, text: String) -> Self {
        Button { rect, text }
    }

    pub(crate) fn render(&self, canvas: &Canvas2D) {
        let (button_color, text_color) = if self.hovered(canvas) {
            (DARKGREEN, WHITE)
        } else {
            (WHITE, DARKGREEN)
        };

        draw_rectangle(
            self.rect.x,
            self.rect.y,
            self.rect.w,
            self.rect.h,
            button_color,
        );
        draw_text(
            &self.text,
            self.rect.x + 10.,
            self.rect.y + self.rect.h * 0.7,
            self.rect.h * 0.65,
            text_color,
        );
    }

    pub fn hovered(&self, canvas: &Canvas2D) -> bool {
        let (mouse_x, mouse_y) = canvas.mouse_position();

        mouse_x >= self.rect.x
            && mouse_x <= self.rect.x + self.rect.w
            && mouse_y >= self.rect.y
            && mouse_y <= self.rect.y + self.rect.h
    }

    pub fn clicked(&self, canvas: &Canvas2D) -> bool {
        is_mouse_button_pressed(MouseButton::Left) && self.hovered(canvas)
    }
}
