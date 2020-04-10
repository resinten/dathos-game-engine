pub mod core;
pub mod draw;

pub trait EngineModule<G>
where
    G: GameState,
{
    fn init(&mut self, _: &mut G) {}
    fn pre_update(&mut self, _: &mut G) {}
    fn update(&mut self, _: &mut G) {}
    fn post_update(&mut self, _: &mut G) {}
}

pub trait GameState {
    fn window_options(&self) -> WindowOptions;
}

#[derive(Clone)]
pub struct WindowOptions {
    pub width: u32,
    pub height: u32,
    pub title: String,
}

impl GameState for WindowOptions {
    fn window_options(&self) -> WindowOptions { self.clone() }
}
