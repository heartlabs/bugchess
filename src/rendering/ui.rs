use crate::constants::*;
use egui_macroquad::egui::TextBuffer;
use macroquad::prelude::*;
use macroquad_canvas::Canvas2D;

#[derive(Debug, Clone)]
pub struct Button {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    text: String,
}

impl Button {
    pub fn new(y: f32, text: String) -> Self {
        let button_x = BOARD_WIDTH as f32 * CELL_ABSOLUTE_WIDTH + PIECE_SCALE * 2. + 10.;
        Button {
            x: button_x,
            y,
            width: 170.,
            height: 60.,
            text,
        }
    }

    pub(crate) fn render(&self, canvas: &Canvas2D) {
        let (button_color, text_color) = if self.hovered(canvas) {
            (DARKGREEN, WHITE)
        } else {
            (WHITE, DARKGREEN)
        };

        draw_rectangle(self.x, self.y, self.width, self.height, button_color);
        draw_text(&*self.text, self.x + 10., self.y + 40., 40., text_color);
    }

    pub fn hovered(&self, canvas: &Canvas2D) -> bool {
        let (mouse_x, mouse_y) = canvas.mouse_position();

        mouse_x >= self.x
            && mouse_x <= self.x + self.width
            && mouse_y >= self.y
            && mouse_y <= self.y + self.height
    }

    pub fn clicked(&self, canvas: &Canvas2D) -> bool {
        is_mouse_button_pressed(MouseButton::Left) && self.hovered(canvas)
    }
}
