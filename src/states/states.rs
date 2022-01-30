
pub trait GameState {
    fn update(&mut self) -> Option<Box<dyn GameState>>;
    fn render(&self);
}

