use macroquad_canvas::Canvas2D;

pub trait GameState {
    fn update(&mut self, canvas: &Canvas2D) -> Option<Box<dyn GameState>>;
    fn render(&self, canvas: &Canvas2D);
}

