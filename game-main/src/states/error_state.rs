use crate::states::GameState;
use macroquad::{color::RED, prelude::draw_text};
use macroquad_canvas::Canvas2D;

pub struct ErrorState {
    description: String,
}

impl ErrorState {
    pub fn new(description: String) -> Self {
        Self { description }
    }
}

impl GameState for ErrorState {
    fn update(&mut self, _canvas: &Canvas2D) -> Option<Box<dyn GameState>> {
        None
    }

    fn render(&self, _canvas: &Canvas2D) {
        draw_text(self.description.as_str(), 100., 100., 50., RED);
    }

    fn uses_egui(&self) -> bool {
        true
    }
}
